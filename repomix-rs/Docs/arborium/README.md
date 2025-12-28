# Arborium Crate Documentation

**Website:** [https://docs.rs/arborium](https://docs.rs/arborium)

## Overview
`arborium` is a Rust crate that provides high-performance syntax highlighting and parsing powered by `tree-sitter`. It bundles grammars for over 60 languages, making it a convenient "batteries-included" solution for tree-sitter support.

## Key Features
- **Tree-sitter 0.24 Compatible:** It uses modern `tree-sitter` (check version compatibility, current docs suggest 0.24 support via `arborium-tree-sitter`).
- **Modular Languages:** Languages are enabled via Cargo features (e.g., `lang-vue`, `lang-rust`).
- **Vue Support:** Unlike standalone `tree-sitter-vue` which might be missing or unmaintained on crates.io, `arborium` provides a maintained Vue grammar via `arborium-vue`.

## API Structure
`arborium` typically re-exports the underlying `tree-sitter` library and the enabled language modules.

- **Tree-sitter Access:** `arborium` re-exports `tree_sitter` (via `arborium_tree_sitter`).
- **Language Access:** When a feature like `lang-vue` is enabled, it pulls in the corresponding crate (e.g., `arborium-vue`). You can access the `language()` function to get the `tree_sitter::Language` struct.

## Usage for Repomix
Since `repomix` needs AST parsing (not just highlighting), we will use `arborium` primarily as a source for the `tree_sitter::Language` object for Vue.
