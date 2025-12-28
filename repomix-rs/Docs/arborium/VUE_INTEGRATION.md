# Vue Integration Guide using Arborium

This guide explains how to integrate Vue support into `repomix-rs` using the `arborium` crate.

## Prerequisites
- `repomix-rs` is currently using `tree-sitter` v0.24.
- `arborium` (and its sub-crates like `arborium-vue`) must be compatible with `tree-sitter` v0.24 to pass the `Language` object to the parser.

## Step 1: Add Dependency

Modify `repomix-rs/Cargo.toml` to add `arborium` with the Vue feature.

```toml
[dependencies]
# ... existing dependencies
arborium = { version = "2.4.6", features = ["lang-vue"] } # Check for latest version
```

**Note:** If `arborium` brings in a conflicting version of `tree-sitter`, you might need to use `arborium`'s re-exported `tree_sitter` or align versions. `arborium` v2.4.x is expected to support `tree-sitter` 0.24.

## Step 2: Update `src/core/compress/languages.rs`

You need to register the Vue language in the `SupportedLanguage` enum and the language loader.

### 2.1 Add Variant
Add `Vue` to `SupportedLanguage` enum.

```rust
pub enum SupportedLanguage {
    // ...
    Vue,
}
```

### 2.2 Update Extension Mapping
Add `.vue` extension support.

```rust
impl SupportedLanguage {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            // ...
            "vue" => Some(Self::Vue),
            _ => None,
        }
    }
}
```

### 2.3 Implement Language Loading
In the method where `tree_sitter::Language` is loaded (likely `get_language()` or similar), add the Vue case.

You need to find where `arborium` exposes the Vue language. It is likely one of the following:

Option A (Direct re-export):
```rust
use arborium::vue::language as vue_language;
```

Option B (Crate re-export):
```rust
use arborium_vue::language as vue_language;
```

**Implementation Code:**

```rust
// In src/core/compress/languages.rs

impl SupportedLanguage {
    pub fn get_language(&self) -> tree_sitter::Language {
        match self {
            // ... other languages
            Self::Vue => {
                // Verify the correct import path
                arborium::vue::language() 
                // OR
                // arborium_vue::language()
            },
        }
    }
}
```

## Step 3: Add Vue Queries

Create a new query file or update `src/core/compress/queries.rs`.
You need `tree-sitter` queries to identify function signatures, classes, etc., in Vue files.

Since Vue contains HTML, CSS, and JS/TS, the queries might need to target the `<script>` sections or use injection logic if `repomix` supports it. For now, simple queries targeting top-level structures or script content might suffice.

Example `query_vue.ts` equivalent in Rust (`queries.rs`):

```rust
pub const VUE_QUERY: &str = r#"
(script_element
  (raw_text) @function.body)
"#;
```
*Note: You will need to inspect the actual grammar node names for `tree-sitter-vue` to write correct queries.*

## Verification

1. Run `cargo check` to ensure dependencies resolve.
2. Run tests to verify the parser loads correctly.
3. Test with a sample `.vue` file.
