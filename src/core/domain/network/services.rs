use std::io::Error;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Receiver;

use crate::core::domain::command::ports::CommandService;
use crate::core::domain::network::entities::NetworkError;
use crate::core::domain::network::entities::ProtocolError;
use crate::core::domain::network::entities::ProtocolMessage;
use crate::core::domain::network::entities::TransferState;
use crate::core::domain::network::ports::NetworkService;

#[derive(Debug, Clone)]
pub struct NetworkServiceImpl<C>
where
    C: CommandService,
{
    pub command_service: C,
    active: Arc<AtomicBool>,
    transfer_state: Arc<Mutex<TransferState>>,
}

impl<C> NetworkServiceImpl<C>
where
    C: CommandService + Clone + Send + Sync + 'static,
{
    pub fn new(command_service: C) -> Self {
        NetworkServiceImpl {
            command_service,
            active: Arc::new(AtomicBool::new(false)),
            transfer_state: Arc::new(Mutex::new(TransferState::Idle)),
        }
    }

    pub async fn reset_transfer_state(&self) {
        let mut state_guard = self.transfer_state.lock().await;
        *state_guard = TransferState::Idle;
        drop(state_guard);
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
            .map_err(NetworkError::ListenerBindFailed)?;
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
            let (read_half, mut write_half) = stream.into_split();
            let mut reader = BufReader::new(read_half);
            let mut buf = Vec::new();

            loop {
                buf.clear();

                // Read one line (terminated by '\n'); returns 0 on EOF
                let n = reader.read_until(b'\n', &mut buf).await?;
                if n == 0 {
                    println!("Client disconnected.");
                    // Mark connection as inactive so listener can accept new ones
                    self.active
                        .store(false, std::sync::atomic::Ordering::SeqCst);
                    self.reset_transfer_state().await;
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

                        match self.trust_protocol(msg).await {
                            Ok(message) => match message {
                                ProtocolMessage::Yeet(yeet_block) => {
                                    let mut bin_buf = vec![0u8; yeet_block.size as usize];
                                    if let Err(e) = reader.read_exact(&mut bin_buf).await {
                                        eprintln!("Error reading binary block: {:?}", e);
                                        let err_msg = ProtocolMessage::Error(String::from(
                                            "Read binary failed",
                                        ));
                                        let s = String::from(err_msg) + "\n";
                                        let _ = write_half.write_all(s.as_bytes()).await;
                                        continue;
                                    }

                                    // Consume the trailing newline after the binary block if any.
                                    let mut _end = Vec::new();
                                    let _ = reader.read_until(b'\n', &mut _end).await;

                                    // Forward the block to the command service for storage.
                                    match self
                                        .command_service
                                        .process_binary_data(
                                            Arc::clone(&self.transfer_state),
                                            &bin_buf,
                                        )
                                        .await
                                    {
                                        Ok(response_msg) => {
                                            let s = String::from(response_msg) + "\n";
                                            if let Err(e) = write_half.write_all(s.as_bytes()).await
                                            {
                                                eprintln!("Error sending response: {:?}", e);
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Error processing binary data: {:?}", e);
                                            let err_msg = ProtocolMessage::Error(String::from(e));
                                            let s = String::from(err_msg) + "\n";
                                            let _ = write_half.write_all(s.as_bytes()).await;
                                        }
                                    }
                                }
                                other => {
                                    // Non-YEET responses (OK, SUCCESS, etc.) are sent back to writer.
                                    let s = String::from(other) + "\n";
                                    if let Err(e) = write_half.write_all(s.as_bytes()).await {
                                        eprintln!("Error sending message: {:?}", e);
                                    }
                                }
                            },
                            Err(e) => eprintln!("Error handling protocol message: {:?}", e),
                        }

                        let guard = self.transfer_state.lock().await;
                        match *guard {
                            TransferState::Closed => {
                                println!("Closing connection.");
                                self.active
                                    .store(false, std::sync::atomic::Ordering::SeqCst);
                                drop(guard);

                                self.reset_transfer_state().await;
                                // shutdown the write half
                                let _ = write_half.shutdown().await;

                                break;
                            }
                            _ => {
                                continue;
                            }
                        }
                    }
                    Err(_) => {
                        if let Err(e) = self
                            .command_service
                            .process_binary_data(Arc::clone(&self.transfer_state), &buf)
                            .await
                        {
                            eprintln!("Error processing binary data: {:?}", e);
                            let err_msg = ProtocolMessage::Error(format!("{:?}", e));
                            let s = String::from(err_msg) + "\n";
                            let _ = write_half.write_all(s.as_bytes()).await;
                        }
                    }
                }
            }
        }

        self.active
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self.reset_transfer_state().await;
        Ok(())
    }

    async fn trust_protocol(
        &self,
        message: ProtocolMessage,
    ) -> Result<ProtocolMessage, ProtocolError> {
        let guard = self.transfer_state.lock().await;
        match *guard {
            TransferState::Idle => {
                if !matches!(message, ProtocolMessage::Hello { .. }) {
                    return Err(ProtocolError::InvalidCommand);
                } else {
                    println!("Transitioning from Idle to Receiving state.");
                }
            }
            TransferState::Receiving { .. } => {
                // Accept either a Yeet message or a MissionAccomplished message while receiving.
                if !(matches!(message, ProtocolMessage::Yeet { .. })
                    || matches!(message, ProtocolMessage::MissionAccomplished))
                {
                    return Err(ProtocolError::InvalidCommand);
                } else {
                    println!("In Receiving state, processing Yeet or MissionAccomplished.");
                }
            }
            TransferState::Finished => {
                if !matches!(message, ProtocolMessage::ByeRis) {
                    return Err(ProtocolError::InvalidCommand);
                } else {
                    println!("Transitioning from Finished to Closed state.");
                }
            }
            _ => {
                return Err(ProtocolError::InvalidCommand);
            }
        }

        drop(guard); // Release the lock before awaiting

        println!("Executing command: {:?}", message);
        self.command_service
            .execute_protocol_command(Arc::clone(&self.transfer_state), &message)
            .await
            .map_err(|e| ProtocolError::CommandExecutionFailed(format!("{:?}", e)))
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
