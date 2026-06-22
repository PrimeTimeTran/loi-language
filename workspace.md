# Workspace Overview

This document gives a high level overview of this workspaces's crates.
The workspace is primarily configured by Rust/Cargo.

## Crates

### [`/crates/vfs`](crates/vfs)

A virtual file system which which enables building browser IDEs, plugins/extensions, analysis tools & more.
The aim of this crate is to be language agnostic, lighting fast, and run anywhere.

### [`/crates/loid`](crates/loid)

LOID is the higher-level development environment/tooling layer built around LOI projects.
It focuses on project intelligence: understanding a workspace, maintaining registries/configuration,
organizing global vs workspace knowledge, caching analysis results, and providing a structured
representation of a codebase. Where FML extracts the flow/skeleton of code, LOID is more about
managing the project model around that information — giving tools and humans a persistent,
layered view of a project.

### [`/crates/loi`](crates/loi)

Loi is the core language/toolchain/paradigm/"dream" crate. It defines the language concepts,
conventions, and execution model around a new .loi programming language files/projects, including
ideas like public-by-default symbols, privacy rules, module organization, project
structure, and the foundation for future compilation/runtime
tooling.

The goal is to create a a new generation language ecosystem where file system, metadata,
comments, analysis, and project context are first-class concepts rather than separate disconnected
tools.

### [`/crates/fml`](crates/fml)

Flow Model Language is a tool for enabling multi levels/layers of a project. Sometimes we want to know how the data flows
throughout the application as a whole so aren't interested in every detail of a function like it's params & types but only the mods &
structures used in the application. FML parses the source code into the skeleton so that humans and machines can get context and strip
away noise consistently across any language and level of analysis.

These crates have all been analyzed by FML for reference.

- [`/.agents/FML.md`](.agents/FML.md)
- [`/.agents/VFS.md`](.agents/VFS.md)
- [`/.agents/FML.md`](.agents/FML.md)
- [`/.agents/LOID.md`](.agents/LOID.md)
- [`/.agents/LLVM.md`](.agents/LLVM.md)

Inspect them to see the I/Os of a crate as appropriate given your current task.
