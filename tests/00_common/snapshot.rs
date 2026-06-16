pub struct SnapshotTester {
    pub sub_dir: &'static str,
}

impl SnapshotTester {
    pub fn assert<A: std::fmt::Debug, B: std::fmt::Debug>(&self, name: &str, a: A, b: B) {
        let path = format!("../snapshots/{}", self.sub_dir);

        insta::with_settings!({
            snapshot_path => path,
        }, {
            insta::assert_debug_snapshot!(name, (a, b));
        });
    }
}

pub const SNAP_FILE: SnapshotTester = SnapshotTester {
    sub_dir: "file_meta",
};
pub const SNAP_CLI: SnapshotTester = SnapshotTester { sub_dir: "cli" };
pub const SNAP_LEXER: SnapshotTester = SnapshotTester { sub_dir: "lexer" };
pub const SNAP_PARSER: SnapshotTester = SnapshotTester { sub_dir: "parser" };
pub const SNAP_LLVM: SnapshotTester = SnapshotTester { sub_dir: "llvm" };
