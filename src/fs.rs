use std::io::{Read, Seek, Write};

use crate::{
    consts::{CLUSTER_SIZE, DIR_ENTRY_SIZE, EOC, FRE},
    Chain, Dir, DirEntry, FatEntry,
};

use self::{
    basic_fs_io::{BaseIO, FileSystemBasicIO},
    directory::DirectoryEntry,
};

pub mod basic_fs_io;
pub mod directory;
pub mod filesystem;

pub struct FileSystem<'a, T>
where
    T: Read + Seek + Write,
{
    pub io: FileSystemBasicIO<'a, T>,
}

impl<'a, T> FileSystem<'a, T>
where
    T: Read + Seek + Write,
{
    pub fn alloc_chunk(&mut self) -> FatEntry {
        let mut next = 1;

        while self.read_fat_entry(next) != FRE {
            next += 1;
        }

        self.write_fat_entry(next, EOC);

        next
    }

    pub fn get_chain(&mut self, mut cluster: FatEntry) -> Chain {
        let mut vec = vec![];

        while cluster != EOC {
            vec.push(cluster);

            cluster = self.read_fat_entry(cluster);
        }

        vec
    }

    pub fn append_to_chain(&mut self, chain: &mut Chain) -> FatEntry {
        let new = self.alloc_chunk();

        if let Some(last) = chain.last() {
            self.write_fat_entry(*last, new)
        }

        chain.push(new);

        new
    }

    pub fn append_dir_to_chain(&mut self, chain: &mut Chain) -> FatEntry {
        let new = self.append_to_chain(chain);

        for i in 0..(CLUSTER_SIZE / DIR_ENTRY_SIZE) {
            self.write_raw_directory_entry(new, i, &DirEntry::from(&DirectoryEntry::Invalid));
        }

        new
    }

    pub fn read_dir(&mut self, chain: &Chain) -> Dir {
        let mut dir = vec![];

        let mut filename = String::new();

        for i in chain {
            for j in 0..(CLUSTER_SIZE / DIR_ENTRY_SIZE) {
                let raw = self.read_raw_directory_entry(*i, j);
                let mut entry = DirectoryEntry::from(&raw);

                match &mut entry {
                    DirectoryEntry::LongFileName(str) => filename = format!("{}{}", filename, str),
                    DirectoryEntry::Invalid => filename = String::new(),
                    DirectoryEntry::File(inode) => {
                        inode.name = format!("{}{}", filename, inode.name);
                        filename = String::new();
                    }
                    DirectoryEntry::Directory(inode) => {
                        inode.name = format!("{}{}", filename, inode.name);
                        filename = String::new();
                    }
                }

                dir.push((entry, *i, j));
            }
        }
        dir
    }
}

impl<'a, T> BaseIO for FileSystem<'a, T>
where
    T: Read + Seek + Write,
{
    fn read_fat_entry(&mut self, cluster: FatEntry) -> FatEntry {
        self.io.read_fat_entry(cluster)
    }

    fn write_fat_entry(&mut self, cluster: FatEntry, entry: FatEntry) {
        self.io.write_fat_entry(cluster, entry)
    }

    fn read_cluster(&mut self, cluster: FatEntry) -> crate::Cluster {
        self.io.read_cluster(cluster)
    }

    fn write_cluster(&mut self, cluster: FatEntry, cluster_content: &crate::Cluster) {
        self.io.write_cluster(cluster, cluster_content)
    }

    fn read_raw_directory_entry(&mut self, cluster: FatEntry, idx: u32) -> crate::DirEntry {
        self.io.read_raw_directory_entry(cluster, idx)
    }

    fn write_raw_directory_entry(&mut self, cluster: FatEntry, idx: u32, entry: &crate::DirEntry) {
        self.io.write_raw_directory_entry(cluster, idx, entry)
    }
}
