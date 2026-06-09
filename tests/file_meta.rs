use std::collections::HashMap;
use std::path::Path;
use std::{fs, path::PathBuf};
use tempfile::tempdir;

use loi::backend::utter::registry::UtterRegistry;
use loi::build_system::BuildSystem;
use loi::registry::file_meta::{FileMeta, GroupKey};
use loi::registry::registry::Registry;
use uuid::Uuid;

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
    let f = meta("00!file.loi");

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
fn tag_is_independent_of_version_and_extension() {
    let f = meta("app#2-alpha.js.loi");
    assert_eq!(f.version, 2);
    assert_eq!(f.tag.as_deref(), Some("alpha"));
    assert!(!f.tag.as_deref().unwrap().contains('.'));
    assert!(!f.tag.as_deref().unwrap().contains('2'));
}

#[test]
fn files_with_different_tags_but_same_identity_group_together_organize() {
    let files = vec![
        meta("app.js.loi"),
        meta("app#1-alpha.js.loi"),
        meta("app#2-bravo.js.loi"),
    ];

    let stacks = Registry::organize(files);

    assert_eq!(stacks.len(), 1);
}

#[test]
fn files_with_different_tags_but_same_identity_group_together_scan() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("app.js.loi"), "").unwrap();
    fs::write(dir.path().join("app#1-alpha.js.loi"), "").unwrap();
    fs::write(dir.path().join("app#2-bravo.js.loi"), "").unwrap();

    let registry = Registry::scan(dir.path());

    assert_eq!(registry.stacks.len(), 1);
}

#[test]
fn tags_are_parsed_independently_of_grouping() {
    let metas: Vec<_> = vec![
        meta("app.js.loi"),
        meta("app#1-alpha.js.loi"),
        meta("app#2-bravo.js.loi"),
    ];

    assert_eq!(metas[1].tag.as_deref(), Some("alpha"));
    assert_eq!(metas[2].tag.as_deref(), Some("bravo"));
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
    let stack = &registry.stacks[0];

    let variants: Vec<_> = stack
        .files
        .iter()
        .filter_map(|f| f.variant.clone())
        .collect();

    assert!(variants.contains(&"$webpack".to_string()));
    assert!(variants.contains(&"$turbopack".to_string()));
}
#[test]
fn multiple_variants_in_same_version() {
    let dir = tempdir().unwrap();

    let files = vec!["app$webpack#1-alpha.js.loi", "app$turbopack#1-alpha.js.loi"];

    for f in &files {
        fs::write(dir.path().join(f), "").unwrap();
    }

    let registry = Registry::scan(dir.path());

    // there should be exactly one stack for "app"
    let stack = registry
        .stacks
        .iter()
        .find(|s| s.active_file.name == "app")
        .expect("stack for app should exist");

    let variants: Vec<String> = stack
        .files
        .iter()
        .filter_map(|f| f.variant.clone())
        .collect();

    assert!(variants.contains(&"$webpack".to_string()));
    assert!(variants.contains(&"$turbopack".to_string()));
    assert_eq!(
        variants.len(),
        2,
        "Should detect variants inside the same stack"
    );
}

// =========================================================
// 📦 Stage 6: full composition sanity check
// =========================================================

#[test]
fn all_features_together_do_not_conflict() {
    let f = meta("00!app@ui#3-feature$webpack.html.loi");
    // sososo
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
