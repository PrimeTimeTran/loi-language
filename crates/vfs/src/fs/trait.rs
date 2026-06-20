use std::sync::Arc;

use async_trait::async_trait;

use crate::fs::{Meta, config::FSConfig, error::FSError, fs::FSHandle, meta::NodeType};

#[async_trait]
pub trait FileSystem: Send + Sync {
    async fn root(&self) -> &str;
    async fn genesis(&self) -> Result<FSConfig, FSError>;

    // --- 1. Path-based entry points (initial lookup) ---
    async fn walk(&self, path: &str) -> Result<FSHandle, FSError>;

    // --- 2. Handle-based Operations (High Performance) ---
    async fn read_node(&self, handle: &FSHandle) -> Result<Vec<u8>, FSError>;
    async fn write_node(&self, handle: &FSHandle, data: Vec<u8>) -> Result<(), FSError>;
    async fn append_node(&self, handle: &FSHandle, data: Vec<u8>) -> Result<(), FSError>;
    async fn meta_node(&self, handle: &FSHandle) -> Result<Meta, FSError>;
    async fn readdir_node(&self, handle: &FSHandle) -> Result<Vec<String>, FSError>;

    // --- 3. Mutative / Structural Operations ---
    // These still use paths because they involve creating or moving nodes
    async fn mkdir(&self, path: &str) -> Result<(), FSError>;
    async fn delete(&self, path: &str) -> Result<(), FSError>;
    async fn rename(&self, src: &str, dest: &str) -> Result<(), FSError>;
    async fn copy(&self, src: &str, dest: &str) -> Result<(), FSError>;

    async fn get_child_handle(
        &self,
        parent_handle: &FSHandle,
        name: &str,
    ) -> Result<FSHandle, FSError>;

    // --- 4. Utilities ---
    async fn exists(&self, path: &str) -> bool;
    async fn path_abs(&self, path: &str) -> String;
    async fn watch(
        &self,
        path: &str,
        handler: Box<dyn Fn(&str) + Send + Sync>,
    ) -> Result<(), FSError>;
    async fn sync(&self) -> Result<(), FSError>;

    async fn sorted_readdir(
        &self,
        fs: &dyn FileSystem,
        handle: &FSHandle,
    ) -> Result<Vec<String>, FSError> {
        let entries = fs.readdir_node(handle).await?;
        let mut entries_with_meta: Vec<(String, Meta)> = Vec::new();

        for name in entries {
            // Assume handle can resolve children or use parent path from handle
            let child_handle = fs.get_child_handle(handle, &name).await?;
            let meta = fs.meta_node(&child_handle).await?;
            entries_with_meta.push((name, meta));
        }

        entries_with_meta.sort_by(|a, b| {
            let (name_a, meta_a) = a;
            let (name_b, meta_b) = b;

            let rank = |name: &str, meta: &Meta| -> u8 {
                let is_hidden = name.starts_with('.');
                match (meta.node_type, is_hidden) {
                    (NodeType::Directory, true) => 0,
                    (NodeType::Directory, false) => 1,
                    (NodeType::File, true) => 2,
                    (NodeType::File, false) => 3,
                }
            };

            let rank_a = rank(name_a, meta_a);
            let rank_b = rank(name_b, meta_b);

            // 2. Compare rank first
            if rank_a != rank_b {
                return rank_a.cmp(&rank_b);
            }

            // 3. Fallback to case-insensitive alphabetical
            name_a.to_lowercase().cmp(&name_b.to_lowercase())
        });

        Ok(entries_with_meta.into_iter().map(|(n, _)| n).collect())
    }
}

/// The central "Inode" trait acts as the common interface for
/// all filesystem nodes (files, directories, symlinks, etc.).
///
/// It enforces the "Bridge Pattern": it separates the Meta
/// operations (InodeOperations) from the I/O operations (FSOperations).

pub trait Inode: Send + Sync {
    fn is_dir(&self) -> bool;
    fn get_meta(&self) -> Meta;
    fn file_ops(&self) -> &dyn FSOperations;
    fn inode_ops(&self) -> &dyn InodeOperations;
}

pub trait InodeOperations: Send + Sync {
    fn lookup(&self, name: &str) -> Result<Arc<dyn Inode>, FSError>;
    fn create(&self, name: &str, mode: u32) -> Result<Arc<dyn Inode>, FSError>;
    fn mkdir(&self, name: &str, mode: u32) -> Result<Arc<dyn Inode>, FSError>;
}

pub trait FSOperations: Send + Sync {
    fn read(&self, buf: &mut [u8], offset: u64) -> Result<usize, FSError>;
    fn write(&self, buf: &[u8], offset: u64) -> Result<usize, FSError>;
    fn fsync(&self) -> Result<(), FSError>;
    fn release(&self) -> Result<(), FSError>;
}
