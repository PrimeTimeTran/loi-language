use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct FileMeta {
    pub id: Uuid,
    pub filename: String,

    // <priority/namespace>.<name>@<utter>#<version>-<tag>$<variant>.<ext>.loi
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

impl FileMeta {
    pub fn identity(&self) -> GroupKey {
        GroupKey {
            namespace: self.namespace.clone(),
            name: self.name.clone(),
            utter: self.utter.clone(),
            variant: self.variant.clone(),
            ext: self.ext.clone(),
        }
    }
    pub fn group_key(&self) -> GroupKey {
        self.identity()
    }

    pub fn variant_key(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            self.namespace,
            self.name,
            self.utter.as_deref().unwrap_or(""),
            self.ext,
            self.variant.as_deref().unwrap_or("")
        )
    }

    pub fn from_path(path: &Path, _root: &Path) -> Self {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let stem = filename.strip_suffix(".loi").unwrap_or(filename);
        let (identity_part, meta_part) = stem.split_once('#').unwrap_or((stem, ""));
        let (namespace, name, _, _) = Self::parse_identity(identity_part);
        let version = Self::get_version(meta_part);
        let tag = Self::get_tag(stem);
        let variant = Self::get_variant(stem);
        let ext = Self::get_ext(stem);
        let utter = Self::get_utter(filename);
        let id = Uuid::new_v4();

        let mut file = Self {
            id,
            filename: filename.to_string(),
            namespace,
            name,
            utter,
            version,
            tag,
            variant,
            ext: ext.clone(),
            path: path.to_path_buf(),
            active: true,
            capabilities: Vec::new(),
        };

        file.capabilities = Self::infer_capabilities(&file.ext);

        file.capabilities.sort();
        file.capabilities.dedup();

        file
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

    fn parse_identity(identity: &str) -> (String, String, Option<String>, String) {
        let known_extensions = ["html", "js", "css", "json"];
        let mut core = identity;
        let mut ext = "loi";
        for e in &known_extensions {
            if identity.ends_with(&format!(".{}", e)) {
                core = &identity[..identity.len() - (e.len() + 1)];
                ext = e;
                break;
            }
        }

        let (namespace, name_with_utter) = match core.split_once('.') {
            Some((ns, n)) => (ns.to_string(), n.to_string()),
            None => ("core".to_string(), core.to_string()),
        };

        let (name, utter) = match name_with_utter.split_once('@') {
            Some((n, u)) => (n.to_string(), Some(u.to_string())),
            None => (name_with_utter, None),
        };

        (namespace, name, utter, ext.to_string())
    }

    fn get_variant(stem: &str) -> Option<String> {
        let after_dollar = stem.split('$').nth(1)?;
        let variant = after_dollar.split('.').next()?;
        if variant.is_empty() {
            None
        } else {
            Some(format!("${}", variant))
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
        let whitelist = ["html", "json", "js", "jsx", "ts", "tsx", "md", "mdx", "css"];
        let parts: Vec<&str> = filename.split('.').collect();
        for i in (0..parts.len()).rev() {
            if whitelist.contains(&parts[i]) {
                return parts[i].to_string();
            }
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
        let path = std::path::PathBuf::from(filename);

        let filename_str = filename.to_string();

        let stem = filename.strip_suffix(".loi").unwrap_or(filename);

        let (identity, meta) = stem.split_once('#').unwrap_or((stem, ""));

        let (namespace, name, _, _) = Self::parse_identity(identity);

        let version = Self::get_version(meta);
        let tag = Self::get_tag(stem);
        let variant = Self::get_variant(stem);
        let ext = Self::get_ext(stem);
        let utter = Self::get_utter(&filename_str);

        let mut file = Self {
            id: uuid::Uuid::new_v4(),

            filename: filename_str,
            namespace,
            name,
            utter,

            version,
            tag,
            variant,

            ext,

            path,
            active: true,
            capabilities: Vec::new(),
        };

        file.capabilities = Self::infer_capabilities(&file.ext);

        file
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
