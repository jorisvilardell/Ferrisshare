#[derive(Debug)]
pub enum CommandError {
    InvalidCommand,
    ExecutionFailed(String),
}
