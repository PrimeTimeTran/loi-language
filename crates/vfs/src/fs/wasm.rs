use wasm_bindgen::prelude::*;

use crate::fs::{AnyFS, FSBuilder, FSInput, FsKind, Storage, builder::TreeBuilder, system::FSNode};
use js_sys::Array;

#[wasm_bindgen]
pub struct Vfs {
    inner: AnyFS,
}

#[wasm_bindgen]
impl Vfs {
    #[wasm_bindgen(constructor)]
    pub fn new(root: JsValue) -> Result<Self, JsValue> {
        let root: FSNode = serde_wasm_bindgen::from_value(root)?;
        let input = FSInput::from_node(root);
        let builder = FSBuilder::new(FsKind::Mem);

        Ok(Self {
            inner: builder.build(input),
        })
    }

    pub fn exists(&self, path: String) -> bool {
        match &self.inner {
            AnyFS::Mem(fs) => fs.core.exists(&path),

            AnyFS::Disk(fs) => fs.core.exists(&path),
        }
    }

    #[wasm_bindgen]
    pub fn add_file(&mut self, path: String) {
        match &mut self.inner {
            AnyFS::Mem(fs) => {
                let input = FSInput::from_files(vec!["hello.txt".into(), "src/main.rs".into()]);

                TreeBuilder::build_into(&fs.core.root, input, &fs.core.allocator);
            }

            AnyFS::Disk(_) => {}
        }
    }

    pub fn mkdir(&mut self, path: String) -> Result<(), JsValue> {
        match &mut self.inner {
            AnyFS::Mem(fs) => fs.core.mkdir(&path).map_err(|e| e.to_string().into()),

            AnyFS::Disk(fs) => fs.core.mkdir(&path).map_err(|e| e.to_string().into()),
        }
    }
    pub fn readdir(&self, path: String) -> Result<Vec<String>, JsValue> {
        let handle = match &self.inner {
            AnyFS::Mem(fs) => futures::executor::block_on(fs.walk(&path)),

            AnyFS::Disk(fs) => futures::executor::block_on(fs.walk(&path)),
        }
        .map_err(|e| e.to_string())?;

        match &self.inner {
            AnyFS::Mem(fs) => futures::executor::block_on(fs.core.storage.readdir(&handle))
                .map_err(|e| e.to_string().into()),

            AnyFS::Disk(fs) => futures::executor::block_on(fs.core.storage.readdir(&handle))
                .map_err(|e| e.to_string().into()),
        }
    }

    pub fn write(&self, path: String, data: Vec<u8>) -> Result<(), JsValue> {
        match &self.inner {
            AnyFS::Mem(fs) => {
                let handle =
                    futures::executor::block_on(fs.walk(&path)).map_err(|e| e.to_string())?;

                futures::executor::block_on(fs.core.storage.write(&handle, data))
                    .map_err(|e| e.to_string().into())
            }

            AnyFS::Disk(fs) => {
                let handle =
                    futures::executor::block_on(fs.walk(&path)).map_err(|e| e.to_string())?;

                futures::executor::block_on(fs.core.storage.write(&handle, data))
                    .map_err(|e| e.to_string().into())
            }
        }
    }

    pub fn read(&self, path: String) -> Result<Vec<u8>, JsValue> {
        match &self.inner {
            AnyFS::Mem(fs) => {
                let handle =
                    futures::executor::block_on(fs.walk(&path)).map_err(|e| e.to_string())?;

                futures::executor::block_on(fs.core.storage.read(&handle))
                    .map_err(|e| e.to_string().into())
            }

            AnyFS::Disk(fs) => {
                let handle =
                    futures::executor::block_on(fs.walk(&path)).map_err(|e| e.to_string())?;

                futures::executor::block_on(fs.core.storage.read(&handle))
                    .map_err(|e| e.to_string().into())
            }
        }
    }
}

impl Vfs {
    pub fn new_empty() -> Self {
        let builder = FSBuilder::new(FsKind::Mem);

        Self {
            inner: builder.build(FSInput::empty()),
        }
    }
}

impl Default for Vfs {
    fn default() -> Self {
        Self::new_empty()
    }
}

use serde::Deserialize;

#[derive(Deserialize)]
struct JsNode {
    name: String,
    #[serde(rename = "type")]
    node_type: String,
    children: Option<Vec<JsNode>>,
    content: Option<String>,
}
