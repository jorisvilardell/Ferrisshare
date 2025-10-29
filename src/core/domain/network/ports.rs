use std::io::Error;

use crate::core::domain::network::entities::{NetworkError, ProtocolError, ProtocolMessage};
use tokio::{
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
};

pub trait NetworkService {
    fn listener(
        &self,
        addr: &str,
        tx: Sender<TcpStream>,
    ) -> impl Future<Output = Result<(), NetworkError>> + Send;
    fn handler(&self, rx: Receiver<TcpStream>) -> impl Future<Output = Result<(), Error>> + Send;
    fn send_message(
        &self,
        stream: &TcpStream,
        message: &ProtocolMessage,
    ) -> Result<(), ProtocolError>;
}

pub trait NetworkClient {
    fn connect(&self, addr: &str) -> Result<(), ProtocolError>;
    fn disconnect(&self) -> Result<(), ProtocolError>;
    fn receive_message(&self) -> Result<ProtocolMessage, ProtocolError>;
}
