use dotenv::dotenv;
use std::sync::Arc;

use tokio::{net::TcpStream, sync::mpsc};

use ferrisshare::{
    application::config::Config,
    core::domain::{
        command::services::CommandServiceImpl,
        network::{ports::NetworkService as _, services::NetworkServiceImpl},
    },
    infra::repositories::fs::fs_storage_repository::FSStorageRepository,
};

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    dotenv().ok();
    let cfg: Config = Config::from_env();

    let (tx, rx) = mpsc::channel::<TcpStream>(1);

    let storage_repo = FSStorageRepository::new(cfg.ferris_base_path);

    let command_service = CommandServiceImpl::new(storage_repo);
    let network_service = NetworkServiceImpl::new(command_service);

    let ferrisshare_state = Arc::new(
        ferrisshare::application::ferrisshare_state::FerrisShareState::new(
            network_service.command_service.clone(),
            network_service.clone(),
        ),
    );

    let ferrisshare_state_clone = ferrisshare_state.clone();

    tokio::spawn(async move {
        if let Err(e) = ferrisshare_state_clone.network_service.handler(rx).await {
            eprintln!("Handler error: {}", e);
        }
    });

    if let Err(e) = ferrisshare_state
        .network_service
        .listener(
            format!("{}:{}", cfg.ferris_host, cfg.ferris_port).as_str(),
            tx,
        )
        .await
    {
        eprintln!("Listener error: {:?}", e);
    }
    Ok(())
}
