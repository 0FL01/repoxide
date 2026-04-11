<div align="center">
  <img src="../../apps/web/client/src/public/images/repomix-title.png" alt="Repomix" width="500" height="auto" />
  <p align="center">
    <b>Pack your codebase into AI-friendly formats</b>
  </p>
</div>

[![npm](https://img.shields.io/npm/v/repomix.svg?maxAge=1000)](https://www.npmjs.com/package/repomix)
[![License](https://img.shields.io/github/license/yamadashy/repomix)](LICENSE)

📦 Repomix is a powerful tool that packs your entire repository into a single, AI-friendly file.
It is perfect for when you need to feed your codebase to Large Language Models (LLMs) or other AI tools like Claude, ChatGPT, DeepSeek, Perplexity, Gemini, Gemma, Llama, Grok, and more.

This directory contains the legacy TypeScript implementation. The default Rust workspace now lives at the repository root.

## 🌟 Features

- **AI-Optimized**: Formats your codebase in a way that's easy for AI to understand and process.
- **Token Counting**: Provides token counts for each file and the entire repository.
- **Simple to Use**: You need just one command to pack your entire repository.
- **Customizable**: Easily configure what to include or exclude.
- **Git-Aware**: Automatically respects your `.gitignore`, `.ignore`, and `.repomixignore` files.
- **Security-Focused**: Incorporates [Secretlint](https://github.com/secretlint/secretlint) checks to detect and prevent inclusion of sensitive information.
- **Code Compression**: The `--compress` option uses [Tree-sitter](https://github.com/tree-sitter/tree-sitter) to extract key code elements, reducing token count while preserving structure.

## 🛠️ Self-Hosting / Deployment

You can host the legacy TypeScript-based web interface using `legacy/compose.yml`.

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/)
- [Docker Compose](https://docs.docker.com/compose/install/)

### Deployment Steps

1. **Clone the repository**:
   ```bash
   git clone https://github.com/yamadashy/repomix.git
   cd repomix
   ```

2. **Start the services**:
    Run the following command to build and start the application in detached mode:
   ```bash
   docker compose -f legacy/compose.yml up -d
   ```

3. **Access the application**:
    Once the containers are running, you can access the web interface at:
    - **Client**: `http://localhost:5173`

### Service Architecture

The legacy compose file orchestrates the following services:

- **client**: The frontend web application.
  - Exposed Port: `5173`
  - Environment: `NODE_ENV=development`
  - Volume: Mounts `../apps/web/client` for development.

- **server**: The backend API service.
  - Exposed Port: `8080`
  - Environment: `NODE_ENV=development`
  - Volume: Mounts `./website-server-ts` for development.

To stop the services, run:
```bash
docker-compose down
```

## 📊 Usage

To pack your entire repository:

```bash
repomix
```

To pack a specific directory:

```bash
repomix path/to/directory
```

To pack specific files using glob patterns:

```bash
repomix --include "src/**/*.ts,**/*.md"
```

To exclude specific files:

```bash
repomix --ignore "**/*.log,tmp/"
```

To pack a remote repository:

```bash
repomix --remote https://github.com/yamadashy/repomix
```

To compress the output (extracts essential code structure):

```bash
repomix --compress
```

To initialize a new configuration file (`repomix.config.json`):

```bash
repomix --init
```

### Output Formats

Repomix supports multiple output formats via the `--style` option:

- **XML** (default): Structured hierarchical format, optimal for Claude and many LLMs.
- **Markdown**: Human-readable structure.
- **JSON**: Structured for programmatic processing.
- **Plain**: Simple text dump.

```bash
repomix --style markdown
```

## ⚙️ Configuration

Create a `repomix.config.json` file to persist your settings:

```json
{
  "output": {
    "filePath": "repomix-output.xml",
    "style": "xml",
    "compress": false,
    "removeComments": false
  },
  "include": ["**/*"],
  "ignore": {
    "useGitignore": true,
    "customPatterns": ["**/*.test.ts"]
  }
}
```

See the [CLI help](#command-line-options) or run `repomix --help` for all available options.

## 🤖 GitHub Actions

You can use Repomix in your GitHub Actions workflows:

```yaml
- name: Pack repository with Repomix
  uses: yamadashy/repomix/.github/actions/repomix@main
  with:
    output: repomix-output.xml
    style: xml
```

## 📚 Library Usage

Repomix can be used as a Node.js library:

```javascript
import { runCli } from 'repomix';

await runCli(['.'], process.cwd(), {
  output: 'output.xml',
  style: 'xml'
});
```

## 📜 License

This project is licensed under the [MIT License](LICENSE).
