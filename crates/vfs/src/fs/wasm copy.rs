use wasm_bindgen::prelude::*;

use crate::fs::{
    AnyFS, FSBuilder, FSInput, FsKind, Storage,
    builder::TreeBuilder,
    meta::NodeType,
    system::{FileEntry, OwnedNode},
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
    pub async fn new(root: JsValue) -> Result<Self, JsValue> {
        web_sys::console::log_1(&format!("RAW: {:#?}", root).into());

        let root: OwnedNode = serde_wasm_bindgen::from_value(root)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        web_sys::console::log_1(&format!("ROOT: {}", root.name).into());
        web_sys::console::log_1(&format!("children: {}", root.children.len()).into());

        let mut files = Vec::new();

        for child in &root.children {
            Self::debug_walk(child, String::new(), 0, &mut files);
        }

        Ok(Self {
            inner: FSBuilder::new(FsKind::Mem).build(FSInput { files }),
        })
    }
    fn debug_walk(node: &OwnedNode, prefix: String, depth: usize, out: &mut Vec<FileEntry>) {
        let indent = "  ".repeat(depth);

        let path = if prefix.is_empty() {
            node.name.clone()
        } else {
            format!("{}/{}", prefix, node.name)
        };

        web_sys::console::log_1(
            &format!("{}visiting: {} ({:?})", indent, path, node.node_type).into(),
        );

        match node.node_type {
            NodeType::File => {
                web_sys::console::log_1(&format!("{}FILE => {}", indent, path).into());

                out.push(FileEntry {
                    path,
                    r#type: NodeType::File,
                });
            }

            NodeType::Directory => {
                web_sys::console::log_1(&format!("{}DIR => {}", indent, path).into());

                for child in &node.children {
                    Self::debug_walk(child, path.clone(), depth + 1, out);
                }
            }
        }
    }
    #[wasm_bindgen]
    pub async fn exists(&self, path: String) -> bool {
        match &self.inner {
            AnyFS::Mem(fs) => fs.core.exists(&path),

            AnyFS::Disk(fs) => fs.core.exists(&path),
        }
    }
    #[wasm_bindgen]
    pub async fn add_file(&mut self, path: String) {
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
    #[wasm_bindgen]
    pub async fn readdir(&self, path: String) -> Result<Vec<String>, JsValue> {
        web_sys::console::log_1(&format!("[VFS] readdir path = {}", path).into());

        let handle = match &self.inner {
            AnyFS::Mem(fs) => {
                let h = futures::executor::block_on(fs.walk(&path));
                web_sys::console::log_1(&format!("[VFS] walk => {:?}", h).into());
                h
            }
            AnyFS::Disk(fs) => {
                let h = futures::executor::block_on(fs.walk(&path));
                web_sys::console::log_1(&format!("[VFS] walk => {:?}", h).into());
                h
            }
        }
        .map_err(|e| {
            web_sys::console::log_1(&format!("[VFS] walk error = {}", e).into());
            e.to_string()
        })?;

        let result = match &self.inner {
            AnyFS::Mem(fs) => {
                let r = futures::executor::block_on(fs.core.storage.readdir(&handle));
                web_sys::console::log_1(&format!("[VFS] storage readdir => {:?}", r).into());
                r
            }
            AnyFS::Disk(fs) => {
                let r = futures::executor::block_on(fs.core.storage.readdir(&handle));
                web_sys::console::log_1(&format!("[VFS] storage readdir => {:?}", r).into());
                r
            }
        };

        result.map_err(|e| {
            web_sys::console::log_1(&format!("[VFS] storage error = {}", e).into());
            e.to_string().into()
        })
    }
    #[wasm_bindgen]
    pub async fn write(&self, path: String, data: Vec<u8>) -> Result<(), JsValue> {
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
    #[wasm_bindgen]
    pub async fn read(&self, path: String) -> Result<Vec<u8>, JsValue> {
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
