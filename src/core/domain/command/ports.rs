use crate::core::domain::{command::entities::CommandError, network::entities::ProtocolMessage};

pub trait CommandService: Send + Sync {
    fn execute_protocol_command(
        &self,
        msg: &ProtocolMessage,
    ) -> impl Future<Output = Result<ProtocolMessage, CommandError>> + Send;
}
