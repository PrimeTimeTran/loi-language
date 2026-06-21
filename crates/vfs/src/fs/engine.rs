use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::fs::{
    Dentry, FSError, FSHandle, FSInput, HandleAllocator, Storage,
    inode::{InMemoryDirectoryInode, InMemoryFileInode},
    meta::{Meta, NodeType},
    r#trait::Inode,
};

pub struct Engine<S: Storage> {
    pub root: Arc<Dentry>,
    pub storage: S,
    pub allocator: HandleAllocator,
    pub lock: Mutex<()>,
    pub index: std::sync::RwLock<HashMap<FSHandle, Arc<Dentry>>>,
    pub cwd: std::sync::RwLock<FSHandle>,
}

impl<S: Storage> Engine<S> {
    fn normalize_path(path: &str) -> Vec<String> {
        let mut parts = Vec::new();

        for part in path.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    parts.pop();
                }
                other => {
                    parts.push(other.to_string());
                }
            }
        }

        parts
    }
    fn find_dentry(&self, node: &Arc<Dentry>, target: &FSHandle) -> Option<Arc<Dentry>> {
        if node.inode.handle() == *target {
            return Some(node.clone());
        }

        let children = node.children.read().unwrap();

        for child in children.values() {
            if let Some(found) = self.find_dentry(child, target) {
                return Some(found);
            }
        }

        None
    }
    pub fn resolve_handle_to_dentry(&self, handle: &FSHandle) -> Result<Arc<Dentry>, FSError> {
        self.find_dentry(&self.root, handle)
            .ok_or(FSError::NotFound)
    }
    pub async fn readdir(&self, path: &str) -> Result<Vec<String>, FSError> {
        let handle = self.resolve_path(path)?;
        let node = self.resolve_handle_to_dentry(&handle)?;
        let children = node.children.read().unwrap();
        Ok(children.keys().cloned().collect())
    }
    pub async fn write(&self, path: &str, data: Vec<u8>) -> Result<(), FSError> {
        let handle = self.walk(path)?;

        self.storage.write(&handle, data).await
    }

    pub fn walk(&self, path: &str) -> Result<FSHandle, FSError> {
        web_sys::console::log_1(&format!("[ENGINE] walk path = {}", path).into());
        let parts = Self::normalize_path(path);
        web_sys::console::log_1(&format!("[ENGINE] parts = {:?}", parts).into());
        let mut current = self.root.clone();

        for part in parts {
            web_sys::console::log_1(
                &format!("[ENGINE] at = {}, looking for = {}", current.name, part).into(),
            );

            let next = {
                let children = current.children.read().unwrap();

                web_sys::console::log_1(
                    &format!(
                        "[ENGINE] children keys = {:?}",
                        children.keys().collect::<Vec<_>>()
                    )
                    .into(),
                );

                children
                    .get(&part)
                    .ok_or_else(|| {
                        web_sys::console::log_1(&format!("[ENGINE] NOT FOUND = {}", part).into());

                        FSError::NotFound
                    })?
                    .clone()
            };

            current = next;
        }

        web_sys::console::log_1(
            &format!("[ENGINE] final inode handle = {:?}", current.inode.handle()).into(),
        );

        Ok(current.inode.handle())
    }
    pub fn rename(&self, src: &str, dst: &str) -> Result<(), FSError> {
        todo!()
    }
    pub fn exists(&self, path: &str) -> bool {
        todo!()
    }

    pub fn mkdir(&self, path: &str) -> Result<(), FSError> {
        let _guard = self.lock.lock().unwrap();

        let parts: Vec<String> = path
            .trim_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        if parts.is_empty() {
            return Ok(());
        }

        let mut current = Arc::clone(&self.root);

        for (i, part) in parts.iter().enumerate() {
            let is_last = i == parts.len() - 1;

            let mut children = current.children.write().unwrap();

            let next = if let Some(existing) = children.get(part) {
                existing.clone()
            } else {
                let handle = self.allocator.new_handle();
                let meta = Meta::new(parts[..=i].to_vec(), NodeType::Directory);
                let inode = Self::build_inode(NodeType::Directory, handle, meta);
                let node = Arc::new(Dentry::new(part, inode, Some(current.clone())));
                children.insert(part.clone(), node.clone());
                self.index.write().unwrap().insert(handle, node.clone());

                node
            };

            drop(children);
            current = next;
        }

        Ok(())
    }
    fn build_inode(node_type: NodeType, handle: FSHandle, meta: Meta) -> Arc<dyn Inode> {
        match node_type {
            NodeType::Directory => Arc::new(InMemoryDirectoryInode::new(handle, meta)),
            NodeType::File => Arc::new(InMemoryFileInode::new(handle, meta)),
        }
    }

    pub fn pwd(&self) -> Result<String, FSError> {
        let handle = *self.cwd.read().unwrap();

        let node = self
            .index
            .read()
            .unwrap()
            .get(&handle)
            .cloned()
            .ok_or(FSError::NotFound)?;

        Ok(format!("/{}", node.name))
    }
    pub fn cd(&self, path: &str) -> Result<(), FSError> {
        let handle = self.resolve_path(path)?;

        let mut cwd = self.cwd.write().unwrap();
        *cwd = handle;

        Ok(())
    }

    pub fn resolve_path(&self, path: &str) -> Result<FSHandle, FSError> {
        let mut current = if path.starts_with('/') {
            self.root.clone()
        } else {
            let cwd = *self.cwd.read().unwrap();

            self.index
                .read()
                .unwrap()
                .get(&cwd)
                .cloned()
                .ok_or(FSError::NotFound)?
        };

        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();

        for part in parts {
            match part {
                "." => {
                    continue;
                }
                ".." => {
                    let parent = current.parent.as_ref().ok_or(FSError::NotFound)?.clone();

                    current = parent;
                }
                name => {
                    let next = {
                        let children = current.children.read().unwrap();
                        children.get(name).cloned().ok_or(FSError::NotFound)?
                    }; // 👈 lock dropped here

                    current = next;
                }
            }
        }

        Ok(current.inode.handle())
    }
}
