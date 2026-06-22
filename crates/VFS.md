### src/fs/builder.rs

```rs
        // ENUMS:
        enum AnyFS { Mem(_0: FS < MemStorage >), Disk(_0: FS < DiskStorage >) }
        enum FsKind { Mem, Disk }

        // STRUCTS:
        struct FSBuilder
            // PROPERTIES:
            kind: FsKind, allocator: HandleAllocator

            // METHODS:
            fn new(
                kind: FsKind
            ) -> Self
            fn build(
                self,
                input: FSInput
            ) -> AnyFS
        struct TreeBuilder
            // METHODS:
            fn build_into(
                engine: &Engine<S>,
                input: FSInput,
                allocator: &HandleAllocator
            )
            fn split_path(
                path: &str
            ) -> Vec<&str>
            fn build_meta_path(
                parts: &[&str],
                i: usize
            ) -> Vec<String>
            fn create_node(
                part: &str,
                parts: &[&str],
                i: usize,
                node_type: NodeType,
                allocator: &HandleAllocator,
                engine: &Engine<S>,
                parent: &Arc<Dentry>
            ) -> Arc<Dentry>
            fn ensure_path(
                engine: &Engine<S>,
                path: &str,
                final_type: NodeType,
                allocator: &HandleAllocator
            )
            fn build_inode(
                node_type: NodeType,
                handle: FSHandle,
                meta: Meta
            ) -> Arc<dyn Inode>
```

### src/fs/config.rs

```rs
        // STRUCTS:
        struct FSConfig
            // PROPERTIES:
            name: String, version: String, entry_points: Vec<String>, ignore: Vec<String>
```

### src/fs/engine.rs

```rs
        // STRUCTS:
        struct Engine
            // PROPERTIES:
            storage: S, lock: Mutex<()>, root: Arc<Dentry>, allocator: HandleAllocator, cwd: std::sync::RwLock<FSHandle>, index: std::sync::RwLock<HashMap<FSHandle, Arc<Dentry>>>
```

### src/fs/error.rs

```rs
        // ENUMS:
        enum FSError { NotFound, PermissionDenied, IoError, AlreadyExists, InvalidPath }
```

### src/fs/inode.rs

```rs
        // STRUCTS:
        struct Dentry
            // PROPERTIES:
            name: String, inode: Arc<dyn Inode>, parent: Option<Arc<Dentry>>, children: RwLock<HashMap<String, Arc<Dentry>>>

            // METHODS:
            fn new(
                name: &str,
                inode: Arc<dyn Inode>,
                parent: Option<Arc<Dentry>>
            ) -> Self
            fn new_root(
                root_inode: Arc<dyn Inode>
            ) -> Arc<Self>
            fn lookup(
                self,
                name: &str
            ) -> Result<Arc<Dentry>, FSError>
        struct DentryDTO
            // PROPERTIES:
            name: String, inode_id: String, children: Vec<DentryDTO>
        struct InMemoryDirectoryInode
            // PROPERTIES:
            meta: Meta, handle: FSHandle

            // METHODS:
            fn is_dir(self) -> bool
            fn meta(self) -> &Meta
            fn handle(self) -> FSHandle
            fn new(
                handle: FSHandle,
                meta: Meta
            ) -> Self
        struct InMemoryFileInode
            // PROPERTIES:
            meta: Meta, handle: FSHandle

            // METHODS:
            fn is_dir(self) -> bool
            fn meta(self) -> &Meta
            fn handle(self) -> FSHandle
            fn new(
                handle: FSHandle,
                meta: Meta
            ) -> Self
            fn handle(self) -> FSHandle
        struct RootInode
            // PROPERTIES:
            meta: Meta, handle: FSHandle

            // METHODS:
            fn new(
                handle: FSHandle,
                meta: Meta
            ) -> Self
            fn is_dir(self) -> bool
            fn meta(self) -> &Meta
            fn handle(self) -> FSHandle
```

### src/fs/meta.rs

```rs
        // ENUMS:
        enum NodeType { File, Directory }

        // STRUCTS:
        struct Meta
            // PROPERTIES:
            handle: FSHandle, size: u64, mode: u32, ext: String, language: String, path_abs: FSPath, path_rel: FSPath, node_type: NodeType

            // METHODS:
            fn default() -> Self
            fn is_dir(self) -> bool
            fn new(
                path_segments: Vec<String>,
                node_type: NodeType
            ) -> Self
            fn with_handle(
                self,
                handle: FSHandle
            ) -> Self
```

### src/fs/system.rs

```rs
        // ENUMS:
        enum FSHandleDTO { Mem(_0: String), Host(_0: PathBuf) }

        // STRUCTS:
        struct FS
            // PROPERTIES:
            core: Engine<S>
        struct FSFile
            // PROPERTIES:
            handle: FSHandle, content: Vec<u8>
        struct FSFileDTO
            // PROPERTIES:
            handle_id: String, content: String
        struct FSHandle
        struct FSInput
            // PROPERTIES:
            files: Vec<FileEntry>

            // METHODS:
            fn empty() -> Self
            fn from_files(
                paths: Vec<String>
            ) -> Self
            fn walk_node_owned(
                node: &OwnedNode,
                prefix: String,
                out: &mut Vec<FileEntry>
            )
            fn from_node(
                root: OwnedNode
            ) -> Self
            fn walk_owned(
                node: &OwnedNode,
                prefix: String,
                out: &mut Vec<FileEntry>
            )
        struct FSPath
            // METHODS:
            fn new(
                segments: Vec<String>
            ) -> Self
            fn empty() -> Self
            fn join(
                self,
                other: &FSPath
            ) -> Self
            fn from_string(
                path: &str
            ) -> Self
        struct FileEntry
            // PROPERTIES:
            path: String, r#type: NodeType
        struct HandleAllocator
            // PROPERTIES:
            counter: Arc<AtomicU64>

            // METHODS:
            fn new() -> Self
            fn new_handle(self) -> FSHandle
        struct OwnedNode
            // PROPERTIES:
            name: String, node_type: NodeType, children: Vec<OwnedNode>, content: Option<String>

            // METHODS:
            fn into_owned(self) -> OwnedNode
```

### src/fs/vfs.rs

```rs
        // STRUCTS:
        struct JsonNode
            // PROPERTIES:
            name: String, r#type: String, content: Option<String>, children: Option<Vec<JsonNode>>
```

### src/fs/wasm.rs

```rs
        // STRUCTS:
        struct Vfs
            // PROPERTIES:
            inner: AnyFS

            // METHODS:
            fn new(
                root: JsValue
            ) -> Result<Self, JsValue>
            fn debug_walk(
                node: &OwnedNode,
                prefix: String,
                depth: usize,
                out: &mut Vec<FileEntry>
            )
            fn exists(
                self,
                path: String
            ) -> bool
            fn add_file(
                self,
                path: String
            )
            fn mkdir(
                self,
                path: String
            ) -> Result<(), JsValue>
            fn readdir(
                self,
                path: String
            ) -> Result<Vec<String>, JsValue>
            fn write(
                self,
                path: String,
                data: Vec<u8>
            ) -> Result<(), JsValue>
            fn read(
                self,
                path: String
            ) -> Result<Vec<u8>, JsValue>
            fn new_empty() -> Self
            fn default() -> Self
```

## src/main.rs

```rs
    // FUNCTIONS:
    fn main()
```

### src/storage/disk.rs

```rs
        // STRUCTS:
        struct DiskStorage
            // PROPERTIES:
            path_root: PathBuf

            // METHODS:
            fn new() -> Self
            fn with_root(
                path: impl Into<PathBuf>
            ) -> Self
            fn walk(
                self,
                path: &str
            ) -> PathBuf
            fn read(
                self,
                h: &FSHandle
            ) -> Result<Vec<u8>, FSError>
            fn write(
                self,
                h: &FSHandle,
                data: Vec<u8>
            ) -> Result<(), FSError>
            fn append(
                self,
                h: &FSHandle,
                data: Vec<u8>
            ) -> Result<(), FSError>
            fn meta(
                self,
                h: &FSHandle
            ) -> Result<Meta, FSError>
```

### src/storage/mem.rs

```rs
        // STRUCTS:
        struct MemStorage
            // PROPERTIES:
            files: std::sync::RwLock<HashMap<FSHandle, Vec<u8>>>, meta: std::sync::RwLock<HashMap<FSHandle, Meta>>

            // METHODS:
            fn new() -> Self
            fn read(
                self,
                h: &FSHandle
            ) -> Result<Vec<u8>, FSError>
            fn write(
                self,
                h: &FSHandle,
                data: Vec<u8>
            ) -> Result<(), FSError>
            fn append(
                self,
                h: &FSHandle,
                data: Vec<u8>
            ) -> Result<(), FSError>
            fn meta(
                self,
                h: &FSHandle
            ) -> Result<Meta, FSError>
```

## tests/builder.rs

```rs
    // FUNCTIONS:
    fn allocator_creates_unique_handles()
    fn build(
        fs: &FS<MemStorage>,
        input: FSInput,
        allocator: &HandleAllocator
    )
    fn builds_nested_directories_and_file()
    fn builds_nested_structure()
    fn builds_single_file()
    fn fs_input(
        pairs: Vec<(&str, NodeType)>
    ) -> FSInput
    fn fsbuilder_disk_builds_successfully()
    fn fsbuilder_mem_builds_successfully()
    fn make_fs() -> (FS<MemStorage>, HandleAllocator)
    fn reuses_existing_directories()
```



# EMPTY FILES
  .DS_Store
  Cargo.toml
  WASM-BUILD.md
  global.md
  llvm.md
  pkg/.gitignore
  pkg/package.json
  pkg/vfs.d.ts
  pkg/vfs.js
  pkg/vfs_bg.js
  pkg/vfs_bg.wasm
  pkg/vfs_bg.wasm.d.ts
  src/fs/mod.rs
  src/fs/trait.rs
  src/lib.rs
  src/storage/mod.rs
  tests/fs.rs
  tests/mod.rs
