use std::sync::Arc;

use crate::fs::{
    Dentry, FS, FSHandle, FSInput, HandleAllocator, InMemoryDirectoryInode, InMemoryFileInode,
    Inode, JsonNode, Meta, NodeType, disk::DiskStorage, mem::MemStorage,
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

                TreeBuilder::build_into(&fs.core.root, input, &allocator);

                AnyFS::Mem(fs)
            }

            FsKind::Disk => {
                let storage = DiskStorage::new();
                let allocator = self.allocator.clone();

                let root_handle = allocator.new_handle();
                let root_meta = Meta::new(vec!["/".into()], NodeType::Directory);

                let fs: FS<DiskStorage> =
                    FS::new(storage, allocator.clone(), root_handle, root_meta);

                TreeBuilder::build_into(&fs.core.root, input, &allocator);

                AnyFS::Disk(fs)
            }
        }
    }
}

pub struct TreeBuilder;

impl TreeBuilder {
    pub fn build_into(root: &Arc<Dentry>, input: FSInput, allocator: &HandleAllocator) {
        for entry in input.files {
            let path = entry.path;
            let explicit_type = entry.r#type;

            let parts: Vec<String> = path
                .trim_matches('/')
                .split('/')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();

            let mut current = Arc::clone(root);

            for (i, part) in parts.iter().enumerate() {
                let is_leaf = i == parts.len() - 1;

                let node_type = if is_leaf {
                    entry.r#type
                } else {
                    NodeType::Directory
                };

                let node = {
                    let mut children = current.children.write().unwrap();

                    children
                        .entry(part.clone())
                        .or_insert_with(|| {
                            let handle = allocator.new_handle();

                            let meta = Meta::new(parts[..=i].to_vec(), node_type);

                            let inode = Self::build_inode(node_type, handle, meta);

                            Arc::new(Dentry::new(part, inode))
                        })
                        .clone()
                };

                current = node;
            }
        }
    }

    fn build_inode(node_type: NodeType, handle: FSHandle, meta: Meta) -> Arc<dyn Inode> {
        match node_type {
            NodeType::Directory => Arc::new(InMemoryDirectoryInode::new(handle, meta)),
            NodeType::File => Arc::new(InMemoryFileInode::new(handle, meta)),
        }
    }
}
