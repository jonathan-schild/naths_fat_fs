use crate::FatEntry;

pub const FS_ID: [u8; 9] = *b"NathFATfs";
pub const FS_VERSION: [u8; 1] = [1u8];

pub const FAT_ENTRY_SIZE: u32 = 4;
pub const DIR_ENTRY_SIZE: u32 = 64;
pub const CLUSTER_SIZE: u32 = 1024; // 8192;

pub const FAT_START_ADDR: u64 = 32;
pub const ALIGNMENT: u32 = 32;

pub const FAT_PADDING: u8 = 0xAAu8;
pub const DATA_REGION: u8 = 0xDDu8;

pub const EOC: FatEntry = 0xFFFF_FFFF; // end of chain
pub const DNA: FatEntry = 0xFFFF_FFFE; // do not allocate
pub const FRE: FatEntry = 0x0000_0000; // free
