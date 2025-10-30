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

pub enum StorageError {
    FileNotFound,
    PermissionDenied,
    AlreadyExists,
    AbsolutePathNotAllowed,
    ParentDirSegmentNotAllowed,
    InvalidFilename,
    Unknown(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::FileNotFound => write!(f, "File not found"),
            StorageError::PermissionDenied => write!(f, "Permission denied"),
            StorageError::AlreadyExists => write!(f, "File already exists"),
            StorageError::AbsolutePathNotAllowed => {
                write!(f, "Absolute paths are not allowed in filenames")
            }
            StorageError::ParentDirSegmentNotAllowed => {
                write!(f, "Parent directory segments are not allowed in filenames")
            }
            StorageError::InvalidFilename => write!(f, "Invalid filename"),
            StorageError::Unknown(msg) => write!(f, "Unknown storage error: {}", msg),
        }
    }
}
