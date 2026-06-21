use wasm_bindgen::prelude::*;

use crate::fs::{
    AnyFS, FSBuilder, FSInput, FsKind, Storage, builder::TreeBuilder, meta::NodeType,
    system::FSNode,
};
use js_sys::Array;
use serde::{Deserialize, Serialize};

#[wasm_bindgen]
pub struct Vfs {
    inner: AnyFS,
}

#[wasm_bindgen]
impl Vfs {
    #[wasm_bindgen(constructor)]
    pub fn new(root: JsValue) -> Result<Self, JsValue> {
        web_sys::console::log_1(&format!("{:?}", root).into());
        let root: FSNode = serde_wasm_bindgen::from_value(root)?;

        let input = {
            let mut files = Vec::new();
            FSInput::walk_node_owned(&root, String::new(), &mut files);
            FSInput { files }
        };

        Ok(Self {
            inner: FSBuilder::new(FsKind::Mem).build(input),
        })
    }
    // pub fn new(input: JsValue) -> Result<Self, JsValue> {
    //     web_sys::console::log_1(&format!("{:?}", input).into());
    //     let input: FSInput = serde_wasm_bindgen::from_value(input)?;
    //     let builder = FSBuilder::new(FsKind::Mem);

    //     Ok(Self {
    //         inner: builder.build(input),
    //     })
    // }

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

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct JsNode {
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    pub children: Option<Vec<JsNode>>,
    pub content: Option<String>,
}
