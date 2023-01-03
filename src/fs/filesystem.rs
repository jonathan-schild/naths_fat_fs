use fuser::{FileAttr, FileType, Filesystem};
use libc::{EBADFD, ENOENT};
use std::{
    ffi::OsString,
    io::{Read, Seek, Write},
    time::{Duration, UNIX_EPOCH},
};

use crate::{
    consts::CLUSTER_SIZE,
    utility::fs_utility::{from_inode, to_inode},
};

use super::{basic_fs_io::BaseIO, directory::DirectoryEntry, FileSystem};

impl<'a, T> Filesystem for FileSystem<'a, T>
where
    T: Read + Seek + Write,
{
    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        let (c, i) = from_inode(parent);
        let parent_dir = DirectoryEntry::from(&self.read_raw_directory_entry(c, i));

        let chain;

        match parent_dir {
            DirectoryEntry::Invalid => reply.error(EBADFD),
            DirectoryEntry::LongFileName(_) => reply.error(EBADFD),
            DirectoryEntry::Directory(inode) => {
                chain = self.get_chain(inode.start_cluster);

                let dir = self.read_dir(&chain);

                for (e, c, i) in &dir {
                    let (t, inode) = match e {
                        DirectoryEntry::Invalid => continue,
                        DirectoryEntry::LongFileName(_) => continue,
                        DirectoryEntry::Directory(i) => (FileType::Directory, i),
                        DirectoryEntry::File(i) => (FileType::RegularFile, i),
                    };

                    if name.to_str().unwrap() == inode.name {
                        reply.entry(
                            &Duration::from_secs(10),
                            &FileAttr {
                                ino: to_inode(*c, *i),
                                size: inode.length,
                                blocks: 0,
                                atime: inode.atime,
                                mtime: inode.mtime,
                                ctime: inode.ctime,
                                crtime: UNIX_EPOCH,
                                kind: t,
                                perm: inode.permission,
                                nlink: 1,
                                uid: inode.uid,
                                gid: inode.gid,
                                rdev: 0,
                                blksize: 0,
                                flags: 0,
                            },
                            0,
                        );
                        return;
                    }
                }

                reply.error(ENOENT);
            }
            DirectoryEntry::File(_) => reply.error(EBADFD),
        }
    }

    fn getattr(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        let (cluster, idx) = from_inode(ino);

        let dir = DirectoryEntry::from(&self.read_raw_directory_entry(cluster, idx));

        let (t, inode) = match dir {
            DirectoryEntry::Invalid => {
                reply.error(EBADFD);
                return;
            }
            DirectoryEntry::LongFileName(_) => {
                reply.error(EBADFD);
                return;
            }
            DirectoryEntry::Directory(i) => (FileType::Directory, i),
            DirectoryEntry::File(i) => (FileType::RegularFile, i),
        };

        reply.attr(
            &Duration::from_secs(10),
            &FileAttr {
                ino,
                size: inode.length,
                blocks: 0,
                atime: inode.atime,
                mtime: inode.mtime,
                ctime: inode.ctime,
                crtime: UNIX_EPOCH,
                kind: t,
                perm: inode.permission,
                nlink: 1,
                uid: inode.uid,
                gid: inode.gid,
                rdev: 0,
                blksize: 0,
                flags: 0,
            },
        );
    }

    fn open(&mut self, _req: &fuser::Request<'_>, _ino: u64, _flags: i32, reply: fuser::ReplyOpen) {
        reply.opened(0, 0);
    }

    fn read(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: fuser::ReplyData,
    ) {
        let (cluster, idx) = from_inode(ino);
        let dir = DirectoryEntry::from(&self.read_raw_directory_entry(cluster, idx));

        match &dir {
            DirectoryEntry::File(i) => {
                let start = i.start_cluster;
                let _length = i.length;
                let _chain = self.get_chain(start);

                let _cluster = offset as u32 / CLUSTER_SIZE;
                let _in_cluster_offset = offset as u32 % CLUSTER_SIZE;

                // TODO

                reply.data(b"hello\0");
            }
            _ => reply.error(ENOENT),
        };
    }

    fn release(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _fh: u64,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
        reply: fuser::ReplyEmpty,
    ) {
        reply.ok();
    }

    fn opendir(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _flags: i32,
        reply: fuser::ReplyOpen,
    ) {
        reply.opened(0, 0);
    }

    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: fuser::ReplyDirectory,
    ) {
        if offset == -1 {
            reply.ok();
            return;
        }

        let (c, i) = from_inode(ino);
        let parent_dir = DirectoryEntry::from(&self.read_raw_directory_entry(c, i));

        let chain;

        match parent_dir {
            DirectoryEntry::Invalid => reply.error(EBADFD),
            DirectoryEntry::LongFileName(_) => reply.error(EBADFD),
            DirectoryEntry::Directory(inode) => {
                chain = self.get_chain(inode.start_cluster);

                let dir = &self.read_dir(&chain)[offset as usize..];

                for (idx, (e, c, i)) in dir.iter().enumerate() {
                    let (t, name) = match e {
                        DirectoryEntry::Directory(i) => (FileType::Directory, i.name.to_string()),
                        DirectoryEntry::File(i) => (FileType::RegularFile, i.name.to_string()),
                        _ => continue,
                    };

                    let buf_full =
                        reply.add(to_inode(*c, *i), (idx + 1) as i64, t, OsString::from(&name));

                    if buf_full {
                        break;
                    }
                }

                reply.ok();
            }
            DirectoryEntry::File(_) => reply.error(EBADFD),
        }
    }

    fn releasedir(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _fh: u64,
        _flags: i32,
        reply: fuser::ReplyEmpty,
    ) {
        reply.ok()
    }
}
