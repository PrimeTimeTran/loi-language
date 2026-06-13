
use harness::{file, get_test_root, setup_test_context};

use loi::backend::utter::registry::UtterRegistry;
use loi::registry::file_meta::FileMeta;
use loi::registry::registry::Registry;

use pretty_assertions::assert_eq;

use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::tempdir;

#[test]
fn parse_standard_filename_format_succeeds() {
    let path = Path::new("05!dashboard@ui#42.jsx.loi");
    let meta = FileMeta::from_path(path, &get_test_root());
    assert_eq!(meta.name, "dashboard");
    assert_eq!(meta.utter, Some("ui".to_string()));
    assert_eq!(meta.version, 42);
    assert_eq!(meta.ext, "jsx");
}

#[test]
fn parse_version_with_suffix_extracts_base_integer() {
    let path = Path::new("00!core@lib#10-try-pnpm.js.loi");
    let meta = FileMeta::from_path(path, &get_test_root());
    assert_eq!(meta.version, 10);
}

#[test]
fn scan_multiple_versions_keeps_only_highest_version() {
    let dir = tempdir().unwrap();
    for f in &[
        "00!core@lib#1.js.loi",
        "00!core@lib#3.js.loi",
        "00!core@lib#2.js.loi",
    ] {
        fs::write(dir.path().join(f), "").unwrap();
    }
    let registry = Registry::scan(dir.path());

    // Convert values to a Vec and find the one that should be active
    let active_files: Vec<&FileMeta> = registry.files.values().collect();

    assert_eq!(active_files.len(), 1);
    assert_eq!(active_files[0].version, 3);
}

#[test]
fn scan_distinct_utters_maintains_separate_entries() {
    let dir = tempdir().unwrap();
    for f in &["00!core@lib#1.js.loi", "00!core@ui#1.js.loi"] {
        fs::write(dir.path().join(f), "").unwrap();
    }
    let registry = Registry::scan(dir.path());
    assert_eq!(registry.files.values().len(), 2);
}

#[test]
fn scan_duplicate_filenames_deduplicates_entry() {
    let dir = tempdir().unwrap();
    let f = "00!app@ui#1.html.loi";
    fs::write(dir.path().join(f), "").unwrap();
    fs::write(dir.path().join(f), "").unwrap();
    let registry = Registry::scan(dir.path());
    assert_eq!(registry.files.values().len(), 1);
}

#[test]
fn scan_files_orders_lexicographically_by_name() {
    let dir = tempdir().unwrap();
    for f in &["b@html.loi", "a@html.loi"] {
        fs::write(dir.path().join(f), "").unwrap();
    }
    let registry = Registry::scan(dir.path());

    // Sort logic is applied to stacks, not the HashMap
    assert_eq!(registry.stacks[0].active_file.name, "a");
    assert_eq!(registry.stacks[1].active_file.name, "b");
}
#[test]
fn test_lexicographical_and_numeric_sorting() {
    let create_registry = |names: Vec<&str>| {
        let dir = tempdir().unwrap();
        for name in names {
            fs::write(dir.path().join(name), "").unwrap();
        }
        Registry::scan(dir.path())
    };

    // 1. Basic lexicographical: a! before b!
    // We check registry.stacks[i].active_file.name instead of registry.files.values()[i]
    let reg1 = create_registry(vec!["b!html.loi", "a!html.loi"]);
    assert_eq!(reg1.stacks[0].active_file.name, "a");
    assert_eq!(reg1.stacks[1].active_file.name, "b");

    // 2. Numeric: 00! before 01!
    let reg2 = create_registry(vec!["01!html.loi", "00!html.loi"]);
    assert_eq!(reg2.stacks[0].active_file.name, "00");
    assert_eq!(reg2.stacks[1].active_file.name, "01");

    // 3. Numeric: 00001! before 2!
    let reg3 = create_registry(vec!["2!html.loi", "00001!html.loi"]);
    assert_eq!(reg3.stacks[0].active_file.name, "00001");
    assert_eq!(reg3.stacks[1].active_file.name, "2");

    // 4. Mixed/Length: aaaaaaaa! before ab!
    let reg4 = create_registry(vec!["ab!html.loi", "aaaaaaaa!html.loi"]);
    assert_eq!(reg4.stacks[0].active_file.name, "aaaaaaaa");
    assert_eq!(reg4.stacks[1].active_file.name, "ab");
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
        FileMeta::mock("00!file.loi").group_key(),
        FileMeta::mock("00!file#1.loi").group_key()
    );

    assert_eq!(
        FileMeta::mock("00!file.loi").group_key(),
        FileMeta::mock("00!file#3.loi").group_key()
    );
}

#[test]
fn groups_tagged_files() {
    assert_eq!(
        FileMeta::mock("00!file@lib.loi").group_key(),
        FileMeta::mock("00!file@lib#1.loi").group_key()
    );

    assert_eq!(
        FileMeta::mock("00!file@lib.loi").group_key(),
        FileMeta::mock("00!file@lib#3.loi").group_key()
    );
}

#[test]
fn preserves_tag_when_grouping() {
    assert_ne!(
        FileMeta::mock("00!file.loi").group_key(),
        FileMeta::mock("00!file@lib.loi").group_key()
    );
}

#[test]
fn preserves_namespace_when_grouping() {
    assert_ne!(
        FileMeta::mock("00!file.loi").group_key(),
        FileMeta::mock("01!file.loi").group_key()
    );
}

#[test]
fn different_tags_are_different_groups() {
    assert_ne!(
        FileMeta::mock("00!file@lib.loi").group_key(),
        FileMeta::mock("00!file@test.loi").group_key()
    );
}

#[test]
fn strips_version_before_extension() {
    let key = FileMeta::mock("file#123.loi").group_key();

    assert_eq!(key.name, "file");
    assert_eq!(key.ext, "loi");
}

#[test]
fn strips_version_before_tag() {
    let key = FileMeta::mock("file#123@lib.loi").group_key();

    assert_eq!(key.name, "file");
    assert_eq!(key.ext, "loi");
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
fn groups_different_versions_together() {
    let key1 = FileMeta::mock("data#1.loi").group_key();
    let key2 = FileMeta::mock("data#2.loi").group_key();
    // These SHOULD be equal
    assert_eq!(key1, key2);
}

#[test]
fn isolates_different_files() {
    let key1 = FileMeta::mock("index.loi").group_key();
    let key2 = FileMeta::mock("about.loi").group_key();
    // These SHOULD NOT be equal
    assert_ne!(key1, key2);
}

#[test]
fn groups_versioned_files_into_single_stack() {
    let files = vec![
        file("00!loi"),
        file("00#0.loi"),
        file("00#1-versions.loi"),
        file("00#2-versions.loi"),
    ];

    let stacks = Registry::organize(files);

    assert_eq!(stacks.len(), 1);

    assert_eq!(stacks[0].files.len(), 4);
}

#[test]
fn latest_version_becomes_active() {
    let mut f1 = file("00!loi");
    f1.version = 0;
    let mut f2 = file("00#0.loi");
    f2.version = 0;
    let mut f3 = file("00#1-versions.loi");
    f3.version = 1;
    let mut f4 = file("00#2-versions.loi");
    f4.version = 2;

    let files = vec![f1, f2, f3, f4];

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
