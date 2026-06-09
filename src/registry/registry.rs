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

    pub fn build_source(root: &Path) -> Vec<FileMeta> {
        walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("loi"))
            .map(|e| FileMeta::from_path(e.path(), root))
            .collect()
    }

    pub fn organize(files: Vec<FileMeta>) -> Vec<FileStack> {
        let mut groups: HashMap<GroupKey, Vec<FileMeta>> = HashMap::new();
        for file in files {
            groups.entry(file.group_key()).or_default().push(file);
        }
        let mut stacks = Vec::new();
        for (_identity, mut group) in groups {
            group.sort_by(|a, b| b.version.cmp(&a.version));

            let active_file = group[0].clone();

            stacks.push(FileStack {
                files: group,
                active_file,
            });
        }

        stacks
    }

    pub fn scan(root: &Path) -> Self {
        let all_files = Self::build_source(root);
        let mut stacks = Self::organize(all_files);

        let sort_logic = |a: &FileMeta, b: &FileMeta| {
            let (ka, kb) = (a.group_key(), b.group_key());
            ka.namespace.cmp(&kb.namespace).then_with(|| {
                match (ka.name.parse::<u64>(), kb.name.parse::<u64>()) {
                    (Ok(n1), Ok(n2)) => n1.cmp(&n2),
                    _ => ka.name.cmp(&kb.name),
                }
            })
        };

        stacks.sort_by(|a, b| sort_logic(&a.active_file, &b.active_file));
        let files = stacks.iter().map(|s| s.active_file.clone()).collect();
        let files_archive = stacks
            .iter()
            .flat_map(|s| s.files.iter().cloned())
            .collect();

        let active_by_group = stacks
            .iter()
            .map(|s| (s.group_key(), s.active_file.id))
            .collect();

        Registry {
            active_by_group,
            files,
            files_archive,
            from_files: Vec::new(),
            stacks,
        }
    }

    pub fn is_active(&self, f: &FileMeta) -> bool {
        self.active_by_group.get(&f.group_key()) == Some(&f.id)
    }

    pub fn find_active_by_name(&self, name: &str) -> Option<&FileMeta> {
        self.stacks
            .iter()
            .find(|s| s.active_file.name == name)
            .map(|s| &s.active_file)
    }
    pub fn list_all(&self) {
        for file in &self.files {
            println!("[{}] {} (ver: {})", file.namespace, file.name, file.version);
        }
    }
}
