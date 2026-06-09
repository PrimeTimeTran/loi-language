use loi::registry::{
    backend::utter::registry::UtterRegistry,
    build_system::BuildSystem,
    file_meta::{FileMeta, group_key},
    registry::Registry,
    registry::registry::Registry,
};
use pretty_assertions::assert_eq;
use pretty_assertions::assert_eq;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::tempdir;

pub fn get_test_root() -> PathBuf {
    PathBuf::from("/virtual/root")
}

pub fn setup_test_context() -> BuildSystem {
    let registry = Registry::from_files(vec![]);
    let utters = UtterRegistry::new();
    BuildSystem::test()
}

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
    assert_eq!(group_key("file#2.loi"), "file.loi");
    assert_eq!(group_key("file#3.loi"), "file.loi");

    // assert_eq!(group_key("00.file.loi"), "00.file.loi");
    // assert_eq!(group_key("00.file#1.loi"), "00.file.loi");

    // assert_eq!(group_key("00.file@lib.loi"), "00.file@lib.loi");
    // assert_eq!(group_key("00.file@lib#3.loi"), "00.file@lib.loi");

    // assert_eq!(group_key("file#123@lib.loi"), "file@lib.loi");
}
