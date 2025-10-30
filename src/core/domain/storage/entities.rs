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
