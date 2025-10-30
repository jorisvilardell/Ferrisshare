#[derive(Debug)]
pub enum CommandError {
    InvalidCommand,
    ExecutionFailed(String),
}

impl From<CommandError> for String {
    fn from(err: CommandError) -> Self {
        match err {
            CommandError::InvalidCommand => "Invalid command".to_string(),
            CommandError::ExecutionFailed(msg) => format!("Command execution failed: {}", msg),
        }
    }
}
