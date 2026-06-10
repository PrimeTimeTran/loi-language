use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct FileMeta {
    pub id: Uuid,
    pub filename: String,

    // <priority/namespace>.<name>@<utter>$<variant>#<version>-<tag>.<ext>.loi
    pub namespace: String,
    pub name: String,
    pub utter: Option<String>,
    pub version: u32,
    pub tag: Option<String>,
    pub variant: Option<String>,
    pub ext: String,

    // Metadata Meta
    pub path: PathBuf,
    pub active: bool,
    pub capabilities: Vec<String>,
}
impl Default for FileMeta {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            namespace: String::new(),
            filename: String::new(),
            name: String::new(),

            utter: None,
            version: 0,
            tag: None,
            variant: None,

            ext: "loi".to_string(),

            path: PathBuf::new(),
            active: true,
            capabilities: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GroupKey {
    pub namespace: String,
    pub name: String,
    pub utter: Option<String>,
    pub variant: Option<String>,
    pub ext: String,
}

impl FileMeta {
    pub fn identity(&self) -> GroupKey {
        GroupKey {
            namespace: self.namespace.clone(),
            name: self.name.clone(),
            utter: self.utter.clone(),
            variant: None,
            ext: self.ext.clone(),
        }
    }
    pub fn group_key(&self) -> GroupKey {
        self.identity()
    }

    pub fn new(stem: &str, filename: String, path: PathBuf, dedup: bool) -> Self {
        // Centralized parsing logic
        let (identity, meta_part) = stem.split_once('#').unwrap_or((stem, ""));

        let mut file = Self {
            id: Uuid::new_v4(),
            filename: filename.clone(),
            path,

            // Map the parsed data directly into the struct
            namespace: Self::get_namespace(stem),
            name: Self::get_name(stem),
            utter: Self::get_utter(identity),
            version: Self::get_version(meta_part),
            tag: Self::get_tag(stem),
            variant: Self::get_variant(stem),

            ext: Self::get_ext(&filename),
            active: true,
            capabilities: Vec::new(),
            ..Default::default()
        };

        file.capabilities = Self::infer_capabilities(&file.ext);

        if dedup {
            file.capabilities.sort();
            file.capabilities.dedup();
        }

        file
    }

    pub fn from_path(path: &Path, _root: &Path) -> Self {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let stem = filename.strip_suffix(".loi").unwrap_or(filename);
        Self::new(stem, filename.to_string(), path.to_path_buf(), true)
    }

    fn get_name(input: &str) -> String {
        // 1. shift origin if namespace exists
        let after_namespace = match input.split_once('!') {
            Some((_, rest)) => rest,
            None => input,
        };

        // 2. name ends at first identity/meta delimiter
        after_namespace
            .split(&['@', '#', '$', '.'][..])
            .next()
            .unwrap_or(after_namespace)
            .to_string()
    }
    fn get_namespace(input: &str) -> String {
        Self::parse_namespace(input)
            .0
            .unwrap_or_else(|| "core".to_string())
    }

    fn parse_namespace(input: &str) -> (Option<String>, &str) {
        match input.split_once('!') {
            Some((ns, rest)) => (Some(ns.to_string()), rest),
            None => (None, input),
        }
    }

    fn get_utter(identity_part: &str) -> Option<String> {
        let start = identity_part.find('@')? + 1;
        let end_anchors = ['#', '.', '-'];
        let end = identity_part[start..]
            .find(|c| end_anchors.contains(&c))
            .map(|idx| start + idx)
            .unwrap_or(identity_part.len());

        let utter = &identity_part[start..end];

        if utter.is_empty() {
            None
        } else {
            Some(utter.to_string())
        }
    }

    fn get_version(meta: &str) -> u32 {
        let version_str: String = meta.chars().take_while(|c| c.is_ascii_digit()).collect();
        version_str.parse::<u32>().unwrap_or(0)
    }

    fn get_variant(stem: &str) -> Option<String> {
        let start = stem.find('$')? + 1;
        let s = &stem[start..];

        let end = s.find(&['#', '.', '-'][..]).unwrap_or(s.len());

        let v = &s[..end];

        if v.is_empty() {
            None
        } else {
            Some(format!("${}", v))
        }
    }

    fn get_tag(stem: &str) -> Option<String> {
        let meta_block = stem.split('#').nth(1)?.split('.').next()?;
        let rest = meta_block.split_once('-')?.1;
        let tag = rest.split('$').next()?;
        if tag.is_empty() {
            None
        } else {
            Some(tag.to_string())
        }
    }

    fn get_ext(filename: &str) -> String {
        let stem = filename.strip_suffix(".loi").unwrap_or(filename);
        let whitelist = ["html", "json", "js", "jsx", "ts", "tsx", "md", "mdx", "css"];

        // find last "." segment
        let last = stem.rsplit('.').next().unwrap_or("");

        if whitelist.contains(&last) {
            return last.to_string();
        }

        "loi".to_string()
    }
    fn capabilities(&self, parsed: &ParsedPath) -> Vec<String> {
        let mut caps = vec![];
        if parsed.variant.as_deref() == Some("ui") {
            caps.push("ui".to_string());
        }
        if parsed.version > 0 {
            caps.push("versioned".to_string());
        }
        caps.sort();
        caps.dedup();
        caps
    }

    fn infer_capabilities(ext: &str) -> Vec<String> {
        let mut caps = Vec::new();

        match ext {
            "js" | "jsx" | "ts" | "tsx" => caps.push("scripting"),
            "html" => caps.push("markup"),
            "css" => caps.push("styling"),
            "md" => caps.push("document"),
            "mdx" => {
                caps.push("document");
                caps.push("ui");
            }
            _ => {}
        }

        caps.into_iter().map(String::from).collect()
    }

    pub fn get_fs_name(&self) -> String {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    pub fn wrapped_extension(&self) -> Option<&str> {
        let file_name = self.path.file_name()?.to_str()?;
        let base = file_name.strip_suffix(".loi")?;
        let ext = base.rsplit('.').next()?;
        match ext {
            "html" | "css" | "js" => Some(ext),
            _ => None,
        }
    }

    pub fn is_loi(&self) -> bool {
        self.path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e == "loi")
    }

    pub fn is_wrapped_loi(&self) -> bool {
        self.wrapped_extension().is_some()
    }
}

pub struct ParsedPath {
    pub variant: Option<String>,
    pub version: u32,
    pub is_versioned: bool,
    pub is_ui: bool,
}
impl From<&Path> for ParsedPath {
    fn from(path: &Path) -> Self {
        let s = path.to_string_lossy();
        let is_ui = s.contains("@ui");
        let version = s
            .split('#')
            .nth(1)
            .and_then(|s| {
                s.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<u32>()
                    .ok()
            })
            .unwrap_or(0);

        let is_versioned = s.contains('#') || true;

        let variant = s
            .split('$')
            .nth(1)
            .map(|s| s.split('.').next().unwrap_or("").to_string())
            .filter(|s| !s.is_empty());

        Self {
            variant,
            version,
            is_versioned,
            is_ui,
        }
    }
}

impl FileMeta {
    pub fn mock(filename: &str) -> Self {
        let stem = filename
            .strip_suffix(".loi")
            .unwrap_or(filename)
            .to_string();
        Self::new(&stem, filename.to_string(), PathBuf::from(filename), true)
    }
}
