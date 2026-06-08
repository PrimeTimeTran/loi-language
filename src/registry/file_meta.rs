use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Default)]
pub struct FileMeta {
    pub path: PathBuf,
    pub active: bool,
    pub namespace: String,
    pub name: String,
    pub version: u32,
    pub priority: Option<u8>,
    pub tag: Option<String>,
    pub utter: Option<String>,
    pub ext: String,
    pub capabilities: Vec<String>,
}
impl FileMeta {
    pub fn default() -> Self {
        Self {
            name: "unknown".to_string(),
            priority: None,
            utter: None,
            version: 0,
            tag: None,
            ext: "loi".to_string(),
            capabilities: Vec::new(),
            namespace: String::new(),
            path: PathBuf::new(),
            active: true,
        }
    }

    pub fn from_path(path: &Path, root: &Path) -> Self {
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        // 1. Split into core components: [Identifier]@[Utter]#[Version]-[Tag].[Ext]
        let (body, ext) = Self::split_extension(stem, path);
        let (body, meta) = body.split_once('#').unwrap_or((body, ""));
        let (id_part, utter) = body.split_once('@').unwrap_or((body, ""));

        // 2. Extract specific data
        let (priority, name) = Self::parse_identifier(id_part);
        let (version, tag) = Self::parse_metadata(meta);

        let mut file = FileMeta {
            active: !path.components().any(|c| c.as_os_str() == "archive"),
            name,
            priority,
            utter: if utter.is_empty() {
                None
            } else {
                Some(utter.to_string())
            },
            version,
            tag: tag.map(|s| s.to_string()),
            ext,
            path: path.to_path_buf(),
            namespace: Self::derive_namespace(path, root),
            ..Default::default()
        };

        let parsed = ParsedPath::from(path);
        file.infer_capabilities(&parsed);
        file
    }

    fn capabilities(&self, parsed: &ParsedPath) -> Vec<String> {
        let mut caps = vec![];

        if parsed.variant.as_deref() == Some("ui") {
            caps.push("ui".to_string());
        }

        if parsed.version.is_some() {
            caps.push("versioned".to_string());
        }

        caps.sort();
        caps.dedup();
        caps
    }

    fn infer_capabilities(&mut self, parsed: &ParsedPath) {
        let mut caps = Vec::new();

        // extension-based
        match self.ext.as_str() {
            "js" | "jsx" | "ts" | "tsx" => {
                caps.push("scripting");
            }
            "html" => caps.push("markup"),
            "css" => caps.push("styling"),
            "md" => caps.push("document"),
            "mdx" => {
                caps.push("document");
                caps.push("ui");
            }
            _ => {}
        }

        // structural (ONLY from parsed path)
        if parsed.is_ui {
            caps.push("ui");
        }

        if parsed.is_versioned {
            caps.push("versioned");
        }

        self.capabilities = caps.into_iter().map(String::from).collect();

        self.capabilities.sort();
        self.capabilities.dedup();
    }

    fn is_versioned(fs: &str) -> bool {
        fs.contains('#')
            && fs
                .split('#')
                .nth(1)
                .map(|s| {
                    s.chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                })
                .unwrap_or(false)
    }

    fn derive_namespace(path: &Path, _root: &Path) -> String {
        let file = path.file_name().and_then(|f| f.to_str()).unwrap_or("");

        if file.contains("@ui") {
            "ui".to_string()
        } else if file.contains("script") {
            "scripting".to_string()
        } else if file.contains("style") {
            "styling".to_string()
        } else {
            "core".to_string()
        }
    }

    fn split_extension<'a>(stem: &'a str, path: &Path) -> (&'a str, String) {
        match stem.rsplit_once('.') {
            Some((base, ext)) => (base, ext.to_string()),
            None => (
                stem,
                path.extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("loi")
                    .to_string(),
            ),
        }
    }

    fn parse_identifier(id: &str) -> (Option<u8>, String) {
        // ONLY dot is structural
        if let Some((p_str, n)) = id.split_once('.') {
            let priority = p_str.parse::<u8>().ok();
            return (priority, n.to_string());
        }

        // If no dot, treat entire thing as name (including hyphens)
        if id.chars().all(|c| c.is_ascii_digit()) {
            return (id.parse::<u8>().ok(), id.to_string());
        }

        (None, id.to_string())
    }

    fn parse_metadata(meta: &str) -> (u32, Option<&str>) {
        if meta.is_empty() {
            return (0, None);
        }
        let (v, t) = meta.split_once('-').unwrap_or((meta, ""));
        (
            v.parse().unwrap_or(0),
            if t.is_empty() { None } else { Some(t) },
        )
    }

    pub fn get_fs_name(&self) -> String {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    pub fn get_ext(&self) -> String {
        let name = self.get_fs_name();
        name.split('@')
            .nth(1)
            .and_then(|s| s.split('.').rev().nth(1))
            .unwrap_or("loi")
            .to_string()
    }
}

pub struct ParsedPath {
    variant: Option<String>,
    version: Option<u32>,
    is_versioned: bool,
    is_ui: bool,
}
impl From<&Path> for ParsedPath {
    fn from(path: &Path) -> Self {
        let path_str = path.to_string_lossy();

        let is_ui = path_str.contains("@ui");

        let is_versioned = path_str.contains('#')
            && path_str
                .split('#')
                .nth(1)
                .map(|s| {
                    s.chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                })
                .unwrap_or(false);

        let version = path_str.split('#').nth(1).and_then(|s| {
            s.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .ok()
        });

        let variant = path_str
            .split('@')
            .nth(1)
            .and_then(|s| s.split('.').next())
            .map(|s| s.to_string());

        Self {
            variant,
            version,
            is_versioned,
            is_ui,
        }
    }
}

/*
 * Filename Parsing Pipeline Specification:
 * Phase 1 (Base): [Priority-]Name.ext
 * Phase 2 (Routing): Name@utter.ext
 * Phase 3 (Versioning): Name#Version[-Tag].ext
 * Phase 4 (Complex Integration): [Priority-]Name@utter#Version[-Tag].ext
 *
 * Logic Precedence (Right-to-Left):
 * 1. ext (via Path)
 * 2. Metadata (split '#') -> Version / Tag
 * 3. Routing (split '@')  -> utter
 * 4. Identifier (split '.' or '-') -> Priority / Name
 *
 * Example Files:
 * 1.  "01-arithmetic.loi"           (Phase 1: Priority 1, Name arithmetic)
 * 2.  "05.loi"                      (Phase 1: Priority 5, Name unknown)
 * 3.  "simple.loi"                  (Phase 1: Name simple)
 * 4.  "utils@helper.loi"            (Phase 2: Name utils, Cap helper)
 * 5.  "profile@ui.loi"              (Phase 2: Name profile, Cap ui)
 * 6.  "index@ui.html.loi"           (Phase 2: Name index, Cap ui, Ext html)
 * 7.  "test#0-run.loi"              (Phase 3: Name test, Ver 0, Tag run)
 * 8.  "base@core#1.loi"             (Phase 3: Name base, Cap core, Ver 1)
 * 9.  "app@web#5.js.loi"            (Phase 3: Name app, Cap web, Ver 5)
 * 10. "config#1-local.toml.loi"     (Phase 3: Name config, Ver 1, Tag local)
 * 11. "02@ui#1.html.loi"            (Phase 4: Priority 2, Name ui, Cap ui, Ver 1)
 * 12. "styles@ui#1.css.loi"         (Phase 4: Name styles, Cap ui, Ver 1, Ext css)
 * 13. "script@lib#2-beta.js.loi"    (Phase 4: Name script, Cap lib, Ver 2, Tag beta)
 * 14. "auth@api#10-prod.json.loi"   (Phase 4: Name auth, Cap api, Ver 10, Tag prod)
 * 15. "data@store#99-debug.bin.loi" (Phase 4: Name data, Cap store, Ver 99, Tag debug)
 * 16. "03@ui#1-alpha.html.loi"      (Phase 4: Priority 3, Name ui, Cap ui, Ver 1, Tag alpha)
 * 17. "api@v1#2.json.loi"           (Phase 4: Name api, Cap v1, Ver 2)
 * 18. "assets@cdn#0-static.png.loi" (Phase 4: Name assets, Cap cdn, Ver 0, Tag static)
 * 19. "ui@menu#4.html.loi"          (Phase 4: Name ui, Cap menu, Ver 4)
 * 20. "log@sys#12-prod.txt.loi"     (Phase 4: Name log, Cap sys, Ver 12, Tag prod)
 */

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_phase_1_basic_identity() {
        let root = PathBuf::from("root");

        let m1 = FileMeta::from_path(&PathBuf::from("root/simple.loi"), &root);
        assert_eq!(m1.name, "simple");

        let m2 = FileMeta::from_path(&PathBuf::from("root/05.loi"), &root);
        assert_eq!(m2.priority, Some(5));
        assert_eq!(m2.name, "05");
    }

    #[test]
    fn test_phase_2_routing() {
        let root = PathBuf::from("root");
        let m = FileMeta::from_path(&PathBuf::from("root/index@ui.html.loi"), &root);

        assert_eq!(m.name, "index");
        assert_eq!(m.utter, Some("ui".to_string()));
        assert_eq!(m.ext, "html"); // Should extract correctly
    }

    #[test]
    fn test_phase_3_versioning() {
        let root = PathBuf::from("root");
        let m = FileMeta::from_path(&PathBuf::from("root/base@core#1.loi"), &root);

        assert_eq!(m.name, "base");
        assert_eq!(m.utter, Some("core".to_string()));
        assert_eq!(m.version, 1);
    }

    #[test]
    fn test_phase_4_complex_stress_test() {
        let root = PathBuf::from("root");
        let path = PathBuf::from("root/02@ui#1-alpha.html.loi");
        let m = FileMeta::from_path(&path, &root);

        assert_eq!(m.priority, Some(2));
        assert_eq!(m.name, "02");
        assert_eq!(m.utter, Some("ui".to_string()));
        assert_eq!(m.version, 1);
        assert_eq!(m.tag, Some("alpha".to_string()));
    }

    #[test]
    fn test_file_meta_contract() {
        let root = PathBuf::from("root");

        // Table-driven tests: (Input, ExpectedPriority, ExpectedName, ExpectedCap, ExpectedVer, ExpectedExt)
        let cases = vec![
            ("simple.loi", None, "simple", None, 0, "loi"),
            ("05.loi", Some(5), "05", None, 0, "loi"),
            ("index@ui.html.loi", None, "index", Some("ui"), 0, "html"),
            ("base@core#1.loi", None, "base", Some("core"), 1, "loi"),
            ("02@ui#1.html.loi", Some(2), "02", Some("ui"), 1, "html"),
            (
                "auth@api#10-prod.json.loi",
                None,
                "auth",
                Some("api"),
                10,
                "json",
            ),
        ];

        for (filename, p, name, cap, ver, ext) in cases {
            let path = root.join(filename);
            let meta = FileMeta::from_path(&path, &root);

            assert_eq!(meta.priority, p, "Failed priority for {}", filename);
            assert_eq!(meta.name, name, "Failed name for {}", filename);
            assert_eq!(
                meta.utter,
                cap.map(|s| s.to_string()),
                "Failed cap for {}",
                filename
            );
            assert_eq!(meta.version, ver, "Failed version for {}", filename);
            assert_eq!(meta.ext, ext, "Failed ext for {}", filename);
        }
    }
}
