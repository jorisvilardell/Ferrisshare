use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolMessage {
    Hello {
        // "HELLO <filename> <filesize>"
        filename: String,
        filesize: u64,
    },
    Ok,           // "OK"
    Nope(String), // "NOPE <reason>"
    Yeet {
        // "YEET <block_index> <block_size> <check_sum>"
        block_index: u64,
        block_size: u32,
        check_sum: u32,
    },
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

                Ok(ProtocolMessage::Yeet {
                    block_index,
                    block_size,
                    check_sum,
                })
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
            _ => return Err(ProtocolError::InvalidCommand),
        }
    }
}
