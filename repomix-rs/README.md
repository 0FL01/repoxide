# Repomix (Rust) 🦀

Rust implementation of [Repomix](https://github.com/yamadashy/repomix) - a powerful tool to pack your entire repository into a single, AI-friendly file.

> **Note**: This is a high-performance port of the original TypeScript tool, designed for speed and singular binary distribution.

## 🚀 Features

- **Blazing Fast**: Written in Rust with parallel file processing (Rayon) and optimized build.
- **AI-Optimized**: Packs your codebase into XML, Markdown, JSON, or Plain text formats.
- **Intelligent compression**: Uses **Tree-sitter** to strip implementation details and keep only signatures (AST-based compression).
- **Token Counting**: Built-in tokenizer (`o200k_base`) to estimate prompt size for LLMs (GPT-4o).
- **Remote Repositories**: Clone and pack remote Git repositories in one command.
- **Security-aware**: Respects `.gitignore` and `.repomixignore` rules.

## 📦 Installation

This project is currently built from source.

### Prerequisites
- Rust toolchain (cargo, rustc)
- Git

### Build
```bash
git clone https://github.com/yourusername/repomix-rs.git
cd repomix-rs
cargo build --release
```

The binary will be available at `./target/release/repomix`.

## 🛠 Usage

### Basic Usage
Pack the current directory into `repomix-output.xml`:
```bash
repomix
```

### Output Formats
Choose between XML (default), Markdown, JSON, or Plain text:
```bash
repomix --style markdown
repomix --style json
repomix --style plain
```

### Compression (Tree-sitter) [Beta]
Extract only essential code structure (functions, classes, interfaces) to save tokens:
```bash
repomix --compress
```

### Remote Repository
Clone and pack a remote repository directly:
```bash
# GitHub shorthand
repomix --remote user/repo

# Full URL
repomix --remote https://github.com/user/repo.git

# With specific branch and options
repomix --remote user/repo --branch main --style markdown --compress
```

### Filtering Files
```bash
# Include specific patterns
repomix --include "src/**/*.rs,Cargo.toml"

# Ignore patterns
repomix --ignore "tests/**,**/examples/**"
```

## 🧩 Comparison with Original (TypeScript)

While this Rust port aims for feature parity, some features are currently placeholders or pending implementation.

| Feature | TypeScript (Original) | Rust (Port) | Status |
|---------|----------------------|-------------|--------|
| **Core Packing** | ✅ | ✅ | Fully Implemented |
| **Output Formats** | ✅ | ✅ | XML, Markdown, JSON, Plain |
| **Tree-sitter Compression** | ✅ (16 languages) | ⚠️ (14 languages) | Missing: Dart, Solidity |
| **Token Counting** | ✅ | ✅ | Uses `tiktoken-rs` |
| **Remote Repositories** | ✅ | ✅ | Supports GitHub, Azure, generic Git |
| **Config File** | ✅ | ✅ | `repomix.config.json` |
| **Clipboard Copy** | ✅ | ❌ | Placeholder (flag exists but does nothing) |
| **Security Check** | ✅ | ❌ | Placeholder (`--no-security-check` ignored) |
| **Token in File Tree** | ✅ | ❌ | Placeholder (Line counts only) |
| **Binary File Reporting** | ✅ (Detailed list) | ✅ (Summary only) |
| **Web Interface** | ✅ | ❌ | CLI tool only |

### ⚠️ Known Placeholders / Limitations in Rust Version

1.  **Clipboard Support**: The `--copy` flag is recognized but currently does not perform any action.
2.  **Security Analysis**: The `--no-security-check` flag is present for compatibility, but the active security scanner (detecting secrets/keys) is not yet implemented.
3.  **Token Count in Tree**: The `--token-count-tree` flag exists but the file tree currently only displays line counts, not token counts.
4.  **Binary File Reporting**: Instead of a detailed list of detected binary files, the tool currently only reports the total count of skipped files.
5.  **Language Support**: 
    -   ✅ Supported: Rust, TypeScript, JavaScript, Python, Go, Java, C, C++, C#, Ruby, PHP, CSS, Vue, Swift
    -   ❌ Missing: Dart, Solidity

## 🤝 Contributing

Contributions are welcome! Please look at the missing features list above for good first issues.

## 📄 License

MIT
