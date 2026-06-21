# browser-bridge.ts

```ts
// VARIABLES:
babel
instance
```

# create-app-core.ts

```ts
// FUNCTIONS:
function createBootRuntime(fs: VFSAdapter) // void
```

# create-legacy-facade.ts

```ts
// FUNCTIONS:
createLifecycle(facade, runtime, platform) // void
createMessaging(facade, runtime, platform) // void
createModuleManagement(facade, runtime) // void
createRendering(facade, runtime, platform) // void
getModType(mod) // void
```

# create-manifest.ts

```ts
// FUNCTIONS:
function collectFrameworkSignals(entries: string[], files: Record) // void
function createEmptyManifest(slug: string[]) // ExhibitManifest
function detectFlutterFromFiles(entries: string[], files: Record) // void
function detectFromDependencies(packageJson: any) // any
function detectProjectType(entries: string[], files: Record, packageJson: any) // ExhibitProjectType
function isVirtualProject(targetDir: string) // void
```

# load-framework-seeds.ts

```ts
// FUNCTIONS:
function classifyFile(file: string) // any
```

# platform.ts

```ts
// FUNCTIONS:
function ensureRoot() // void
```

# react-renderer.ts

```ts
// FUNCTIONS:
function ensureRoot() // void
```

# resolve-runtime.ts

```ts
// FUNCTIONS:
function buildAssets(projectType: ExhibitProjectType) // SeedFile[]
function findEntryByProjectType(
  projectType: ExhibitProjectType,
  entries: string[],
) // void
function findFlutterEntry(entries: string[]) // void
function findRNEntry(entries: string[]) // void
function findWebEntry(entries: string[]) // void
```

# EMPTY FILES

ast/extractImports.ts
ast/extractRequires.ts
babel-setup.ts
cjs-transformer.ts
detectProjectType.ts
global.d.ts
index.ts
module-error.ts
pipelines/cjsPipeline.ts
pipelines/create-pipeline.ts
pipelines/esmPipeline.ts
runtime/cjs.ts
runtime/create-runtime.ts
runtime/esm.ts
runtime/helpers.ts
static-loader.ts
types.ts
