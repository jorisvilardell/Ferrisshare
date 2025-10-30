use std::convert::TryFrom;

use crate::core::domain::storage::entities::YeetBlock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolMessage {
    Hello {
        // "HELLO <filename> <filesize>"
        filename: String,
        filesize: u64,
    },
    Ok,                  // "OK"
    Nope(String),        // "NOPE <reason>"
    Yeet(YeetBlock),     // "YEET <block_index> <block_size> <check_sum>"
    OkHousten(u64),      // "OK-HOUSTEN <block_index>"
    MissionAccomplished, // "MISSION-ACCOMPLISHED"
    Success,             // "SUCCESS"
    Error(String),       // "ERROR <reason>"
    ByeRis,              // "BYE-RIS"
}

#[derive(Debug)]
pub enum ProtocolError {
    InvalidUtf8,
    InvalidCommand,
    MissingArgs,
    InvalidNumber,
    Incomplete,
    CommandExecutionFailed(String),
}

impl TryFrom<&str> for ProtocolMessage {
    type Error = ProtocolError;

    fn try_from(value: &str) -> Result<Self, ProtocolError> {
        let line = value.trim();
        let tokens: Vec<&str> = line.split_whitespace().collect();

        match tokens.first().copied() {
            Some("HELLO") => {
                let filename = tokens.get(1).ok_or(ProtocolError::MissingArgs)?.to_string();
                let filesize = tokens
                    .get(2)
                    .ok_or(ProtocolError::MissingArgs)?
                    .parse::<u64>()
                    .map_err(|_| ProtocolError::InvalidNumber)?;
                Ok(ProtocolMessage::Hello { filename, filesize })
            }
            Some("OK") => Ok(ProtocolMessage::Ok),
            Some("NOPE") => {
                if tokens.len() < 2 {
                    return Err(ProtocolError::MissingArgs);
                }
                let reason = tokens[1..].join(" ");
                Ok(ProtocolMessage::Nope(reason))
            }
            Some("YEET") => {
                let block_index = tokens
                    .get(1)
                    .ok_or(ProtocolError::MissingArgs)?
                    .parse::<u64>()
                    .map_err(|_| ProtocolError::InvalidNumber)?;
                let block_size = tokens
                    .get(2)
                    .ok_or(ProtocolError::MissingArgs)?
                    .parse::<u32>()
                    .map_err(|_| ProtocolError::InvalidNumber)?;
                let check_sum = tokens
                    .get(3)
                    .ok_or(ProtocolError::MissingArgs)?
                    .parse::<u32>()
                    .map_err(|_| ProtocolError::InvalidNumber)?;

                Ok(ProtocolMessage::Yeet(YeetBlock::new(
                    block_index,
                    block_size,
                    check_sum,
                )))
            }
            Some("OK-HOUSTEN") => {
                let block_index = tokens
                    .get(1)
                    .ok_or(ProtocolError::MissingArgs)?
                    .parse::<u64>()
                    .map_err(|_| ProtocolError::InvalidNumber)?;
                Ok(ProtocolMessage::OkHousten(block_index))
            }
            Some("MISSION-ACCOMPLISHED") => Ok(ProtocolMessage::MissionAccomplished),
            Some("SUCCESS") => Ok(ProtocolMessage::Success),
            Some("ERROR") => {
                if tokens.len() < 2 {
                    return Err(ProtocolError::MissingArgs);
                }
                let reason = tokens[1..].join(" ");
                Ok(ProtocolMessage::Error(reason))
            }
            Some("BYE-RIS") => Ok(ProtocolMessage::ByeRis),
            _ => Err(ProtocolError::InvalidCommand),
        }
    }
}

impl From<ProtocolMessage> for String {
    fn from(msg: ProtocolMessage) -> Self {
        match msg {
            ProtocolMessage::Hello { filename, filesize } => {
                format!("HELLO {} {}", filename, filesize)
            }
            ProtocolMessage::Ok => "OK".to_string(),
            ProtocolMessage::Nope(reason) => format!("NOPE {}", reason),
            ProtocolMessage::Yeet(yeet_block) => format!(
                "YEET {} {} {}",
                yeet_block.index, yeet_block.size, yeet_block.checksum
            ),
            ProtocolMessage::OkHousten(block_index) => format!("OK-HOUSTEN {}", block_index),
            ProtocolMessage::MissionAccomplished => "MISSION-ACCOMPLISHED".to_string(),
            ProtocolMessage::Success => "SUCCESS".to_string(),
            ProtocolMessage::Error(reason) => format!("ERROR: {}", reason),
            ProtocolMessage::ByeRis => "BYE-RIS".to_string(),
        }
    }
}

impl From<ProtocolError> for String {
    fn from(err: ProtocolError) -> Self {
        match err {
            ProtocolError::InvalidUtf8 => "Invalid UTF-8 sequence".to_string(),
            ProtocolError::InvalidCommand => "Invalid command".to_string(),
            ProtocolError::MissingArgs => "Missing arguments".to_string(),
            ProtocolError::InvalidNumber => "Invalid number format".to_string(),
            ProtocolError::Incomplete => "Incomplete command".to_string(),
            ProtocolError::CommandExecutionFailed(msg) => {
                format!("Command execution failed: {}", msg)
            }
        }
    }
}

#[derive(Debug)]
pub enum NetworkError {
    ListenerBindFailed(std::io::Error),
    TransferInterrupted,
    TooManyConnections,
    ConnectionLost,
    Timeout,
    InvalidData,
    ProtocolError(ProtocolError),
}

impl From<ProtocolError> for NetworkError {
    fn from(err: ProtocolError) -> Self {
        NetworkError::ProtocolError(err)
    }
}

impl From<NetworkError> for String {
    fn from(err: NetworkError) -> Self {
        match err {
            NetworkError::ListenerBindFailed(e) => {
                format!("ERROR Listener bind failed: {}", e)
            }
            NetworkError::TransferInterrupted => "ERROR Transfer interrupted".to_string(),
            NetworkError::TooManyConnections => "ERROR Too many connections".to_string(),
            NetworkError::ConnectionLost => "ERROR Connection lost".to_string(),
            NetworkError::Timeout => "ERROR Timeout occurred".to_string(),
            NetworkError::InvalidData => "ERROR Invalid data received".to_string(),
            NetworkError::ProtocolError(proto_err) => String::from(proto_err),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TransferState {
    Idle,
    Receiving {
        current_file: String,
        expected_blocks: u64,
        focused_block: Option<YeetBlock>,
        received_blocks: Vec<u64>,
    },
    Finished,
    Closed,
}
