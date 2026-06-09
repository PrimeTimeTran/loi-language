use uuid::Uuid;

use crate::backend::utter::registry::UtterRegistry;
use crate::registry::file_meta::{FileMeta, group_key};

use std::collections::HashMap;
use std::path::Path;

#[derive(Clone)]
pub struct Registry {
    pub files: Vec<FileMeta>,
    pub files_archive: Vec<FileMeta>,
    pub from_files: Vec<FileMeta>,
    pub stacks: Vec<FileStack>,
    pub active_by_group: HashMap<String, Uuid>,
}
#[derive(Clone)]
pub struct FileStack {
    pub group_key: String,
    pub active_file: FileMeta,
    pub archive_files: Vec<FileMeta>,
}

impl Registry {
    pub fn group_key(fs_name: &str) -> String {
        let mut result = String::with_capacity(fs_name.len());
        let mut chars = fs_name.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '#' {
                // skip until '.' or end
                while let Some(&next) = chars.peek() {
                    if next == '.' {
                        break;
                    }
                    chars.next();
                }
                continue;
            }
            result.push(c);
        }

        result
    }

    pub fn from_files(files: Vec<FileMeta>) -> Self {
        Registry {
            files,
            files_archive: Vec::new(),
            from_files: Vec::new(),
            stacks: Vec::new(),
            active_by_group: HashMap::new(),
        }
    }
    pub fn find_file(&self, name: &str) -> Option<&FileMeta> {
        self.files
            .iter()
            .find(|f| f.name == name)
            // 2. Fallback to archive if not found
            .or_else(|| self.files_archive.iter().find(|f| f.name == name))
    }

    pub fn is_active(&self, f: &FileMeta) -> bool {
        self.active_by_group
            .get(&f.name)
            .is_some_and(|id| id == &f.id)
    }

    pub fn find_active(&self, group: &str) -> Option<&FileMeta> {
        let id = self.active_by_group.get(group)?;
        self.files.iter().find(|f| f.id == *id)
    }

    pub fn build_file(&self, name: &str, utter_reg: &UtterRegistry) {
        if let Some(file) = self.get_active_by_name(name) {
            if let Some(cap) = &file.utter {
                if let Some(utter) = utter_reg.get_utter(cap) {
                    println!("Found utter for {}: {}", name, utter.name());
                    // Now you can call utter.to_ir(file)
                }
            }
        }
    }
    fn discover_files(root: &Path) -> Vec<FileMeta> {
        walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("loi"))
            .map(|e| FileMeta::from_path(e.path(), root))
            .collect()
    }

    fn organize(files: Vec<FileMeta>) -> Vec<FileStack> {
        use std::collections::HashMap;

        let mut groups: HashMap<String, Vec<FileMeta>> = HashMap::new();

        for file in files {
            groups.entry(file.group_key()).or_default().push(file);
        }

        let mut group_vec: Vec<_> = groups.into_iter().collect();
        group_vec.sort_by(|a, b| a.0.cmp(&b.0));

        let mut stacks = Vec::new();

        for (_, mut group) in group_vec {
            group.sort_by(|a, b| b.version.cmp(&a.version));

            let active_file = group.remove(0);
            let archive_files = group;
            let group_key = group_key(&active_file.filename);

            stacks.push(FileStack {
                group_key,
                active_file,
                archive_files,
            });
        }

        stacks.sort_by(|a, b| a.active_file.path.cmp(&b.active_file.path));

        stacks
    }
    pub fn scan(root: &Path) -> Self {
        let all_files = Self::discover_files(root);

        // organize MUST use group_key internally
        let stacks = Self::organize(all_files);

        let active_by_group: HashMap<String, Uuid> = stacks
            .iter()
            .map(|s| (s.group_key.clone(), s.active_file.id))
            .collect();

        let active: Vec<FileMeta> = stacks.iter().map(|s| s.active_file.clone()).collect();
        let archive: Vec<FileMeta> = stacks
            .iter()
            .flat_map(|s| s.archive_files.clone())
            .collect();

        Registry {
            active_by_group,
            files: active,
            files_archive: archive,
            from_files: Vec::new(),
            stacks,
        }
    }
    pub fn list_all(&self) {
        for file in &self.files {
            println!("[{}] {} (ver: {})", file.namespace, file.name, file.version);
        }
    }

    fn resolve_versioning(&mut self) {
        let mut identity_groups: HashMap<(String, Option<String>, String), Vec<*mut FileMeta>> =
            HashMap::new();

        for file in &mut self.files {
            let key = (file.name.clone(), file.utter.clone(), file.ext.clone());
            identity_groups
                .entry(key)
                .or_default()
                .push(file as *mut FileMeta);
        }

        for group in identity_groups.values() {
            let max_version = group
                .iter()
                .map(|&f| unsafe { (*f).version })
                .max()
                .unwrap_or(0);

            for &file_ptr in group {
                unsafe {
                    (*file_ptr).active = (*file_ptr).version == max_version;
                }
            }
        }
    }

    pub fn get_active_by_name(&self, name: &str) -> Option<&FileMeta> {
        self.files.iter().find(|f| f.name == name)
    }
}
