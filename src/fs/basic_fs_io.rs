use std::{
    io::{Read, Seek, SeekFrom, Write},
    str::from_utf8,
};

use crate::{
    consts::{CLUSTER_SIZE, DIR_ENTRY_SIZE, FAT_ENTRY_SIZE, FAT_START_ADDR, FS_ID, FS_VERSION},
    utility::{
        fs_utility::{check_cluster, get_data_section_address},
        le_bytes_to_u32,
    },
    Cluster, DirEntry, FatEntry,
};

pub trait BaseIO {
    fn read_fat_entry(&mut self, cluster: FatEntry) -> FatEntry;
    fn write_fat_entry(&mut self, cluster: FatEntry, entry: FatEntry);
    fn read_cluster(&mut self, cluster: FatEntry) -> Cluster;
    fn write_cluster(&mut self, cluster: FatEntry, cluster_content: &Cluster);
    fn read_raw_directory_entry(&mut self, cluster: FatEntry, idx: u32) -> DirEntry;
    fn write_raw_directory_entry(&mut self, cluster: FatEntry, idx: u32, entry: &DirEntry);
}

pub struct FileSystemBasicIO<'a, T>
where
    T: Read + Seek + Write,
{
    pub device: &'a mut T,
    pub fat_length: FatEntry,
    pub start_data_region: u64,
}
impl<'a, T> FileSystemBasicIO<'a, T>
where
    T: Read + Seek + Write,
{
    pub fn open_file_system(device: &'a mut T) -> Self
    where
        T: Read + Write + Seek,
    {
        device.rewind().unwrap();

        let mut fat_prelude_buffer = [0u8; 14];

        device.read_exact(&mut fat_prelude_buffer).unwrap();

        if fat_prelude_buffer[0..=8] != FS_ID {
            panic!(
                "invalid filesystem: {}",
                from_utf8(&fat_prelude_buffer[0..=8]).unwrap()
            );
        }

        if fat_prelude_buffer[9..10] != FS_VERSION {
            panic!("invalid filesystem version {}", fat_prelude_buffer[9]);
        }

        let fat_length = le_bytes_to_u32(&fat_prelude_buffer[10..14]);

        let start_data_region = get_data_section_address(fat_length);

        FileSystemBasicIO {
            device,
            fat_length,
            start_data_region,
        }
    }
}

impl<'a, T> BaseIO for FileSystemBasicIO<'a, T>
where
    T: Read + Seek + Write,
{
    fn read_fat_entry(&mut self, cluster: FatEntry) -> FatEntry {
        if let Err(s) = check_cluster(self.fat_length, cluster) {
            panic!("{}", s)
        }

        let addr = (cluster - 1) * FAT_ENTRY_SIZE;

        let mut buf = [0u8; 4];

        self.device
            .seek(SeekFrom::Start(FAT_START_ADDR + addr as u64))
            .unwrap();

        self.device.read_exact(&mut buf).unwrap();

        le_bytes_to_u32(&buf)
    }

    fn write_fat_entry(&mut self, cluster: FatEntry, entry: FatEntry) {
        if let Err(s) = check_cluster(self.fat_length, cluster) {
            panic!("{}", s)
        }

        let addr = (cluster - 1) * FAT_ENTRY_SIZE;

        self.device
            .seek(SeekFrom::Start(FAT_START_ADDR + addr as u64))
            .unwrap();

        self.device.write_all(&entry.to_le_bytes()).unwrap();
    }

    fn read_cluster(&mut self, cluster: FatEntry) -> Cluster {
        let mut cluster_content = [0u8; CLUSTER_SIZE as usize];

        if let Err(s) = check_cluster(self.fat_length, cluster) {
            panic!("{}", s)
        }

        let addr = ((cluster - 1) * CLUSTER_SIZE) as u64;

        self.device
            .seek(SeekFrom::Start(self.start_data_region + addr))
            .unwrap();

        self.device.read_exact(&mut cluster_content).unwrap();

        cluster_content
    }

    fn write_cluster(&mut self, cluster: FatEntry, cluster_content: &Cluster) {
        if let Err(s) = check_cluster(self.fat_length, cluster) {
            panic!("{}", s)
        }

        let addr = ((cluster - 1) * CLUSTER_SIZE) as u64;

        self.device
            .seek(SeekFrom::Start(self.start_data_region + addr))
            .unwrap();

        self.device.write_all(cluster_content).unwrap();
    }

    fn read_raw_directory_entry(&mut self, cluster: FatEntry, idx: u32) -> DirEntry {
        if let Err(s) = check_cluster(self.fat_length, cluster) {
            panic!("{}", s)
        }

        let addr = ((cluster - 1) * CLUSTER_SIZE) as u64;
        let offset = (idx * DIR_ENTRY_SIZE) as u64;

        let mut buf = [0u8; DIR_ENTRY_SIZE as usize];

        self.device
            .seek(SeekFrom::Start(self.start_data_region + addr + offset))
            .unwrap();

        self.device.read_exact(&mut buf).unwrap();

        buf
    }

    fn write_raw_directory_entry(&mut self, cluster: FatEntry, idx: u32, entry: &DirEntry) {
        if let Err(s) = check_cluster(self.fat_length, cluster) {
            panic!("{}", s)
        }

        let addr = ((cluster - 1) * CLUSTER_SIZE) as u64;
        let offset = (idx * DIR_ENTRY_SIZE) as u64;

        self.device
            .seek(SeekFrom::Start(self.start_data_region + addr + offset))
            .unwrap();

        self.device.write_all(entry).unwrap();
    }
}
