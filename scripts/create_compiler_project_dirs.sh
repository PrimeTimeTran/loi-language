#!/usr/bin/env bash

set -euo pipefail

mkdir -p src/{backend/{bundle,symbol,utter},build,cli,diagnostics,frontend,middle,pipeline,registry,ui,watcher} && \
touch src/{lib.rs,main.rs,build_system.rs} && \
touch src/backend/{compile.rs,link_with_clang.rs,llvm.rs,mod.rs} && \
touch src/backend/bundle/{artifact.rs,mod.rs,service.rs,target.rs} && \
touch src/backend/symbol/{mod.rs,registry.rs} && \
touch src/backend/utter/{handler.rs,mod.rs,registry.rs,utter.rs} && \
touch src/build/target.rs && \
touch src/cli/{command.rs,controller.rs,display.rs,ir_runner.rs,mod.rs} && \
touch src/diagnostics/mod.rs && \
touch src/frontend/{ast.rs,lexer.rs,mod.rs,parser.rs,token.rs} && \
touch src/middle/{ir.rs,mod.rs,optimize.rs,semantic.rs} && \
touch src/pipeline/mod.rs && \
touch src/registry/{display.rs,extended.rs,file_meta.rs,mod.rs,registry.rs,test_utils.rs} && \
touch src/ui/mod.rs && \
touch src/watcher/mod.rs
