use std::sync::Arc;

use crate::{
    fs::{
        Dentry, FS, FSHandle, FSInput, HandleAllocator, InMemoryDirectoryInode, InMemoryFileInode,
        Inode, Meta, NodeType, engine::Engine, r#trait::Storage,
    },
    storage::{disk::DiskStorage, mem::MemStorage},
};

pub enum AnyFS {
    Mem(FS<MemStorage>),
    Disk(FS<DiskStorage>),
}

pub enum FsKind {
    Mem,
    Disk,
}

pub struct FSBuilder {
    kind: FsKind,
    allocator: HandleAllocator,
}

impl FSBuilder {
    pub fn new(kind: FsKind) -> Self {
        Self {
            kind,
            allocator: HandleAllocator::new(),
        }
    }

    pub fn build(self, input: FSInput) -> AnyFS {
        match self.kind {
            FsKind::Mem => {
                let storage = MemStorage::new();
                let allocator = self.allocator.clone();

                let root_handle = allocator.new_handle();
                let root_meta = Meta::new(vec!["/".into()], NodeType::Directory);

                let fs: FS<MemStorage> =
                    FS::new(storage, allocator.clone(), root_handle, root_meta);

                TreeBuilder::build_into(&fs.core, input, &allocator);

                AnyFS::Mem(fs)
            }

            FsKind::Disk => {
                let storage = DiskStorage::new();
                let allocator = self.allocator.clone();

                let root_handle = allocator.new_handle();
                let root_meta = Meta::new(vec!["/".into()], NodeType::Directory);

                let fs: FS<DiskStorage> =
                    FS::new(storage, allocator.clone(), root_handle, root_meta);

                TreeBuilder::build_into(&fs.core, input, &allocator);

                AnyFS::Disk(fs)
            }
        }
    }
}

pub struct TreeBuilder;

impl TreeBuilder {
    pub fn build_into<S: Storage>(engine: &Engine<S>, input: FSInput, allocator: &HandleAllocator) {
        for entry in input.files {
            Self::ensure_path(engine, &entry.path, entry.r#type, allocator);
        }
    }

    fn split_path(path: &str) -> Vec<&str> {
        path.trim_matches('/')
            .split('/')
            .filter(|p| !p.is_empty())
            .collect()
    }
    fn build_meta_path(parts: &[&str], i: usize) -> Vec<String> {
        parts[..=i].iter().map(|s| s.to_string()).collect()
    }
    fn create_node<S: Storage>(
        part: &str,
        parts: &[&str],
        i: usize,
        node_type: NodeType,
        allocator: &HandleAllocator,
        engine: &Engine<S>,
        parent: &Arc<Dentry>,
    ) -> Arc<Dentry> {
        let handle = allocator.new_handle();
        let meta = Meta::new(Self::build_meta_path(parts, i), node_type);
        let inode = Self::build_inode(node_type, handle, meta);
        let node = Arc::new(Dentry::new(part, inode, Some(parent.clone())));
        engine.index.write().unwrap().insert(handle, node.clone());
        if node_type == NodeType::File {
            futures::executor::block_on(engine.storage.write(&handle, Vec::new())).unwrap();
        }
        node
    }
    fn ensure_path<S: Storage>(
        engine: &Engine<S>,
        path: &str,
        final_type: NodeType,
        allocator: &HandleAllocator,
    ) {
        let parts = Self::split_path(path);
        let mut current = Arc::clone(&engine.root);
        for (i, part) in parts.iter().enumerate() {
            let is_leaf = i == parts.len() - 1;

            let node_type = if is_leaf {
                final_type
            } else {
                NodeType::Directory
            };

            let next = {
                let mut children = current.children.write().unwrap();

                children
                    .entry(part.to_string())
                    .or_insert_with(|| {
                        Self::create_node(part, &parts, i, node_type, allocator, engine, &current)
                    })
                    .clone()
            };

            current = next;
        }
    }
    fn build_inode(node_type: NodeType, handle: FSHandle, meta: Meta) -> Arc<dyn Inode> {
        match node_type {
            NodeType::Directory => Arc::new(InMemoryDirectoryInode::new(handle, meta)),
            NodeType::File => Arc::new(InMemoryFileInode::new(handle, meta)),
        }
    }
}
