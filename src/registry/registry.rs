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
    // PHASE 1: Discovery - Just turn path to metadata
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

            stacks.push(FileStack {
                active_file,
                archive_files,
            });
        }

        stacks.sort_by(|a, b| a.active_file.path.cmp(&b.active_file.path));

        stacks
    }
    pub fn scan(root: &Path) -> Self {
        let all_files = Self::discover_files(root);
        let stacks = Self::organize(all_files);

        let mut active_by_group: HashMap<String, Uuid> = HashMap::new();

        for stack in &stacks {
            let key = stack.active_file.group_key();
            active_by_group.insert(key, stack.active_file.id);
        }

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

        // 2. Resolve versioning for each group
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::file_meta::FileMeta;
    use crate::registry::registry::Registry;
    use std::{fs, path::Path};
    use tempfile::tempdir;

    mod parsing {
        use super::*;
        use crate::registry::test_utils::test_helpers::get_test_root;
        use pretty_assertions::assert_eq;

        #[test]
        fn parse_standard_filename_format_succeeds() {
            let path = Path::new("05.dashboard@ui#42.jsx.loi");
            let meta = FileMeta::from_path(path, &get_test_root());
            assert_eq!(meta.name, "dashboard");
            assert_eq!(meta.utter, Some("ui".to_string()));
            assert_eq!(meta.version, 42);
            assert_eq!(meta.ext, "jsx");
        }

        #[test]
        fn parse_version_with_suffix_extracts_base_integer() {
            let path = Path::new("00.core@lib#10-try-pnpm.js.loi");
            let meta = FileMeta::from_path(path, &get_test_root());
            assert_eq!(meta.version, 10);
        }
    }

    mod resolution {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn scan_multiple_versions_keeps_only_highest_version() {
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
        fn scan_distinct_utters_maintains_separate_entries() {
            let dir = tempdir().unwrap();
            for f in &["00.core@lib#1.js.loi", "00.core@ui#1.js.loi"] {
                fs::write(dir.path().join(f), "").unwrap();
            }
            let registry = Registry::scan(dir.path());
            assert_eq!(registry.files.len(), 2);
        }

        #[test]
        fn scan_duplicate_filenames_deduplicates_entry() {
            let dir = tempdir().unwrap();
            let f = "00.app@ui#1.html.loi";
            fs::write(dir.path().join(f), "").unwrap();
            fs::write(dir.path().join(f), "").unwrap();
            let registry = Registry::scan(dir.path());
            assert_eq!(registry.files.len(), 1);
        }
    }

    mod ordering {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn scan_files_orders_lexicographically_by_name() {
            let dir = tempdir().unwrap();
            for f in &["b@html.loi", "a@html.loi"] {
                fs::write(dir.path().join(f), "").unwrap();
            }
            let registry = Registry::scan(dir.path());
            assert_eq!(registry.files[0].name, "a");
            assert_eq!(registry.files[1].name, "b");
        }
    }

    mod groups {
        use crate::registry::file_meta::group_key;

        use super::*;

        #[test]
        fn groups_base_and_versions() {
            assert_eq!(group_key("file.loi"), group_key("file#1.loi"));

            assert_eq!(group_key("file.loi"), group_key("file#3.loi"));
        }

        #[test]
        fn groups_numbered_namespaces() {
            assert_eq!(group_key("00.file.loi"), group_key("00.file#1.loi"));

            assert_eq!(group_key("00.file.loi"), group_key("00.file#3.loi"));
        }

        #[test]
        fn groups_tagged_files() {
            assert_eq!(group_key("00.file@lib.loi"), group_key("00.file@lib#1.loi"));

            assert_eq!(group_key("00.file@lib.loi"), group_key("00.file@lib#3.loi"));
        }

        #[test]
        fn preserves_tag_when_grouping() {
            assert_ne!(group_key("00.file.loi"), group_key("00.file@lib.loi"));
        }

        #[test]
        fn preserves_namespace_when_grouping() {
            assert_ne!(group_key("00.file.loi"), group_key("01.file.loi"));
        }

        #[test]
        fn different_tags_are_different_groups() {
            assert_ne!(group_key("00.file@lib.loi"), group_key("00.file@test.loi"));
        }

        #[test]
        fn strips_version_before_extension() {
            assert_eq!(group_key("file#123.loi"), "file.loi");
        }

        #[test]
        fn strips_version_before_tag() {
            assert_eq!(group_key("file#123@lib.loi"), "file@lib.loi");
        }

        #[test]
        fn multiple_versions_group_together() {
            let key = group_key("file.loi");

            assert_eq!(key, group_key("file#1.loi"));
            assert_eq!(key, group_key("file#2.loi"));
            assert_eq!(key, group_key("file#999.loi"));
        }
        #[test]
        fn normalization_examples() {
            assert_eq!(group_key("file.loi"), "file.loi");
            assert_eq!(group_key("file#1.loi"), "file.loi");
            assert_eq!(group_key("file#3.loi"), "file.loi");

            assert_eq!(group_key("00.file.loi"), "00.file.loi");
            assert_eq!(group_key("00.file#1.loi"), "00.file.loi");

            assert_eq!(group_key("00.file@lib.loi"), "00.file@lib.loi");
            assert_eq!(group_key("00.file@lib#3.loi"), "00.file@lib.loi");

            assert_eq!(group_key("file#123@lib.loi"), "file@lib.loi");
        }
    }
}
