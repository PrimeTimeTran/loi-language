use std::sync::Arc;

use crate::fs::{Dentry, FSError, FSHandle, FSInput, HandleAllocator, Storage};

pub struct Engine<S: Storage> {
    pub root: Arc<Dentry>,
    pub storage: S,
    pub allocator: HandleAllocator,
}

impl<S: Storage> Engine<S> {
    pub fn build_tree(&self, input: FSInput, allocator: &HandleAllocator) -> Arc<Dentry> {
        todo!()
    }
    pub fn walk(&self, path: &str) -> Result<FSHandle, FSError> {
        todo!()
    }
    pub fn rename(&self, src: &str, dst: &str) -> Result<(), FSError> {
        todo!()
    }
    pub fn exists(&self, path: &str) -> bool {
        todo!()
    }
    pub fn mkdir(&self, path: &str) -> Result<(), FSError> {
        todo!()
    }
}
