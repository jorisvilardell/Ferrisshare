#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct YeetBlock {
    pub index: u64,
    pub size: u32,
    pub checksum: u32,
}

impl YeetBlock {
    pub fn new(index: u64, size: u32, checksum: u32) -> Self {
        YeetBlock {
            index,
            size,
            checksum,
        }
    }
}

pub struct File {
    pub id: u64,
    pub name: String,
    pub size: u64,
}

#[derive(Debug)]
pub enum StorageError {
    FileNotFound,
    PermissionDenied,
    AlreadyExists,
    AbsolutePathNotAllowed,
    ParentDirSegmentNotAllowed,
    InvalidFilename,
    Unknown(String),
}

impl From<StorageError> for String {
    fn from(error: StorageError) -> Self {
        match error {
            StorageError::FileNotFound => "File not found".into(),
            StorageError::PermissionDenied => "Permission denied".into(),
            StorageError::AlreadyExists => "File already exists".into(),
            StorageError::AbsolutePathNotAllowed => {
                "Absolute paths are not allowed in filenames".into()
            }
            StorageError::ParentDirSegmentNotAllowed => {
                "Parent directory segments are not allowed in filenames".into()
            }
            StorageError::InvalidFilename => "Invalid filename".into(),
            StorageError::Unknown(msg) => format!("Unknown storage error: {}", msg),
        }
    }
}
