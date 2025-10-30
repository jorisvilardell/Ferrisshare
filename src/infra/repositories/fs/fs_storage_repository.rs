use std::future::Future;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

use crate::core::domain::storage::{
    entities::{StorageError, YeetBlock},
    ports::StorageRepository,
};

#[derive(Clone)]
pub struct FSStorageRepository {
    base_path: String,
}

impl FSStorageRepository {
    pub fn new(base_path: String) -> Self {
        FSStorageRepository { base_path }
    }

    // helper pour sécuriser le filename (simple)
    fn sanitize_filename(filename: &str) -> Result<(), StorageError> {
        let p = Path::new(filename);
        // refuse les chemins absolus ou qui remontent (..).
        if p.is_absolute() {
            return Err(StorageError::AbsolutePathNotAllowed);
        }
        if p.components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(StorageError::ParentDirSegmentNotAllowed);
        }
        // Optionnel: s'assurer qu'on a bien un file name (pas "dir/")
        if p.file_name().is_none() {
            return Err(StorageError::InvalidFilename);
        }
        Ok(())
    }

    fn file_path_for(&self, filename: &str) -> PathBuf {
        PathBuf::from(&self.base_path).join(filename)
    }
}

impl StorageRepository for FSStorageRepository {
    fn open_file(&self, filename: &str) -> impl Future<Output = Result<(), StorageError>> + Send {
        let filename = filename.to_string();

        async move {
            // sanitize
            if let Err(e) = FSStorageRepository::sanitize_filename(&filename) {
                return Err(e);
            }

            let path = self.file_path_for(&filename);
            // Use a temporary extension during transfer
            let part_path = path.with_extension("ferrisshare");

            // create parent dirs if needed
            if let Some(parent) = path.parent() {
                if let Err(e) = tokio::fs::create_dir_all(parent).await {
                    return Err(StorageError::Unknown(format!(
                        "Failed to create dir: {}",
                        e
                    )));
                }
            }

            match tokio::fs::File::create(&part_path).await {
                Ok(_) => Ok(()),
                Err(e) => Err(StorageError::Unknown(e.to_string())),
            }
        }
    }

    fn write_block(
        &self,
        filename: &str,
        block: &YeetBlock,
        data: &[u8],
    ) -> impl Future<Output = Result<(), StorageError>> + Send {
        let filename = filename.to_string();
        let data = data.to_vec();
        let block_index = block.index;

        async move {
            // sanitize
            if let Err(e) = FSStorageRepository::sanitize_filename(&filename) {
                return Err(e);
            }

            let path = self.file_path_for(&filename);
            // write into a .ferrisshare temporary file while transferring
            let part_path = path.with_extension("ferrisshare");

            // ensure parent dir exists before open
            if let Some(parent) = path.parent() {
                if let Err(e) = tokio::fs::create_dir_all(parent).await {
                    return Err(StorageError::Unknown(format!(
                        "Failed to create dir: {}",
                        e
                    )));
                }
            }

            match tokio::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(&part_path)
                .await
            {
                Ok(mut file) => {
                    let offset = block_index * block.size as u64;
                    if let Err(e) = file.seek(std::io::SeekFrom::Start(offset)).await {
                        return Err(StorageError::Unknown(e.to_string()));
                    }
                    if let Err(e) = file.write_all(&data).await {
                        return Err(StorageError::Unknown(e.to_string()));
                    }
                    // flush si tu veux (optionnel)
                    if let Err(e) = file.flush().await {
                        return Err(StorageError::Unknown(e.to_string()));
                    }
                    Ok(())
                }
                Err(e) => Err(StorageError::Unknown(e.to_string())),
            }
        }
    }

    fn finalize(&self, filename: &str) -> impl Future<Output = Result<(), StorageError>> + Send {
        let base = self.base_path.clone();
        let filename = filename.to_string();

        async move {
            // sanitize
            if let Err(e) = FSStorageRepository::sanitize_filename(&filename) {
                return Err(e);
            }

            let path = PathBuf::from(&base).join(&filename);
            // Rename the .ferrisshare temp file to the final filename
            let part = path.with_extension("ferrisshare");
            match tokio::fs::rename(&part, &path).await {
                Ok(_) => Ok(()),
                Err(e) => Err(StorageError::Unknown(e.to_string())),
            }
        }
    }
}
