use consts::{CLUSTER_SIZE, DIR_ENTRY_SIZE};
use fs::directory::DirectoryEntry;

pub mod consts;
pub mod fs;
pub mod mkfs;
pub mod utility;

pub type FatEntry = u32;
pub type DirEntry = [u8; DIR_ENTRY_SIZE as usize];
pub type Cluster = [u8; CLUSTER_SIZE as usize];
pub type Chain = Vec<FatEntry>;
pub type Dir = Vec<(DirectoryEntry, FatEntry, u32)>;
