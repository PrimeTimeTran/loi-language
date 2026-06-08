use crate::backend::utter_registry::UtterRegistry;
use crate::registry::file_meta::FileMetadata;

use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedManifest {
    pub active: Vec<FileMetadata>,
    pub archive: Vec<FileMetadata>,
}

pub struct Registry {
    pub files: Vec<FileMetadata>,
    pub filesArchive: Vec<FileMetadata>,
}

impl Registry {
    pub fn find_file(&self, name: &str) -> Option<&FileMetadata> {
        // 1. Check active files first (most common case)
        self.files
            .iter()
            .find(|f| f.name == name)
            // 2. Fallback to archive if not found
            .or_else(|| self.filesArchive.iter().find(|f| f.name == name))
    }

    /// Explicitly find only in active files
    pub fn find_active(&self, name: &str) -> Option<&FileMetadata> {
        self.files.iter().find(|f| f.name == name)
    }
    pub fn build_file(&self, name: &str, utter_reg: &UtterRegistry) {
        if let Some(file) = self.get_active_by_name(name) {
            if let Some(cap) = &file.capability {
                if let Some(utter) = utter_reg.get_utter(cap) {
                    println!("Found utter for {}: {}", name, utter.name());
                    // Now you can call utter.to_ir(file)
                }
            }
        }
    }
    // PHASE 1: Discovery - Just turn path to metadata
    fn discover_files(root: &Path) -> Vec<FileMetadata> {
        walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("loi"))
            // Pass 'root' into the closure here:
            .map(|e| FileMetadata::from_path(e.path(), root))
            .collect()
    }

    // PHASE 2: Organize - Group metadata into Active vs Archive
    fn organize(files: Vec<FileMetadata>) -> (Vec<FileMetadata>, Vec<FileMetadata>) {
        let mut groups: HashMap<
            (Vec<String>, String, Option<String>, Option<String>),
            Vec<FileMetadata>,
        > = HashMap::new();

        for file in files {
            let key = (
                file.namespace.clone(),
                file.name.clone(),
                file.capability.clone(),
                file.tag.clone(),
            );
            groups.entry(key).or_default().push(file);
        }

        let mut active = Vec::new();
        let mut archive = Vec::new();
        for (_, mut group) in groups {
            group.sort_by(|a, b| b.version.cmp(&a.version));

            active.push(group.remove(0));
            archive.extend(group);
        }

        active.sort();
        (active, archive)
    }

    pub fn scan(root: &Path) -> Self {
        let all_files = Self::discover_files(root);
        let (active, archive) = Self::organize(all_files);

        Registry {
            files: active,
            filesArchive: archive,
        }
    }
    pub fn list_all(&self) {
        for file in &self.files {
            println!(
                "[{}] {} (ver: {})",
                file.namespace.join("/"),
                file.name,
                file.version
            );
        }
    }

    pub fn get_active_by_name(&self, name: &str) -> Option<&FileMetadata> {
        self.files.iter().find(|f| f.name == name)
    }
}

#[cfg(test)]
mod test_utils {
    use std::path::{Path, PathBuf};

    pub fn get_test_root() -> PathBuf {
        PathBuf::from("/virtual/root")
    }
}

#[cfg(test)]
mod tests {
    use crate::registry::{file_meta::FileMetadata, registry::Registry};

    use super::test_utils::get_test_root;
    use pretty_assertions::assert_eq;
    use std::{fs, path::Path};
    use tempfile::tempdir;
    #[test]
    fn test_registry_sorting_and_auto_promotion() {
        let dir = tempdir().unwrap();
        let files = vec![
            "00.index@ui#1.html.loi",
            "00.index@ui#2.html.loi",
            "00.index@ui#3.html.loi",
        ];

        for f in &files {
            fs::write(dir.path().join(f), "").unwrap();
        }

        // 2. Execute: Scan and build registry
        let registry = Registry::scan(dir.path());

        // Fix: It should be 1, because v3 shadows v1 and v2
        assert_eq!(registry.files.len(), 1, "Should only have 1 active file");

        // Check the active file is indeed v3
        assert_eq!(
            registry.files[0].version, 3,
            "Should pick #3 as active version"
        );

        // Check the other 2 are in the archive
        assert_eq!(
            registry.filesArchive.len(),
            2,
            "Should have 2 archived files"
        );
    }

    #[test]
    fn test_metadata_parsing() {
        let path = Path::new("05.dashboard@ui#42.jsx.loi");
        let meta = FileMetadata::from_path(path, &get_test_root());
        assert_eq!(meta.priority, Some(5));
        assert_eq!(meta.name, "dashboard");
        assert_eq!(meta.capability, Some("ui".to_string()));
        assert_eq!(meta.version, 42);
        assert_eq!(meta.extension, "loi");
    }

    #[test]
    fn test_version_auto_promotion() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        let files = vec![
            "00.core@lib#1.js.loi",
            "00.core@lib#3.js.loi",
            "00.core@lib#2.js.loi",
        ];
        for f in &files {
            fs::write(root.join(f), "").unwrap();
        }

        let registry = Registry::scan(root);

        // Assert counts
        assert_eq!(registry.files.len(), 1, "Should only have 1 active file");

        // Assert the correct version was promoted
        let active = &registry.files[0];
        assert_eq!(active.version, 3, "Should have promoted version 3");

        // Assert the name is correct (your parser strips the version/tag part)
        assert_eq!(active.name, "core");

        // Check that the other two are in the archive
        assert_eq!(
            registry.filesArchive.len(),
            2,
            "Should have 2 archived files"
        );
    }

    #[test]
    fn test_capability_grouping() {
        let dir = tempdir().unwrap();

        // Setup: Same name, different capabilities
        // Should NOT shadow each other
        let files = vec!["00.core@lib#1.js.loi", "00.core@ui#1.js.loi"];
        for f in &files {
            fs::write(dir.path().join(f), "").unwrap();
        }

        let registry = Registry::scan(dir.path());

        dbg!("test_capability_grouping", &registry.files);

        assert_eq!(
            registry.files.len(),
            2,
            "Different capabilities should coexist"
        );
    }

    #[test]
    fn test_complex_version_string() {
        let path = Path::new("00.core@lib#10-try-pnpm.js.loi");
        let meta = FileMetadata::from_path(path, &get_test_root());

        assert_eq!(
            meta.version, 10,
            "Should extract version 10 before the hyphen"
        );
    }

    #[test]
    fn test_version_normalization_and_padding() {
        // We want 02, 003, and 0001 to all be parsed as their numeric values (2, 3, 1)
        let files = vec![
            "00.core@lib#02.js.loi",
            "00.core@lib#003.js.loi",
            "00.core@lib#0001.js.loi",
        ];

        let mut results = Vec::new();
        for f in &files {
            results.push(FileMetadata::from_path(Path::new(f), &get_test_root()).version);
        }

        dbg!("test_version_normalization_and_padding", &files);

        assert_eq!(results[0], 2);
        assert_eq!(results[1], 3);
        assert_eq!(results[2], 1);
    }

    #[test]
    fn test_flexible_tag_grouping_and_ordering() {
        let dir = tempdir().unwrap();

        let files = vec![
            "00.app@lib#1-feature.js.loi",
            "00.app@lib#2-bugfix.js.loi",
            "00.app@lib#1-alpha.js.loi",
            "00.app@lib#2-alpha.js.loi",
        ];

        for f in &files {
            fs::write(dir.path().join(f), "").unwrap();
        }

        let registry = Registry::scan(dir.path());

        // Verify Active counts (3 winners)
        assert_eq!(registry.files.len(), 3);

        // Verify Archive counts (1 shadowed file: #1-alpha)
        assert_eq!(registry.filesArchive.len(), 1);

        // Assert the order of the winners
        // Based on Ord: 1. Name, 2. Version (Asc), 3. Tag
        // Sorted result:
        // 0: #1-feature (v1)
        // 1: #2-alpha   (v2)
        // 2: #2-bugfix  (v2)

        assert_eq!(registry.files[0].tag.as_deref(), Some("feature"));
        assert_eq!(registry.files[1].tag.as_deref(), Some("alpha"));
        assert_eq!(registry.files[2].tag.as_deref(), Some("bugfix"));
    }

    #[test]
    fn test_lexicographical_ordering() {
        let dir = tempdir().unwrap();

        let files = vec!["00.b@ui#1.html.loi", "00.a@ui#1.html.loi"];

        for f in &files {
            fs::write(dir.path().join(f), "").unwrap();
        }

        let registry = Registry::scan(dir.path());
        dbg!("test_lexicographical_ordering", &registry.files);
        assert_eq!(registry.files[0].name, "a");
        assert_eq!(registry.files[1].name, "b");
    }

    #[test]
    fn test_version_collision_tie_break() {
        let dir = tempdir().unwrap();

        let files = vec!["00.app@ui#1.html.loi", "00.app@ui#1.html.loi"];

        for f in &files {
            fs::write(dir.path().join(f), "").unwrap();
        }

        dbg!("test_version_collision_tie_break", &files);

        let registry = Registry::scan(dir.path());
        assert_eq!(registry.files.len(), 1);
    }

    #[test]
    fn test_recursive_namespace_ordering() {
        //
        let files = vec![
            "a.constants.loi",
            "a.b.constants.loi",
            "a.a.constants.loi", // Should sort before a.b
        ];
        // Registry::scan should output them as:
        // 1. a.constants
        // 2. a.a.constants
        // 3. a.b.constants
    }
}
