use axum::{
    extract::{Multipart, Path, State},
    http::{
        header::{CONTENT_TYPE, HOST, ORIGIN, REFERER},
        HeaderMap, StatusCode,
    },
    response::{Html, IntoResponse, Response},
};
use maud::{html, Markup, PreEscaped, DOCTYPE};
use std::{path::Component, path::PathBuf, sync::Arc};

use crate::{
    state::AppState,
    types::{PackOptions, PackResponse},
};

use super::pack::{build_folder_upload, execute_pack_job, PackJob, PackSource};

const APP_CSS: &str = r#"
:root {
  color-scheme: light dark;
  --bg: #f8f7f4;
  --bg-accent: #fff4e8;
  --surface: rgba(255, 255, 255, 0.94);
  --surface-strong: #ffffff;
  --surface-muted: #fffaf5;
  --border: rgba(15, 23, 42, 0.1);
  --border-strong: rgba(249, 115, 22, 0.22);
  --text: #0f172a;
  --muted: #526072;
  --muted-strong: #334155;
  --brand: #f97316;
  --brand-strong: #ea580c;
  --brand-soft: rgba(249, 115, 22, 0.12);
  --success: #15803d;
  --danger: #b91c1c;
  --danger-soft: rgba(185, 28, 28, 0.1);
  --shadow: 0 18px 40px rgba(15, 23, 42, 0.08);
  --shadow-soft: 0 8px 24px rgba(15, 23, 42, 0.05);
  --radius-xl: 24px;
  --radius-lg: 18px;
  --radius-md: 14px;
}

@media (prefers-color-scheme: dark) {
  :root {
    --bg: #0b1120;
    --bg-accent: #1b130b;
    --surface: rgba(15, 23, 42, 0.94);
    --surface-strong: #111827;
    --surface-muted: #0f172a;
    --border: rgba(148, 163, 184, 0.18);
    --border-strong: rgba(251, 146, 60, 0.28);
    --text: #e5eefc;
    --muted: #a5b4cc;
    --muted-strong: #d7e2f3;
    --brand: #fb923c;
    --brand-strong: #fdba74;
    --brand-soft: rgba(251, 146, 60, 0.16);
    --success: #86efac;
    --danger: #fca5a5;
    --danger-soft: rgba(252, 165, 165, 0.12);
    --shadow: 0 18px 45px rgba(2, 6, 23, 0.38);
    --shadow-soft: 0 10px 26px rgba(2, 6, 23, 0.26);
  }
}

* { box-sizing: border-box; }

html { scroll-behavior: smooth; }

body {
  margin: 0;
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: linear-gradient(180deg, var(--bg-accent) 0%, var(--bg) 220px);
  color: var(--text);
}

a { color: inherit; }

button,
input,
select,
textarea {
  font: inherit;
}

.page {
  max-width: 1160px;
  margin: 0 auto;
  padding: 32px 20px 56px;
}

.stack {
  display: grid;
  gap: 20px;
}

.hero {
  display: grid;
  gap: 18px;
  margin-bottom: 12px;
}

.hero-top {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.hero-copy {
  display: grid;
  gap: 14px;
  max-width: 760px;
}

.badge {
  width: fit-content;
  padding: 6px 10px;
  border-radius: 999px;
  background: var(--brand-soft);
  color: var(--brand-strong);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.hero h1 {
  margin: 0;
  font-size: clamp(2.3rem, 4vw, 3.6rem);
  line-height: 1.05;
  letter-spacing: -0.03em;
}

.hero p {
  margin: 0;
  color: var(--muted);
  font-size: 1.02rem;
  line-height: 1.65;
}

.hero-points {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.hero-point {
  padding: 8px 12px;
  border-radius: 999px;
  background: var(--surface);
  border: 1px solid var(--border);
  color: var(--muted-strong);
  font-size: 0.93rem;
  font-weight: 600;
  box-shadow: var(--shadow-soft);
}

.locale-switcher {
  display: inline-flex;
  width: fit-content;
  gap: 8px;
  padding: 6px;
  border-radius: 999px;
  background: var(--surface);
  border: 1px solid var(--border);
  box-shadow: var(--shadow-soft);
}

.locale-link {
  padding: 8px 12px;
  border-radius: 999px;
  text-decoration: none;
  color: var(--muted);
  font-weight: 600;
}

.locale-link.active {
  background: var(--surface-strong);
  color: var(--text);
  box-shadow: inset 0 0 0 1px var(--border);
}

.card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  padding: 24px;
  box-shadow: var(--shadow-soft);
}

.card h2,
.card h3 {
  margin: 0;
}

.card p {
  color: var(--muted);
}

.section-heading {
  display: grid;
  gap: 8px;
}

.section-heading p {
  max-width: 720px;
}

.notice {
  border-color: rgba(185, 28, 28, 0.18);
  background: linear-gradient(180deg, var(--danger-soft), var(--surface));
}

.notice h2 {
  color: var(--danger);
}

.notice-message {
  color: var(--text) !important;
  font-weight: 600;
}

.notice-hint {
  color: var(--danger) !important;
  font-size: 0.95rem;
}

.notice-inline {
  padding: 12px 14px;
  border-radius: var(--radius-md);
  background: var(--brand-soft);
  color: var(--brand-strong);
  font-size: 0.94rem;
  font-weight: 600;
}

.workspace-form {
  display: grid;
  gap: 20px;
  margin-top: 20px;
}

.source-switch {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
}

.source-tab {
  display: block;
  cursor: pointer;
}

.source-tab-body {
  text-align: left;
  padding: 16px;
  border-radius: var(--radius-lg);
  border: 1px solid var(--border);
  background: var(--surface-muted);
  display: grid;
  gap: 6px;
  transition: border-color 0.2s ease, transform 0.2s ease, box-shadow 0.2s ease, background 0.2s ease;
}

.source-tab:hover .source-tab-body {
  border-color: var(--border-strong);
  transform: translateY(-1px);
}

.source-tab-input:checked + .source-tab-body {
  background: var(--surface-strong);
  border-color: var(--brand);
  box-shadow: 0 0 0 3px var(--brand-soft);
}

.source-tab-title {
  font-size: 0.95rem;
  font-weight: 700;
  color: var(--muted-strong);
}

.source-tab-copy {
  font-size: 0.92rem;
  color: var(--muted);
  line-height: 1.4;
}

.source-panel {
  display: grid;
  gap: 14px;
  padding: 20px;
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  background: var(--surface-muted);
}

.source-panel[hidden] {
  display: none;
}

.source-panel-header {
  display: grid;
  gap: 8px;
}

.source-panel-title-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
}

.source-pill {
  display: inline-flex;
  align-items: center;
  padding: 5px 10px;
  border-radius: 999px;
  background: var(--brand-soft);
  color: var(--brand-strong);
  font-size: 0.82rem;
  font-weight: 700;
}

.field {
  display: grid;
  gap: 8px;
}

.field-row {
  display: grid;
  gap: 16px;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
}

label {
  font-weight: 700;
  color: var(--muted-strong);
}

input[type="text"],
input[type="url"],
select,
textarea {
  width: 100%;
  min-height: 48px;
  border-radius: var(--radius-md);
  border: 1px solid var(--border);
  background: var(--surface-strong);
  color: var(--text);
  padding: 12px 14px;
}

input[type="file"] {
  display: none;
}

input[type="text"]::placeholder,
input[type="url"]::placeholder,
textarea::placeholder {
  color: color-mix(in srgb, var(--muted) 80%, transparent);
}

input[type="text"]:focus,
input[type="url"]:focus,
select:focus,
textarea:focus,
button:focus-visible,
.upload-zone:focus-visible,
.locale-link:focus-visible {
  outline: none;
  border-color: var(--brand);
  box-shadow: 0 0 0 4px var(--brand-soft);
}

.source-tab-input:focus-visible + .source-tab-body {
  outline: none;
  border-color: var(--brand);
  box-shadow: 0 0 0 4px var(--brand-soft);
}

textarea {
  min-height: 340px;
  resize: vertical;
  font-family: "SFMono-Regular", "Cascadia Code", "JetBrains Mono", monospace;
  font-size: 0.95rem;
  line-height: 1.55;
}

.helper {
  margin: 0;
  font-size: 0.93rem;
  line-height: 1.5;
  color: var(--muted);
}

.upload-zone {
  display: grid;
  gap: 12px;
  padding: 18px;
  border: 1.5px dashed rgba(37, 99, 235, 0.26);
  border-radius: var(--radius-lg);
  background: linear-gradient(180deg, var(--surface-strong), var(--surface-muted));
  cursor: pointer;
  transition: border-color 0.2s ease, box-shadow 0.2s ease, background 0.2s ease;
}

.upload-zone:hover {
  border-color: var(--brand);
}

.upload-zone.is-dragover {
  border-color: var(--brand);
  box-shadow: 0 0 0 4px var(--brand-soft);
}

.upload-zone-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.upload-zone-title {
  font-weight: 700;
  color: var(--muted-strong);
}

.upload-zone-copy {
  color: var(--muted);
  font-size: 0.95rem;
  line-height: 1.5;
}

.upload-zone-action {
  width: fit-content;
  padding: 8px 12px;
  border-radius: 999px;
  background: var(--brand-soft);
  color: var(--brand-strong);
  font-size: 0.92rem;
  font-weight: 700;
}

.upload-summary {
  color: var(--muted-strong);
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.options-shell {
  display: grid;
  gap: 16px;
  padding: 20px;
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  background: var(--surface-muted);
}

.options-header {
  display: grid;
  gap: 6px;
}

.checkbox-grid {
  display: grid;
  gap: 12px;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
}

.checkbox {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px;
  border-radius: var(--radius-md);
  border: 1px solid var(--border);
  background: var(--surface-strong);
}

.checkbox input {
  margin-top: 4px;
  accent-color: var(--brand);
}

.checkbox-copy {
  display: grid;
  gap: 4px;
}

.checkbox-copy strong {
  font-size: 0.97rem;
}

.checkbox-copy span {
  color: var(--muted);
  font-size: 0.9rem;
  line-height: 1.4;
}

.form-actions,
.result-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
}

.form-footnote {
  margin: 0;
  color: var(--muted);
  font-size: 0.92rem;
}

button,
.button-link {
  appearance: none;
  border: none;
  cursor: pointer;
  border-radius: 999px;
  padding: 12px 16px;
  font-weight: 700;
  text-decoration: none;
  transition: transform 0.2s ease, background 0.2s ease, border-color 0.2s ease, color 0.2s ease;
}

button:hover,
.button-link:hover {
  transform: translateY(-1px);
}

button.primary,
.button-link.primary {
  background: var(--brand);
  color: white;
  box-shadow: 0 10px 24px rgba(37, 99, 235, 0.18);
}

button.primary:hover,
.button-link.primary:hover {
  background: var(--brand-strong);
}

button.secondary,
.button-link.secondary {
  background: var(--surface-strong);
  color: var(--text);
  border: 1px solid var(--border);
}

.result-card {
  padding: 0;
  overflow: hidden;
}

.result-shell {
  display: grid;
  grid-template-columns: minmax(0, 1.6fr) minmax(280px, 0.95fr);
}

.result-main,
.result-sidebar {
  padding: 24px;
  display: grid;
  gap: 18px;
}

.result-main {
  border-right: 1px solid var(--border);
  background: var(--surface);
}

.result-sidebar {
  background: linear-gradient(180deg, var(--surface-muted), var(--surface));
}

.result-header {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.output-panel {
  display: grid;
  gap: 10px;
}

.overview-grid,
.summary-grid,
.top-files-list {
  display: grid;
  gap: 12px;
}

.summary-grid {
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
}

.meta-card,
.metric-card,
.top-file-item {
  padding: 16px;
  border-radius: var(--radius-lg);
  border: 1px solid var(--border);
  background: var(--surface-strong);
}

.meta-label,
.metric-label {
  display: block;
  margin-bottom: 8px;
  color: var(--muted);
  font-size: 0.84rem;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.meta-value {
  font-weight: 600;
  line-height: 1.45;
  word-break: break-word;
}

.metric-value {
  font-size: 1.35rem;
  font-weight: 800;
  letter-spacing: -0.02em;
  font-variant-numeric: tabular-nums;
}

.metric-subtext {
  margin-top: 4px;
  font-size: 0.9rem;
  color: var(--muted);
}

.top-files {
  display: grid;
  gap: 12px;
}

.top-files-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.top-file-path {
  font-weight: 600;
  line-height: 1.45;
  word-break: break-word;
}

.top-file-stats {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 10px;
}

.stat-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border-radius: 999px;
  background: var(--surface-muted);
  color: var(--muted-strong);
  font-size: 0.88rem;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.stat-chip.strong {
  background: var(--brand-soft);
  color: var(--brand-strong);
}

.footer {
  margin-top: 24px;
  color: var(--muted);
  text-align: center;
  font-size: 0.95rem;
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

@media (max-width: 980px) {
  .source-switch {
    grid-template-columns: 1fr;
  }

  .result-shell {
    grid-template-columns: 1fr;
  }

  .result-main {
    border-right: none;
    border-bottom: 1px solid var(--border);
  }
}

@media (max-width: 720px) {
  .page {
    padding: 24px 16px 40px;
  }

  .card,
  .result-main,
  .result-sidebar {
    padding: 20px;
  }

  .hero h1 {
    font-size: clamp(2rem, 9vw, 2.8rem);
  }

  .form-actions,
  .result-actions {
    flex-direction: column;
    align-items: stretch;
  }

  button,
  .button-link {
    width: 100%;
  }

  .upload-zone-head {
    flex-direction: column;
    align-items: flex-start;
  }
}

.feedback-stack {
  display: grid;
  gap: 20px;
}

.state-card {
  display: grid;
  gap: 14px;
}

.state-card[hidden] {
  display: none;
}

.state-title-row,
.toolbar-row,
.selection-toolbar,
.selection-summary,
.result-tab-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.state-title-row h3,
.selection-toolbar h3,
.result-tab-row h3 {
  margin: 0;
}

.status-chip-row,
.micro-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.status-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border-radius: 999px;
  background: var(--surface-muted);
  border: 1px solid var(--border);
  color: var(--muted-strong);
  font-size: 0.88rem;
  font-weight: 600;
}

.status-chip.strong {
  background: var(--brand-soft);
  color: var(--brand-strong);
  border-color: color-mix(in srgb, var(--brand) 30%, var(--border));
}

.progress-shell {
  display: grid;
  gap: 8px;
}

.progress-track {
  width: 100%;
  height: 10px;
  border-radius: 999px;
  background: var(--surface-muted);
  border: 1px solid var(--border);
  overflow: hidden;
}

.progress-value {
  height: 100%;
  background: linear-gradient(90deg, var(--brand), color-mix(in srgb, var(--brand) 65%, white));
  transition: width 0.2s ease;
}

.progress-copy {
  color: var(--muted);
  font-size: 0.92rem;
}

.result-tabs {
  display: flex;
  gap: 8px;
}

.result-tab {
  padding: 10px 14px;
  border-radius: 10px;
  background: transparent;
  border: 1px solid var(--border);
  color: var(--muted);
}

.result-tab.active {
  background: var(--brand-soft);
  border-color: color-mix(in srgb, var(--brand) 35%, var(--border));
  color: var(--brand-strong);
}

.output-viewer {
  min-height: 420px;
  max-height: 70vh;
  padding: 18px;
  border-radius: var(--radius-lg);
  border: 1px solid var(--border);
  background: color-mix(in srgb, var(--surface-strong) 90%, black 10%);
  color: var(--text);
  font-family: "SFMono-Regular", "Cascadia Code", "JetBrains Mono", monospace;
  font-size: 0.94rem;
  line-height: 1.6;
  white-space: pre;
}

.output-viewer[readonly] {
  cursor: text;
}

.output-viewer::selection {
  background: var(--brand-soft);
}

.selection-shell {
  display: grid;
  gap: 14px;
}

.selection-table-wrap {
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  overflow: hidden;
  background: var(--surface-strong);
}

.selection-table-scroll {
  max-height: 420px;
  overflow: auto;
}

.selection-table {
  width: 100%;
  border-collapse: collapse;
}

.selection-table th,
.selection-table td {
  padding: 12px 14px;
  border-bottom: 1px solid var(--border);
  text-align: left;
}

.selection-table th {
  position: sticky;
  top: 0;
  z-index: 1;
  background: var(--surface-muted);
  color: var(--muted);
  font-size: 0.85rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.selection-table tr[data-selected="true"] {
  background: color-mix(in srgb, var(--brand-soft) 70%, var(--surface-strong));
}

.selection-table tbody tr:hover {
  background: color-mix(in srgb, var(--brand-soft) 35%, var(--surface-strong));
}

.selection-checkbox,
.selection-master-checkbox {
  width: 16px;
  height: 16px;
  accent-color: var(--brand);
}

.selection-path {
  font-weight: 600;
  word-break: break-word;
}

.selection-token-cell,
.selection-char-cell {
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.subtle-copy {
  color: var(--muted);
  margin: 0;
}

.mobile-share-note {
  color: var(--muted);
  font-size: 0.9rem;
}

.button-row {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.empty-state {
  color: var(--muted);
}

@media (max-width: 720px) {
  .result-tabs,
  .status-chip-row,
  .micro-actions,
  .button-row {
    width: 100%;
  }

  .result-tabs button,
  .micro-actions button,
  .button-row button {
    flex: 1 1 100%;
  }

  .selection-table th:nth-child(3),
  .selection-table td:nth-child(3) {
    display: none;
  }
}
"#;

const APP_JS: &str = r#"
const numberFormatter = new Intl.NumberFormat(document.documentElement.lang || undefined);
const dateTimeFormatter = new Intl.DateTimeFormat(document.documentElement.lang || undefined, {
  dateStyle: 'medium',
  timeStyle: 'short',
});
const DEFAULT_OPTIONS = {
  format: 'xml',
  removeComments: false,
  removeEmptyLines: false,
  showLineNumbers: false,
  fileSummary: true,
  directoryStructure: true,
  includePatterns: '',
  ignorePatterns: '',
  outputParsable: false,
  compress: false,
};
const MAX_CLIENT_UPLOAD_BYTES = 100 * 1024 * 1024;
const MAX_FOLDER_FILES = 50000;
const MAX_FOLDER_DEPTH = 50;
const MAX_BROWSER_FILE_SELECTION_FILES = 2000;
const FILE_SELECTION_WARNING_THRESHOLD = 500;
const CHUNK_SIZE = 1024 * 1024;
const CHUNKED_UPLOAD_THRESHOLD = 2 * 1024 * 1024;
const TIMEOUT_MS = 600000;
const URL_HISTORY_KEY = 'repomix-url-history';
const locale = document.documentElement.lang === 'ru' ? 'ru' : 'en';
const TEXT = locale === 'ru' ? {
  invalidUrl: 'Введите корректный URL Git-репозитория, SSH-адрес или shorthand вида owner/repo.',
  loadingTitle: 'Собираем пакет',
  loadingBody: 'Запрос выполняется на Rust-бэкенде. Для крупных архивов это может занять несколько минут.',
  uploadProgress: 'Загрузка ZIP: {progress}% ({current}/{total} чанков)',
  processing: 'Обработка репозитория и построение результата...',
  timeout: 'Запрос превысил лимит в 10 минут. Попробуйте сузить набор файлов через include/ignore паттерны.',
  cancelled: 'Запрос отменен.',
  requestFailed: 'Запрос завершился ошибкой',
  requestFailedHint: 'Проверьте источник, загруженные данные и паттерны, затем повторите попытку.',
  packedOutput: 'Готовый результат',
  packedOutputBody: 'Скопируйте результат, скачайте его или откройте в совместимом мобильном приложении.',
  copy: 'Копировать',
  copied: 'Скопировано',
  download: 'Скачать',
  share: 'Поделиться',
  shared: 'Поделились',
  shareUnavailable: 'Поделиться можно только на мобильных устройствах с поддержкой Web Share API.',
  repository: 'Репозиторий',
  format: 'Формат',
  generatedAt: 'Сгенерировано',
  files: 'Файлы',
  characters: 'Символы',
  tokens: 'Токены',
  topFiles: 'Топ файлов по токенам',
  topFilesBody: 'Начните с самых тяжелых файлов, если хотите уменьшить размер промпта.',
  resultTab: 'Результат',
  filesTab: 'Выбор файлов',
  fileSelection: 'Выбор файлов',
  fileSelectionBody: 'Выберите файлы из `metadata.allFiles` и пересоберите пакет только по ним.',
  fileSelectionUnavailable: 'Выбор файлов в браузере доступен только для пакетов до {threshold} файлов.',
  selectAll: 'Выбрать все',
  deselectAll: 'Снять выделение',
  repack: 'Пересобрать',
  repacking: 'Пересборка...',
  selectedSummary: '{selected} из {total} файлов, {tokens} токенов ({percent})',
  selectionTableLabel: 'Таблица выбора файлов',
  toggleAllFiles: 'Переключить выбор всех файлов',
  toggleFile: 'Переключить выбор файла {path}',
  emptyResult: 'Результат пока не получен.',
  pack: 'Сформировать пакет',
  cancel: 'Отменить',
  reset: 'Сбросить',
  zipRequired: 'Выберите ZIP-файл.',
  zipOnly: 'Допускаются только ZIP-файлы.',
  zipTooLarge: 'ZIP-файл должен быть не больше 100 МБ для браузерной загрузки.',
  folderRequired: 'Выберите папку.',
  folderEmpty: 'Выбранная папка не содержит файлов.',
  uploadTooLarge: 'Для прямой загрузки папка слишком большая.',
  folderTooManyFiles: 'Папка содержит слишком много файлов для браузерной загрузки.',
  folderTooDeep: 'Папка содержит слишком глубокую вложенность.',
  fileSelectionWarning: 'Вы выбрали более {threshold} файлов. Пересборка может занять заметно больше времени.',
} : {
  invalidUrl: 'Please enter a valid Git repository URL, SSH address, or owner/repo shorthand.',
  loadingTitle: 'Packing repository',
  loadingBody: 'The Rust backend is generating the pack. Larger archives can take a few minutes.',
  uploadProgress: 'Uploading ZIP: {progress}% ({current}/{total} chunks)',
  processing: 'Processing repository and generating output...',
  timeout: 'The request hit the 10 minute timeout. Try narrowing the scope with include/ignore patterns.',
  cancelled: 'The request was cancelled.',
  requestFailed: 'Request failed',
  requestFailedHint: 'Check the selected source, uploaded data, and patterns, then try again.',
  packedOutput: 'Packed output',
  packedOutputBody: 'Copy the result, download it, or hand it off to a compatible mobile app.',
  copy: 'Copy',
  copied: 'Copied',
  download: 'Download',
  share: 'Share',
  shared: 'Shared',
  shareUnavailable: 'Share is only available on mobile devices with Web Share API support.',
  repository: 'Repository',
  format: 'Format',
  generatedAt: 'Generated At',
  files: 'Files',
  characters: 'Characters',
  tokens: 'Tokens',
  topFiles: 'Top files by token count',
  topFilesBody: 'Start with the biggest prompt contributors if you want to shrink the output.',
  resultTab: 'Result',
  filesTab: 'File Selection',
  fileSelection: 'File Selection',
  fileSelectionBody: 'Choose files from `metadata.allFiles` and re-pack only that subset.',
  fileSelectionUnavailable: 'Browser file selection is available only for packs up to {threshold} files.',
  selectAll: 'Select All',
  deselectAll: 'Deselect All',
  repack: 'Re-pack Selected',
  repacking: 'Re-packing...',
  selectedSummary: '{selected} of {total} files, {tokens} tokens ({percent})',
  selectionTableLabel: 'File selection table',
  toggleAllFiles: 'Toggle all files',
  toggleFile: 'Toggle file {path}',
  emptyResult: 'No result yet.',
  pack: 'Generate pack',
  cancel: 'Cancel',
  reset: 'Reset',
  zipRequired: 'Choose a ZIP file.',
  zipOnly: 'Only ZIP files are supported.',
  zipTooLarge: 'ZIP uploads are limited to 100 MB in the browser UI.',
  folderRequired: 'Choose a folder.',
  folderEmpty: 'The selected folder is empty.',
  uploadTooLarge: 'The selected folder is too large for direct upload.',
  folderTooManyFiles: 'The selected folder contains too many files for browser upload.',
  folderTooDeep: 'The selected folder is nested too deeply for browser upload.',
  fileSelectionWarning: 'You selected more than {threshold} files. Re-packing may take noticeably longer.',
};

function fillTemplate(template, values) {
  return template.replace(/\{(\w+)\}/g, (_, key) => values[key] ?? '');
}

function formatBytes(bytes) {
  if (!Number.isFinite(bytes) || bytes <= 0) {
    return '0 B';
  }
  const units = ['B', 'KB', 'MB', 'GB'];
  let value = bytes;
  let index = 0;
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024;
    index += 1;
  }
  const precision = value >= 10 || index === 0 ? 0 : 1;
  return `${value.toFixed(precision)} ${units[index]}`;
}

function formatTimestamp(value) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return dateTimeFormatter.format(date);
}

function escapeHtml(value) {
  return String(value)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function isValidRemoteValue(remoteValue) {
  const namePattern = '[a-zA-Z0-9](?:[a-zA-Z0-9._-]*[a-zA-Z0-9])?';
  const shortFormRegex = new RegExp(`^${namePattern}/${namePattern}$`);
  const sshRegex = /^git@[^:]+:[^/]+\/[^/]+(?:\.git)?$/;
  if (shortFormRegex.test(remoteValue)) {
    return true;
  }
  if (sshRegex.test(remoteValue)) {
    return true;
  }
  try {
    new URL(remoteValue);
    return true;
  } catch (_) {
    return false;
  }
}

function parseUrlParameters() {
  const params = new URLSearchParams(window.location.search);
  const next = {};
  const repo = params.get('repo');
  if (repo) {
    next.repo = repo.trim();
  }
  const format = params.get('format');
  const legacyStyle = params.get('style');
  const formatValue = ['xml', 'markdown', 'plain'].includes(format || '')
    ? format
    : (['xml', 'markdown', 'plain'].includes(legacyStyle || '') ? legacyStyle : null);
  if (formatValue) {
    next.format = formatValue;
  }
  const include = params.get('include');
  if (include) {
    next.includePatterns = include;
  }
  const ignore = params.get('ignore');
  if (ignore) {
    next.ignorePatterns = ignore;
  }
  ['removeComments', 'removeEmptyLines', 'showLineNumbers', 'fileSummary', 'directoryStructure', 'outputParsable', 'compress'].forEach((key) => {
    const value = params.get(key);
    if (value !== null) {
      next[key] = ['true', '1', 'yes', 'on'].includes(value.toLowerCase());
    }
  });
  return next;
}

function buildDownloadName(format) {
  if (format === 'xml') return 'repomix-output.xml';
  if (format === 'markdown') return 'repomix-output.md';
  return 'repomix-output.txt';
}

function buildContentType(format) {
  if (format === 'xml') return 'application/xml;charset=utf-8';
  if (format === 'markdown') return 'text/markdown;charset=utf-8';
  return 'text/plain;charset=utf-8';
}

function escapeGlobLiteral(path) {
  return Array.from(String(path)).map((char) => {
    if (char === '?') return '[?]';
    if (char === '*') return '[*]';
    if (char === '[') return '[[]';
    if (char === ']') return '[]]';
    if (char === '{') return '[{]';
    if (char === '}') return '[}]';
    if (char === ',') return '\\,';
    return char;
  }).join('');
}

function isZipFile(file) {
  const mime = String(file.type || '').toLowerCase();
  return file.name.toLowerCase().endsWith('.zip')
    || mime === 'application/zip'
    || mime === 'application/x-zip-compressed'
    || mime === 'multipart/x-zip';
}

function setActiveSource(form, kind) {
  form.dataset.activeSource = kind;
  form.querySelectorAll('input[name="sourceKind"][data-source-choice]').forEach((input) => {
    if (input instanceof HTMLInputElement) {
      input.checked = input.value === kind;
    }
  });
  form.querySelectorAll('[data-source-panel]').forEach((panel) => {
    const active = panel.dataset.sourcePanel === kind;
    panel.hidden = !active;
    panel.querySelectorAll('[data-source-control="true"]').forEach((field) => {
      if (field instanceof HTMLInputElement || field instanceof HTMLTextAreaElement || field instanceof HTMLSelectElement) {
        field.disabled = !active;
      }
    });
  });
}

function updateFileSummary(input) {
  const summaryId = input.dataset.summaryTarget;
  const summary = summaryId ? document.getElementById(summaryId) : null;
  if (!(summary instanceof HTMLElement)) {
    return;
  }
  const files = input.files ? Array.from(input.files) : [];
  if (!files.length) {
    summary.textContent = summary.dataset.emptyLabel || '';
    return;
  }
  const totalSize = files.reduce((size, file) => size + file.size, 0);
  const first = files[0];
  summary.textContent = fillTemplate(summary.dataset.selectedTemplate || '', {
    count: numberFormatter.format(files.length),
    size: formatBytes(totalSize),
    name: first.name,
    sample: first.webkitRelativePath || first.name,
  });
}

function initUploadZone(zone) {
  const targetId = zone.dataset.fileTarget;
  const input = targetId ? document.getElementById(targetId) : null;
  if (!(input instanceof HTMLInputElement)) {
    return;
  }
  zone.addEventListener('keydown', (event) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      input.click();
    }
  });
  if (zone.dataset.dropEnabled !== 'true') {
    return;
  }
  const clearDrag = () => zone.classList.remove('is-dragover');
  zone.addEventListener('dragover', (event) => {
    event.preventDefault();
    zone.classList.add('is-dragover');
  });
  zone.addEventListener('dragleave', clearDrag);
  zone.addEventListener('dragend', clearDrag);
  zone.addEventListener('drop', (event) => {
    event.preventDefault();
    clearDrag();
    if (!(event.dataTransfer instanceof DataTransfer) || !event.dataTransfer.files.length) {
      return;
    }
    if (input.id === 'folder-files') {
      return;
    }
    const file = event.dataTransfer.files[0];
    const transfer = new DataTransfer();
    transfer.items.add(file);
    input.files = transfer.files;
    input.dispatchEvent(new Event('change', { bubbles: true }));
  });
}

function loadUrlHistory() {
  try {
    const value = window.localStorage.getItem(URL_HISTORY_KEY);
    return value ? JSON.parse(value) : [];
  } catch (_) {
    return [];
  }
}

function saveUrlHistory(url) {
  if (!isValidRemoteValue(url)) {
    return;
  }
  const trimmed = url.trim();
  const current = loadUrlHistory().filter((item) => item !== trimmed);
  const next = [trimmed].concat(current).slice(0, 5);
  try {
    window.localStorage.setItem(URL_HISTORY_KEY, JSON.stringify(next));
  } catch (_) {}
}

function renderHistory(datalist, values) {
  datalist.innerHTML = values.map((value) => `<option value="${escapeHtml(value)}"></option>`).join('');
}

async function jsonOrError(response) {
  const text = await response.text();
  if (!text) {
    return {};
  }
  try {
    return JSON.parse(text);
  } catch (_) {
    return { error: text };
  }
}

window.addEventListener('DOMContentLoaded', () => {
  const form = document.querySelector('[data-source-form="true"]');
  if (!(form instanceof HTMLFormElement)) {
    return;
  }

  const refs = {
    urlInput: document.getElementById('url-source'),
    repoHistory: document.getElementById('repo-history'),
    zipInput: document.getElementById('zip-file'),
    folderInput: document.getElementById('folder-files'),
    folderManifest: form.querySelector('input[name="folderManifest"]'),
    formatInput: form.querySelector('select[name="format"]'),
    includeInput: form.querySelector('input[name="includePatterns"]'),
    ignoreInput: form.querySelector('input[name="ignorePatterns"]'),
    submitButton: form.querySelector('[data-pack-button]'),
    resetButton: form.querySelector('[data-reset-button]'),
    cancelButton: form.querySelector('[data-cancel-button]'),
    urlWarning: document.getElementById('url-validation-message'),
    feedbackRoot: document.getElementById('app-feedback-root'),
    legacyRoot: document.getElementById('legacy-response-root'),
    appStatus: document.getElementById('app-status'),
  };

  if (!(refs.urlInput instanceof HTMLInputElement)
    || !(refs.repoHistory instanceof HTMLDataListElement)
    || !(refs.zipInput instanceof HTMLInputElement)
    || !(refs.folderInput instanceof HTMLInputElement)
    || !(refs.folderManifest instanceof HTMLInputElement)
    || !(refs.formatInput instanceof HTMLSelectElement)
    || !(refs.includeInput instanceof HTMLInputElement)
    || !(refs.ignoreInput instanceof HTMLInputElement)
    || !(refs.submitButton instanceof HTMLButtonElement)
    || !(refs.resetButton instanceof HTMLButtonElement)
    || !(refs.cancelButton instanceof HTMLButtonElement)
    || !(refs.urlWarning instanceof HTMLElement)
    || !(refs.feedbackRoot instanceof HTMLElement)) {
    return;
  }

  const checkboxRefs = {
    removeComments: form.querySelector('input[type="checkbox"][name="removeComments"][value="true"]'),
    removeEmptyLines: form.querySelector('input[type="checkbox"][name="removeEmptyLines"][value="true"]'),
    showLineNumbers: form.querySelector('input[type="checkbox"][name="showLineNumbers"][value="true"]'),
    fileSummary: form.querySelector('input[type="checkbox"][name="fileSummary"][value="true"]'),
    directoryStructure: form.querySelector('input[type="checkbox"][name="directoryStructure"][value="true"]'),
    outputParsable: form.querySelector('input[type="checkbox"][name="outputParsable"][value="true"]'),
    compress: form.querySelector('input[type="checkbox"][name="compress"][value="true"]'),
  };

  const state = {
    activeTab: 'result',
    loading: false,
    error: null,
    errorType: 'error',
    result: null,
    uploadProgress: 0,
    uploadChunks: { current: 0, total: 0 },
    controller: null,
    selectedPaths: new Set(),
  };

  renderHistory(refs.repoHistory, loadUrlHistory());
  form.querySelectorAll('[data-file-target]').forEach((zone) => initUploadZone(zone));
  form.querySelectorAll('input[type="file"][data-summary-target]').forEach((input) => {
    if (input instanceof HTMLInputElement) {
      input.addEventListener('change', () => updateFileSummary(input));
      updateFileSummary(input);
    }
  });

  function announce(message) {
    if (refs.appStatus instanceof HTMLElement) {
      refs.appStatus.textContent = message;
    }
  }

  function currentMode() {
    const checked = form.querySelector('input[name="sourceKind"][data-source-choice]:checked');
    return checked instanceof HTMLInputElement ? checked.value : (form.dataset.activeSource || 'url');
  }

  function buildOptions() {
    return {
      removeComments: checkboxRefs.removeComments instanceof HTMLInputElement ? checkboxRefs.removeComments.checked : false,
      removeEmptyLines: checkboxRefs.removeEmptyLines instanceof HTMLInputElement ? checkboxRefs.removeEmptyLines.checked : false,
      showLineNumbers: checkboxRefs.showLineNumbers instanceof HTMLInputElement ? checkboxRefs.showLineNumbers.checked : false,
      fileSummary: checkboxRefs.fileSummary instanceof HTMLInputElement ? checkboxRefs.fileSummary.checked : true,
      directoryStructure: checkboxRefs.directoryStructure instanceof HTMLInputElement ? checkboxRefs.directoryStructure.checked : true,
      includePatterns: refs.includeInput.value.trim() || undefined,
      ignorePatterns: refs.ignoreInput.value.trim() || undefined,
      outputParsable: checkboxRefs.outputParsable instanceof HTMLInputElement ? checkboxRefs.outputParsable.checked : false,
      compress: checkboxRefs.compress instanceof HTMLInputElement ? checkboxRefs.compress.checked : false,
    };
  }

  function hasNonDefaultValues() {
    if (refs.urlInput.value.trim() !== '') {
      return true;
    }
    if (refs.formatInput.value !== DEFAULT_OPTIONS.format) {
      return true;
    }
    const options = buildOptions();
    return Object.keys(DEFAULT_OPTIONS).some((key) => {
      if (key === 'format') {
        return false;
      }
      const current = options[key];
      const defaultValue = DEFAULT_OPTIONS[key];
      if (typeof current === 'string') {
        return current !== '' && current !== defaultValue;
      }
      return current !== defaultValue;
    }) || (currentMode() !== 'url') || refs.zipInput.files?.length || refs.folderInput.files?.length;
  }

  function syncResetVisibility() {
    refs.resetButton.hidden = !hasNonDefaultValues();
  }

  function updateButtons() {
    const mode = currentMode();
    const urlValid = refs.urlInput.value.trim() !== '' && isValidRemoteValue(refs.urlInput.value.trim());
    const zipValid = Boolean(refs.zipInput.files && refs.zipInput.files[0]);
    const folderValid = Boolean(refs.folderInput.files && refs.folderInput.files.length > 0);
    const isValid = mode === 'url' ? urlValid : (mode === 'zip' ? zipValid : folderValid);
    refs.submitButton.disabled = !isValid || state.loading;
    refs.cancelButton.hidden = !state.loading;
    refs.submitButton.textContent = state.loading ? TEXT.processing : TEXT.pack;
    refs.urlWarning.hidden = !(mode === 'url' && refs.urlInput.value.trim() !== '' && !urlValid);
    syncResetVisibility();
  }

  function applyQueryState() {
    const query = parseUrlParameters();
    if (typeof query.repo === 'string') {
      refs.urlInput.value = query.repo;
    }
    if (typeof query.format === 'string') {
      refs.formatInput.value = query.format;
    }
    if (typeof query.includePatterns === 'string') {
      refs.includeInput.value = query.includePatterns;
    }
    if (typeof query.ignorePatterns === 'string') {
      refs.ignoreInput.value = query.ignorePatterns;
    }
    Object.keys(checkboxRefs).forEach((key) => {
      const input = checkboxRefs[key];
      if (input instanceof HTMLInputElement && typeof query[key] === 'boolean') {
        input.checked = query[key];
      }
    });
  }

  function syncQueryString() {
    const url = new URL(window.location.href);
    ['repo', 'format', 'style', 'include', 'ignore', 'removeComments', 'removeEmptyLines', 'showLineNumbers', 'fileSummary', 'directoryStructure', 'outputParsable', 'compress'].forEach((key) => {
      url.searchParams.delete(key);
    });
    const repo = refs.urlInput.value.trim();
    if (repo && isValidRemoteValue(repo)) {
      url.searchParams.set('repo', repo);
    }
    if (refs.formatInput.value !== DEFAULT_OPTIONS.format) {
      url.searchParams.set('format', refs.formatInput.value);
    }
    if (refs.includeInput.value.trim()) {
      url.searchParams.set('include', refs.includeInput.value.trim());
    }
    if (refs.ignoreInput.value.trim()) {
      url.searchParams.set('ignore', refs.ignoreInput.value.trim());
    }
    const options = buildOptions();
    Object.keys(options).forEach((key) => {
      if (key === 'includePatterns' || key === 'ignorePatterns') {
        return;
      }
      if (options[key] !== DEFAULT_OPTIONS[key]) {
        url.searchParams.set(key, String(options[key]));
      }
    });
    if (url.toString().length > 2000) {
      return;
    }
    window.history.replaceState({}, '', url.toString());
  }

  function resetAll() {
    if (state.controller) {
      state.controller.abort('reset');
    }
    refs.urlInput.value = '';
    refs.formatInput.value = DEFAULT_OPTIONS.format;
    refs.includeInput.value = '';
    refs.ignoreInput.value = '';
    Object.keys(checkboxRefs).forEach((key) => {
      const input = checkboxRefs[key];
      if (input instanceof HTMLInputElement) {
        input.checked = DEFAULT_OPTIONS[key];
      }
    });
    refs.zipInput.value = '';
    refs.folderInput.value = '';
    refs.folderManifest.value = '';
    setActiveSource(form, 'url');
    form.querySelectorAll('input[type="file"][data-summary-target]').forEach((input) => {
      if (input instanceof HTMLInputElement) {
        updateFileSummary(input);
      }
    });
    state.error = null;
    state.errorType = null;
    state.result = null;
    state.selectedPaths = new Set();
    state.activeTab = 'result';
    state.uploadProgress = 0;
    state.uploadChunks = { current: 0, total: 0 };
    syncQueryString();
    updateButtons();
    renderFeedback();
  }

  function setLoading(loading) {
    state.loading = loading;
    updateButtons();
    renderFeedback();
  }

  function scrollToFeedback() {
    const target = refs.feedbackRoot.firstElementChild || refs.legacyRoot?.firstElementChild;
    if (target instanceof HTMLElement) {
      target.scrollIntoView({ block: 'start', behavior: 'smooth' });
    }
  }

  function setSelectionFromResult(previousSelection) {
    state.selectedPaths = new Set();
    const allFiles = state.result?.metadata?.allFiles || [];
    if (previousSelection && previousSelection.size) {
      allFiles.forEach((file) => {
        if (previousSelection.has(file.path)) {
          state.selectedPaths.add(file.path);
        }
      });
    }
    if (!state.selectedPaths.size) {
      allFiles.forEach((file) => state.selectedPaths.add(file.path));
    }
  }

  function topFilesMarkup(result) {
    const summary = result.metadata.summary;
    const topFiles = Array.isArray(result.metadata.topFiles) ? result.metadata.topFiles : [];
    const allFiles = Array.isArray(result.metadata.allFiles) ? result.metadata.allFiles : [];
    const fileTokenTotal = allFiles.reduce((sum, file) => sum + file.tokenCount, 0);
    if (!topFiles.length) {
      return '';
    }
    return `
      <section class="top-files">
        <div class="section-heading">
          <h3>${escapeHtml(TEXT.topFiles)}</h3>
          <p>${escapeHtml(TEXT.topFilesBody)}</p>
        </div>
        <ol class="top-files-list">
          ${topFiles.map((file) => `
            <li class="top-file-item">
              <div class="top-file-path">${escapeHtml(file.path)}</div>
              <div class="top-file-stats">
                <span class="stat-chip strong">${numberFormatter.format(file.tokenCount)} ${escapeHtml(TEXT.tokens.toLowerCase())}</span>
                <span class="stat-chip">${numberFormatter.format(file.charCount)} ${escapeHtml(TEXT.characters.toLowerCase())}</span>
                ${fileTokenTotal > 0 ? `<span class="stat-chip">${(((file.tokenCount / fileTokenTotal) * 100) || 0).toFixed(1)}%</span>` : ''}
              </div>
            </li>
          `).join('')}
        </ol>
      </section>
    `;
  }

  function selectionMarkup(result) {
    const files = Array.isArray(result.metadata.allFiles) ? [...result.metadata.allFiles].sort((a, b) => b.tokenCount - a.tokenCount) : [];
    const selectedFiles = files.filter((file) => state.selectedPaths.has(file.path));
    const selectedTokens = selectedFiles.reduce((sum, file) => sum + file.tokenCount, 0);
    const totalTokens = files.reduce((sum, file) => sum + file.tokenCount, 0);
    const allSelected = files.length > 0 && selectedFiles.length === files.length;
    const warning = selectedFiles.length > FILE_SELECTION_WARNING_THRESHOLD
      ? `<div class="notice-inline">${escapeHtml(fillTemplate(TEXT.fileSelectionWarning, { threshold: numberFormatter.format(FILE_SELECTION_WARNING_THRESHOLD) }))}</div>`
      : '';
    const summary = fillTemplate(TEXT.selectedSummary, {
      selected: numberFormatter.format(selectedFiles.length),
      total: numberFormatter.format(files.length),
      tokens: numberFormatter.format(selectedTokens),
      percent: `${totalTokens > 0 ? ((selectedTokens / totalTokens) * 100).toFixed(1) : '0.0'}%`,
    });
    return `
      <div class="selection-shell">
        <div class="selection-toolbar">
          <div class="section-heading">
            <h3>${escapeHtml(TEXT.fileSelection)}</h3>
            <p>${escapeHtml(TEXT.fileSelectionBody)}</p>
          </div>
          ${warning}
          <div class="button-row">
            <button class="secondary" type="button" data-selection-action="select-all">${escapeHtml(TEXT.selectAll)}</button>
            <button class="secondary" type="button" data-selection-action="deselect-all">${escapeHtml(TEXT.deselectAll)}</button>
            <button class="primary" type="button" data-selection-action="repack" ${selectedFiles.length ? '' : 'disabled'}>${escapeHtml(state.loading ? TEXT.repacking : TEXT.repack)}</button>
          </div>
        </div>
        <p class="subtle-copy">${escapeHtml(summary)}</p>
        <div class="selection-table-wrap">
          <div class="selection-table-scroll">
            <table class="selection-table" aria-label="${escapeHtml(TEXT.selectionTableLabel)}">
              <thead>
                <tr>
                  <th><input class="selection-master-checkbox" type="checkbox" aria-label="${escapeHtml(TEXT.toggleAllFiles)}" data-selection-action="toggle-all" ${allSelected ? 'checked' : ''}></th>
                  <th>${escapeHtml(TEXT.fileSelection)}</th>
                  <th>${escapeHtml(TEXT.tokens)}</th>
                  <th>${escapeHtml(TEXT.characters)}</th>
                </tr>
              </thead>
              <tbody>
                ${files.map((file) => {
                    const selected = state.selectedPaths.has(file.path);
                    const toggleLabel = fillTemplate(TEXT.toggleFile, { path: file.path });
                    return `
                      <tr data-selection-row="${escapeHtml(file.path)}" data-selected="${selected}">
                        <td><input class="selection-checkbox" type="checkbox" aria-label="${escapeHtml(toggleLabel)}" data-selection-path="${escapeHtml(file.path)}" ${selected ? 'checked' : ''}></td>
                        <td class="selection-path">${escapeHtml(file.path)}</td>
                        <td class="selection-token-cell">${numberFormatter.format(file.tokenCount)}</td>
                        <td class="selection-char-cell">${numberFormatter.format(file.charCount)}</td>
                    </tr>
                  `;
                }).join('')}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    `;
  }

  function resultMarkup(result) {
    const summary = result.metadata.summary;
    const hasFiles = Array.isArray(result.metadata.allFiles) && result.metadata.allFiles.length > 0;
    const selectionUnavailable = !hasFiles && summary && summary.totalFiles > MAX_BROWSER_FILE_SELECTION_FILES
      ? `<div class="notice-inline">${escapeHtml(fillTemplate(TEXT.fileSelectionUnavailable, { threshold: numberFormatter.format(MAX_BROWSER_FILE_SELECTION_FILES) }))}</div>`
      : '';
    return `
      <section class="card result-card" id="result">
        <div class="result-shell">
          <div class="result-main">
            <div class="result-header">
              <div class="section-heading">
                <h2>${escapeHtml(TEXT.packedOutput)}</h2>
                <p>${escapeHtml(TEXT.packedOutputBody)}</p>
              </div>
              <div class="micro-actions">
                <button class="secondary" type="button" data-result-action="copy">${escapeHtml(TEXT.copy)}</button>
                <button class="secondary" type="button" data-result-action="download">${escapeHtml(TEXT.download)}</button>
                <button class="secondary" type="button" data-result-action="share">${escapeHtml(TEXT.share)}</button>
              </div>
            </div>
            ${hasFiles ? `
              <div class="result-tab-row">
                <div class="result-tabs">
                  <button class="result-tab ${state.activeTab === 'result' ? 'active' : ''}" type="button" data-result-tab="result">${escapeHtml(TEXT.resultTab)}</button>
                  <button class="result-tab ${state.activeTab === 'files' ? 'active' : ''}" type="button" data-result-tab="files">${escapeHtml(TEXT.filesTab)}</button>
                </div>
              </div>
            ` : ''}
            <div ${hasFiles && state.activeTab === 'files' ? 'hidden' : ''}>
              <div class="output-panel">
                <textarea id="result-output-view" class="output-viewer" aria-label="${escapeHtml(TEXT.packedOutput)}" readonly spellcheck="false"></textarea>
              </div>
            </div>
            ${hasFiles ? `<div ${state.activeTab === 'result' ? 'hidden' : ''}>${selectionMarkup(result)}</div>` : ''}
          </div>
          <aside class="result-sidebar">
            <div class="overview-grid">
              <div class="meta-card"><span class="meta-label">${escapeHtml(TEXT.repository)}</span><div class="meta-value">${escapeHtml(result.metadata.repository)}</div></div>
              <div class="meta-card"><span class="meta-label">${escapeHtml(TEXT.format)}</span><div class="meta-value">${escapeHtml(result.format.toUpperCase())}</div></div>
              <div class="meta-card"><span class="meta-label">${escapeHtml(TEXT.generatedAt)}</span><div class="meta-value">${escapeHtml(formatTimestamp(result.metadata.timestamp))}</div></div>
            </div>
            ${summary ? `
              <div class="summary-grid">
                <div class="metric-card"><span class="metric-label">${escapeHtml(TEXT.files)}</span><div class="metric-value">${numberFormatter.format(summary.totalFiles)}</div></div>
                <div class="metric-card"><span class="metric-label">${escapeHtml(TEXT.characters)}</span><div class="metric-value">${numberFormatter.format(summary.totalCharacters)}</div></div>
                <div class="metric-card"><span class="metric-label">${escapeHtml(TEXT.tokens)}</span><div class="metric-value">${numberFormatter.format(summary.totalTokens)}</div></div>
              </div>
            ` : ''}
            ${selectionUnavailable}
            ${topFilesMarkup(result)}
          </aside>
        </div>
      </section>
    `;
  }

  function renderFeedback() {
    if (refs.legacyRoot instanceof HTMLElement && (state.loading || state.error || state.result)) {
      refs.legacyRoot.hidden = true;
    }
    if (state.loading) {
      const progressText = state.uploadChunks.total > 0
        ? fillTemplate(TEXT.uploadProgress, {
            progress: state.uploadProgress.toFixed(0),
            current: numberFormatter.format(state.uploadChunks.current),
            total: numberFormatter.format(state.uploadChunks.total),
          })
        : TEXT.processing;
      refs.feedbackRoot.innerHTML = `
        <section class="card state-card">
          <div class="state-title-row">
            <div class="section-heading">
              <h3>${escapeHtml(TEXT.loadingTitle)}</h3>
              <p>${escapeHtml(TEXT.loadingBody)}</p>
            </div>
            <span class="status-chip strong">${escapeHtml(TEXT.cancel)}</span>
          </div>
          <div class="progress-shell">
            <div class="progress-track"><div class="progress-value" style="width:${Math.max(8, state.uploadProgress)}%"></div></div>
            <div class="progress-copy">${escapeHtml(progressText)}</div>
          </div>
        </section>
      `;
      return;
    }
    if (state.error) {
      refs.feedbackRoot.innerHTML = `
        <section class="card notice state-card">
          <div class="section-heading">
            <h2>${escapeHtml(TEXT.requestFailed)}</h2>
            <p class="notice-message">${escapeHtml(state.error)}</p>
            <p class="notice-hint">${escapeHtml(TEXT.requestFailedHint)}</p>
          </div>
        </section>
      `;
      scrollToFeedback();
      return;
    }
    if (state.result) {
      refs.feedbackRoot.innerHTML = resultMarkup(state.result);
      const output = document.getElementById('result-output-view');
      if (output instanceof HTMLTextAreaElement) {
        output.value = state.result.content;
      }
      const masterCheckbox = refs.feedbackRoot.querySelector('.selection-master-checkbox');
      if (masterCheckbox instanceof HTMLInputElement && Array.isArray(state.result.metadata.allFiles)) {
        const selectedCount = state.result.metadata.allFiles.filter((file) => state.selectedPaths.has(file.path)).length;
        masterCheckbox.indeterminate = selectedCount > 0 && selectedCount < state.result.metadata.allFiles.length;
      }
      const resultSection = document.getElementById('result');
      if (resultSection instanceof HTMLElement) {
        resultSection.scrollIntoView({ block: 'start', behavior: 'smooth' });
      }
      scrollToFeedback();
      return;
    }
    refs.feedbackRoot.innerHTML = '';
  }

  async function initChunkedUpload(file, signal) {
    const totalChunks = Math.ceil(file.size / CHUNK_SIZE);
    const response = await fetch('/api/upload/init', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ fileName: file.name, fileSize: file.size, totalChunks }),
      signal,
    });
    const data = await jsonOrError(response);
    if (!response.ok) {
      throw new Error(data.error || 'Failed to initialize chunked upload');
    }
    return data.uploadId;
  }

  async function uploadChunk(uploadId, chunkIndex, chunkData, signal) {
    const response = await fetch(`/api/upload/chunk?uploadId=${encodeURIComponent(uploadId)}&chunkIndex=${chunkIndex}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/octet-stream' },
      body: chunkData,
      signal,
    });
    const data = await jsonOrError(response);
    if (!response.ok) {
      throw new Error(data.error || 'Failed to upload chunk');
    }
  }

  async function chunkedUpload(file, signal) {
    const totalChunks = Math.ceil(file.size / CHUNK_SIZE);
    state.uploadChunks = { current: 0, total: totalChunks };
    const uploadId = await initChunkedUpload(file, signal);
    for (let index = 0; index < totalChunks; index += 1) {
      const start = index * CHUNK_SIZE;
      const end = Math.min(start + CHUNK_SIZE, file.size);
      const chunkBuffer = await file.slice(start, end).arrayBuffer();
      await uploadChunk(uploadId, index, chunkBuffer, signal);
      state.uploadChunks.current = index + 1;
      state.uploadProgress = ((index + 1) / totalChunks) * 100;
      renderFeedback();
    }
    return uploadId;
  }

  function currentSourcePayload() {
    const mode = currentMode();
    if (mode === 'url') {
      const value = refs.urlInput.value.trim();
      if (!value || !isValidRemoteValue(value)) {
        throw new Error(TEXT.invalidUrl);
      }
      saveUrlHistory(value);
      renderHistory(refs.repoHistory, loadUrlHistory());
      return { kind: 'url', value };
    }
    if (mode === 'zip') {
      const file = refs.zipInput.files && refs.zipInput.files[0];
      if (!file) {
        throw new Error(TEXT.zipRequired);
      }
      if (!isZipFile(file)) {
        throw new Error(TEXT.zipOnly);
      }
      if (file.size > MAX_CLIENT_UPLOAD_BYTES) {
        throw new Error(TEXT.zipTooLarge);
      }
      return { kind: 'zip', file };
    }
    const files = refs.folderInput.files ? Array.from(refs.folderInput.files) : [];
    if (!files.length) {
      throw new Error(TEXT.folderRequired);
    }
    const totalSize = files.reduce((sum, file) => sum + file.size, 0);
    if (totalSize > MAX_CLIENT_UPLOAD_BYTES) {
      throw new Error(TEXT.uploadTooLarge);
    }
    if (files.length > MAX_FOLDER_FILES) {
      throw new Error(TEXT.folderTooManyFiles);
    }
    const maxDepth = files.reduce((depth, file) => {
      const path = (file.webkitRelativePath || file.name).split('/').filter(Boolean);
      return Math.max(depth, path.length);
    }, 0);
    if (maxDepth > MAX_FOLDER_DEPTH) {
      throw new Error(TEXT.folderTooDeep);
    }
    return { kind: 'folder', files };
  }

  async function submitRequest(overrides) {
    const previousSelection = new Set(state.selectedPaths);
    const source = currentSourcePayload();
    if (state.controller) {
      state.controller.abort('cancel');
    }
    const controller = new AbortController();
    state.controller = controller;
    state.error = null;
    state.result = null;
    state.uploadProgress = 0;
    state.uploadChunks = { current: 0, total: 0 };
    state.activeTab = 'result';
    setLoading(true);
    const timeoutId = window.setTimeout(() => controller.abort('timeout'), TIMEOUT_MS);

    try {
      const formData = new FormData();
      formData.append('format', refs.formatInput.value);
      const options = buildOptions();
      if (overrides) {
        Object.assign(options, overrides);
      }
      formData.append('options', JSON.stringify(options));

      if (source.kind === 'url') {
        formData.append('url', source.value);
      } else if (source.kind === 'zip') {
        if (source.file.size > CHUNKED_UPLOAD_THRESHOLD) {
          const uploadId = await chunkedUpload(source.file, controller.signal);
          formData.append('uploadId', uploadId);
        } else {
          formData.append('file', source.file, source.file.name);
        }
      } else {
        const paths = source.files.map((file) => file.webkitRelativePath || file.name);
        refs.folderManifest.value = JSON.stringify({ paths });
        formData.append('folderManifest', refs.folderManifest.value);
        source.files.forEach((file) => {
          formData.append('folderFiles', file, file.name);
        });
      }

      const response = await fetch('/api/pack', {
        method: 'POST',
        body: formData,
        signal: controller.signal,
      });
      const data = await jsonOrError(response);
      if (!response.ok) {
        throw new Error(data.error || 'Pack request failed');
      }
      state.result = data;
      setSelectionFromResult(previousSelection);
      announce(TEXT.packedOutput);
      scrollToFeedback();
    } catch (error) {
      if (controller.signal.aborted) {
        if (controller.signal.reason === 'reset') {
          state.error = null;
          state.errorType = null;
        } else {
          state.error = controller.signal.reason === 'timeout' ? TEXT.timeout : TEXT.cancelled;
          state.errorType = 'warning';
        }
      } else {
        state.error = error instanceof Error ? error.message : 'Unexpected error';
        state.errorType = 'error';
      }
      if (state.error) {
        announce(state.error);
        scrollToFeedback();
      }
    } finally {
      clearTimeout(timeoutId);
      if (state.controller === controller) {
        state.controller = null;
      }
      setLoading(false);
      renderFeedback();
    }
  }

  form.addEventListener('submit', (event) => {
    event.preventDefault();
    submitRequest().catch((error) => {
      state.error = error instanceof Error ? error.message : 'Unexpected error';
      renderFeedback();
    });
  });

  refs.resetButton.addEventListener('click', () => {
    resetAll();
    announce(TEXT.reset);
  });

  refs.cancelButton.addEventListener('click', () => {
    if (state.controller) {
      state.controller.abort('cancel');
    }
  });

  refs.feedbackRoot.addEventListener('click', async (event) => {
    const target = event.target;
    if (!(target instanceof Element)) {
      return;
    }
    const tab = target.closest('[data-result-tab]');
    if (tab instanceof HTMLButtonElement) {
      state.activeTab = tab.dataset.resultTab || 'result';
      renderFeedback();
      return;
    }
    const resultAction = target.closest('[data-result-action]');
    if (resultAction instanceof HTMLButtonElement && state.result) {
      const action = resultAction.dataset.resultAction;
      if (action === 'copy') {
        try {
          await navigator.clipboard.writeText(state.result.content);
          announce(TEXT.copied);
          resultAction.textContent = TEXT.copied;
          window.setTimeout(() => { resultAction.textContent = TEXT.copy; }, 1600);
        } catch (_) {}
      }
      if (action === 'download') {
        const blob = new Blob([state.result.content], { type: buildContentType(state.result.format) });
        const url = URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = buildDownloadName(state.result.format);
        link.click();
        URL.revokeObjectURL(url);
      }
      if (action === 'share') {
        if (!navigator.share || window.innerWidth > 768) {
          announce(TEXT.shareUnavailable);
          return;
        }
        try {
          await navigator.share({
            title: buildDownloadName(state.result.format),
            text: state.result.content,
          });
          announce(TEXT.shared);
        } catch (_) {}
      }
      return;
    }
    const selectionAction = target.closest('[data-selection-action]');
    if (selectionAction instanceof HTMLButtonElement || selectionAction instanceof HTMLInputElement) {
      const action = selectionAction.dataset.selectionAction;
      const files = Array.isArray(state.result?.metadata?.allFiles) ? state.result.metadata.allFiles : [];
      if (action === 'select-all' || (action === 'toggle-all' && selectionAction.checked)) {
        state.selectedPaths = new Set(files.map((file) => file.path));
        renderFeedback();
        return;
      }
      if (action === 'deselect-all' || (action === 'toggle-all' && !selectionAction.checked)) {
        state.selectedPaths = new Set();
        renderFeedback();
        return;
      }
      if (action === 'repack') {
        if (!state.selectedPaths.size) {
          return;
        }
        const includePatterns = Array.from(state.selectedPaths).map((path) => escapeGlobLiteral(path)).join(',');
        submitRequest({ includePatterns, ignorePatterns: undefined }).catch((error) => {
          state.error = error instanceof Error ? error.message : 'Unexpected error';
          renderFeedback();
        });
      }
    }
    const selectionCheckbox = target.closest('[data-selection-path]');
    if (selectionCheckbox instanceof HTMLInputElement) {
      const path = selectionCheckbox.dataset.selectionPath;
      if (!path) {
        return;
      }
      if (selectionCheckbox.checked) {
        state.selectedPaths.add(path);
      } else {
        state.selectedPaths.delete(path);
      }
      renderFeedback();
      return;
    }
    const row = target.closest('[data-selection-row]');
    if (row instanceof HTMLElement) {
      const path = row.dataset.selectionRow;
      if (!path) {
        return;
      }
      if (state.selectedPaths.has(path)) {
        state.selectedPaths.delete(path);
      } else {
        state.selectedPaths.add(path);
      }
      renderFeedback();
    }
  });

  [refs.urlInput, refs.formatInput, refs.includeInput, refs.ignoreInput, refs.zipInput, refs.folderInput].forEach((input) => {
    input.addEventListener('input', () => {
      syncQueryString();
      updateButtons();
    });
    input.addEventListener('change', () => {
      if (input instanceof HTMLInputElement && input.type === 'file') {
        updateFileSummary(input);
      }
      syncQueryString();
      updateButtons();
    });
  });

  document.addEventListener('click', async (event) => {
    const target = event.target;
    if (!(target instanceof Element)) {
      return;
    }
    const copyButton = target.closest('[data-copy-target]');
    if (copyButton instanceof HTMLButtonElement) {
      const selector = copyButton.dataset.copyTarget;
      const field = selector ? document.querySelector(selector) : null;
      const value = field instanceof HTMLTextAreaElement ? field.value : field?.textContent;
      if (!value) {
        return;
      }
      try {
        await navigator.clipboard.writeText(value);
        announce(copyButton.dataset.copiedLabel || TEXT.copied);
      } catch (_) {}
      return;
    }
    const downloadButton = target.closest('[data-download-target]');
    if (downloadButton instanceof HTMLButtonElement) {
      const selector = downloadButton.dataset.downloadTarget;
      const field = selector ? document.querySelector(selector) : null;
      const value = field instanceof HTMLTextAreaElement ? field.value : field?.textContent;
      if (!value) {
        return;
      }
      const blob = new Blob([value], { type: downloadButton.dataset.downloadType || 'text/plain;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = downloadButton.dataset.downloadName || 'repomix-output.txt';
      link.click();
      URL.revokeObjectURL(url);
    }
  });

  Object.values(checkboxRefs).forEach((input) => {
    if (input instanceof HTMLInputElement) {
      input.addEventListener('change', () => {
        syncQueryString();
        updateButtons();
      });
    }
  });

  form.querySelectorAll('input[name="sourceKind"][data-source-choice]').forEach((input) => {
    if (input instanceof HTMLInputElement) {
      input.addEventListener('change', () => {
        setActiveSource(form, input.value);
        syncQueryString();
        updateButtons();
      });
    }
  });

  applyQueryState();
  setActiveSource(form, currentMode());
  updateButtons();
  renderFeedback();

  const autoQuery = parseUrlParameters();
  if (typeof autoQuery.repo === 'string' && isValidRemoteValue(autoQuery.repo.trim())) {
    submitRequest().catch((error) => {
      state.error = error instanceof Error ? error.message : 'Unexpected error';
      renderFeedback();
    });
  }

  if (document.body.dataset.hasResult === 'true') {
    const result = document.getElementById('legacy-result');
    if (result instanceof HTMLElement) {
      result.scrollIntoView({ block: 'start' });
    }
  }
});
"#;

#[derive(Debug, Clone)]
struct WebFormState {
    source_kind: SourceKind,
    url: String,
    format: String,
    options: PackOptions,
}

impl WebFormState {
    fn new() -> Self {
        Self {
            source_kind: SourceKind::Url,
            url: String::new(),
            format: "xml".to_string(),
            options: PackOptions::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Locale {
    En,
    Ru,
}

impl Locale {
    fn from_value(value: &str) -> Self {
        if value.eq_ignore_ascii_case("ru") {
            Self::Ru
        } else {
            Self::En
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ru => "ru",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum SourceKind {
    Url,
    Zip,
    Folder,
}

impl SourceKind {
    fn from_value(value: &str) -> Option<Self> {
        match value {
            "url" => Some(Self::Url),
            "zip" => Some(Self::Zip),
            "folder" => Some(Self::Folder),
            _ => None,
        }
    }

    fn value(self) -> &'static str {
        match self {
            Self::Url => "url",
            Self::Zip => "zip",
            Self::Folder => "folder",
        }
    }
}

pub async fn index() -> Html<String> {
    render_index(Locale::En)
}

pub async fn index_ru() -> Html<String> {
    render_index(Locale::Ru)
}

pub async fn site_fallback(Path(path): Path<String>) -> Response {
    if path == "api" || path.starts_with("api/") {
        return StatusCode::NOT_FOUND.into_response();
    }

    let locale = if path == "ru" || path.starts_with("ru/") {
        Locale::Ru
    } else {
        Locale::En
    };

    Html(render_page(locale, &WebFormState::new(), None, None).into_string()).into_response()
}

pub async fn schema_asset(Path(path): Path<String>) -> Response {
    let relative_path = match normalize_schema_asset_path(&path) {
        Ok(path) => path,
        Err(message) => {
            return (StatusCode::BAD_REQUEST, message).into_response();
        }
    };

    let full_path = schema_root_dir().join(relative_path);
    match tokio::fs::read_to_string(&full_path).await {
        Ok(content) => {
            ([(CONTENT_TYPE, "application/json; charset=utf-8")], content).into_response()
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            StatusCode::NOT_FOUND.into_response()
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read schema asset: {error}"),
        )
            .into_response(),
    }
}

pub async fn pack_page(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    if !is_same_origin_form_request(&headers) {
        let form = WebFormState::new();
        return render_response(
            StatusCode::FORBIDDEN,
            Locale::En,
            &form,
            None,
            Some("Cross-origin form submissions are not allowed. Use the same-origin web UI or /api/pack."),
        );
    }

    let (locale, form, job) = match parse_web_form(multipart).await {
        Ok(parsed) => parsed,
        Err((locale, form, message, status)) => {
            return render_response(status, locale, &form, None, Some(&message));
        }
    };

    match execute_pack_job(state, job).await {
        Ok(result) => render_response(StatusCode::OK, locale, &form, Some(&result), None),
        Err(err) => render_response(err.status, locale, &form, None, Some(&err.message)),
    }
}

fn is_same_origin_form_request(headers: &HeaderMap) -> bool {
    let mut hosts = Vec::new();
    if let Some(host) = headers.get(HOST).and_then(|value| value.to_str().ok()) {
        hosts.push(host.trim());
    }
    if let Some(forwarded_host) = headers
        .get("x-forwarded-host")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        hosts.push(forwarded_host);
    }

    if hosts.is_empty() {
        return false;
    }

    let mut schemes = vec!["http", "https"];
    if let Some(forwarded_proto) = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        schemes.insert(0, forwarded_proto);
    }

    if let Some(origin) = headers.get(ORIGIN).and_then(|value| value.to_str().ok()) {
        return hosts
            .iter()
            .any(|host| matches_origin_host(origin, host, &schemes));
    }

    if let Some(referer) = headers.get(REFERER).and_then(|value| value.to_str().ok()) {
        return hosts
            .iter()
            .any(|host| matches_referer_host(referer, host, &schemes));
    }

    false
}

fn matches_origin_host(origin: &str, host: &str, schemes: &[&str]) -> bool {
    schemes
        .iter()
        .any(|scheme| origin == format!("{scheme}://{host}"))
}

fn matches_referer_host(referer: &str, host: &str, schemes: &[&str]) -> bool {
    schemes.iter().any(|scheme| {
        referer.starts_with(&format!("{scheme}://{host}/"))
            || referer == format!("{scheme}://{host}")
    })
}

fn render_index(locale: Locale) -> Html<String> {
    Html(render_page(locale, &WebFormState::new(), None, None).into_string())
}

fn normalize_schema_asset_path(path: &str) -> Result<PathBuf, &'static str> {
    let mut normalized = PathBuf::new();

    for component in std::path::Path::new(path).components() {
        match component {
            Component::Normal(segment) => normalized.push(segment),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err("Invalid schema path");
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        return Err("Schema path is required");
    }

    Ok(normalized)
}

fn schema_root_dir() -> PathBuf {
    std::env::var_os("REPOMIX_SCHEMA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../client/src/public/schemas")
        })
}

async fn parse_web_form(
    mut multipart: Multipart,
) -> Result<(Locale, WebFormState, PackJob), (Locale, WebFormState, String, StatusCode)> {
    let mut locale = Locale::En;
    let mut form = WebFormState::new();
    let mut source_kind: Option<SourceKind> = None;
    let mut zip_upload: Option<(String, Vec<u8>)> = None;
    let mut folder_manifest: Option<String> = None;
    let mut folder_files: Vec<Vec<u8>> = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            locale,
            form.clone(),
            format!(
                "{}: {e}",
                t(
                    locale,
                    "Failed to read multipart form",
                    "Не удалось прочитать multipart-форму"
                )
            ),
            StatusCode::BAD_REQUEST,
        )
    })? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "locale" => {
                let value = field.text().await.map_err(|e| {
                    (
                        locale,
                        form.clone(),
                        format!(
                            "{}: {e}",
                            t(locale, "Failed to read locale", "Не удалось прочитать язык")
                        ),
                        StatusCode::BAD_REQUEST,
                    )
                })?;
                locale = Locale::from_value(&value);
            }
            "sourceKind" => {
                let value = field.text().await.map_err(|e| {
                    (
                        locale,
                        form.clone(),
                        format!(
                            "{}: {e}",
                            t(
                                locale,
                                "Failed to read source type",
                                "Не удалось прочитать тип источника"
                            )
                        ),
                        StatusCode::BAD_REQUEST,
                    )
                })?;
                source_kind = SourceKind::from_value(&value);
                if let Some(kind) = source_kind {
                    form.source_kind = kind;
                }
            }
            "url" => {
                form.url = field.text().await.map_err(|e| {
                    (
                        locale,
                        form.clone(),
                        format!(
                            "{}: {e}",
                            t(
                                locale,
                                "Failed to read repository URL",
                                "Не удалось прочитать URL репозитория"
                            )
                        ),
                        StatusCode::BAD_REQUEST,
                    )
                })?;
            }
            "format" => {
                form.format = field.text().await.map_err(|e| {
                    (
                        locale,
                        form.clone(),
                        format!(
                            "{}: {e}",
                            t(
                                locale,
                                "Failed to read format",
                                "Не удалось прочитать формат"
                            )
                        ),
                        StatusCode::BAD_REQUEST,
                    )
                })?;
            }
            "includePatterns" => {
                let value = field.text().await.map_err(|e| {
                    (
                        locale,
                        form.clone(),
                        format!(
                            "{}: {e}",
                            t(
                                locale,
                                "Failed to read include patterns",
                                "Не удалось прочитать include-паттерны"
                            )
                        ),
                        StatusCode::BAD_REQUEST,
                    )
                })?;
                form.options.include_patterns = normalize_optional_text(value);
            }
            "ignorePatterns" => {
                let value = field.text().await.map_err(|e| {
                    (
                        locale,
                        form.clone(),
                        format!(
                            "{}: {e}",
                            t(
                                locale,
                                "Failed to read ignore patterns",
                                "Не удалось прочитать ignore-паттерны"
                            )
                        ),
                        StatusCode::BAD_REQUEST,
                    )
                })?;
                form.options.ignore_patterns = normalize_optional_text(value);
            }
            "removeComments" => {
                let value = field.text().await.unwrap_or_else(|_| "true".to_string());
                form.options.remove_comments = parse_bool_field(&value);
            }
            "removeEmptyLines" => {
                let value = field.text().await.unwrap_or_else(|_| "true".to_string());
                form.options.remove_empty_lines = parse_bool_field(&value);
            }
            "showLineNumbers" => {
                let value = field.text().await.unwrap_or_else(|_| "true".to_string());
                form.options.show_line_numbers = parse_bool_field(&value);
            }
            "fileSummary" => {
                let value = field.text().await.unwrap_or_else(|_| "true".to_string());
                form.options.file_summary = parse_bool_field(&value);
            }
            "directoryStructure" => {
                let value = field.text().await.unwrap_or_else(|_| "true".to_string());
                form.options.directory_structure = parse_bool_field(&value);
            }
            "outputParsable" => {
                let value = field.text().await.unwrap_or_else(|_| "true".to_string());
                form.options.output_parsable = parse_bool_field(&value);
            }
            "compress" => {
                let value = field.text().await.unwrap_or_else(|_| "true".to_string());
                form.options.compress = parse_bool_field(&value);
            }
            "file" => {
                let file_name = field.file_name().unwrap_or("upload.zip").to_string();
                let data = field.bytes().await.map_err(|e| {
                    (
                        locale,
                        form.clone(),
                        format!(
                            "{}: {e}",
                            t(
                                locale,
                                "Failed to read ZIP upload",
                                "Не удалось прочитать ZIP-файл"
                            )
                        ),
                        StatusCode::BAD_REQUEST,
                    )
                })?;
                zip_upload = Some((file_name, data.to_vec()));
            }
            "folderManifest" => {
                folder_manifest = Some(field.text().await.map_err(|e| {
                    (
                        locale,
                        form.clone(),
                        format!(
                            "{}: {e}",
                            t(
                                locale,
                                "Failed to read folder manifest",
                                "Не удалось прочитать манифест папки"
                            )
                        ),
                        StatusCode::BAD_REQUEST,
                    )
                })?);
            }
            "folderFiles" => {
                let bytes = field.bytes().await.map_err(|e| {
                    (
                        locale,
                        form.clone(),
                        format!(
                            "{}: {e}",
                            t(
                                locale,
                                "Failed to read folder upload",
                                "Не удалось прочитать загрузку папки"
                            )
                        ),
                        StatusCode::BAD_REQUEST,
                    )
                })?;
                folder_files.push(bytes.to_vec());
            }
            _ => {}
        }
    }

    let source_kind = source_kind.ok_or_else(|| {
        (
            locale,
            form.clone(),
            t(locale, "Missing source type", "Не указан тип источника").to_string(),
            StatusCode::BAD_REQUEST,
        )
    })?;

    let source = match source_kind {
        SourceKind::Url => {
            if form.url.trim().is_empty() {
                return Err((
                    locale,
                    form.clone(),
                    t(
                        locale,
                        "Repository URL is required",
                        "Нужно указать URL репозитория",
                    )
                    .to_string(),
                    StatusCode::BAD_REQUEST,
                ));
            }
            PackSource::Url(form.url.trim().to_string())
        }
        SourceKind::Zip => {
            let (file_name, data) = zip_upload.ok_or_else(|| {
                (
                    locale,
                    form.clone(),
                    t(locale, "ZIP file is required", "Нужно выбрать ZIP-файл").to_string(),
                    StatusCode::BAD_REQUEST,
                )
            })?;

            PackSource::ZipUpload { file_name, data }
        }
        SourceKind::Folder => {
            let files = build_folder_upload(folder_manifest, folder_files)
                .map_err(|err| (locale, form.clone(), err.message, err.status))?;
            PackSource::FolderUpload { files }
        }
    };

    let job = PackJob {
        format: form.format.clone(),
        options: form.options.clone(),
        source,
    };

    Ok((locale, form, job))
}

fn render_response(
    status: StatusCode,
    locale: Locale,
    form: &WebFormState,
    result: Option<&PackResponse>,
    error: Option<&str>,
) -> Response {
    (
        status,
        Html(render_page(locale, form, result, error).into_string()),
    )
        .into_response()
}

fn render_page(
    locale: Locale,
    form: &WebFormState,
    result: Option<&PackResponse>,
    error: Option<&str>,
) -> Markup {
    html! {
        (DOCTYPE)
        html lang=(locale.code()) {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (t(locale, "Repomix Web", "Repomix Веб")) }
                meta name="description" content=(t(
                    locale,
                    "Rust-native web frontend for packing repositories into AI-friendly context.",
                    "Rust-native веб-интерфейс для упаковки репозиториев в AI-friendly контекст."
                ));
                style { (PreEscaped(APP_CSS)) }
            }
            body data-has-result=(if result.is_some() { "true" } else { "false" }) {
                main class="page" {
                    p id="app-status" class="sr-only" aria-live="polite" aria-atomic="true" {}
                    header class="hero" {
                        div class="hero-top" {
                            div class="hero-copy" {
                                div class="badge" { (t(locale, "Repomix web", "Repomix веб")) }
                                h1 { (t(locale, "Pack repositories into clean AI context", "Упаковывайте репозитории в чистый AI-контекст")) }
                                p {
                                    (t(
                                        locale,
                                        "Generate readable repository packs from a Git URL, a ZIP archive, or a local folder. Pick one source, tune the output once, and copy or download the result.",
                                        "Создавайте читаемые пакеты репозиториев из Git URL, ZIP-архива или локальной папки. Выберите источник, один раз настройте вывод и сразу копируйте или скачивайте результат."
                                    ))
                                }
                            }
                            nav class="locale-switcher" aria-label=(t(locale, "Language switcher", "Переключатель языка")) {
                                a href="/en" class=(locale_class(locale, Locale::En)) { "EN" }
                                a href="/ru" class=(locale_class(locale, Locale::Ru)) { "RU" }
                            }
                        }
                        div class="hero-points" {
                            span class="hero-point" { (t(locale, "Single workflow", "Единый сценарий")) }
                            span class="hero-point" { (t(locale, "Readable metrics", "Читаемые метрики")) }
                            span class="hero-point" { (t(locale, "Local uploads", "Локальные загрузки")) }
                        }
                    }

                    div class="stack" {
                        (render_workspace_form(locale, form))
                        div id="legacy-response-root" class="feedback-stack" {
                            @if let Some(error_message) = error {
                                section class="card notice" {
                                    div class="section-heading" {
                                        h2 { (t(locale, "Request failed", "Запрос завершился ошибкой")) }
                                        p class="notice-message" { (error_message) }
                                        p class="notice-hint" {
                                            (t(
                                                locale,
                                                "Check the selected source, upload content, and patterns, then try again.",
                                                "Проверьте выбранный источник, загруженные данные и паттерны, затем попробуйте снова."
                                            ))
                                        }
                                    }
                                }
                            }

                            @if let Some(response) = result {
                                (render_result(locale, response))
                            }
                        }
                        div id="app-feedback-root" class="feedback-stack" {}
                    }

                    footer class="footer" {
                        (t(
                            locale,
                            "Built on Axum + repomix. JSON API remains available under /api/*.",
                            "Построено на Axum + repomix. JSON API по-прежнему доступен под /api/*."
                        ))
                    }
                }

                script { (PreEscaped(APP_JS)) }
            }
        }
    }
}

fn render_workspace_form(locale: Locale, form: &WebFormState) -> Markup {
    html! {
        section class="card" {
            div class="section-heading" {
                h2 { (t(locale, "Choose a source and build the pack", "Выберите источник и соберите пакет")) }
                p {
                    (t(
                        locale,
                        "Use one source at a time. Shared output options stay in one place, so the flow remains short and predictable.",
                        "Используйте один источник за раз. Общие параметры вывода собраны в одном месте, чтобы сценарий оставался коротким и предсказуемым."
                    ))
                }
            }
            form
                class="workspace-form"
                method="post"
                action="/pack"
                enctype="multipart/form-data"
                data-folder-form="true"
                data-source-form="true"
                data-active-source=(form.source_kind.value()) {
                input type="hidden" name="locale" value=(locale.code());

                div class="source-switch" role="radiogroup" aria-label=(t(locale, "Source selector", "Выбор источника")) {
                    label class="source-tab" {
                        input class="source-tab-input sr-only" type="radio" name="sourceKind" value="url" checked[form.source_kind == SourceKind::Url] data-source-choice="url";
                        span class="source-tab-body" {
                            span class="source-tab-title" { "URL" }
                            span class="source-tab-copy" {
                                (t(locale, "Clone a public repository on the server.", "Клонируйте публичный репозиторий на сервере."))
                            }
                        }
                    }
                    label class="source-tab" {
                        input class="source-tab-input sr-only" type="radio" name="sourceKind" value="zip" checked[form.source_kind == SourceKind::Zip] data-source-choice="zip";
                        span class="source-tab-body" {
                            span class="source-tab-title" { "ZIP" }
                            span class="source-tab-copy" {
                                (t(locale, "Upload an archive you already prepared locally.", "Загрузите архив, который уже подготовили локально."))
                            }
                        }
                    }
                    label class="source-tab" {
                        input class="source-tab-input sr-only" type="radio" name="sourceKind" value="folder" checked[form.source_kind == SourceKind::Folder] data-source-choice="folder";
                        span class="source-tab-body" {
                            span class="source-tab-title" { (t(locale, "Folder", "Папка")) }
                            span class="source-tab-copy" {
                                (t(locale, "Select a local directory and keep browser paths.", "Выберите локальную директорию и сохраните пути из браузера."))
                            }
                        }
                    }
                }

                (render_url_source_panel(locale, form))
                (render_zip_source_panel(locale, form))
                (render_folder_source_panel(locale, form))

                (render_shared_options(locale, form, "workspace"))

                div class="form-actions" {
                    button class="primary" type="submit" data-pack-button="true" { (t(locale, "Generate pack", "Сформировать пакет")) }
                    button class="secondary" type="button" data-reset-button="true" hidden { (t(locale, "Reset", "Сбросить")) }
                    button class="secondary" type="button" data-cancel-button="true" hidden { (t(locale, "Cancel", "Отменить")) }
                    p class="form-footnote" {
                        (t(
                            locale,
                            "The browser form stays same-origin. API clients can continue to use /api/pack.",
                            "Браузерная форма остается same-origin. API-клиенты по-прежнему могут использовать /api/pack."
                        ))
                    }
                }
            }
        }
    }
}

fn render_url_source_panel(locale: Locale, form: &WebFormState) -> Markup {
    html! {
        section class="source-panel" data-source-panel="url" {
            div class="source-panel-header" {
                div class="source-panel-title-row" {
                    h3 { (t(locale, "Repository URL", "URL репозитория")) }
                    span class="source-pill" { "Git" }
                }
                p {
                    (t(
                        locale,
                        "Use a full Git URL or a shorthand like owner/repo.",
                        "Используйте полный Git URL или сокращение вида owner/repo."
                    ))
                }
            }

            div class="field" {
                label for="url-source" { (t(locale, "Repository URL or shorthand", "URL репозитория или shorthand")) }
                input
                    id="url-source"
                    type="text"
                    inputmode="url"
                    autocapitalize="off"
                    spellcheck="false"
                    name="url"
                    list="repo-history"
                    data-source-control="true"
                    placeholder="github.com/yamadashy/repomix or yamadashy/repomix"
                    value=(form.url.as_str());
                datalist id="repo-history" {}
                p class="helper" {
                    (t(
                        locale,
                        "Examples: https://github.com/yamadashy/repomix or yamadashy/repomix",
                        "Примеры: https://github.com/yamadashy/repomix или yamadashy/repomix"
                    ))
                }
                p class="helper notice-hint" id="url-validation-message" hidden {
                    (t(
                        locale,
                        "Enter a valid Git repository URL, SSH address, or owner/repo shorthand.",
                        "Введите корректный URL Git-репозитория, SSH-адрес или shorthand вида owner/repo."
                    ))
                }
            }
        }
    }
}

fn render_zip_source_panel(locale: Locale, _form: &WebFormState) -> Markup {
    html! {
        section class="source-panel" data-source-panel="zip" {
            div class="source-panel-header" {
                div class="source-panel-title-row" {
                    h3 { (t(locale, "ZIP archive", "ZIP-архив")) }
                    span class="source-pill" { "ZIP" }
                }
                p {
                    (t(
                        locale,
                        "Best when you want to preserve an exact local folder snapshot or upload a larger input in one file.",
                        "Лучше всего подходит, когда нужно сохранить точный локальный снимок папки или загрузить крупный набор данных одним файлом."
                    ))
                }
            }

            label class="upload-zone" for="zip-file" tabindex="0" data-file-target="zip-file" data-drop-enabled="true" {
                span class="upload-zone-head" {
                    span class="upload-zone-title" { (t(locale, "Drop a ZIP here or choose a file", "Перетащите ZIP сюда или выберите файл")) }
                    span class="upload-zone-action" { (t(locale, "Choose ZIP", "Выбрать ZIP")) }
                }
                span class="upload-zone-copy" {
                    (t(
                        locale,
                        "Drag and drop works for quick tests. For repeatable results, keep the archive structure exactly as you want it packed.",
                        "Перетаскивание подходит для быстрых проверок. Для повторяемого результата сохраните структуру архива ровно в том виде, в котором хотите ее упаковать."
                    ))
                }
            }
            input
                class="sr-only"
                id="zip-file"
                type="file"
                name="file"
                accept=".zip,application/zip"
                data-source-control="true"
                data-summary-target="zip-summary";
            p
                class="helper upload-summary"
                id="zip-summary"
                aria-live="polite"
                data-empty-label=(t(locale, "No ZIP selected yet.", "ZIP еще не выбран."))
                data-selected-template=(t(locale, "Selected ZIP: {name} · {size}", "Выбран ZIP: {name} · {size}")) {
                (t(locale, "No ZIP selected yet.", "ZIP еще не выбран."))
            }
        }
    }
}

fn render_folder_source_panel(locale: Locale, _form: &WebFormState) -> Markup {
    html! {
        section class="source-panel" data-source-panel="folder" {
            div class="source-panel-header" {
                div class="source-panel-title-row" {
                    h3 { (t(locale, "Local folder", "Локальная папка")) }
                    span class="source-pill" { (t(locale, "Browser paths", "Пути браузера")) }
                }
                p {
                    (t(
                        locale,
                        "Choose a folder in the browser and keep its relative paths without creating an archive first.",
                        "Выберите папку в браузере и сохраните ее относительные пути без предварительного создания архива."
                    ))
                }
            }

            input type="hidden" name="folderManifest" value="" data-source-control="true";
            label class="upload-zone" for="folder-files" tabindex="0" data-file-target="folder-files" {
                span class="upload-zone-head" {
                    span class="upload-zone-title" { (t(locale, "Choose a folder to pack", "Выберите папку для упаковки")) }
                    span class="upload-zone-action" { (t(locale, "Choose folder", "Выбрать папку")) }
                }
                span class="upload-zone-copy" {
                    (t(
                        locale,
                        "The browser will send file contents plus relative paths. This depends on directory upload support in your browser.",
                        "Браузер отправит содержимое файлов и их относительные пути. Для этого нужна поддержка загрузки директорий в вашем браузере."
                    ))
                }
            }
            input
                class="sr-only"
                id="folder-files"
                type="file"
                name="folderFiles"
                multiple="multiple"
                webkitdirectory="webkitdirectory"
                directory="directory"
                data-source-control="true"
                data-summary-target="folder-summary";
            p
                class="helper upload-summary"
                id="folder-summary"
                aria-live="polite"
                data-empty-label=(t(locale, "No folder selected yet.", "Папка еще не выбрана."))
                data-selected-template=(t(locale, "Selected {count} files · {size} · Starts with {sample}", "Выбрано {count} файлов · {size} · Начинается с {sample}")) {
                (t(locale, "No folder selected yet.", "Папка еще не выбрана."))
            }
            p class="helper" {
                (t(locale, "Folder mode requires a JavaScript-enabled browser so relative paths can be prepared before upload.", "Режим папки требует браузер с включенным JavaScript, чтобы перед загрузкой подготовить относительные пути."))
            }
        }
    }
}

fn render_shared_options(locale: Locale, form: &WebFormState, prefix: &str) -> Markup {
    let format_id = format!("{prefix}-format");
    let include_id = format!("{prefix}-include");
    let ignore_id = format!("{prefix}-ignore");

    html! {
        div class="options-shell" {
            div class="options-header" {
                h3 { (t(locale, "Output options", "Параметры вывода")) }
                p {
                    (t(
                        locale,
                        "Tune the result once. These options apply to whichever source is currently selected.",
                        "Настройте результат один раз. Эти параметры применяются к текущему выбранному источнику."
                    ))
                }
            }

            div class="field-row" {
                div class="field" {
                    label for=(format_id.as_str()) { (t(locale, "Output format", "Формат вывода")) }
                    select id=(format_id.as_str()) name="format" {
                        option value="xml" selected[form.format == "xml"] { "XML" }
                        option value="markdown" selected[form.format == "markdown"] { "Markdown" }
                        option value="plain" selected[form.format == "plain"] { (t(locale, "Plain text", "Обычный текст")) }
                    }
                    p class="helper" {
                        (t(
                            locale,
                            "XML keeps structure explicit, Markdown reads easier, Plain stays lightweight.",
                            "XML лучше сохраняет структуру, Markdown проще читать, а обычный текст остается самым легким вариантом."
                        ))
                    }
                }
                div class="field" {
                    label for=(include_id.as_str()) { (t(locale, "Include patterns", "Include-паттерны")) }
                    input
                        id=(include_id.as_str())
                        type="text"
                        name="includePatterns"
                        placeholder="*.rs,*.toml"
                        value=(form.options.include_patterns.as_deref().unwrap_or(""));
                    p class="helper" {
                        (t(
                            locale,
                            "Comma-separated globs. Use this to narrow the output to the files you actually need.",
                            "Глоб-паттерны через запятую. Используйте их, чтобы сузить вывод до действительно нужных файлов."
                        ))
                    }
                }
                div class="field" {
                    label for=(ignore_id.as_str()) { (t(locale, "Ignore patterns", "Ignore-паттерны")) }
                    input
                        id=(ignore_id.as_str())
                        type="text"
                        name="ignorePatterns"
                        placeholder="target/**,node_modules/**"
                        value=(form.options.ignore_patterns.as_deref().unwrap_or(""));
                    p class="helper" {
                        (t(
                            locale,
                            "Exclude generated, vendored, or heavy directories to keep the pack smaller and cleaner.",
                            "Исключайте сгенерированные, вендорные или тяжелые директории, чтобы сделать пакет меньше и чище."
                        ))
                    }
                }
            }

            div class="checkbox-grid" {
                label class="checkbox" {
                    input type="hidden" name="removeComments" value="false";
                    input type="checkbox" name="removeComments" value="true" checked[form.options.remove_comments];
                    span class="checkbox-copy" {
                        strong { (t(locale, "Remove comments", "Удалять комментарии")) }
                        span { (t(locale, "Useful when you need denser output for large repositories.", "Полезно, когда нужен более плотный вывод для больших репозиториев.")) }
                    }
                }
                label class="checkbox" {
                    input type="hidden" name="removeEmptyLines" value="false";
                    input type="checkbox" name="removeEmptyLines" value="true" checked[form.options.remove_empty_lines];
                    span class="checkbox-copy" {
                        strong { (t(locale, "Remove empty lines", "Удалять пустые строки")) }
                        span { (t(locale, "Cuts visual noise in the final bundle.", "Убирает визуальный шум в итоговом пакете.")) }
                    }
                }
                label class="checkbox" {
                    input type="hidden" name="showLineNumbers" value="false";
                    input type="checkbox" name="showLineNumbers" value="true" checked[form.options.show_line_numbers];
                    span class="checkbox-copy" {
                        strong { (t(locale, "Show line numbers", "Показывать номера строк")) }
                        span { (t(locale, "Helpful when you quote snippets back into reviews or prompts.", "Удобно, когда вы вставляете фрагменты обратно в ревью или промпты.")) }
                    }
                }
                label class="checkbox" {
                    input type="hidden" name="fileSummary" value="false";
                    input type="checkbox" name="fileSummary" value="true" checked[form.options.file_summary];
                    span class="checkbox-copy" {
                        strong { (t(locale, "Include file summary", "Включать сводку по файлам")) }
                        span { (t(locale, "Keeps the file metrics section in the generated output.", "Сохраняет секцию с метриками файлов в итоговом выводе.")) }
                    }
                }
                label class="checkbox" {
                    input type="hidden" name="directoryStructure" value="false";
                    input type="checkbox" name="directoryStructure" value="true" checked[form.options.directory_structure];
                    span class="checkbox-copy" {
                        strong { (t(locale, "Include directory structure", "Включать структуру директорий")) }
                        span { (t(locale, "Keeps the repository tree in the generated output.", "Сохраняет дерево репозитория в сгенерированном выводе.")) }
                    }
                }
                label class="checkbox" {
                    input type="hidden" name="outputParsable" value="false";
                    input type="checkbox" name="outputParsable" value="true" checked[form.options.output_parsable];
                    span class="checkbox-copy" {
                        strong { (t(locale, "Output parsable format", "Выводить parsable-формат")) }
                        span { (t(locale, "Escapes the output to better match the selected schema style.", "Экранирует вывод, чтобы лучше соответствовать выбранной схеме формата.")) }
                    }
                }
                label class="checkbox" {
                    input type="hidden" name="compress" value="false";
                    input type="checkbox" name="compress" value="true" checked[form.options.compress];
                    span class="checkbox-copy" {
                        strong { (t(locale, "Enable compression", "Включить сжатие")) }
                        span { (t(locale, "Uses tree-sitter based compression when the language is supported.", "Использует tree-sitter сжатие, когда язык поддерживается.")) }
                    }
                }
            }
        }
    }
}

fn render_result(locale: Locale, response: &PackResponse) -> Markup {
    let download_name = download_file_name(response);
    let download_type = download_content_type(&response.format);
    let total_tokens = response
        .metadata
        .summary
        .as_ref()
        .map(|summary| summary.total_tokens)
        .unwrap_or_default();
    let format_label = response.format.to_uppercase();

    html! {
        section class="card result-card" id="legacy-result" {
            div class="result-shell" {
                div class="result-main" {
                    div class="result-header" {
                        div class="section-heading" {
                            h2 { (t(locale, "Packed output", "Готовый результат")) }
                            p { (t(locale, "Copy the generated bundle directly or download it for later use.", "Скопируйте сгенерированный пакет сразу или скачайте его для дальнейшей работы.")) }
                        }
                        div class="result-actions" {
                            button
                                class="secondary"
                                type="button"
                                data-copy-target="#result-output"
                                data-copied-label=(t(locale, "Copied", "Скопировано")) {
                                (t(locale, "Copy result", "Копировать результат"))
                            }
                            button
                                class="secondary"
                                type="button"
                                data-download-target="#result-output"
                                data-download-name=(download_name)
                                data-download-type=(download_type) {
                                (t(locale, "Download", "Скачать"))
                            }
                        }
                    }

                    div class="output-panel" {
                        label for="result-output" { (t(locale, "Output", "Вывод")) }
                        p class="helper" {
                            (t(locale, "This is the final packed text that Repomix produced from your selected source.", "Это итоговый упакованный текст, который Repomix создал из выбранного источника."))
                        }
                        textarea id="result-output" readonly { (&response.content) }
                    }
                }

                aside class="result-sidebar" {
                    div class="overview-grid" {
                        div class="meta-card" {
                            span class="meta-label" { (t(locale, "Repository", "Репозиторий")) }
                            div class="meta-value" { (&response.metadata.repository) }
                        }
                        div class="meta-card" {
                            span class="meta-label" { (t(locale, "Format", "Формат")) }
                            div class="meta-value" { (format_label) }
                        }
                        div class="meta-card" {
                            span class="meta-label" { (t(locale, "Generated at", "Сгенерировано")) }
                            div class="meta-value" { (&response.metadata.timestamp) }
                        }
                    }

                    @if let Some(summary) = &response.metadata.summary {
                        div class="summary-grid" {
                            div class="metric-card" {
                                span class="metric-label" { (t(locale, "Files", "Файлы")) }
                                div class="metric-value" { (format_number(summary.total_files)) }
                                div class="metric-subtext" { (t(locale, "processed files", "обработано файлов")) }
                            }
                            div class="metric-card" {
                                span class="metric-label" { (t(locale, "Characters", "Символы")) }
                                div class="metric-value" { (format_number(summary.total_characters)) }
                                div class="metric-subtext" { (t(locale, "characters in output", "символов в выводе")) }
                            }
                            div class="metric-card" {
                                span class="metric-label" { (t(locale, "Tokens", "Токены")) }
                                div class="metric-value" { (format_number(summary.total_tokens)) }
                                div class="metric-subtext" { (t(locale, "estimated prompt size", "оценка размера промпта")) }
                            }
                        }
                    }

                    @if let Some(top_files) = &response.metadata.top_files {
                        @if !top_files.is_empty() {
                            section class="top-files" {
                                div class="section-heading" {
                                    h3 { (t(locale, "Top files by token count", "Топ файлов по токенам")) }
                                    p { (t(locale, "Use this list to spot the biggest prompt contributors first.", "Используйте этот список, чтобы сначала увидеть самые тяжелые вкладчики в промпт.")) }
                                }
                                ol class="top-files-list" {
                                    @for file in top_files {
                                        li class="top-file-item" {
                                            div class="top-file-path" { (&file.path) }
                                            div class="top-file-stats" {
                                                span class="stat-chip strong" {
                                                    (format!("{} {}", format_number(file.token_count), t(locale, "tokens", "токенов")))
                                                }
                                                span class="stat-chip" {
                                                    (format!("{} {}", format_number(file.char_count), t(locale, "chars", "символов")))
                                                }
                                                @if total_tokens > 0 {
                                                    span class="stat-chip" {
                                                        (format!("{} {}", format_percentage(file.token_count, total_tokens), t(locale, "of total tokens", "от всех токенов")))
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn locale_class(active: Locale, current: Locale) -> &'static str {
    if active == current {
        "locale-link active"
    } else {
        "locale-link"
    }
}

fn normalize_optional_text(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn parse_bool_field(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "yes" | "on"
    )
}

fn download_file_name(response: &PackResponse) -> String {
    match response.format.as_str() {
        "xml" => "repomix-output.xml",
        "markdown" => "repomix-output.md",
        _ => "repomix-output.txt",
    }
    .to_string()
}

fn download_content_type(format: &str) -> &'static str {
    match format {
        "xml" => "application/xml;charset=utf-8",
        "markdown" => "text/markdown;charset=utf-8",
        _ => "text/plain;charset=utf-8",
    }
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*c);
    }

    result
}

fn format_percentage(numerator: usize, denominator: usize) -> String {
    if denominator == 0 {
        "0.0%".to_string()
    } else {
        format!("{:.1}%", (numerator as f64 / denominator as f64) * 100.0)
    }
}

fn t(locale: Locale, en: &'static str, ru: &'static str) -> &'static str {
    match locale {
        Locale::En => en,
        Locale::Ru => ru,
    }
}
