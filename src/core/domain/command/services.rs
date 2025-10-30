use std::sync::Arc;

use crate::core::domain::{
    command::{entities::CommandError, ports::CommandService},
    network::entities::{ProtocolMessage, TransferState},
};

#[derive(Clone)]
pub struct CommandServiceImpl {}

impl CommandServiceImpl {
    pub fn new() -> Self {
        CommandServiceImpl {}
    }
}

impl CommandService for CommandServiceImpl {
    async fn execute_protocol_command(
        &self,
        state: Arc<tokio::sync::Mutex<TransferState>>,
        msg: &ProtocolMessage,
    ) -> Result<ProtocolMessage, CommandError> {
        match msg {
            ProtocolMessage::Hello { filename, filesize } => {
                // Démarrer la logique de préparation de réception
                Ok(ProtocolMessage::Ok)
            }
            ProtocolMessage::Yeet {
                block_index,
                block_size,
                check_sum,
            } => {
                // Logique pour gérer la réception d'un bloc de données
                Ok(ProtocolMessage::OkHousten(block_index.clone()))
            }
            // ... autres commandes
            _ => Err(CommandError::InvalidCommand),
        }
    }
}
