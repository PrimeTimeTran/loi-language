use uuid::Uuid;

use crate::backend::utter::registry::UtterRegistry;
use crate::registry::file_meta::{FileMeta, GroupKey};
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone)]
pub struct Registry {
    pub files: Vec<FileMeta>,
    pub files_archive: Vec<FileMeta>,
    pub from_files: Vec<FileMeta>,
    pub stacks: Vec<FileStack>,
    pub active_by_group: HashMap<GroupKey, Uuid>,
}
#[derive(Clone)]
pub struct FileStack {
    pub files: Vec<FileMeta>,
    pub active_file: FileMeta,
}
impl FileStack {
    pub fn group_key(&self) -> GroupKey {
        self.active_file.identity()
    }
}

impl Registry {
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
            .find(|f| f.name == name && f.utter.is_none())
            .or_else(|| self.files_archive.iter().find(|f| f.name == name))
    }

    pub fn is_active(&self, f: &FileMeta) -> bool {
        self.active_by_group
            .get(&f.identity())
            .is_some_and(|id| id == &f.id)
    }

    pub fn find_active(&self, group: &GroupKey) -> Option<&FileMeta> {
        let id = self.active_by_group.get(group)?;
        self.files.iter().find(|f| f.id == *id)
    }

    pub fn build_file(&self, name: &str, utter_reg: &UtterRegistry) {
        if let Some(file) = self.get_active_by_name(name) {
            if let Some(cap) = &file.utter {
                if let Some(utter) = utter_reg.get_utter(cap) {
                    println!("Found utter for {}: {}", name, utter.name());
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

    fn group_key(file: &FileMeta) -> String {
        format!(
            "{}:{}:{}:{}",
            file.namespace,
            file.name,
            file.utter.as_deref().unwrap_or(""),
            file.ext
        )
    }

    pub fn organize(files: Vec<FileMeta>) -> Vec<FileStack> {
        use std::collections::HashMap;

        let mut groups: HashMap<GroupKey, Vec<FileMeta>> = HashMap::new();

        for file in files {
            let key = file.group_key();
            groups.entry(key).or_default().push(file);
        }

        let mut group_vec: Vec<_> = groups.into_iter().collect();
        group_vec.sort_by(|a, b| a.0.cmp(&b.0));

        let mut stacks = Vec::new();

        for (group_key, mut group) in group_vec {
            group.sort_by(|a, b| b.version.cmp(&a.version));

            let active_file = group.remove(0);

            stacks.push(FileStack {
                files: group,
                active_file,
            });
        }

        stacks.sort_by(|a, b| a.active_file.path.cmp(&b.active_file.path));

        stacks
    }
    pub fn scan(root: &Path) -> Self {
        let all_files = Self::discover_files(root);

        let stacks = Self::organize(all_files);

        let active_by_group: HashMap<GroupKey, Uuid> = stacks
            .iter()
            .map(|s| (s.group_key(), s.active_file.id))
            .collect();

        let active: Vec<FileMeta> = stacks.iter().map(|s| s.active_file.clone()).collect();

        let archive: Vec<FileMeta> = stacks.iter().flat_map(|s| s.files.clone()).collect();

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

    pub fn get_active_by_name(&self, name: &str) -> Option<&FileMeta> {
        self.stacks
            .iter()
            .find(|s| s.active_file.name == name)
            .map(|s| &s.active_file)
    }
}
