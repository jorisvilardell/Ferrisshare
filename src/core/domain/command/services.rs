use std::sync::Arc;

use crate::core::domain::{
    command::{entities::CommandError, ports::CommandService},
    network::entities::{ProtocolMessage, TransferState},
    storage::ports::StorageRepository,
};

#[derive(Clone)]
pub struct CommandServiceImpl<C>
where
    C: StorageRepository,
{
    storage: C,
}

impl<C> CommandServiceImpl<C>
where
    C: StorageRepository + Clone + Send + Sync + 'static,
{
    pub fn new(storage: C) -> Self {
        CommandServiceImpl { storage }
    }
}

impl<C> CommandService for CommandServiceImpl<C>
where
    C: StorageRepository + Clone + Send + Sync + 'static,
{
    async fn execute_protocol_command(
        &self,
        state: Arc<tokio::sync::Mutex<TransferState>>,
        msg: &ProtocolMessage,
    ) -> Result<ProtocolMessage, CommandError> {
        match msg {
            ProtocolMessage::Hello {
                filename: _filename,
                filesize,
            } => {
                println!("Execute HELLO command.");
                let expected_blocks = (*filesize + 1023) / 1024;
                let mut state_guard = state.lock().await;
                println!(
                    "Setting state to Receiving with expected_blocks={}",
                    expected_blocks
                );
                *state_guard = TransferState::Receiving {
                    current_file: _filename.clone(),
                    expected_blocks,
                    focused_block: None,
                    received_blocks: Vec::with_capacity(expected_blocks as usize),
                };

                drop(state_guard);

                Ok(ProtocolMessage::Ok)
            }
            ProtocolMessage::Yeet(yeet_block) => {
                let mut state_guard = state.lock().await;
                let (expected_blocks, focused_block, received_blocks, current_file) =
                    match &mut *state_guard {
                        TransferState::Receiving {
                            expected_blocks,
                            focused_block,
                            received_blocks,
                            current_file,
                        } => (
                            expected_blocks,
                            focused_block,
                            received_blocks,
                            current_file,
                        ),
                        _ => {
                            return Err(CommandError::ExecutionFailed(
                                "Error transfer state is not equal Receiving".to_string(),
                            ));
                        }
                    };

                // Ensure we don't exceed the expected number of blocks.
                if received_blocks.len() >= *expected_blocks as usize {
                    println!("Received all expected blocks.");
                    return Err(CommandError::ExecutionFailed(
                        "Received block index exceeds expected blocks".to_string(),
                    ));
                }

                if let Some(focused_block) = focused_block
                    && !received_blocks.contains(&focused_block.index)
                {
                    println!("Received expected block index: {}", focused_block.index);
                    return Err(CommandError::ExecutionFailed(
                        "Block not received. Can't proceed with next block.".to_string(),
                    ));
                }

                // Reuse the mutable guard to update the state without locking again.
                *state_guard = TransferState::Receiving {
                    current_file: current_file.clone(),
                    expected_blocks: *expected_blocks,
                    focused_block: Some(yeet_block.clone()),
                    received_blocks: received_blocks.clone(),
                };

                drop(state_guard);

                Ok(ProtocolMessage::Yeet(yeet_block.clone()))
            }
            ProtocolMessage::MissionAccomplished => {
                let mut state_guard = state.lock().await;
                let current_file = match &*state_guard {
                    TransferState::Receiving { current_file, .. } => current_file,
                    _ => {
                        return Err(CommandError::ExecutionFailed(
                            "Error transfer state is not equal Receiving".to_string(),
                        ));
                    }
                };

                self.storage.finalize(current_file).await.map_err(|e| {
                    CommandError::ExecutionFailed(format!("Storage error: {:?}", e))
                })?;
                *state_guard = TransferState::Finished;
                drop(state_guard);
                Ok(ProtocolMessage::Success)
            }
            ProtocolMessage::ByeRis => {
                *state.lock().await = TransferState::Closed;
                Ok(ProtocolMessage::ByeRis)
            }
            _ => Err(CommandError::InvalidCommand),
        }
    }

    async fn process_binary_data(
        &self,
        state: Arc<tokio::sync::Mutex<TransferState>>,
        data: &[u8],
    ) -> Result<ProtocolMessage, CommandError> {
        // Lock once and extract what we need.
        let mut state_guard = state.lock().await;

        let (
            mut expected_blocks_val,
            maybe_focused_block,
            received_blocks_clone,
            current_file_clone,
        ) = match &mut *state_guard {
            TransferState::Receiving {
                expected_blocks,
                focused_block,
                received_blocks,
                current_file,
            } => {
                // take the focused block out (leaves None in the guard)
                let taken_block = focused_block.take();
                (
                    *expected_blocks,
                    taken_block,
                    received_blocks.clone(),
                    current_file.clone(),
                )
            }
            _ => {
                return Err(CommandError::ExecutionFailed(
                    "Error transfer state is not equal Receiving".to_string(),
                ));
            }
        };

        // If there was no focused block, nothing to do.
        let focused_block = match maybe_focused_block {
            Some(b) => b,
            None => {
                println!("No focused block to store data for.");
                return Ok(ProtocolMessage::Ok);
            }
        };

        // If block already received, restore focused_block into the state and return.
        if received_blocks_clone.contains(&focused_block.index) {
            // restore focused_block back into the guard before returning
            if let TransferState::Receiving {
                focused_block: guard_focused_block,
                ..
            } = &mut *state_guard
            {
                *guard_focused_block = Some(focused_block.clone());
            }
            println!("Block {} already received, ignoring.", focused_block.index);
            return Ok(ProtocolMessage::Ok);
        }

        println!("Stored binary data block: {:?}", focused_block);

        // Clone what we need for the async storage write, then drop the guard before awaiting.
        let file_for_write = current_file_clone.clone();
        let block_for_write = focused_block.clone();
        drop(state_guard);

        // Perform the async write while not holding the mutex.
        self.storage
            .write_block(&file_for_write, &block_for_write, data)
            .await
            .map_err(|e| CommandError::ExecutionFailed(format!("Storage error: {:?}", e)))?;

        // Re-lock and update received_blocks + clear focused_block.
        let mut state_guard = state.lock().await;
        match &mut *state_guard {
            TransferState::Receiving {
                received_blocks,
                focused_block,
                ..
            } => {
                received_blocks.push(block_for_write.index);
                *focused_block = None;
            }
            _ => {
                return Err(CommandError::ExecutionFailed(
                    "Transfer state changed while writing block".to_string(),
                ));
            }
        };

        Ok(ProtocolMessage::OkHousten(block_for_write.index))
    }
}
