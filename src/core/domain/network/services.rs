use std::io::Error;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::stream;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use crate::core::domain::command::ports::CommandService;
use crate::core::domain::network::entities::NetworkError;
use crate::core::domain::network::entities::ProtocolError;
use crate::core::domain::network::entities::ProtocolMessage;
use crate::core::domain::network::ports::NetworkService;

#[derive(Debug, Clone)]
pub struct NetworkServiceImpl<C>
where
    C: CommandService,
{
    command_service: C,
    active: Arc<AtomicBool>,
}

impl<C> NetworkServiceImpl<C>
where
    C: CommandService + Clone + Send + Sync + 'static,
{
    pub fn new(command_service: C) -> Self {
        NetworkServiceImpl {
            command_service,
            active: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl<C> NetworkService for NetworkServiceImpl<C>
where
    C: CommandService + Clone + Send + Sync + 'static,
{
    async fn listener(
        &self,
        addr: &str,
        tx: tokio::sync::mpsc::Sender<TcpStream>,
    ) -> Result<(), NetworkError> {
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| NetworkError::ListenerBindFailed(e))?;
        println!("Listening on {}", addr);

        loop {
            let (mut stream, addr) = listener
                .accept()
                .await
                .map_err(|_| NetworkError::ConnectionLost)?;
            println!("New connection from {}", addr);

            // Vérifie si une connexion est déjà active
            if !self.active.load(std::sync::atomic::Ordering::SeqCst) {
                self.active.store(true, std::sync::atomic::Ordering::SeqCst);
            } else {
                eprintln!(
                    "A connection is already active. Rejecting new connection from {}",
                    addr
                );
                let _ = stream.shutdown().await;
                continue;
            }

            // Essaye d’envoyer la connexion au handler
            match tx.try_send(stream) {
                Ok(_) => println!("Connection sent to handler."),
                Err(e) => {
                    eprintln!("Failed to send connection to handler: {}", e);
                    // Optionally close the connection if it can't be handled
                }
            }
        }
    }

    async fn handler(&self, mut rx: Receiver<TcpStream>) -> Result<(), Error> {
        while let Some(stream) = rx.recv().await {
            let mut reader = BufReader::new(stream);
            let mut buf = Vec::new();

            loop {
                buf.clear();

                // Read one line (terminated by '\n'); returns 0 on EOF
                let n = reader.read_until(b'\n', &mut buf).await?;
                if n == 0 {
                    println!("Client disconnected.");
                    break;
                }

                // Trim trailing LF/CRLF
                if buf.ends_with(b"\n") {
                    buf.pop();
                }
                if buf.ends_with(b"\r") {
                    buf.pop();
                }

                // Convert to &str and parse into your ProtocolMessage
                let line = std::str::from_utf8(&buf)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

                match ProtocolMessage::try_from(line) {
                    Ok(msg) => {
                        println!("Received message: {:?}", msg);
                        match self.command_service.execute_protocol_command(&msg).await {
                            Ok(response) => {
                                println!("Response: {:?}", response);
                                // Clone the underlying TcpStream to avoid moving reader
                                let mut stream_ref = reader.get_mut();
                                match self.send_message(&mut stream_ref, response).await {
                                    Ok(_) => println!("Response sent successfully."),
                                    Err(e) => eprintln!("Failed to send response: {:?}", e),
                                }
                            }
                            Err(e) => {
                                println!("Command execution error: {:?}", e);
                                // Handle command execution error
                            }
                        }
                    }
                    Err(e) => {
                        // You have From<ProtocolError> for String
                        println!("Failed to parse message: {}", String::from(e));
                    }
                }

                println!("Received line: {}", line);
                // handle msg here...
            }
        }

        Ok(())
    }

    async fn send_message(
        &self,
        stream: &mut TcpStream,
        message: ProtocolMessage,
    ) -> Result<(), ProtocolError> {
        let msg_str = String::from(message) + "\n";
        if let Err(e) = stream.write_all(msg_str.as_bytes()).await {
            eprintln!("Failed to send message: {}", e);
        }
        Ok(())
    }
}
