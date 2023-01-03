use std::{
    str::from_utf8,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::{
    consts::DIR_ENTRY_SIZE,
    utility::{le_bytes_to_u32, le_bytes_to_u64_padded},
    DirEntry, FatEntry,
};

#[derive(Debug)]
pub enum DirectoryEntry {
    Invalid,

    LongFileName(String),

    Directory(Inode),
    File(Inode),
}

impl DirectoryEntry {
    pub fn split(&self) -> Vec<DirEntry> {
        todo!()

        // let mut vec = vec![];

        // let inode = match self {
        //     DirectoryEntry::Invalid => None,
        //     DirectoryEntry::LongFileName(_) => None,
        //     DirectoryEntry::Directory(i) => Some(i),
        //     DirectoryEntry::File(i) => Some(i),
        // };

        // match inode {
        //     Some(inode) => {
        //         todo!()
        //     }
        //     None => (),
        // };

        // vec
    }
}

impl From<&DirEntry> for DirectoryEntry {
    fn from(raw_entry: &DirEntry) -> Self {
        let type_indicator = le_bytes_to_u32(raw_entry);

        if (0b1 << 0) & type_indicator == 0 {
            DirectoryEntry::Invalid
        } else if (0b1 << 1) & type_indicator != 0 {
            DirectoryEntry::LongFileName(from_utf8(&raw_entry[1..]).unwrap().to_owned())
        } else if (0b1 << 2) & type_indicator != 0 {
            DirectoryEntry::Directory(Inode::from(raw_entry))
        } else {
            DirectoryEntry::File(Inode::from(raw_entry))
        }
    }
}

impl From<&DirectoryEntry> for DirEntry {
    fn from(value: &DirectoryEntry) -> Self {
        let mut raw = [0u8; DIR_ENTRY_SIZE as usize];

        match value {
            DirectoryEntry::Invalid => (),
            DirectoryEntry::LongFileName(file_name) => {
                raw[0] |= 0b1 << 0;
                raw[0] |= 0b1 << 1;
                let name_bytes = file_name.as_bytes();
                raw[1..1 + name_bytes.len()].copy_from_slice(name_bytes);
            }
            DirectoryEntry::Directory(inode) => {
                raw = DirEntry::from(inode);
                raw[0] |= 0b1 << 0;
                raw[0] |= 0b1 << 2;
            }
            DirectoryEntry::File(inode) => {
                raw = DirEntry::from(inode);
                raw[0] |= 0b1 << 0;
            }
        }

        raw
    }
}

#[derive(Debug)]
pub struct Inode {
    pub name: String,
    pub length: u64,
    pub uid: u32,
    pub gid: u32,
    pub permission: u16,
    pub ctime: SystemTime,
    pub mtime: SystemTime,
    pub atime: SystemTime,
    pub number_of_hlinks: u8,
    pub start_cluster: FatEntry,
}

impl Inode {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        length: u64,
        uid: u32,
        gid: u32,
        permission: u16,
        ctime: SystemTime,
        mtime: SystemTime,
        atime: SystemTime,
        number_of_hlinks: u8,
        start_cluster: FatEntry,
    ) -> Self {
        Inode {
            name,
            length,
            uid,
            gid,
            permission,
            ctime,
            mtime,
            atime,
            number_of_hlinks,
            start_cluster,
        }
    }
}

impl From<&Inode> for DirEntry {
    fn from(value: &Inode) -> Self {
        let mut raw = [0u8; DIR_ENTRY_SIZE as usize];

        raw[0] |= (0x0F & value.number_of_hlinks) << 4;
        raw[1..5].copy_from_slice(&value.uid.to_le_bytes());
        raw[5..9].copy_from_slice(&value.gid.to_le_bytes());

        raw[9..15].copy_from_slice(
            &value
                .ctime
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_le_bytes()[0..6],
        );

        raw[15..21].copy_from_slice(
            &value
                .mtime
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_le_bytes()[0..6],
        );

        raw[21..27].copy_from_slice(
            &value
                .atime
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_le_bytes()[0..6],
        );
        raw[27..29].copy_from_slice(&value.permission.to_le_bytes());
        raw[29..33].copy_from_slice(&value.start_cluster.to_le_bytes());
        raw[33..39].copy_from_slice(&(value.length.to_le_bytes()[0..6]));

        let name_bytes = value.name.as_bytes();
        raw[39..39 + name_bytes.len()].copy_from_slice(name_bytes);

        raw
    }
}

impl From<&DirEntry> for Inode {
    fn from(value: &DirEntry) -> Self {
        let number_of_hlinks = (value[0] & 0xF0) >> 4;
        let uid = le_bytes_to_u64_padded(&value[1..5]) as u32;
        let gid = le_bytes_to_u64_padded(&value[5..9]) as u32;
        let ctime = UNIX_EPOCH + Duration::from_millis(le_bytes_to_u64_padded(&value[9..15]));
        let mtime = UNIX_EPOCH + Duration::from_millis(le_bytes_to_u64_padded(&value[15..21]));
        let atime = UNIX_EPOCH + Duration::from_millis(le_bytes_to_u64_padded(&value[21..27]));
        let permission = le_bytes_to_u64_padded(&value[27..29]) as u16;
        let start_cluster = le_bytes_to_u64_padded(&value[29..33]) as FatEntry;
        let length = le_bytes_to_u64_padded(&value[33..39]);
        let name = from_utf8(&value[39..64])
            .unwrap()
            .trim_matches('\0')
            .to_owned();

        Inode {
            name,
            length,
            uid,
            gid,
            permission,
            ctime,
            mtime,
            atime,
            number_of_hlinks,
            start_cluster,
        }
    }
}
