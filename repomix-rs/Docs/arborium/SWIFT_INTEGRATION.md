# Swift Integration Guide using Arborium

This guide explains how to integrate Swift support into `repomix-rs` using the `arborium-swift` crate.

## Prerequisites
- `repomix-rs` is currently using `tree-sitter` v0.24.
- `arborium-swift` must be compatible with `tree-sitter` v0.24.

## Step 1: Add Dependency

Modify `repomix-rs/Cargo.toml` to add `arborium-swift`.

```toml
[dependencies]
# ... existing dependencies
arborium-swift = "2.4.6" # Check for latest version matching other arborium crates
```

## Step 2: Update `src/core/compress/languages.rs`

You need to register the Swift language in the `SupportedLanguage` enum and the language loader.

### 2.1 Add Variant
Add `Swift` to `SupportedLanguage` enum.

```rust
pub enum SupportedLanguage {
    // ...
    Swift,
}
```

### 2.2 Update Extension Mapping
Add `.swift` extension support.

```rust
impl SupportedLanguage {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            // ...
            "swift" => Some(Self::Swift),
            _ => None,
        }
    }
}
```

### 2.3 Implement Language Loading
In the `get_ts_language()` method, add the Swift case.

```rust
// In src/core/compress/languages.rs

impl SupportedLanguage {
    pub fn get_ts_language(&self) -> tree_sitter::Language {
        match self {
            // ... other languages
            Self::Swift => arborium_swift::language().into(),
        }
    }
}
```

### 2.4 Implement Helper Methods
Update `name()` and `extensions()` methods to include Swift.

```rust
impl SupportedLanguage {
    pub fn name(&self) -> &'static str {
        match self {
            // ...
            Self::Swift => "swift",
        }
    }

    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            // ...
            Self::Swift => &["swift"],
        }
    }
}
```

### 2.5 Update Extension Map Initialization
Add `SupportedLanguage::Swift` to the list in `get_extension_map()` function.

```rust
fn get_extension_map() -> &'static HashMap<&'static str, SupportedLanguage> {
    EXTENSION_MAP.get_or_init(|| {
        let languages = [
            // ...
            SupportedLanguage::Vue,
            SupportedLanguage::Swift,
        ];
        // ...
    })
}
```
```

## Step 3: Add Swift Queries

Update `src/core/compress/queries.rs`.
You need `tree-sitter` queries to identify function signatures, classes, etc., in Swift files.

Add `QUERY_SWIFT` constant:

```rust
/// Swift query - captures class, struct, enum, function, protocol definitions
pub const QUERY_SWIFT: &str = r#"
(comment) @comment
(import_declaration) @definition.import

(class_declaration
  name: (type_identifier) @name.definition.class) @definition.class

(struct_declaration
  name: (type_identifier) @name.definition.class) @definition.class

(enum_declaration
  name: (type_identifier) @name.definition.class) @definition.class

(protocol_declaration
  name: (type_identifier) @name.definition.interface) @definition.interface

(extension_declaration
  name: (type_identifier) @name.definition.class) @definition.class

(function_declaration
  name: (simple_identifier) @name.definition.function) @definition.function
"#;
```

Then link it in `languages.rs`:

```rust
impl SupportedLanguage {
    pub fn get_query(&self) -> &'static str {
        match self {
            // ...
            Self::Swift => queries::QUERY_SWIFT,
        }
    }
}
```

## Verification

1. Run `cargo check` to ensure dependencies resolve.
2. Run tests to verify the parser loads correctly.
3. Test with a sample `.swift` file using `repomix . --compress --style plain`.
