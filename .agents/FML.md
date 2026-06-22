### src/analyzer/error.rs

```rs
        // ENUMS:
        enum AnalysisError { Parse(_0: String), UnsupportedLanguage(_0: String), Io(_0: String) }
```

### src/analyzer/my_analyzer.rs

```rs
        // STRUCTS:
        struct MyAnalyzer
            // PROPERTIES:
            config: &'a Config, items: &'a [syn::Item], rendered_output: Vec<String>, registry: SymbolRegistry
```

### src/analyzer/rust.rs

```rs
        // FUNCTIONS:
        fn visibility(
            vis: &syn::Visibility
        ) -> Visibility

        // STRUCTS:
        struct RustAnalyzer
            // METHODS:
            fn analyze(
                self,
                source: &str,
                options: &AnalyzerOptions
            ) -> Result<Vec<Symbol>, AnalysisError>
        struct RustVisitor
            // PROPERTIES:
            options: &'a AnalyzerOptions, symbols: Vec<Symbol>
```

### src/analyzer/trait.rs

```rs
        // STRUCTS:
        struct AnalyzerOptions
            // PROPERTIES:
            include_private: bool, include_tests: bool
```

### src/analyzer/ts.rs

```rs
        // STRUCTS:
        struct TsVisitor
            // PROPERTIES:
            options: &'a AnalyzerOptions, symbols: Vec<Symbol>
        struct TypeScriptAnalyzer
            // METHODS:
            fn analyze(
                self,
                source: &str,
                options: &AnalyzerOptions
            ) -> Result<Vec<Symbol>, AnalysisError>
```

## src/config.rs

```rs
    // FUNCTIONS:
    fn clean_rust_syntax(
        input: &str
    ) -> String
    fn clean_type_spacing(
        s: &str
    ) -> String
    fn format_type(
        s: &str
    ) -> String

    // STRUCTS:
    struct CliArgs
        // PROPERTIES:
        name: Option<String>, root: Option<PathBuf>, path: Option<PathBuf>
    struct Config
        // PROPERTIES:
        analysis_root: PathBuf, output_name: String, output_path: PathBuf, extract: ExtractConfig, format: FormatConfig, render_policy: RenderPolicy, layout: DenseConfig

        // METHODS:
        fn load() -> Self
        fn method_scope(
            self,
            struct_scope: &str
        ) -> String
        fn format_signature(
            self,
            name: &str,
            params: &[String],
            ret: Option<String>,
            fn_indent: &str
        ) -> String
        fn default() -> Self
    struct ExtractConfig
        // PROPERTIES:
        rules: Vec<Rule>

        // METHODS:
        fn default() -> Self
    struct FormatConfig
        // PROPERTIES:
        comment_mark: String, line_style: LineStyle, header: HeaderFormat, codeblock: Option<CodeBlockConfig>, wrap_in_code_blocks: bool, dense: DenseConfig

        // METHODS:
        fn default() -> Self
    struct RenderConfig
        // PROPERTIES:
        policy: RenderPolicy, format: HeaderFormat
    struct RenderPolicy
        // PROPERTIES:
        mode: ViewMode, include_properties: bool, include_functions: bool, include_params: bool, include_nested_types: bool

        // METHODS:
        fn default() -> Self
```

## src/detector.rs

```rs
    // STRUCTS:
    struct LanguageDetector
        // METHODS:
        fn detect(
            path: &Path
        ) -> Language
```

## src/evaluator.rs

```rs
    // STRUCTS:
    struct Evaluator
        // PROPERTIES:
        config: Config, scanner: FileScanner, renderers: HashMap<Language, Box<dyn FileRenderer>>, writer: Box<dyn OutputWriter>

        // METHODS:
        fn default() -> Self
        fn new(
            config: Config
        ) -> Self
        fn evaluate_fs(self)
```

## src/extract.rs

```rs
    // ENUMS:
    enum DepthConstraint { Any, Exact(_0: usize), Range(from: usize, to: usize) }
    enum IncludePolicy { Only, IncludeDerived, IncludeNested }
    enum Matcher { Symbol(_0: SymbolMatcher), File(_0: FileMatcher) }
    enum ParentConstraint { Any, Within(_0: SymbolKind), WithinPath(_0: Vec < SymbolKind >) }
    enum ScopeRoot { File, Module, Symbol(_0: SymbolKind) }

    // STRUCTS:
    struct FileMatcher
        // PROPERTIES:
        extensions: HashSet<String>, path_contains: Option<String>, ignore_tests: bool

        // METHODS:
        fn default() -> Self
    struct Rule
        // PROPERTIES:
        languages: HashSet<Language>, matchers: Vec<Matcher>

        // METHODS:
        fn default() -> Self
    struct StructuralFilter
        // PROPERTIES:
        depth: DepthConstraint, parent: Option<ParentConstraint>

        // METHODS:
        fn default() -> Self
    struct SymbolMatcher
        // PROPERTIES:
        kinds: HashSet<SymbolKind>, structural: Option<StructuralFilter>
```

### src/format/mod.rs

```rs
        // ENUMS:
        enum DepthConstraint { Any, Exact(_0: usize), Range(from: usize, to: usize) }
        enum EnumFormat { NameOnly, NameWithTypes }
        enum ExtractMode { SymbolsOnly, FullBody }
        enum FieldFormat { None, Name, NameAndType, All }
        enum HeaderFormat { None, Flat, DepthHash }
        enum HeaderMode { Flat, DepthHash }
        enum IncludePolicy { Only, IncludeDerived, IncludeNested }
        enum LineStyle { Compact, ExpandedParams, Block }
        enum Matcher { Symbol(_0: SymbolMatcher), File(_0: FileMatcher) }
        enum ParamFormat { PartialEq, Eq, None, NameOnly, NameList, NameType, TypeOnly }
        enum ParentConstraint { Any, Within(_0: SymbolKind), WithinPath(_0: Vec < SymbolKind >) }
        enum PathFormat { FileName, Relative, ModulePath, Absolute }
        enum PathMode { FileName, Relative, ModulePath }
        enum ScopeRoot { File, Module, Symbol(_0: SymbolKind) }

        // STRUCTS:
        struct CodeBlockConfig
            // PROPERTIES:
            enabled: bool, language_override: Option<String>, preserve_indentation: bool

            // METHODS:
            fn default() -> Self
        struct DenseConfig
            // PROPERTIES:
            enabled: bool, line_style: LineStyle

            // METHODS:
            fn default() -> Self
        struct EnumDenseConfig
            // PROPERTIES:
            variants: ParamFormat

            // METHODS:
            fn default() -> Self
        struct FunctionDenseConfig
            // PROPERTIES:
            params: ParamFormat

            // METHODS:
            fn default() -> Self
        struct OutputConfig
            // PROPERTIES:
            path_format: PathFormat, header: HeaderFormat, codeblock: Option<CodeBlockConfig>, dense: DenseConfig

            // METHODS:
            fn default() -> Self
        struct StructDenseConfig
            // PROPERTIES:
            fields: ParamFormat, functions: FunctionDenseConfig

            // METHODS:
            fn default() -> Self
        struct StructuralFilter
            // PROPERTIES:
            depth: DepthConstraint, parent: Option<ParentConstraint>

            // METHODS:
            fn default() -> Self
        struct SymbolMatcher
            // PROPERTIES:
            kinds: HashSet<SymbolKind>, structural: Option<StructuralFilter>
```

## src/ir.rs

```rs
    // ENUMS:
    enum FunctionKind { Free, Method, Associated, Lambda }
    enum Language { Rust, JavaScript, TypeScript, JSX, TSX, Python, Go, Java, CSharp, Unknown }
    enum SymbolKind { Function(_0: FunctionKind), Variable(_0: VariableKind), Type(_0: TypeKind) }
    enum TypeKind { Struct, Enum, Class, Trait, Interface, TypeAlias }
    enum VariableKind { Let, Const, Var, Field }
    enum Visibility { Public, Private, Protected, Internal }

    // STRUCTS:
    struct Signature
        // PROPERTIES:
        params: Vec<(String, String)>, return_type: String
    struct Symbol
        // PROPERTIES:
        name: String, kind: SymbolKind, visibility: Visibility, params: Option<Vec<(String, String)>>, return_type: Option<String>
```

## src/main.rs

```rs
    // FUNCTIONS:
    fn main()
```

## src/mode.rs

```rs
    // ENUMS:
    enum ViewMode { System, SystemFlow, SystemFlowDetailed, Structures, Interface, FullDetail }
```

## src/parser.rs

```rs
    // STRUCTS:
    struct ParserContext
        // PROPERTIES:
        cm: Lrc<SourceMap>

        // METHODS:
        fn with_parser(
            self,
            name: &str,
            source: &str,
            f: F
        ) -> R
```

### src/render/common.rs

```rs
        // FUNCTIONS:
        fn get_path_metadata(
            path: &Path,
            root: &Path
        ) -> (PathBuf, usize, String)
        fn group_items(
            ast: &syn::File,
            config: Config,
            sym_indent: &str
        ) -> BTreeMap<String, Vec<String>>
```

### src/render/file.rs

```rs
        // STRUCTS:
        struct RenderedFile
            // PROPERTIES:
            path: PathBuf, header: String, body: String, is_empty: bool
```

### src/render/rust.rs

```rs
        // STRUCTS:
        struct RustFileRenderer
            // PROPERTIES:
            config: Config

            // METHODS:
            fn render(
                self,
                path: &Path,
                source: &str
            ) -> RenderedFile
```

### src/render/ts.rs

```rs
        // FUNCTIONS:
        fn extract_params_from_params(
            params: &[Param]
        ) -> Vec<String>
        fn extract_params_from_pat(
            params: &[Pat]
        ) -> Vec<String>
        fn extract_pat(
            config: &Config,
            pat: &Pat,
            init: &Option<Box<Expr>>,
            sym_indent: &str,
            groups: &mut BTreeMap<String, Vec<String>>
        )
        fn get_return_type(
            rt: &Option<Box<TsTypeAnn>>
        ) -> String
        fn group_items_ts(
            config: &Config,
            module: &Module,
            sym_indent: &str
        ) -> BTreeMap<String, Vec<String>>
        fn type_to_string(
            ts_type: &TsType
        ) -> String

        // STRUCTS:
        struct TypeScriptFileRenderer
            // PROPERTIES:
            config: Config

            // METHODS:
            fn render(
                self,
                path: &Path,
                source: &str
            ) -> RenderedFile
```

## src/scanner.rs

```rs
    // STRUCTS:
    struct FileScanner
        // PROPERTIES:
        root: PathBuf

        // METHODS:
        fn new(
            root: PathBuf
        ) -> Self
        fn scan(self) -> Vec<PathBuf>
        fn root(self) -> &Path
```

## src/ui.rs

```rs
    // ENUMS:
    enum RenderSig { Function(_0: & 'a ItemFn), Method(_0: & 'a Signature, _1: & 'a str) }

    // FUNCTIONS:
    fn extract_fields(
        s: &ItemStruct,
        policy: &RenderPolicy
    ) -> Vec<String>
    fn extract_params(
        sig: &Signature,
        config: &Config
    ) -> Vec<String>
    fn extract_ret(
        sig: &Signature
    ) -> Option<String>
    fn render_blocks(
        config: &Config,
        groups: BTreeMap<String, Vec<String>>,
        sym_indent: &str
    ) -> String
    fn render_enum(
        e: &ItemEnum,
        config: &Config,
        indent: String
    ) -> String
    fn render_enum_payload(
        fields: &Fields,
        policy: &RenderPolicy
    ) -> Vec<String>
    fn render_header(
        rel: &Path,
        file_depth: usize,
        config: &Config
    ) -> String
    fn render_indent(
        level: usize
    ) -> String
    fn render_output(
        output: &str,
        _config: &Config
    ) -> String
    fn render_signature(
        kind: RenderSig,
        config: &Config,
        scope: &str
    ) -> String
    fn render_struct(
        s: &ItemStruct,
        config: &Config,
        indent: String,
        items: &[Item]
    ) -> String
    fn render_sym_item(
        config: Config,
        item: &'a Item,
        ast: &'a File,
        sym_indent: &str
    ) -> Option<(String, String)>
```

## src/writer.rs

```rs
    // STRUCTS:
    struct MarkdownWriter
        // METHODS:
        fn write_file(
            self,
            files: Vec<RenderedFile>,
            config: &Config
        ) -> String
```



# EMPTY FILES
  Cargo.toml
  RESOLVE.md
  System.txt
  SystemFlow.txt
  SystemFlowDetailed.txt
  eval-.txt
  eval-rs.md
  eval-ts.md
  src/analyzer/mod.rs
  src/context/mod.rs
  src/context/ts.rs
  src/lib.rs
  src/render/mod.rs
  src/render/trait.rs
