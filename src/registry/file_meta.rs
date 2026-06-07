use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct FileMetadata {
    // The "Virtual Path": ["00", "3", "a", "prebuild-next"]
    // This replaces filename and segments.
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
