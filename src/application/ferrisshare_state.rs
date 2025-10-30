use crate::core::domain::{command::ports::CommandService, network::ports::NetworkService};

#[derive(Clone, Copy)]
pub struct FerrisShareState<C, N>
where
    C: CommandService,
    N: NetworkService,
{
    pub command_service: C,
    pub network_service: N,
}

impl<C, N> FerrisShareState<C, N>
where
    C: CommandService + Clone + Send + Sync + 'static,
    N: NetworkService + Clone + Send + Sync + 'static,
{
    pub fn new(command_service: C, network_service: N) -> Self {
        FerrisShareState {
            command_service,
            network_service,
        }
    }
}
