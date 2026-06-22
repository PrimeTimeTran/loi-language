use wasm_bindgen::prelude::*;

use crate::fs::{
    AnyFS, FSBuilder, FSInput, FsKind, Storage,
    builder::TreeBuilder,
    meta::NodeType,
    system::{FileEntry, OwnedNode},
};

#[wasm_bindgen]
pub struct Vfs {
    inner: AnyFS,
}

#[wasm_bindgen]
impl Vfs {
    #[wasm_bindgen(constructor)]
    pub fn new(root: JsValue) -> Result<Self, JsValue> {
        crate::vfs_log!("RAW: {:#?}", root);

        let root: OwnedNode = serde_wasm_bindgen::from_value(root)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        crate::vfs_log!("ROOT: {}", root.name);
        crate::vfs_log!("children: {}", root.children.len());

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

        crate::vfs_log!("{}visiting: {} ({:?})", indent, path, node.node_type);

        match node.node_type {
            NodeType::File => {
                crate::vfs_log!("{}FILE => {}", indent, path);

                out.push(FileEntry {
                    path,
                    r#type: NodeType::File,
                });
            }

            NodeType::Directory => {
                crate::vfs_log!("{}DIR => {}", indent, path);

                for child in &node.children {
                    Self::debug_walk(child, path.clone(), depth + 1, out);
                }
            }
        }
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

                TreeBuilder::build_into(&fs.core, input, &fs.core.allocator);
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
        match &self.inner {
            AnyFS::Mem(fs) => fs
                .core
                .readdir(&path)
                .await
                .map_err(|e| e.to_string().into()),

            AnyFS::Disk(fs) => fs
                .core
                .readdir(&path)
                .await
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
