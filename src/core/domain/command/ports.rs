use std::sync::Arc;

use crate::core::domain::{
    command::entities::CommandError,
    network::entities::{ProtocolMessage, TransferState},
};

pub trait CommandService: Send + Sync {
    fn execute_protocol_command(
        &self,
        state: Arc<tokio::sync::Mutex<TransferState>>,
        msg: &ProtocolMessage,
    ) -> impl Future<Output = Result<ProtocolMessage, CommandError>> + Send + Sync;
    fn process_binary_data(
        &self,
        state: Arc<tokio::sync::Mutex<TransferState>>,
        data: &[u8],
    ) -> impl Future<Output = Result<ProtocolMessage, CommandError>>;
}
