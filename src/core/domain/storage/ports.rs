use crate::core::domain::storage::entities::{StorageError, YeetBlock};

pub trait StorageRepository {
    fn open_file(&self, filename: &str) -> impl Future<Output = Result<(), StorageError>> + Send;
    fn write_block(
        &self,
        filename: &str,
        block: &YeetBlock,
        data: &[u8],
    ) -> impl Future<Output = Result<(), StorageError>> + Send;
    fn finalize(&self, filename: &str) -> impl Future<Output = Result<(), StorageError>> + Send;
}
