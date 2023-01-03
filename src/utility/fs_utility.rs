use crate::{
    consts::{
        ALIGNMENT, CLUSTER_SIZE, DIR_ENTRY_SIZE, DNA, EOC, FAT_ENTRY_SIZE, FAT_START_ADDR, FRE,
    },
    FatEntry,
};

pub fn get_prelude_padding_size(fat_size: FatEntry) -> u64 {
    (ALIGNMENT - (fat_size * FAT_ENTRY_SIZE % ALIGNMENT)) as u64
}

pub fn get_data_section_address(fat_size: FatEntry) -> u64 {
    let used_size = FAT_START_ADDR + (fat_size * FAT_ENTRY_SIZE) as u64;
    used_size + get_prelude_padding_size(fat_size)
}

pub fn get_data_region_size(fat_size: FatEntry) -> u64 {
    (CLUSTER_SIZE * fat_size) as u64
}

pub fn check_cluster(fat_length: FatEntry, cluster: FatEntry) -> Result<(), String> {
    if cluster == FRE {
        Err("unallocated cluster".to_string())
    } else if cluster == DNA {
        Err("bad block".to_string())
    } else if cluster == EOC {
        Err("end of chain".to_string())
    } else if cluster > fat_length {
        Err(format!("fat out of bounds {:#010X}", cluster))
    } else {
        Ok(())
    }
}

pub fn to_inode(cluster: FatEntry, index: u32) -> u64 {
    (cluster - 1) as u64 * (CLUSTER_SIZE / DIR_ENTRY_SIZE) as u64 + index as u64 + 1
}

pub fn from_inode(mut inode: u64) -> (FatEntry, u32) {
    inode -= 1;

    let cluster = (inode / (CLUSTER_SIZE / DIR_ENTRY_SIZE) as u64) + 1;
    let index = inode % (CLUSTER_SIZE / DIR_ENTRY_SIZE) as u64;

    (cluster as FatEntry, index as u32)
}
