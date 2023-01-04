# Nath's FAT Filesystem

This is not a filesystem for productive application. I just want to learn something about FUSE and file systems in general. Maybe it's helpful for others who are also interested in these topics.

## General

- little-endian encoding
- cluster size: `8192 Bytes`
- directory entry size: `64 Bytes`
- FAT entry size: `4 Byte | 32 bit` (little-endian)
- start address of FAT: `0x0000_0000_0000_0020`
- start of data region: next `32 Byte` aligned address behind FAT

## The first 32 byte

| Bytes    | 0 - 8          | 9    | 10 - 13          | 14 - 31 |
|---       |---             |---   |---               |---      |
| Content  | `b"NathFATfs"` | 0x01 | size of FAT [^0] | padding |

[^0]: in number of entries

## The FAT

Each four bytes of the FAT represent one cluster in the data region. So the first FAT entry represents the first `8192 Bytes` of the data region (first cluster), the next 4 bytes in the FAT the second `8192 Bytes` of the data region (second cluster) and so one.

The FAT stores information about its corresponding cluster. Possible entires are:

- `0x0000_0000`: This cluster is free and not used. It can be allocated if necessary.
- `0xFFFF_FFFE`: This cluster is not used but do not allocate. In read block devices used e.g. for bad blocks.
- `0x<next-cluster>`: Contains the address of the next cluster. E.g. a file needs 16,384 Bytes of storage, so you need two cluster. The FAT entry of the fist cluster tells you the cluster of the second cluster of this file.
- `0xFFFF_FFFF`: End-of-chain - this is the last cluster of a file.

## The Directory

The FAT tells you which chunks belong to a file but you don't know where to start. This information can be acquired through a directory. A directory just a special file that tells you which files and subdirectories it contains and at which cluster they start.

The root directory starts at the first cluster.

### Directory Entry

| Byte(s)  | Mask | Meaning
|:---:     |:---: |:---
| 0        | 0x01 | Entry is valid
| 0        | 0x02 | All other bytes encode the begin of the name of the following entry
| 0        | 0x04 | Entry is a directory
| 0        | 0x08 | Reserved
| 0        | 0xF0 | Reserved
| 1 - 4    | ---  | UID (POSIX User ID | Owner)
| 5 - 8    | ---  | GID (Group ID)
| 9 - 14   | ---  | ctime (Last change of meta-data)
| 15 - 20  | ---  | mtime (Last change of data)
| 21 - 26  | ---  | atime (Last access of data)
| 27 - 28  | ---  | Permission (POSIX-permission)
| 29 - 32  | ---  | Start cluster of content
| 33 - 38  | ---  | Length in Bytes
| 39 - 63  | ---  | Name (padded with \0)

## The FAT, Directory and the Inode

The Linux Virtual File System (VFS) is an abstraction layer between the actual File System and userspace. The VFS uses Inodes (index-nodes) to work with files, directories and their meta-data. The Inode number identifies as file or directory uniquely (per file system). To be able to use FUSE to mount this file system we must provide the FUSE Kernel Driver unique Inodes for every directory or file. The nearest equivalent to an Inode is a Directory Entry. It can be uniquely identified by the cluster and offset in that cluster.