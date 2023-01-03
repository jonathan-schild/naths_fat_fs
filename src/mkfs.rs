use std::{
    io::{Read, Seek, SeekFrom, Write},
    time::SystemTime,
};

use crate::{
    consts::{
        CLUSTER_SIZE, DATA_REGION, DIR_ENTRY_SIZE, EOC, FAT_ENTRY_SIZE, FAT_PADDING, FS_ID,
        FS_VERSION,
    },
    fs::{
        basic_fs_io::BaseIO,
        directory::{DirectoryEntry, Inode},
        FileSystem,
    },
    utility::fs_utility::{
        get_data_region_size, get_data_section_address, get_prelude_padding_size,
    },
    DirEntry,
};

pub fn write_prelude<W: Write + Seek>(fat_size: u32, dest: &mut W) {
    dest.rewind().unwrap();

    dest.write_all(&FS_ID).unwrap();
    dest.write_all(&FS_VERSION).unwrap();
    dest.write_all(&fat_size.to_le_bytes()).unwrap();

    dest.write_all(&[FAT_PADDING; 18]).unwrap();

    let fat_bytes = FAT_ENTRY_SIZE * fat_size;

    for _ in 0..fat_bytes {
        dest.write_all(&[0u8]).unwrap();
    }

    for _ in 0..get_prelude_padding_size(fat_size) {
        dest.write_all(&[FAT_PADDING]).unwrap();
    }
}

pub fn write_data_section<W: Write + Seek>(fat_size: u32, dest: &mut W) {
    dest.seek(SeekFrom::Start(get_data_section_address(fat_size)))
        .unwrap();

    for _ in 0..get_data_region_size(fat_size) {
        dest.write_all(&[DATA_REGION]).unwrap();
    }
}

pub fn write_root_dir<T: Read + Write + Seek>(fs: &mut FileSystem<T>) {
    fs.write_fat_entry(1, EOC);

    for i in 0..(CLUSTER_SIZE / DIR_ENTRY_SIZE) {
        fs.write_raw_directory_entry(1, i, &DirEntry::from(&DirectoryEntry::Invalid));
    }

    fs.write_raw_directory_entry(
        1,
        0,
        &DirEntry::from(&DirectoryEntry::Directory(Inode::new(
            String::from("."),
            64,
            0,
            0,
            0o755,
            SystemTime::now(),
            SystemTime::now(),
            SystemTime::now(),
            2,
            1,
        ))),
    );

    fs.write_raw_directory_entry(
        1,
        1,
        &DirEntry::from(&DirectoryEntry::Directory(Inode::new(
            String::from(".."),
            64,
            0,
            0,
            0o755,
            SystemTime::now(),
            SystemTime::now(),
            SystemTime::now(),
            2,
            1,
        ))),
    );
}
