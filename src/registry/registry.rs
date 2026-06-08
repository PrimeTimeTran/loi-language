use crate::backend::utter::registry::UtterRegistry;
use crate::registry::file_meta::FileMeta;

use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedManifest {
    pub active: Vec<FileMeta>,
    pub archive: Vec<FileMeta>,
}
#[derive(Clone)]
pub struct Registry {
    pub files: Vec<FileMeta>,
    pub files_archive: Vec<FileMeta>,
    pub from_files: Vec<FileMeta>,
    pub stacks: Vec<FileStack>,
}
#[derive(Clone)]
pub struct FileStack {
    pub active_file: FileMeta,
    pub archive_files: Vec<FileMeta>,
}

impl Registry {
    pub fn from_files(files: Vec<FileMeta>) -> Self {
        Registry {
            files,
            files_archive: Vec::new(),
            from_files: Vec::new(),
            stacks: Vec::new(),
        }
    }
    pub fn find_file(&self, name: &str) -> Option<&FileMeta> {
        self.files
            .iter()
            .find(|f| f.name == name)
            // 2. Fallback to archive if not found
            .or_else(|| self.files_archive.iter().find(|f| f.name == name))
    }

    pub fn find_active(&self, name: &str) -> Option<&FileMeta> {
        self.files.iter().find(|f| f.name == name)
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
    // PHASE 1: Discovery - Just turn path to metadata
    fn discover_files(root: &Path) -> Vec<FileMeta> {
        walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("loi"))
            .map(|e| FileMeta::from_path(e.path(), root))
            .collect()
    }

    fn organize(files: Vec<FileMeta>) -> (Vec<FileMeta>, Vec<FileMeta>, Vec<FileStack>) {
        use std::collections::HashMap;

        let mut groups: HashMap<(String, String, Option<String>, String), Vec<FileMeta>> =
            HashMap::new();

        for file in files {
            let key = (
                file.namespace.clone(),
                file.name.clone(),
                file.utter.clone(),
                file.ext.clone(),
            );

            groups.entry(key).or_default().push(file);
        }

        let mut active = Vec::new();
        let mut archive = Vec::new();
        let mut stacks = Vec::new();

        for (_, mut group) in groups {
            group.sort_by(|a, b| b.version.cmp(&a.version));

            if let Some(head) = group.first_mut() {
                head.active = true;
            }

            let head = group.remove(0);

            active.push(head.clone());

            for item in group.into_iter() {
                archive.push(item);
            }

            let active_name = head.name.clone();

            stacks.push(FileStack {
                active_file: head,
                archive_files: archive
                    .iter()
                    .filter(|f: &&FileMeta| f.name == active_name)
                    .cloned()
                    .collect(),
            });
        }

        active.sort_by(|a, b| a.path.cmp(&b.path));
        archive.sort_by(|a, b| a.path.cmp(&b.path));
        stacks.sort_by(|a, b| a.active_file.path.cmp(&b.active_file.path));

        (active, archive, stacks)
    }
    pub fn scan(root: &Path) -> Self {
        let all_files = Self::discover_files(root);
        let (active, archive, stacks) = Self::organize(all_files);

        Registry {
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

        // 1. Group pointers to all files by their identity
        for file in &mut self.files {
            let key = (file.name.clone(), file.utter.clone(), file.ext.clone());
            identity_groups
                .entry(key)
                .or_default()
                .push(file as *mut FileMeta);
        }

        // 2. Resolve versioning for each group
        for group in identity_groups.values() {
            let max_version = group
                .iter()
                .map(|&f| unsafe { (*f).version })
                .max()
                .unwrap_or(0);

            for &file_ptr in group {
                unsafe {
                    // Only the highest version (or version 0 if no versions exist) is active
                    (*file_ptr).active = (*file_ptr).version == max_version;
                }
            }
        }
    }

    pub fn get_active_by_name(&self, name: &str) -> Option<&FileMeta> {
        self.files.iter().find(|f| f.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{file_meta::FileMeta, registry::Registry};
    use std::{fs, path::Path};
    use tempfile::tempdir;

    // --- DOMAIN 1: Filename Parsing Logic (The "Parser" Unit Tests) ---
    mod parsing {
        use crate::registry::test_utils::test_helpers::get_test_root;

        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn test_metadata_parsing() {
            let path = Path::new("05.dashboard@ui#42.jsx.loi");
            let meta = FileMeta::from_path(path, &get_test_root());
            assert_eq!(meta.priority, Some(5));
            assert_eq!(meta.name, "dashboard");
            assert_eq!(meta.utter, Some("ui".to_string()));
            assert_eq!(meta.version, 42);
            assert_eq!(meta.ext, "jsx");
        }

        #[test]
        fn test_complex_version_string() {
            let path = Path::new("00.core@lib#10-try-pnpm.js.loi");
            let meta = FileMeta::from_path(path, &get_test_root());
            assert_eq!(meta.version, 10);
        }

        #[test]
        fn test_version_normalization() {
            let files = vec!["00.core@lib#02.js.loi", "00.core@lib#003.js.loi"];
            let results: Vec<u32> = files
                .iter()
                .map(|f| FileMeta::from_path(Path::new(f), &get_test_root()).version)
                .collect();
            assert_eq!(results, vec![2, 3]);
        }
    }

    // --- DOMAIN 2: Resolution & Shadowing (Integration Tests) ---
    mod resolution {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn test_version_auto_promotion() {
            let dir = tempdir().unwrap();
            for f in &[
                "00.core@lib#1.js.loi",
                "00.core@lib#3.js.loi",
                "00.core@lib#2.js.loi",
            ] {
                fs::write(dir.path().join(f), "").unwrap();
            }
            let registry = Registry::scan(dir.path());
            assert_eq!(registry.files.len(), 1);
            assert_eq!(registry.files[0].version, 3);
        }

        #[test]
        fn test_utter_grouping() {
            let dir = tempdir().unwrap();
            for f in &["00.core@lib#1.js.loi", "00.core@ui#1.js.loi"] {
                fs::write(dir.path().join(f), "").unwrap();
            }
            let registry = Registry::scan(dir.path());
            assert_eq!(
                registry.files.len(),
                2,
                "Capabilities should group independently"
            );
        }

        #[test]
        fn test_version_collision_tie_break() {
            let dir = tempdir().unwrap();
            let f = "00.app@ui#1.html.loi";
            fs::write(dir.path().join(f), "").unwrap();
            fs::write(dir.path().join(f), "").unwrap();
            let registry = Registry::scan(dir.path());
            assert_eq!(registry.files.len(), 1);
        }
    }

    // --- DOMAIN 3: Sorting & Lifecycle ---
    mod lifecycle {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn test_lexicographical_ordering() {
            let dir = tempdir().unwrap();
            for f in &["00.b@ui#1.html.loi", "00.a@ui#1.html.loi"] {
                fs::write(dir.path().join(f), "").unwrap();
            }
            let registry = Registry::scan(dir.path());
            assert_eq!(registry.files[0].name, "a");
        }

        #[test]
        fn test_tag_grouping_and_ordering() {
            let dir = tempdir().unwrap();
            let files = vec![
                "00.app@lib#1-feature.js.loi",
                "00.app@lib#2-alpha.js.loi",
                "00.app@lib#2-bugfix.js.loi",
            ];
            for f in &files {
                fs::write(dir.path().join(f), "").unwrap();
            }

            let registry = Registry::scan(dir.path());
            assert_eq!(registry.files[0].tag.as_deref(), Some("feature"));
            assert_eq!(registry.files[1].tag.as_deref(), Some("alpha"));
            assert_eq!(registry.files[2].tag.as_deref(), Some("bugfix"));
        }
    }
}
