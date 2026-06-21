use std::sync::Arc;

use async_trait::async_trait;

use crate::fs::{
    Meta,
    config::FSConfig,
    engine::Engine,
    error::FSError,
    system::{FSHandle, HandleAllocator},
    inode::Dentry,
    meta::NodeType,
};

// Storage = “how bytes + metadata are persisted”
#[async_trait]
pub trait Storage: Send + Sync {
    async fn read(&self, h: &FSHandle) -> Result<Vec<u8>, FSError>;
    async fn write(&self, h: &FSHandle, data: Vec<u8>) -> Result<(), FSError>;
    async fn append(&self, h: &FSHandle, data: Vec<u8>) -> Result<(), FSError>;
    async fn meta(&self, h: &FSHandle) -> Result<Meta, FSError>;
    async fn readdir(&self, h: &FSHandle) -> Result<Vec<String>, FSError>;
}

#[async_trait]
pub trait FileSystem: Send + Sync {
    async fn walk(&self, path: &str) -> Result<FSHandle, FSError>;
}
// #[async_trait]
// pub trait FileSystem: Send + Sync {
//     async fn get_child_handle(
//         &self,
//         parent_handle: &FSHandle,
//         name: &str,
//     ) -> Result<FSHandle, FSError>;
//     async fn watch(
//         &self,
//         path: &str,
//         handler: Box<dyn Fn(&str) + Send + Sync>,
//     ) -> Result<(), FSError>;
//     async fn sync(&self) -> Result<(), FSError>;

//     // identity
//     fn new_handle(&self) -> FSHandle;

//     // async fn resolve_child_handle(
//     //     &self,
//     //     parent: &FSHandle,
//     //     name: &str,
//     // ) -> Result<(FSHandle, Meta), FSError> {
//     //     let child = self.get_child_handle(parent, name).await?;
//     //     let meta = self.meta_node(&child).await?;
//     //     Ok((child, meta))
//     // }
// }

// impl dyn FileSystem {
//     async fn walk(&self, path: &str) -> Result<FSHandle, FSError> {
//         todo!("Rename")
//     }
//     async fn sorted_readdir(&self, handle: &FSHandle) -> Result<Vec<String>, FSError> {
//         todo!("sort")
//     }
// }

// impl dyn FileSystem {
//     async fn walk(&self, path: &str) -> Result<FSHandle, FSError> {
//         todo!("Rename")
//     }
//     async fn sorted_readdir(&self, handle: &FSHandle) -> Result<Vec<String>, FSError> {
//         let entries = self.readdir_node(handle).await?;

//         let mut entries_with_meta: Vec<(String, Meta)> = Vec::new();

//         for name in entries {
//             let child_handle = self.get_child_handle(handle, &name).await?;
//             let meta = self.meta_node(&child_handle).await?;

//             entries_with_meta.push((name, meta));
//         }

//         entries_with_meta.sort_by(|a, b| {
//             let (name_a, meta_a) = a;
//             let (name_b, meta_b) = b;

//             let rank = |name: &str, meta: &Meta| -> u8 {
//                 let is_hidden = name.starts_with('.');

//                 match (meta.node_type, is_hidden) {
//                     (NodeType::Directory, true) => 0,
//                     (NodeType::Directory, false) => 1,
//                     (NodeType::File, true) => 2,
//                     (NodeType::File, false) => 3,
//                 }
//             };

//             let rank_a = rank(name_a, meta_a);
//             let rank_b = rank(name_b, meta_b);

//             if rank_a != rank_b {
//                 return rank_a.cmp(&rank_b);
//             }

//             name_a.to_lowercase().cmp(&name_b.to_lowercase())
//         });

//         Ok(entries_with_meta.into_iter().map(|(n, _)| n).collect())
//     }
// }
/// The central "Inode" trait acts as the common interface for
/// all filesystem nodes (files, directories, symlinks, etc.).
///
/// It enforces the "Bridge Pattern": it separates the Meta
/// operations (InodeOperations) from the I/O operations (InodeOperations).
#[async_trait]
pub trait Inode: Send + Sync {
    fn is_dir(&self) -> bool;

    /// Return owned metadata snapshot (NOT reference)
    fn meta(&self) -> &Meta;
}

pub trait InodeOperations: Send + Sync {
    fn lookup(&self, name: &str) -> Result<Arc<dyn Inode>, FSError>;
    fn create(&self, name: &str, mode: u32) -> Result<Arc<dyn Inode>, FSError>;
    fn mkdir(&self, name: &str, mode: u32) -> Result<Arc<dyn Inode>, FSError>;
}

pub trait FileOperations: Send + Sync {
    fn read(&self, buf: &mut [u8], offset: u64) -> Result<usize, FSError>;
    fn write(&self, buf: &[u8], offset: u64) -> Result<usize, FSError>;
    fn fsync(&self) -> Result<(), FSError>;
    fn release(&self) -> Result<(), FSError>;
}

pub trait Backend: Send + Sync {
    fn read(&self, handle: &FSHandle) -> Result<Vec<u8>, FSError>;
    fn write(&self, handle: &FSHandle, data: Vec<u8>) -> Result<(), FSError>;
    fn append(&self, handle: &FSHandle, data: Vec<u8>) -> Result<(), FSError>;
}
