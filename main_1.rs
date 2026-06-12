#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Lexer Error: {0}")]
    Lexer(String),
    #[error("Parser Error: {0}")]
    Parser(String),
    #[error("Analysis Error: {0}")]
    Analysis(String),
    #[error("Backend Error: {0}")]
    Backend(String),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Type {
    I32,
    F64,
    Bool,
    Str,
    Void,
    Ptr(Box<Type>),
    Array(Box<Type>),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedExpr {
    pub expr: Expr,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Serialize, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Cmp,
    Neg,
}

pub enum IR {
    Raw(String),
    Structured {
        body: Vec<IROp>,
        symbols: HashMap<String, Symbol>,
        metadata: HashMap<String, String>,
    },
}

#[derive(Debug, Clone)]
pub enum LoweredOp {
    Binary {
        target: String,
        left: String,
        op: Op,
        right: String,
    },
    Move {
        target: String,
        source: String,
    },
    Label(String),
    Jump(String),
    JumpIf {
        condition: String,
        label: String,
    },
    Nop,
}

#[derive(Debug, Clone)]
pub enum IROp {
    Binary {
        target: String,
        left: TypedExpr,
        op: BinOp,
        right: TypedExpr,
    },
    Module {
        body: Vec<IROp>,
    },
    Function {
        name: String,
        params: Vec<(String, Type)>,
        body: Vec<IROp>,
        return_type: Type,
    },
    Block {
        body: Vec<IROp>,
    },
    Declare {
        name: String,
        value: TypedExpr,
        mutable: bool,
        dynamic: bool,
    },
    Assign {
        name: String,
        value: TypedExpr,
    },
    Load {
        name: String,
    },
    ExprStmt {
        expr: TypedExpr,
    },
    If {
        condition: TypedExpr,
        then_branch: Vec<IROp>,
        else_branch: Vec<IROp>,
        scope_id: usize,
    },
    Return {
        value: Option<TypedExpr>,
    },
    Call {
        name: String,
        args: Vec<TypedExpr>,
    },
    Print {
        value: TypedExpr,
    },
    ExternalCall {
        namespace: String,
        function: String,
        args: Vec<TypedExpr>,
    },
    ModuleScope {
        name: String,
        body: Vec<IROp>,
    },
    While {
        condition: TypedExpr,
        body: Vec<IROp>,
    },
    DoWhile {
        body: Vec<IROp>,
        condition: TypedExpr,
    },
    Loop {
        body: Vec<IROp>,
    },
    For {
        iterator: String,
        iterable: TypedExpr,
        body: Vec<IROp>,
    },
    Lowered(LoweredOp),
}

pub struct SemanticAnalyzer {
    symbols: HashMap<String, Type>,
    scope_counter: AtomicUsize,
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f\r]+")]
#[logos(skip r"#[^\n]\*")]
pub enum Token {
    #[token("==")]
    Eq,
    #[token("!=")]
    Neq,
    #[token("=!")]
    Immutable,
    #[token("=?")]
    Dynamic,
    #[token("=:")]
    EqualsColon,
    #[token("||")]
    Or,
    #[token("&&")]
    And,
    #[token("+=")]
    Inc,
    #[token("-=")]
    Dec,
    #[token("//")]
    Floor,
    #[token(">=")]
    Ge,
    #[token("<=")]
    Le,
    #[token("=")]
    Assign,
    #[token("!")]
    Not,
    #[token("&")]
    Ampersand,
    #[token(":")]
    Colon,
    #[token("|")]
    Pipe,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("/")]
    Slash,
    #[token(">")]
    Gt,
    #[token("<")]
    Lt,
    #[token("*")]
    Star,
    #[token("%")]
    Mod,
    #[token("^")]
    Power,
    #[token(".")]
    Dot,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
    #[token("#", lex_line_note)]
    LineNote,
    #[token("`>", lex_block_note)]
    BlockNote,
    #[token("@{", lex_raw_block)]
    RawStart,
    #[token("}@", lex_raw_block)]
    RawEnd,
    #[token("dep")]
    Dependency,
    #[token("pkg")]
    Package,
    #[token("mod")]
    Module,
    #[token("pub")]
    Public,
    #[token("priv")]
    Private,
    #[token("print")]
    Print,
    #[token("if")]
    If,
    #[token("elif")]
    ElseIf,
    #[token("else")]
    Else,
    #[token("unless")]
    Unless,
    #[token("switch")]
    Switch,
    #[token("case")]
    Case,
    #[token("default")]
    Default,
    #[token("match")]
    Match,
    #[token("pipe")]
    Pipeline,
    #[token("fn")]
    Function,
    #[token("yield")]
    Yield,
    #[token("next")]
    Next,
    #[token("return")]
    Return,
    #[token("Do")]
    Do,
    #[token("loop")]
    Loop,
    #[token("until")]
    Until,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("of")]
    Of,
    #[token("in")]
    In,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("is")]
    Is,
    #[token("assert")]
    Assert,
    #[token("try")]
    Try,
    #[token("catch")]
    Catch,
    #[token("finally")]
    Finally,
    #[token("throw")]
    Throw,
    #[token("enum")]
    Enum,
    #[token("struct")]
    Struct,
    #[token("trait")]
    Trait,
    #[token("impl")]
    Impl,
    #[token("as")]
    As,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("or")]
    OrAlias,
    #[token("and")]
    AndAlias,
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex|lex.slice().parse::<f64>().ok())]
    Number(f64),
    #[regex(r#""[^"]*""#, |lex|lex.slice()[1..lex.slice().len()-1].to_string())]
    String(String),
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex|lex.slice().to_string())]
    Ident(String),
    Error,
    EOF,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum DeclKind {
    MutableStatic,
    ImmutableStatic,
    Dynamic,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
    And,
    Or,
    Assign,
    Mod,
    Power,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum UnOp {
    Neg,
    Not,
    AddrOf,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AssignOp {
    Assign,
    Immutable,
    Dynamic,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Expr {
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    Assign {
        left: Box<Expr>,
        right: Box<Expr>,
        op: AssignOp,
    },
    Number(f64),
    Bool(bool),
    String(String),
    Var(String),
    Array(Vec<Expr>),
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    Member {
        target: Box<Expr>,
        field: String,
    },
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        kind: DeclKind,
        value: Expr,
    },
    Print {
        expr: Expr,
    },
    ExprStmt {
        expr: Expr,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Return {
        value: Option<Expr>,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Loop {
        body: Vec<Stmt>,
    },
    For {
        iterator: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    DoWhile {
        body: Vec<Stmt>,
        condition: Expr,
    },
    Block {
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Serialize)]
pub struct AST {
    pub stmts: Vec<Stmt>,
    pub expr: Option<Expr>,
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Meta {
    #[token("#", lex_line_note)]
    LineNote,
    #[token("`>", lex_block_note)]
    BlockNote,
    #[token("@{", lex_raw_block)]
    RawStart,
    #[token("}@")]
    RawEnd,
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum MultiChar_02 {
    #[token("==")]
    Eq,
    #[token("!=")]
    Neq,
    #[token("=!")]
    Immutable,
    #[token("=?")]
    Dynamic,
    #[token("=:")]
    EqualsColon,
    #[token("||")]
    Or,
    #[token("&&")]
    And,
    #[token("+=")]
    Inc,
    #[token("-=")]
    Dec,
    #[token("//")]
    Floor,
    #[token(">=")]
    Ge,
    #[token("<=")]
    Le,
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum SingleChar_03 {
    #[token("=")]
    Assign,
    #[token("!")]
    Not,
    #[token("&")]
    Ampersand,
    #[token(":")]
    Colon,
    #[token("|")]
    Pipe,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("/")]
    Slash,
    #[token(">")]
    Gt,
    #[token("<")]
    Lt,
    #[token("*")]
    Star,
    #[token("%")]
    Mod,
    #[token("^")]
    Power,
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Structural_04 {
    #[token(".")]
    Dot,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Meta_05 {
    #[token("#", lex_line_note)]
    LineNote,
    #[token("`>", lex_block_note)]
    BlockNote,
    #[token("@{", lex_raw_block)]
    RawStart,
    #[token("}@", lex_raw_block)]
    RawEnd,
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Identifiers_06 {
    #[token("dep")]
    Dependency,
    #[token("pkg")]
    Package,
    #[token("mod")]
    Module,
    #[token("pub")]
    Public,
    #[token("priv")]
    Private,
    #[token("print")]
    Print,
    #[token("if")]
    If,
    #[token("elif")]
    ElseIf,
    #[token("else")]
    Else,
    #[token("unless")]
    Unless,
    #[token("switch")]
    Switch,
    #[token("case")]
    Case,
    #[token("default")]
    Default,
    #[token("match")]
    Match,
    #[token("pipe")]
    Pipeline,
    #[token("fn")]
    Function,
    #[token("yield")]
    Yield,
    #[token("next")]
    Next,
    #[token("return")]
    Return,
    #[token("Do")]
    Do,
    #[token("loop")]
    Loop,
    #[token("until")]
    Until,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("of")]
    Of,
    #[token("in")]
    In,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("is")]
    Is,
    #[token("assert")]
    Assert,
    #[token("try")]
    Try,
    #[token("catch")]
    Catch,
    #[token("finally")]
    Finally,
    #[token("throw")]
    Throw,
    #[token("enum")]
    Enum,
    #[token("struct")]
    Struct,
    #[token("trait")]
    Trait,
    #[token("impl")]
    Impl,
    #[token("as")]
    As,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("or")]
    OrAlias,
    #[token("and")]
    AndAlias,
}

pub struct CompilerContext {
    pub root_dir: PathBuf,
    pub output_dir: PathBuf,
    pub mode: CompileMode,
    pub registry: Registry,
    pub build: BuildSystem,
}

pub enum CompileMode {
    Batch,
    Interactive,
    Watch,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Config {
    #[arg(short, long, default_value = "output/syntax")]
    pub input: PathBuf,
    #[arg(short, long, default_value = "output/syntax")]
    pub output: PathBuf,
    #[arg(short, long)]
    pub watch: bool,
}

#[derive(Clone)]
pub struct Registry {
    pub files: HashMap<Uuid, FileMeta>,
    pub files_archive: Vec<FileMeta>,
    pub from_files: Vec<FileMeta>,
    pub stacks: Vec<FileStack>,
    pub active_by_group: HashMap<GroupKey, Uuid>,
}

#[derive(Clone)]
pub struct BundleManifest {
    pub dir_root: PathBuf,
    pub dir_out: PathBuf,
    pub strip_namespace: bool,
    pub strip_tag: bool,
    pub strip_utter: bool,
    pub strip_variant: bool,
    pub strip_version: bool,
    pub minify: bool,
    pub remove_comments: bool,
}

#[derive(Clone)]
pub struct BundleConfig {
    pub dir_root: PathBuf,
    pub dir_out: PathBuf,
}

pub struct BundleService {
    pub registry: Registry,
    pub utter_registry: UtterRegistry,
    pub symbols: SymbolRegistry,
    pub manifest: BundleManifest,
    pub resolver: OutputResolver,
    pub optimizer: AssetOptimizer,
}

pub struct Runtime<'ctx> {
    pub main: FunctionValue<'ctx>,
    pub printf: FunctionValue<'ctx>,
    pub fmt_f64: PointerValue<'ctx>,
    pub fmt_i32: PointerValue<'ctx>,
    pub fmt_str: PointerValue<'ctx>,
}

pub struct CodegenState<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub env: HashMap<String, PointerValue<'ctx>>,
}

pub struct LLVM<'ctx> {
    pub module: Module<'ctx>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Constant,
    Variable,
    Function,
    Method,
    Component,
    Action,
    Style,
    Theme,
    Type,
    Interface,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub value: String,
    pub file: FileMeta,
    pub origin: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct SymbolId {
    pub name: String,
    pub origin: String,
}

pub struct SymbolRegistry {
    pub table: HashMap<SymbolId, Symbol>,
    pub warnings: Vec<String>,
}

#[derive(Clone)]
pub struct UtterRegistry {
    pub utters: HashMap<String, Box<dyn Utter>>,
    pub handlers: HashMap<String, Box<dyn Handler>>,
}

#[derive(Clone, Copy)]
pub enum RenderTarget {
    Html,
    Css,
    Js,
    Ts,
    Json,
    Md,
    Loi,
}

#[derive(Clone)]
pub struct GenericHandler {
    pub target: RenderTarget,
}

#[derive(Debug, Clone)]
pub struct UtterFlags {
    pub browser_dom: bool,
    pub allow_network: bool,
    pub fs_access: bool,
    pub db_access: bool,
}

#[derive(Clone)]
pub struct LanguageConfig {
    pub name: String,
    pub flags: UtterFlags,
    pub symbol_patterns: Vec<(&'static str, SymbolKind)>,
    pub to_ir: Option<ToIrFn>,
}

#[derive(Clone)]
pub struct GenericUtter {
    name: String,
    config: LanguageConfig,
}

pub struct CodegenState<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub env: HashMap<String, PointerValue<'ctx>>,
}

struct ModuleState {}

#[derive(Tabled)]
pub struct FileView {
    pub namespace: String,
    #[tabled(rename = "#")]
    pub index: usize,
    #[tabled(rename = "FS Name")]
    pub filename: String,
    #[tabled(rename = "Active")]
    pub active: String,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Utter")]
    pub utter: String,
    #[tabled(rename = "Version")]
    pub version: String,
    #[tabled(rename = "Tag")]
    pub tag: String,
    #[tabled(rename = "Ext")]
    pub ext: String,
    #[tabled(rename = "Capabilities")]
    pub capabilities: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ListFilter {
    #[default]
    Active,
    Archived,
    All,
}

pub struct RegistryRenderer;

pub struct Theme;

pub struct CliController {
    pub system: BuildSystem,
    pub history_path: PathBuf,
    pub current_namespace: Vec<String>,
    pub verbosity: u8,
}

#[derive(Debug, PartialEq)]
pub enum BuildTarget {
    ByName(String),
    ByIndex(usize),
}

pub struct CommandMeta {
    pub label: &'static str,
    pub alias: Option<&'static str>,
    pub description: &'static str,
    pub hidden: bool,
    pub weight: u32,
}

#[derive(Debug, Default, PartialEq)]
pub struct ViewArgs {
    pub target: Option<BuildTarget>,
    pub flags: ViewFlags,
}

#[derive(Debug, Default, PartialEq)]
pub struct ViewFlags {
    pub name: Option<String>,
    pub number: Option<i32>,
    pub sort: Option<SortOrder>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Default, PartialEq)]
pub struct BuildAllArgs {
    pub target: Option<BuildTarget>,
    pub flags: BuildFlags,
}

#[derive(Debug, Default, PartialEq)]
pub struct BuildFlags {
    pub force: bool,
    pub ext: Option<String>,
    pub filter: Option<String>,
}

#[derive(EnumIter, Display, Debug, PartialEq)]
pub enum Command {
    #[strum(serialize = "mode")]
    Mode(String),
    List(ListFilter),
    #[strum(serialize = "tree")]
    Tree,
    #[strum(serialize = "history")]
    History(Option<String>),
    #[strum(serialize = "caps")]
    CapabilityMap,
    #[strum(serialize = "diff")]
    Diff(String, String),
    #[strum(serialize = "build-target")]
    Build(BuildTarget),
    #[strum(serialize = "build")]
    BuildAll(BuildAllArgs),
    #[strum(serialize = "view")]
    View(ViewArgs),
    #[strum(serialize = "clear")]
    Clear,
    #[strum(serialize = "help")]
    Help,
    #[strum(serialize = "exit")]
    Exit,
}

pub enum WebTarget {
    Html,
    Css,
    Js,
}

#[derive(Clone)]
pub struct Registry {
    pub files: HashMap<Uuid, FileMeta>,
    pub files_archive: Vec<FileMeta>,
    pub from_files: Vec<FileMeta>,
    pub stacks: Vec<FileStack>,
    pub active_by_group: HashMap<GroupKey, Uuid>,
}

#[derive(Clone)]
pub struct FileStack {
    pub files: Vec<FileMeta>,
    pub active_file: FileMeta,
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct FileMeta {
    pub id: Uuid,
    pub filename: String,
    pub namespace: String,
    pub name: String,
    pub utter: Option<String>,
    pub version: u32,
    pub tag: Option<String>,
    pub variant: Option<String>,
    pub ext: String,
    pub path: PathBuf,
    pub active: bool,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GroupKey {
    pub namespace: String,
    pub name: String,
    pub utter: Option<String>,
    pub variant: Option<String>,
    pub ext: String,
}

pub struct ParsedPath {
    pub variant: Option<String>,
    pub version: u32,
    pub is_versioned: bool,
    pub is_ui: bool,
}

pub struct OutputResolver {
    manifest: BundleManifest,
}

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

pub struct AssetOptimizer {
    pub minify: bool,
    pub remove_comments: bool,
}

#[derive(Clone)]
pub struct BundleManifest {
    pub dir_root: PathBuf,
    pub dir_out: PathBuf,
    pub strip_namespace: bool,
    pub strip_tag: bool,
    pub strip_utter: bool,
    pub strip_variant: bool,
    pub strip_version: bool,
    pub minify: bool,
    pub remove_comments: bool,
}

#[derive(Clone)]
pub struct BundleConfig {
    pub dir_root: PathBuf,
    pub dir_out: PathBuf,
}

pub struct BundleService {
    pub registry: Registry,
    pub utter_registry: UtterRegistry,
    pub symbols: SymbolRegistry,
    pub manifest: BundleManifest,
    pub resolver: OutputResolver,
    pub optimizer: AssetOptimizer,
}

pub struct BuildContext {
    pub build_id: u64,
    pub started_at: Instant,
    pub dir_root: PathBuf,
    pub dir_out: PathBuf,
    pub watch: bool,
    pub clean: bool,
    pub verbose: bool,
}

pub struct BuildSystem {
    pub context: BuildContext,
    pub registry: Registry,
    pub utters: UtterRegistry,
    pub bundle_service: BundleService,
}
