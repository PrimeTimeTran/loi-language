use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct FileMetadata {
    pub namespace: Vec<String>,
    pub name: String,
    // Core metadata
    pub version: u32,
    pub priority: Option<u8>,
    pub tag: Option<String>,
    pub capability: Option<String>,

    // For when you need to write it back to disk
    pub extension: String,
    pub path: PathBuf,
}

impl FileMetadata {
    fn parse_metadata(raw: Option<&str>) -> (Option<String>, u32, Option<String>) {
        let Some(s) = raw else { return (None, 0, None) };

        let cap_ver: Vec<&str> = s.split('#').collect();
        let capability = Some(cap_ver[0].to_string());

        if cap_ver.len() < 2 {
            return (capability, 0, None);
        }

        // Logic to extract version and tag
        let raw_ver_tag = cap_ver[1];
        let digits: String = raw_ver_tag.chars().take_while(|c| c.is_numeric()).collect();
        let version = digits.parse::<u32>().unwrap_or(0);

        let remainder = &raw_ver_tag[digits.len()..];
        let tag = if remainder.starts_with('-') {
            Some(remainder.trim_start_matches('-').to_string())
        } else {
            None
        };

        (capability, version, tag)
    }
    pub fn from_path(path: &Path, root: &Path) -> Self {
        // 1. Calculate Namespace from directory structure (relative to root)
        let relative = path.strip_prefix(root).unwrap_or(path);
        let mut namespace: Vec<String> = relative
            .parent()
            .map(|p| {
                p.components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();

        // 2. Parse Filename
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let parts: Vec<&str> = stem.split('.').collect();

        let (priority, start_idx) = if parts.len() > 0 && parts[0].parse::<u8>().is_ok() {
            namespace.push(parts[0].to_string());
            (Some(parts[0].parse::<u8>().unwrap()), 1)
        } else {
            (None, 0)
        };

        // SAFE ACCESS: Check if start_idx is within bounds
        let name_part = if start_idx < parts.len() {
            parts[start_idx]
        } else {
            // Handle files with no name part, e.g., "00.loi"
            "unknown"
        };

        let cap_split: Vec<&str> = name_part.split('@').collect();
        let name = cap_split[0].to_string();
        // Logic for Capability, Version, Tag remains similar but uses name_part
        let (capability, version, tag) = Self::parse_metadata(cap_split.get(1).copied());

        FileMetadata {
            namespace,
            name,
            priority,
            tag,
            capability,
            version,
            path: path.to_path_buf(),
            extension: path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string(),
        }
    }
}

/*
Expected filename support patterns:
1. "01-arithmetic.loi"           (Priority 1, name "arithmetic")
2. "index@ui.html.loi"           (Name "index", cap "ui", ext "html")
3. "styles@ui#1.css.loi"         (Name "styles", cap "ui", ver 1, ext "css")
4. "script@lib#2-beta.js.loi"    (Name "script", cap "lib", ver 2, tag "beta", ext "js")
5. "05.loi"                      (Priority 5, name "unknown")
6. "auth@api#10-prod.json.loi"   (Name "auth", cap "api", ver 10, tag "prod")
7. "profile@ui.loi"              (No explicit ext in logic, defaults handled)
8. "base@core#1.loi"             (Name "base", cap "core", ver 1)
9. "data@store#99-debug.bin.loi" (Complex metadata)
10. "utils@helper.loi"           (Name "utils", cap "helper")
11. "02@ui#1.html.loi"           (Priority 2, name "ui", cap "ui"?) -> Needs careful parsing
12. "test#0-run.loi"             (Name "test", ver 0, tag "run")
13. "app@web#5.js.loi"           (Name "app", cap "web", ver 5)
14. "config#1-local.toml.loi"    (Name "config", ver 1, tag "local")
15. "simple.loi"                 (No priority, no metadata)
*/

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_standard_file() {
        let path = PathBuf::from("root/simple.loi");
        let root = PathBuf::from("root");
        let meta = FileMetadata::from_path(&path, &root);

        assert_eq!(meta.name, "simple");
        assert_eq!(meta.extension, "loi");
        assert!(meta.capability.is_none());
    }

    #[test]
    fn test_priority_and_metadata() {
        // Filename: 03@ui#1-alpha.html.loi
        let path = PathBuf::from("root/03@ui#1-alpha.html.loi");
        let root = PathBuf::from("root");
        let meta = FileMetadata::from_path(&path, &root);

        assert_eq!(meta.priority, Some(3));
        assert_eq!(meta.name, "ui"); // Because 03 is the priority, "ui" is the name part
        assert_eq!(meta.capability, Some("ui".to_string()));
        assert_eq!(meta.version, 1);
        assert_eq!(meta.tag, Some("alpha".to_string()));
        assert_eq!(meta.extension, "loi");
    }

    #[test]
    fn test_namespace_derivation() {
        let path = PathBuf::from("root/api/v1/user@auth.json.loi");
        let root = PathBuf::from("root");
        let meta = FileMetadata::from_path(&path, &root);

        assert_eq!(meta.namespace, vec!["api", "v1"]);
        assert_eq!(meta.name, "user");
        assert_eq!(meta.capability, Some("auth".to_string()));
    }

    #[test]
    fn test_missing_name_part() {
        let path = PathBuf::from("root/05.loi");
        let root = PathBuf::from("root");
        let meta = FileMetadata::from_path(&path, &root);

        assert_eq!(meta.priority, Some(5));
        assert_eq!(meta.name, "unknown");
    }
}
