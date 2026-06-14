#[derive(Debug, Clone)]
pub enum BuildArtifact {
    Object(Vec<u8>),
    Llvm(Vec<u8>),
    Wasm(Vec<u8>),
    Bytecode(Vec<u8>),
}
