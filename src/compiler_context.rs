pub struct CompilerContext {
    pub global_symbols: HashMap<String, SymbolDefinition>,
    pub ir_modules: Vec<IR>,
}
