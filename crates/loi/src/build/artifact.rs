use std::path::PathBuf;

use crate::middle::ir::IR;

#[derive(Clone, Debug)]
pub enum ArtifactKind {
    Web,
    Loi,
}

#[derive(Clone, Debug)]
pub struct Artifact {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
    pub kind: ArtifactKind,
}

pub struct CompiledArtifact {
    pub ir: IR,
    pub bundle: Vec<Artifact>,
}
