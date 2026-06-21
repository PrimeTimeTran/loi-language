use std::sync::Arc;

use crate::fs::{
    Dentry, FS, FSHandle, FSInput, HandleAllocator, InMemoryDirectoryInode, InMemoryFileInode,
    Inode, JsonNode, Meta, NodeType, disk::DiskStorage, mem::MemStorage,
};

pub enum AnyFS {
    Mem(FS<MemStorage>),
    Disk(FS<DiskStorage>),
}
pub struct FSBuilder {
    kind: FsKind,
    allocator: HandleAllocator,
}

pub enum FsKind {
    Mem,
    Disk,
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
                let fs: FS<MemStorage> = FS::new(storage, self.allocator.clone());
                TreeBuilder::build_into(&fs.core.root, input, &self.allocator);
                AnyFS::Mem(fs)
            }

            FsKind::Disk => {
                let storage = DiskStorage::new();
                let fs: FS<DiskStorage> = FS::new(storage, self.allocator.clone());
                TreeBuilder::build_into(&fs.core.root, input, &self.allocator);
                AnyFS::Disk(fs)
            }
        }
    }
}

pub struct TreeBuilder;

impl TreeBuilder {
    pub fn build_into(root: &Arc<Dentry>, input: FSInput, allocator: &HandleAllocator) {
        for (path, _entry) in input.files {
            let parts: Vec<String> = path
                .trim_matches('/')
                .split('/')
                .map(|s| s.to_string())
                .collect();

            let mut current = Arc::clone(root);

            for (i, part) in parts.iter().enumerate() {
                let node = {
                    let mut children = current.children.write().unwrap();

                    children
                        .entry(part.clone())
                        .or_insert_with(|| {
                            let node_type = if i == parts.len() - 1 {
                                NodeType::File
                            } else {
                                NodeType::Directory
                            };

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
