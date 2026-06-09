use std::{fs, path::PathBuf};
use tempfile::tempdir;

use loi::backend::utter::registry::UtterRegistry;
use loi::build_system::BuildSystem;
use loi::registry::file_meta::FileMeta;
use loi::registry::registry::Registry;

/*
 * Filename Parsing Pipeline Specification:
 * Phase 1 (Base): [namespace-]Name.ext
 * Phase 2 (Routing): Name@utter.ext
 * Phase 3 (Versioning): Name#Version[-Tag].ext
 * Phase 4 (Complex Integration): [namespace-]Name@utter#Version[-Tag].ext
 *
 * Logic Precedence (Right-to-Left):
 * 1. ext (via Path)
 * 2. Metadata (split '#') -> Version / Tag
 * 3. Routing (split '@')  -> utter
 * 4. Identifier (split '.' or '-') -> namespace / Name
 *
 * Example Files:
 * 1.  "01-arithmetic.loi"           (Phase 1: namespace 1, Name arithmetic)
 * 2.  "05.loi"                      (Phase 1: namespace 5, Name unknown)
 * 3.  "simple.loi"                  (Phase 1: Name simple)
 * 4.  "utils@helper.loi"            (Phase 2: Name utils, Cap helper)
 * 5.  "profile@ui.loi"              (Phase 2: Name profile, Cap ui)
 * 6.  "index@ui.html.loi"           (Phase 2: Name index, Cap ui, Ext html)
 * 7.  "test#0-run.loi"              (Phase 3: Name test, Ver 0, Tag run)
 * 8.  "base@core#1.loi"             (Phase 3: Name base, Cap core, Ver 1)
 * 9.  "app@web#5.js.loi"            (Phase 3: Name app, Cap web, Ver 5)
 * 10. "config#1-local.toml.loi"     (Phase 3: Name config, Ver 1, Tag local)
 * 11. "02@ui#1.html.loi"            (Phase 4: namespace 2, Name ui, Cap ui, Ver 1)
 * 12. "styles@ui#1.css.loi"         (Phase 4: Name styles, Cap ui, Ver 1, Ext css)
 * 13. "script@lib#2-beta.js.loi"    (Phase 4: Name script, Cap lib, Ver 2, Tag beta)
 * 14. "auth@api#10-prod.json.loi"   (Phase 4: Name auth, Cap api, Ver 10, Tag prod)
 * 15. "data@store#99-debug.bin.loi" (Phase 4: Name data, Cap store, Ver 99, Tag debug)
 * 16. "03@ui#1-alpha.html.loi"      (Phase 4: namespace 3, Name ui, Cap ui, Ver 1, Tag alpha)
 * 17. "api@v1#2.json.loi"           (Phase 4: Name api, Cap v1, Ver 2)
 * 18. "assets@cdn#0-static.png.loi" (Phase 4: Name assets, Cap cdn, Ver 0, Tag static)
 * 19. "ui@menu#4.html.loi"          (Phase 4: Name ui, Cap menu, Ver 4)
 * 20. "log@sys#12-prod.txt.loi"     (Phase 4: Name log, Cap sys, Ver 12, Tag prod)
 */

fn root() -> PathBuf {
    std::env::temp_dir()
}

fn meta(input: &str) -> FileMeta {
    FileMeta::from_path(std::path::Path::new(input), &root())
}

#[test]
fn test_file_extension_parsing() {
    let cases: Vec<(&str, &str)> = vec![
        ("index.html.loi", "html"),
        ("index.js.loi", "js"),
        ("index.css.loi", "css"),
        ("main.loi", "loi"),
    ];

    for (input, expected_ext) in cases {
        let file = FileMeta::from_path(std::path::Path::new(input), &root());
        assert_eq!(file.ext, expected_ext, "failed for input: {}", input);
    }
}
// =========================================================
// 📦 Stage 1: basic identity + namespace parsing
// =========================================================
//
#[test]
fn plain_file() {
    let f = meta("file.loi");

    assert_eq!(f.name, "file");
    assert_eq!(f.ext, "loi");
    assert_eq!(f.version, 0);
    assert_eq!(f.tag, None);
    assert_eq!(f.variant, None);
}

#[test]
fn priority_prefix() {
    let f = meta("00.file.loi");

    assert_eq!(f.name, "file");
    assert_eq!(f.namespace, "00");
    assert_eq!(f.version, 0);
}

// =========================================================
// 📦 Stage 2: extension parsing
// =========================================================

#[test]
fn extracts_known_extensions() {
    let cases = vec![
        ("index.html.loi", "html"),
        ("index.js.loi", "js"),
        ("index.css.loi", "css"),
        ("main.loi", "loi"),
    ];

    for (input, ext) in cases {
        let f = meta(input);
        assert_eq!(f.ext, ext, "failed for {}", input);
    }
}

// =========================================================
// 📦 Stage 3: version parsing (independent axis)
// =========================================================

#[test]
fn sequential_versions() {
    let cases = vec![("app.js.loi", 0), ("app#1.js.loi", 1), ("app#2.js.loi", 2)];

    for (input, expected) in cases {
        let f = meta(input);
        // Assert that the parsed version matches your expectation
        assert_eq!(f.version, expected, "Failed for input: {}", input);
    }
}
#[test]
fn version_does_not_affect_name() {
    let f = meta("app#12.js.loi");
    assert_eq!(f.name, "app");
}

// =========================================================
// 📦 Stage 4: tags (human metadata layer)
// =========================================================

#[test]
fn tag_is_independent_of_version() {
    let f = meta("app#2-alpha.js.loi");

    assert_eq!(f.version, 2);
    assert_eq!(f.tag.as_deref(), Some("alpha"));
}

#[test]
fn multiple_tags_are_distinct() {
    let files = vec!["app.js.loi", "app#1-alpha.js.loi", "app#2-bravo.js.loi"];

    let metas: Vec<_> = files.iter().map(|f| meta(f)).collect();

    let alpha = metas
        .iter()
        .find(|f| f.tag.as_deref() == Some("alpha"))
        .expect("Alpha file missing");

    let bravo = metas
        .iter()
        .find(|f| f.tag.as_deref() == Some("bravo"))
        .expect("Bravo file missing");

    assert_eq!(alpha.version, 1);
    assert_eq!(bravo.version, 2);
}

// =========================================================
// 📦 Stage 5: variants (execution branches)
// =========================================================

#[test]
fn variant_parsing() {
    let f = meta("app#1-$webpack.js.loi");

    assert_eq!(f.version, 1);
    assert_eq!(f.variant.as_deref(), Some("$webpack"));
}

#[test]
fn multiple_variants() {
    let dir = tempdir().unwrap();
    let files = vec!["app$webpack.js.loi", "app$turbopack.js.loi"];
    for f in &files {
        fs::write(dir.path().join(f), "").unwrap();
    }
    let registry = Registry::scan(dir.path());
    let variants: Vec<String> = registry
        .files
        .iter()
        .filter_map(|f| f.variant.clone())
        .collect();

    assert!(variants.contains(&"$webpack".to_string()));
    assert!(variants.contains(&"$turbopack".to_string()));
    assert_eq!(variants.len(), 2, "Should detect variants");
}
#[test]
fn multiple_variants_in_same_version() {
    let dir = tempdir().unwrap();
    let files = vec!["app#1-alpha$webpack.js.loi", "app#1-bravo$turbopack.js.loi"];
    for f in &files {
        fs::write(dir.path().join(f), "").unwrap();
    }
    let registry = Registry::scan(dir.path());
    let variants: Vec<String> = registry
        .files
        .iter()
        .filter_map(|f| f.variant.clone())
        .collect();

    assert!(variants.contains(&"$webpack".to_string()));
    assert!(variants.contains(&"$turbopack".to_string()));
    assert_eq!(
        variants.len(),
        2,
        "Should detect variants after version tags"
    );
}

// =========================================================
// 📦 Stage 6: full composition sanity check
// =========================================================

#[test]
fn all_features_together_do_not_conflict() {
    let f = meta("00.app@ui#3-feature$webpack.html.loi");

    assert_eq!(f.name, "app");
    assert_eq!(f.namespace, "00");
    assert_eq!(f.utter.as_deref(), Some("ui"));

    assert_eq!(f.version, 3);
    assert_eq!(f.tag.as_deref(), Some("feature"));
    assert_eq!(f.variant.as_deref(), Some("$webpack"));

    assert_eq!(f.ext, "html");
}

#[test]
fn test_phase_2_routing() {
    let root = PathBuf::from("root");
    // "index@ui.html.loi" -> name: index, utter: ui, ext: html
    let m = FileMeta::from_path(&PathBuf::from("root/index@ui.html.loi"), &root);

    assert_eq!(m.name, "index");
    assert_eq!(m.utter.as_deref(), Some("ui"));
    assert_eq!(m.ext, "html");
}

#[test]
fn test_phase_3_versioning() {
    let root = PathBuf::from("root");
    // "base@core#1.loi" -> name: base, utter: core, version: 1
    let m = FileMeta::from_path(&PathBuf::from("root/base@core#1.loi"), &root);

    assert_eq!(m.name, "base");
    assert_eq!(m.utter.as_deref(), Some("core"));
    assert_eq!(m.version, 1);
}

#[test]
fn test_phase_4_complex_stress_test() {
    let root = PathBuf::from("root");
    // "02@ui#1-alpha.html.loi" -> name: 02, utter: ui, version: 1, tag: alpha, ext: html
    let path = PathBuf::from("root/02@ui#1-alpha.html.loi");
    let m = FileMeta::from_path(&path, &root);

    assert_eq!(m.name, "02");
    assert_eq!(m.utter.as_deref(), Some("ui"));
    assert_eq!(m.version, 1);
    assert_eq!(m.tag.as_deref(), Some("alpha"));
    assert_eq!(m.ext, "html");
}

#[test]
fn test_file_meta_contract() {
    let root = PathBuf::from("root");

    // Input, Name, Utter, Version, Tag, Ext
    let cases = vec![
        ("simple.loi", "simple", None, 0, None, "loi"),
        ("base@core#1.loi", "base", Some("core"), 1, None, "loi"),
        ("index@ui.html.loi", "index", Some("ui"), 0, None, "html"),
        ("02@ui#1.html.loi", "02", Some("ui"), 1, None, "html"),
        (
            "auth@api#10-prod.json.loi",
            "auth",
            Some("api"),
            10,
            Some("prod"),
            "json",
        ),
    ];

    for (filename, name, utter, ver, tag, ext) in cases {
        let path = root.join(filename);
        let meta = FileMeta::from_path(&path, &root);

        assert_eq!(meta.name, name, "Failed name for {}", filename);
        assert_eq!(
            meta.utter.as_deref(),
            utter,
            "Failed utter for {}",
            filename
        );
        assert_eq!(meta.version, ver, "Failed version for {}", filename);
        assert_eq!(meta.tag.as_deref(), tag, "Failed tag for {}", filename);
        assert_eq!(meta.ext, ext, "Failed ext for {}", filename);
    }
}
