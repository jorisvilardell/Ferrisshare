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
    ) -> impl Future<Output = Result<ProtocolMessage, CommandError>> + Send;
}
