use std::sync::Arc;

use tokio::{net::TcpStream, sync::mpsc};

use ferrisshare::core::domain::{
    command::services::CommandServiceImpl,
    network::{ports::NetworkService as _, services::NetworkServiceImpl},
};

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let (tx, rx) = mpsc::channel::<TcpStream>(1);

    let command_service = CommandServiceImpl::new();
    let network_service = Arc::new(NetworkServiceImpl::new(command_service));

    let handler_service = Arc::clone(&network_service);
    tokio::spawn(async move {
        if let Err(e) = handler_service.handler(rx).await {
            eprintln!("Handler error: {}", e);
        }
    });

    if let Err(e) = network_service.listener("127.0.0.1:9000", tx).await {
        eprintln!("Listener error: {:?}", e);
    }
    Ok(())
}
