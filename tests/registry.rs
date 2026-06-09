use loi::backend::utter::registry::UtterRegistry;
use loi::build_system::BuildSystem;
use loi::registry::file_meta::FileMeta;
use loi::registry::registry::Registry;

use pretty_assertions::assert_eq;

use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::tempdir;

pub fn get_test_root() -> PathBuf {
    PathBuf::from("/virtual/root")
}

fn file(name: &str) -> FileMeta {
    FileMeta {
        path: PathBuf::from(name),
        ..Default::default()
    }
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
    assert_eq!(
        FileMeta::mock("file.loi").group_key(),
        FileMeta::mock("file#1.loi").group_key()
    );

    assert_eq!(
        FileMeta::mock("file.loi").group_key(),
        FileMeta::mock("file#3.loi").group_key()
    );
}
#[test]
fn groups_numbered_namespaces() {
    assert_eq!(
        FileMeta::mock("00.file.loi").group_key(),
        FileMeta::mock("00.file#1.loi").group_key()
    );

    assert_eq!(
        FileMeta::mock("00.file.loi").group_key(),
        FileMeta::mock("00.file#3.loi").group_key()
    );
}

#[test]
fn groups_tagged_files() {
    assert_eq!(
        FileMeta::mock("00.file@lib.loi").group_key(),
        FileMeta::mock("00.file@lib#1.loi").group_key()
    );

    assert_eq!(
        FileMeta::mock("00.file@lib.loi").group_key(),
        FileMeta::mock("00.file@lib#3.loi").group_key()
    );
}

#[test]
fn preserves_tag_when_grouping() {
    assert_ne!(
        FileMeta::mock("00.file.loi").group_key(),
        FileMeta::mock("00.file@lib.loi").group_key()
    );
}

#[test]
fn preserves_namespace_when_grouping() {
    assert_ne!(
        FileMeta::mock("00.file.loi").group_key(),
        FileMeta::mock("01.file.loi").group_key()
    );
}

#[test]
fn different_tags_are_different_groups() {
    assert_ne!(
        FileMeta::mock("00.file@lib.loi").group_key(),
        FileMeta::mock("00.file@test.loi").group_key()
    );
}

#[test]
fn strips_version_before_extension() {
    let key = FileMeta::mock("file#123.loi").group_key();

    assert_eq!(key.name, "file");
    assert_eq!(key.ext, "loi");
    assert_eq!(key.variant, None);
}

#[test]
fn strips_version_before_tag() {
    let key = FileMeta::mock("file#123@lib.loi").group_key();

    assert_eq!(key.name, "file");
    assert_eq!(key.ext, "loi");
    assert_eq!(key.variant, Some("lib".to_string()));
}

#[test]
fn multiple_versions_group_together() {
    let key = FileMeta::mock("file.loi").group_key();

    assert_eq!(key, FileMeta::mock("file#1.loi").group_key());
    assert_eq!(key, FileMeta::mock("file#2.loi").group_key());
    assert_eq!(key, FileMeta::mock("file#999.loi").group_key());
}

#[test]
fn normalization_examples() {
    let key = FileMeta::mock("file.loi").group_key();

    assert_eq!(key, FileMeta::mock("file#1.loi").group_key());
    assert_eq!(key, FileMeta::mock("file#2.loi").group_key());
    assert_eq!(key, FileMeta::mock("file#3.loi").group_key());
}

#[test]
fn strips_version() {
    let key = FileMeta::mock("00.loi").group_key();

    assert_eq!(key, FileMeta::mock("00#0.loi").group_key());
    assert_eq!(key, FileMeta::mock("00#1.loi").group_key());
    assert_eq!(key, FileMeta::mock("00#99.loi").group_key());
}
#[test]
fn strips_version_and_tag() {
    let key = FileMeta::mock("00.loi").group_key();

    assert_eq!(key, FileMeta::mock("00#1-versions.loi").group_key());
    assert_eq!(key, FileMeta::mock("00#2-draft.loi").group_key());
    assert_eq!(key, FileMeta::mock("00#500-whatever.loi").group_key());
}

#[test]
fn preserves_non_version_names() {
    let key = FileMeta::mock("index.loi").group_key();
    // hello
    assert_eq!(key, FileMeta::mock("about.loi").group_key());
}

#[test]
fn groups_versioned_files_into_single_stack() {
    let files = vec![
        file("00.loi"),
        file("00#0.loi"),
        file("00#1-versions.loi"),
        file("00#2-versions.loi"),
    ];

    let stacks = Registry::organize(files);

    assert_eq!(stacks.len(), 1);

    let stack = &stacks[0];

    assert_eq!(stack.files.len(), 4);
}

#[test]
fn latest_version_becomes_active() {
    let files = vec![
        file("00.loi"),
        file("00#0.loi"),
        file("00#1-versions.loi"),
        file("00#2-versions.loi"),
    ];

    let stacks = Registry::organize(files);

    let stack = &stacks[0];

    assert_eq!(stack.active_file.version, 2);
}
#[test]
fn all_version_formats_map_to_same_group() {
    let a = FileMeta::mock("00.loi");
    let b = FileMeta::mock("00#0.loi");
    let c = FileMeta::mock("00#1-versions.loi");
    let d = FileMeta::mock("00#2-versions.loi");

    assert_eq!(a.group_key(), b.group_key());
    assert_eq!(a.group_key(), c.group_key());
    assert_eq!(a.group_key(), d.group_key());
}
