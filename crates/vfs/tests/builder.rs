use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use vfs::fs::{
    // Dentry, FSInput, HandleAllocator, Meta, NodeType,
    builder::{AnyFS, FSBuilder, FsKind, TreeBuilder},
    inode::Dentry,
    meta::{Meta, NodeType},
    storage::mem::MemStorage,
    system::{FS, FSInput, HandleAllocator},
};

fn make_fs() -> (FS<MemStorage>, HandleAllocator) {
    let allocator = HandleAllocator::new();
    let storage = MemStorage::new();

    let root_handle = allocator.new_handle();
    let root_meta = Meta::new(vec!["/".into()], NodeType::Directory);

    let fs = FS::new(storage, allocator.clone(), root_handle, root_meta);

    (fs, allocator)
}
fn fs_input(pairs: Vec<(&str, NodeType)>) -> FSInput {
    let mut files = HashMap::new();

    for (path, node_type) in pairs {
        files.insert(
            path.to_string(),
            Meta::new(vec![path.to_string()], node_type),
        );
    }

    FSInput { files }
}

fn build(fs: &FS<MemStorage>, input: FSInput, allocator: &HandleAllocator) {
    TreeBuilder::build_into(&fs.core.root, input, allocator);
}

#[test]
fn builds_single_file() {
    let (fs, allocator) = make_fs();

    let input = fs_input(vec![("file.txt", NodeType::File)]);

    build(&fs, input, &allocator);

    let root = &fs.core.root;
    let children = root.children.read().unwrap();

    assert!(children.contains_key("file.txt"));
}

#[test]
fn builds_nested_structure() {
    let (fs, allocator) = make_fs();

    let input = fs_input(vec![
        ("dir1/b.txt", NodeType::File),
        ("dir1/dir2/c.txt", NodeType::File),
    ]);

    build(&fs, input, &allocator);

    let root = &fs.core.root;
    let level1 = root.children.read().unwrap();

    assert!(level1.contains_key("dir1"));
}

#[test]
fn builds_nested_directories_and_file() {
    let (fs, allocator) = make_fs();

    let input = fs_input(vec![("dir1/dir2/file.txt", NodeType::File)]);

    TreeBuilder::build_into(&fs.core.root, input, &allocator);

    let level1 = fs.core.root.children.read().unwrap();
    let dir1 = level1.get("dir1").unwrap().clone();

    let level2 = dir1.children.read().unwrap();
    let dir2 = level2.get("dir2").unwrap().clone();

    let level3 = dir2.children.read().unwrap();
    let file = level3.get("file.txt").unwrap();

    assert_eq!(file.inode.meta().node_type, NodeType::File);
    assert_eq!(dir2.inode.meta().node_type, NodeType::Directory);
}

#[test]
fn reuses_existing_directories() {
    let (fs, allocator) = make_fs();

    let input = fs_input(vec![
        ("dir/a.txt", NodeType::File),
        ("dir/b.txt", NodeType::File),
    ]);

    TreeBuilder::build_into(&fs.core.root, input, &allocator);

    let children = fs.core.root.children.read().unwrap();
    let dir1 = children.get("dir").unwrap().clone();

    let dir_children = dir1.children.read().unwrap();
    assert!(dir_children.contains_key("a.txt"));
    assert!(dir_children.contains_key("b.txt"));

    drop(children);

    let children2 = fs.core.root.children.read().unwrap();
    let dir2 = children2.get("dir").unwrap().clone();

    assert!(Arc::ptr_eq(&dir1, &dir2));
}

#[test]
fn allocator_creates_unique_handles() {
    let (fs, allocator) = make_fs();

    let input = fs_input(vec![
        ("a.txt", NodeType::File),
        ("b.txt", NodeType::File),
        ("c.txt", NodeType::File),
    ]);

    TreeBuilder::build_into(&fs.core.root, input, &allocator);

    let children = fs.core.root.children.read().unwrap();

    let handles: HashSet<_> = children.values().map(|n| n.inode.handle()).collect();

    assert_eq!(handles.len(), 3);
}

#[test]
fn fsbuilder_mem_builds_successfully() {
    let builder = FSBuilder::new(FsKind::Mem);

    let input = fs_input(vec![
        ("a.txt", NodeType::File),
        ("dir/b.txt", NodeType::File),
    ]);

    let fs = builder.build(input);

    match fs {
        AnyFS::Mem(fs) => {
            let root = fs.core.root.clone();
            let children = root.children.read().unwrap();

            assert!(children.contains_key("a.txt"));
            assert!(children.contains_key("dir"));
        }
        _ => panic!("expected Mem FS"),
    }
}

#[test]
fn fsbuilder_disk_builds_successfully() {
    let builder = FSBuilder::new(FsKind::Disk);

    let input = fs_input(vec![("x/y/z.txt", NodeType::File)]);

    let fs = builder.build(input);

    match fs {
        AnyFS::Disk(fs) => {
            let root = fs.core.root.clone();
            let children = root.children.read().unwrap();

            assert!(children.contains_key("x"));
        }
        _ => panic!("expected Disk FS"),
    }
}
