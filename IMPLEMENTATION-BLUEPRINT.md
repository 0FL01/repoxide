# IMPLEMENTATION BLUEPRINT: Repomix TypeScript → Rust

> **Target**: Linux binary для локального запуска
> **Dev Environment**: `toolbox enter ag_dev`
> **Repo Location**: Создать новую директорию `repomix-rs/` в корне

---

## PHASE 1: Project Scaffold ✅ DONE

**Status**: Завершено 2025-12-28
**Goal**: Создать структуру Rust проекта с базовыми зависимостями

**Reference Files (read before implementing):**
- `package.json` — список зависимостей для маппинга на Rust crates
- `src/index.ts` — точка входа и экспорты модулей

**Create:**
```
repomix-rs/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── cli/mod.rs
│   ├── config/mod.rs
│   ├── core/mod.rs
│   ├── remote/mod.rs
│   └── shared/mod.rs
```

**Cargo.toml dependencies:**
```toml
clap = { version = "4", features = ["derive"] }
walkdir = "2"
globset = "0.4"
ignore = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
quick-xml = "0.36"
minijinja = "2"
tiktoken-rs = "0.5"
tree-sitter = "0.22"
encoding_rs = "0.8"
content_inspector = "0.2"
colored = "2"
rayon = "1"
anyhow = "1"
thiserror = "1"
tempfile = "3"
```

**Acceptance**: `cargo check` passes ✅

**Implementation Notes:**
- Создано 26 файлов в директории `repomix-rs/`
- Все модули содержат заглушки для будущей реализации
- `cargo check` проходит успешно (только warnings о неиспользуемом коде)
- Release profile настроен для минимального размера бинарника (LTO, strip, codegen-units=1)

---

## PHASE 2: CLI & Config ✅ DONE

**Status**: Завершено 2025-12-28
**Goal**: Реализовать CLI парсинг и загрузку конфигурации

**Reference Files (read before implementing):**
- `src/cli/cliRun.ts` — CLI опции и их обработка
- `src/cli/types.ts` — типы CLI опций
- `src/config/configSchema.ts` — схема конфигурации с дефолтами
- `src/config/configLoad.ts` — логика загрузки конфига из файла

**Implement:**
- `src/cli/args.rs` — Clap derive structs ✅
- `src/cli/run.rs` — CLI entry point ✅
- `src/config/schema.rs` — Config structs with serde ✅
- `src/config/loader.rs` — Load repomix.config.json ✅

**CLI Options:**
```
repomix [DIRECTORY]           # default: current dir
repomix remote <URL>          # clone and process

--output, -o <PATH>           # output file path
--style <xml|markdown|json|plain>
--compress                    # enable tree-sitter compression
--include <PATTERNS>          # glob patterns to include
--ignore <PATTERNS>           # glob patterns to ignore
--remove-comments             # strip comments
--show-line-numbers           # add line numbers
--copy                        # copy to clipboard
--stdout                      # output to stdout
```

**Acceptance**: `repomix --help` works ✅, config file loads ✅

**Implementation Notes:**
- Полный набор CLI опций (30+ флагов) с использованием `clap` derive macros
- Поддержка subcommands: `remote`, `init`
- Загрузка конфигурации из JSON/JSONC/JSON5 с поддержкой комментариев и trailing commas
- Поиск конфига в локальной директории и глобально (`~/.config/repomix/`)
- Мёрдж CLI args + file config + defaults
- 16 unit-тестов для args, schema и loader
- `repomix --help` выводит полную справку
- `repomix --init` создаёт дефолтный конфиг файл
- `repomix . --verbose` показывает debug информацию

---

## PHASE 3: File System ✅ DONE

**Status**: Завершено 2025-12-28
**Goal**: Поиск, фильтрация и чтение файлов

**Reference Files (read before implementing):**
- `src/core/file/fileSearch.ts` — glob search logic
- `src/core/file/fileCollect.ts` — file collection
- `src/core/file/fileRead.ts` — reading with encoding detection
- `src/core/file/fileTreeGenerate.ts` — directory tree generation
- `src/config/defaultIgnore.ts` — default ignore patterns

**Implement:**
- `src/core/file/search.rs` — walkdir + globset filtering ✅
- `src/core/file/collect.rs` — parallel file reading with rayon ✅
- `src/core/file/tree.rs` — ASCII tree generation ✅
- `src/core/file/mod.rs` — exports ✅

**Logic:**
1. Walk directory with `walkdir` ✅
2. Apply .gitignore via `ignore` crate ✅
3. Apply --include/--ignore glob patterns ✅
4. Detect binary files with `content_inspector` ✅
5. Read text files with `encoding_rs` for charset detection ✅
6. Generate directory tree string ✅

**Acceptance**: Can list and read files from a directory ✅

**Implementation Notes:**
- `search.rs`: Полная поддержка 100+ default ignore patterns, gitignore, .repomixignore
- `collect.rs`: Параллельное чтение через `rayon`, 80+ binary extensions, BOM handling
- `tree.rs`: ASCII дерево с сортировкой (директории первыми), поддержка line counts
- 19 unit-тестов для всех модулей (все проходят)
- Поддержка пустых директорий для вывода в дерево
- Определение кодировки: UTF-8 (с BOM), UTF-16LE/BE с fallback
- Progress callback для отображения прогресса при сборе файлов

---

## PHASE 4: Tree-sitter Compression ✅ DONE

**Status**: Завершено 2025-12-28
**Goal**: Реализовать `--compress` через tree-sitter

**Reference Files (read before implementing):**
- `src/core/treeSitter/parseFile.ts` — main parsing logic, CHUNK_SEPARATOR
- `src/core/treeSitter/languageConfig.ts` — language extensions mapping
- `src/core/treeSitter/languageParser.ts` — parser initialization
- `src/core/treeSitter/queries/` — all 17 query files (queryRust.ts, queryTypescript.ts, etc.)
- `src/core/file/fileProcessContent.ts` — how compression is applied

**Implement:**
- `src/core/compress/parser.rs` — tree-sitter integration ✅
- `src/core/compress/languages.rs` — extension to language mapping ✅
- `src/core/compress/queries.rs` — all queries in single module ✅

**Languages supported (12 из 16):**
| Language | Extensions | Status |
|----------|------------|--------|
| Rust | .rs | ✅ |
| TypeScript | .ts, .tsx, .mts, .mtsx, .cts | ✅ |
| JavaScript | .js, .jsx, .cjs, .mjs, .mjsx | ✅ |
| Python | .py | ✅ |
| Go | .go | ✅ |
| Java | .java | ✅ |
| C | .c, .h | ✅ |
| C++ | .cpp, .hpp, .cc, .cxx, .hxx | ✅ |
| C# | .cs | ✅ |
| Ruby | .rb | ✅ |
| PHP | .php | ✅ |
| CSS | .css | ✅ |
| Swift | .swift | ❌ (нет Rust crate) |
| Dart | .dart | ❌ (нет Rust crate) |
| Solidity | .sol | ❌ (нет Rust crate) |
| Vue | .vue | ✅ |

**Chunk separator:** `⋮----` ✅

**Acceptance**: `--compress` extracts function/class signatures ✅

**Implementation Notes:**
- Миграция на `arborium-*` crates для всех 13 языков (включая Vue)
- `queries.rs`: Все tree-sitter запросы в одном файле, порты из TypeScript
- `languages.rs`: `SupportedLanguage` enum с lazy-init HashMap для маппинга расширений
- `parser.rs`: Полная логика парсинга с StreamingIterator для tree-sitter 0.24
- Поддержка извлечения сигнатур функций/методов (без тела)
- Поддержка извлечения заголовков классов/интерфейсов
- Фильтрация дубликатов по начальной строке
- Мёрж смежных chunks в один блок
- 12 unit-тестов для модуля compress (все проходят)
- Release бинарник: 881KB (stripped, LTO)

---


## PHASE 5: Output Generation

**Goal**: Генерация XML/Markdown/JSON/Plain вывода

**Reference Files (read before implementing):**
- `src/core/output/outputGenerate.ts` — main generation logic
- `src/core/output/outputStyles/xmlStyle.ts` — XML template
- `src/core/output/outputStyles/markdownStyle.ts` — Markdown template
- `src/core/output/outputStyles/plainStyle.ts` — Plain template
- `src/core/output/outputStyleDecorate.ts` — header/summary generation

**Implement:**
- `src/core/output/generate.rs` — orchestration
- `src/core/output/xml.rs` — XML output
- `src/core/output/markdown.rs` — Markdown output
- `src/core/output/json.rs` — JSON output
- `src/core/output/plain.rs` — Plain text output

**XML Structure:**
```xml
<file_summary>...</file_summary>
<directory_structure>...</directory_structure>
<files>
  <file path="path/to/file">content</file>
</files>
```

**Markdown Structure:**
```markdown
# File Summary
...
# Directory Structure
```
...
```
# Files
## File: path/to/file
```lang
content
```
```

**Acceptance**: All 4 output formats generate correctly

---

## PHASE 6: Remote Repository

**Goal**: Поддержка `repomix remote <URL>`

**Reference Files (read before implementing):**
- `src/cli/actions/remoteAction.ts` — remote action logic
- `src/core/git/gitRemoteParse.ts` — URL parsing and validation

**Implement:**
- `src/remote/clone.rs` — git clone to temp directory
- `src/remote/parse.rs` — URL format parsing

**Supported URL formats:**
- `https://github.com/user/repo`
- `https://github.com/user/repo.git`
- `github:user/repo`
- `user/repo` (shorthand for GitHub)

**Logic:**
1. Parse URL format
2. Create temp directory
3. `git clone --depth 1 <url> <temp_dir>`
4. Run packager on temp directory
5. Cleanup temp on exit

**Acceptance**: `repomix remote user/repo` works

---

## PHASE 7: Token Counting & Metrics

**Goal**: Подсчёт токенов и метрики

**Reference Files (read before implementing):**
- `src/core/metrics/TokenCounter.ts` — tiktoken usage
- `src/core/metrics/calculateMetrics.ts` — metrics calculation

**Implement:**
- `src/core/metrics/tokens.rs` — tiktoken-rs integration
- `src/core/metrics/mod.rs` — metrics struct

**Metrics to report:**
- Total files
- Total characters
- Total tokens (o200k_base encoding)
- Per-file token counts (top N)

**Acceptance**: Token count displayed after packing

---

## PHASE 8: Integration & Polish

**Goal**: Собрать всё вместе, тестирование

**Tasks:**
1. Wire all modules in `main.rs`
2. Add colored CLI output
3. Progress indicators
4. Error handling improvements
5. Release build optimization

**Build:**
```bash
toolbox enter ag_dev
cd repomix-rs
cargo build --release
# Binary: target/release/repomix
```

**Final Tests:**
```bash
# Local directory
./target/release/repomix . --style xml
./target/release/repomix . --style markdown --compress

# Remote repository  
./target/release/repomix remote yamadashy/repomix --style xml

# With options
./target/release/repomix . --include "src/**" --ignore "tests/**"
```

**Acceptance**: All commands work, binary size < 15MB

---

## Quick Reference

| TS Module | Rust Module | Key Files |
|-----------|-------------|-----------|
| cli/ | cli/ | cliRun.ts → run.rs |
| config/ | config/ | configSchema.ts → schema.rs |
| core/file/ | core/file/ | fileSearch.ts → search.rs |
| core/treeSitter/ | core/compress/ | parseFile.ts → parser.rs |
| core/output/ | core/output/ | outputGenerate.ts → generate.rs |
| core/metrics/ | core/metrics/ | TokenCounter.ts → tokens.rs |
