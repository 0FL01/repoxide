<div align="center">
  <img src="../../apps/web/client/src/public/images/repoxide-title.png" alt="Repoxide" width="500" height="auto" />
  <p align="center">
    <b>Pack your codebase into AI-friendly formats</b>
  </p>
</div>

[![npm](https://img.shields.io/npm/v/repoxide.svg?maxAge=1000)](https://www.npmjs.com/package/repoxide)
[![License](https://img.shields.io/github/license/yamadashy/repoxide)](LICENSE)

📦 Repoxide is a powerful tool that packs your entire repository into a single, AI-friendly file.
It is perfect for when you need to feed your codebase to Large Language Models (LLMs) or other AI tools like Claude, ChatGPT, DeepSeek, Perplexity, Gemini, Gemma, Llama, Grok, and more.

This directory contains the legacy TypeScript implementation. The default Rust workspace now lives at the repository root.

## 🌟 Features

- **AI-Optimized**: Formats your codebase in a way that's easy for AI to understand and process.
- **Token Counting**: Provides token counts for each file and the entire repository.
- **Simple to Use**: You need just one command to pack your entire repository.
- **Customizable**: Easily configure what to include or exclude.
- **Git-Aware**: Automatically respects your `.gitignore`, `.ignore`, and `.repoxideignore` files.
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
   git clone https://github.com/yamadashy/repoxide.git
   cd repoxide
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
repoxide
```

To pack a specific directory:

```bash
repoxide path/to/directory
```

To pack specific files using glob patterns:

```bash
repoxide --include "src/**/*.ts,**/*.md"
```

To exclude specific files:

```bash
repoxide --ignore "**/*.log,tmp/"
```

To pack a remote repository:

```bash
repoxide --remote https://github.com/yamadashy/repoxide
```

To compress the output (extracts essential code structure):

```bash
repoxide --compress
```

To initialize a new configuration file (`repoxide.config.json`):

```bash
repoxide --init
```

### Output Formats

Repoxide supports multiple output formats via the `--style` option:

- **XML** (default): Structured hierarchical format, optimal for Claude and many LLMs.
- **Markdown**: Human-readable structure.
- **JSON**: Structured for programmatic processing.
- **Plain**: Simple text dump.

```bash
repoxide --style markdown
```

## ⚙️ Configuration

Create a `repoxide.config.json` file to persist your settings:

```json
{
  "output": {
    "filePath": "repoxide-output.xml",
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

See the [CLI help](#command-line-options) or run `repoxide --help` for all available options.

## 🤖 GitHub Actions

You can use Repoxide in your GitHub Actions workflows:

```yaml
- name: Pack repository with Repoxide
  uses: yamadashy/repoxide/.github/actions/repoxide@main
  with:
    output: repoxide-output.xml
    style: xml
```

## 📚 Library Usage

Repoxide can be used as a Node.js library:

```javascript
import { runCli } from 'repoxide';

await runCli(['.'], process.cwd(), {
  output: 'output.xml',
  style: 'xml'
});
```

## 📜 License

This project is licensed under the [MIT License](LICENSE).
