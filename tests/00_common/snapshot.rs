use insta::assert_debug_snapshot;
use loi::{compiler::diagnostic::DiagnosticStore, frontend::ast::AST, middle::ir::IR};

pub struct SnapshotBundle {
    pub ast: Option<AST>,
    pub ir: Option<IR>,
    pub llvm: Option<String>,
    pub diagnostics: DiagnosticStore,
}

impl std::fmt::Debug for SnapshotBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnapshotBundle")
            .field("ast", &self.ast)
            .field("diagnostics", &self.diagnostics)
            .finish()
    }
}

pub struct SnapshotContext {
    pub stage: &'static str,
    pub name: String,
}

#[derive(Debug)]
pub struct SnapshotPair<A, B> {
    pub left: A,
    pub right: B,
}

pub struct SnapshotTester {
    pub sub_dir: &'static str,
}

impl SnapshotTester {
    fn path(&self) -> &str {
        self.sub_dir
    }

    pub fn assert_value<T: std::fmt::Debug>(&self, name: &str, value: T) {
        insta::with_settings!({
            snapshot_path => self.path(),
        }, {
            assert_debug_snapshot!(name, value);
        });
    }

    pub fn assert_pair<A: std::fmt::Debug, B: std::fmt::Debug>(&self, name: &str, a: A, b: B) {
        insta::with_settings!({
            snapshot_path => self.path(),
        }, {
            assert_debug_snapshot!(name, SnapshotPair {
                left: a,
                right: b,
            });
        });
    }

    pub fn assert_stage<T: std::fmt::Debug>(&self, stage: &str, name: &str, value: T) {
        insta::with_settings!({
            snapshot_path => format!("{}/{}", self.path(), stage),
        }, {
            assert_debug_snapshot!(name, value);
        });
    }
}

pub const SNAP_FILE: SnapshotTester = SnapshotTester {
    sub_dir: "file_meta",
};
pub const SNAP_FRONTEND: SnapshotTester = SnapshotTester {
    sub_dir: "frontend",
};
pub const SNAP_MIDDLE: SnapshotTester = SnapshotTester { sub_dir: "middle" };
pub const SNAP_BACKEND: SnapshotTester = SnapshotTester { sub_dir: "backend" };

pub const SNAP_AST: SnapshotTester = SnapshotTester { sub_dir: "ast" };
pub const SNAP_IR: SnapshotTester = SnapshotTester { sub_dir: "ir" };
pub const SNAP_SYMBOLS: SnapshotTester = SnapshotTester { sub_dir: "symbols" };
pub const SNAP_DIAGNOSTICS: SnapshotTester = SnapshotTester {
    sub_dir: "diagnostics",
};
