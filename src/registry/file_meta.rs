use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Default)]
pub struct FileMeta {
    pub path: PathBuf,
    pub active: bool,
    pub namespace: Vec<String>,
    pub name: String,
    pub version: u32,
    pub priority: Option<u8>,
    pub tag: Option<String>,
    pub capability: Option<String>,
    pub extension: String,
}

impl FileMeta {
    pub fn default() -> Self {
        Self {
            name: "unknown".to_string(),
            priority: None,
            capability: None,
            version: 0,
            tag: None,
            extension: "loi".to_string(),
            namespace: Vec::new(),
            path: PathBuf::new(),
            active: true,
        }
    }
    pub fn from_path(path: &Path, root: &Path) -> Self {
        let container_ext = path.extension().and_then(|s| s.to_str()).unwrap_or("loi");

        // 2. Get the stem (e.g., "02@ui#1.html")
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        // 3. Extract functional extension (e.g., "html") if it exists
        // rsplit_once finds the LAST dot, separating "02@ui#1" and "html"
        let (stem_without_ext, functional_ext) = if let Some((base, ext)) = stem.rsplit_once('.') {
            (base, Some(ext.to_string()))
        } else {
            (stem, None)
        };

        // 4. Metadata Layer: Split at '#'
        let (name_and_cap, meta_part) = stem_without_ext
            .split_once('#')
            .unwrap_or((stem_without_ext, ""));

        // 5. Routing Layer: Split at '@'
        // 1. ISOLATE NAME AND CAPABILITY (Everything left of '@' is the identifier)
        let (raw_identifier, capability) = match name_and_cap.split_once('@') {
            Some((b, c)) => (b, Some(c.to_string())),
            None => (name_and_cap, None),
        };

        // 2. DERIVE PRIORITY AND NAME FROM raw_identifier
        let (priority, name) = if let Some((p_str, n)) = raw_identifier.split_once('.') {
            // Case: "02.name" -> Priority 2, Name "name"
            (p_str.parse::<u8>().ok(), n.to_string())
        } else if let Some((p_str, n)) = raw_identifier.split_once('-') {
            // Case: "02-name" -> Priority 2, Name "name"
            (p_str.parse::<u8>().ok(), n.to_string())
        } else if raw_identifier.chars().all(|c| c.is_ascii_digit()) {
            // Case: "02" -> Priority 2, Name "02" (Name is same as priority)
            (
                raw_identifier.parse::<u8>().ok(),
                raw_identifier.to_string(),
            )
        } else {
            // Case: "name" (No priority) -> Name is the whole string
            (None, raw_identifier.to_string())
        };

        let (version, tag) = if !meta_part.is_empty() {
            let (v, t) = meta_part.split_once('-').unwrap_or((meta_part, ""));
            (
                v.parse().unwrap_or(0),
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                },
            )
        } else {
            (0, None)
        };

        FileMeta {
            active: !path
                .components()
                .any(|c| c.as_os_str() == "archive" || c.as_os_str() == ".hidden"),
            name,
            priority,
            capability,
            version,
            tag,
            extension: functional_ext.unwrap_or_else(|| container_ext.to_string()),
            namespace: Self::derive_namespace(path, root),
            path: path.to_path_buf(),
        }
    }

    fn derive_namespace(path: &Path, root: &Path) -> Vec<String> {
        path.strip_prefix(root)
            .unwrap_or(path)
            .parent()
            .map(|p| {
                p.components()
                    .filter_map(|c| {
                        if let std::path::Component::Normal(n) = c {
                            Some(n.to_string_lossy().into_owned())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

/*
 * Filename Parsing Pipeline Specification:
 * Phase 1 (Base): [Priority-]Name.ext
 * Phase 2 (Routing): Name@Capability.ext
 * Phase 3 (Versioning): Name#Version[-Tag].ext
 * Phase 4 (Complex Integration): [Priority-]Name@Capability#Version[-Tag].ext
 *
 * Logic Precedence (Right-to-Left):
 * 1. Extension (via Path)
 * 2. Metadata (split '#') -> Version / Tag
 * 3. Routing (split '@')  -> Capability
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
mod test_utils {
    use crate::{
        backend::{compiler_service::CompilerService, utter::registry::UtterRegistry},
        context::LoiContext,
        registry::registry::Registry,
    };

    use super::*;
    pub fn setup_test_context() -> LoiContext {
        let registry = Registry::from_files(vec![]);
        let utters = UtterRegistry::new();

        LoiContext {
            compiler_service: CompilerService::new(registry.clone(), utters.clone()),
            registry,
            utters,
        }
    }
}

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
        assert_eq!(m.capability, Some("ui".to_string()));
        assert_eq!(m.extension, "html"); // Should extract correctly
    }

    #[test]
    fn test_phase_3_versioning() {
        let root = PathBuf::from("root");
        let m = FileMeta::from_path(&PathBuf::from("root/base@core#1.loi"), &root);

        assert_eq!(m.name, "base");
        assert_eq!(m.capability, Some("core".to_string()));
        assert_eq!(m.version, 1);
    }

    #[test]
    fn test_phase_4_complex_stress_test() {
        let root = PathBuf::from("root");
        let path = PathBuf::from("root/02@ui#1-alpha.html.loi");
        let m = FileMeta::from_path(&path, &root);

        assert_eq!(m.priority, Some(2));
        assert_eq!(m.name, "02");
        assert_eq!(m.capability, Some("ui".to_string()));
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
                meta.capability,
                cap.map(|s| s.to_string()),
                "Failed cap for {}",
                filename
            );
            assert_eq!(meta.version, ver, "Failed version for {}", filename);
            assert_eq!(meta.extension, ext, "Failed ext for {}", filename);
        }
    }
}
