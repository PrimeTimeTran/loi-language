use std::{collections::HashMap, path::PathBuf};

use crate::{
    backend::{
        symbol::registry::SymbolRegistry,
        utter::{handler::Handler, registry::UtterRegistry, utter::Utter},
    },
    middle::ir::IR,
    registry::{file_meta::FileMeta, registry::Registry},
};

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
