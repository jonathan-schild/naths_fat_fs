use fuser::{mount2, MountOption};
use naths_fat_fs::{
    fs::{
        basic_fs_io::{BaseIO, FileSystemBasicIO},
        directory::{DirectoryEntry, Inode},
        FileSystem,
    },
    mkfs::{write_data_section, write_prelude, write_root_dir},
    DirEntry,
};
use std::{fs::OpenOptions, path::Path, time::SystemTime};

#[test]
fn test() {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .open(Path::new("test.hex"))
        .unwrap();

    write_prelude(16, &mut file);
    write_data_section(16, &mut file);

    let mut fs = FileSystem {
        io: FileSystemBasicIO::open_file_system(&mut file),
    };

    write_root_dir(&mut fs);

    let mut new_file = vec![];

    fs.append_to_chain(&mut new_file);

    let mut block = fs.read_cluster(new_file[0]);
    let cont = b"Das ist eine Hello.txt file :)";

    block[0..cont.len()].copy_from_slice(cont);

    fs.write_cluster(new_file[0], &block);

    fs.write_raw_directory_entry(
        1,
        2,
        &DirEntry::from(&DirectoryEntry::File(Inode::new(
            "Hello.txt".to_string(),
            cont.len() as u64,
            1000,
            1000,
            0o666,
            SystemTime::now(),
            SystemTime::now(),
            SystemTime::now(),
            1,
            new_file[0],
        ))),
    );

    mount2(
        fs,
        Path::new("mnt/"),
        &[MountOption::Sync, MountOption::AutoUnmount, MountOption::RO],
    )
    .unwrap();
}
