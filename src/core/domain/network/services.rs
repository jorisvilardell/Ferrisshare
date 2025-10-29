use std::io::Error;

use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncReadExt;
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

#[derive(Debug, Copy, Clone)]
pub struct NetworkServiceImpl<C>
where
    C: CommandService,
{
    command_service: C,
}

impl<C> NetworkServiceImpl<C>
where
    C: CommandService,
{
    pub fn new(command_service: C) -> Self {
        NetworkServiceImpl { command_service }
    }
}

impl<C> NetworkService for NetworkServiceImpl<C>
where
    C: CommandService,
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
            let (stream, addr) = listener
                .accept()
                .await
                .map_err(|_| NetworkError::ConnectionLost)?;
            println!("New connection from {}", addr);

            // Essaye d’envoyer la connexion au handler
            if let Err(_) = tx.try_send(stream) {
                println!("Connection refused: another one is active");
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
                                // Here you would send the response back to the client
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

    fn send_message(
        &self,
        stream: &TcpStream,
        message: &ProtocolMessage,
    ) -> Result<(), ProtocolError> {
        todo!()
    }
}
