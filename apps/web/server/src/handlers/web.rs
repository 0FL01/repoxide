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
  --bg: #f4f7fb;
  --bg-accent: #e9eef8;
  --surface: rgba(255, 255, 255, 0.94);
  --surface-strong: #ffffff;
  --surface-muted: #f8fafc;
  --border: rgba(15, 23, 42, 0.1);
  --border-strong: rgba(37, 99, 235, 0.18);
  --text: #0f172a;
  --muted: #526072;
  --muted-strong: #334155;
  --brand: #2563eb;
  --brand-strong: #1d4ed8;
  --brand-soft: rgba(37, 99, 235, 0.1);
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
    --bg-accent: #121a2d;
    --surface: rgba(15, 23, 42, 0.94);
    --surface-strong: #111827;
    --surface-muted: #0f172a;
    --border: rgba(148, 163, 184, 0.18);
    --border-strong: rgba(96, 165, 250, 0.28);
    --text: #e5eefc;
    --muted: #a5b4cc;
    --muted-strong: #d7e2f3;
    --brand: #60a5fa;
    --brand-strong: #93c5fd;
    --brand-soft: rgba(96, 165, 250, 0.16);
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
"#;

const APP_JS: &str = r#"
const numberFormatter = new Intl.NumberFormat();

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

function fillTemplate(template, values) {
  return template.replace(/\{(\w+)\}/g, (_, key) => values[key] ?? '');
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
  const values = {
    count: numberFormatter.format(files.length),
    size: formatBytes(totalSize),
    name: first.name,
    sample: first.webkitRelativePath || first.name,
  };

  summary.textContent = fillTemplate(summary.dataset.selectedTemplate || '', values);
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

    const file = event.dataTransfer.files[0];
    const transfer = new DataTransfer();
    transfer.items.add(file);
    input.files = transfer.files;
    input.dispatchEvent(new Event('change', { bubbles: true }));
  });
}

function initSourceForm(form) {
  const sourceChoices = Array.from(form.querySelectorAll('input[name="sourceKind"][data-source-choice]'));
  const checkedChoice = sourceChoices.find((input) => input instanceof HTMLInputElement && input.checked);
  const initial = checkedChoice instanceof HTMLInputElement ? checkedChoice.value : (form.dataset.activeSource || 'url');
  setActiveSource(form, initial);

  sourceChoices.forEach((input) => {
    if (input instanceof HTMLInputElement) {
      input.addEventListener('change', () => {
        setActiveSource(form, input.value);
      });
    }
  });

  form.querySelectorAll('input[type="file"][data-summary-target]').forEach((input) => {
    input.addEventListener('change', () => updateFileSummary(input));
    updateFileSummary(input);
  });

  form.querySelectorAll('[data-file-target]').forEach((zone) => initUploadZone(zone));
}

document.addEventListener('submit', (event) => {
  const form = event.target;
  if (!(form instanceof HTMLFormElement) || form.dataset.folderForm !== 'true') {
    return;
  }

  const input = form.querySelector('input[name="folderFiles"]');
  const manifest = form.querySelector('input[name="folderManifest"]');
  if (!(input instanceof HTMLInputElement) || !(manifest instanceof HTMLInputElement) || !input.files) {
    return;
  }

  const paths = [];
  for (const file of input.files) {
    paths.push(file.webkitRelativePath || file.name);
  }

  manifest.value = JSON.stringify({ paths });
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
    const liveRegion = document.getElementById('app-status');

    if (!value) {
      return;
    }

    try {
      await navigator.clipboard.writeText(value);
    } catch (_) {
      return;
    }

    const original = copyButton.textContent;
    copyButton.textContent = copyButton.dataset.copiedLabel || 'Copied';
    if (liveRegion instanceof HTMLElement) {
      liveRegion.textContent = copyButton.dataset.copiedLabel || 'Copied';
    }
    window.setTimeout(() => {
      copyButton.textContent = original;
    }, 1800);
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

window.addEventListener('DOMContentLoaded', () => {
  document.querySelectorAll('[data-source-form="true"]').forEach((form) => {
    if (form instanceof HTMLFormElement) {
      initSourceForm(form);
    }
  });

  if (document.body.dataset.hasResult !== 'true') {
    return;
  }

  const result = document.getElementById('result');
  if (result instanceof HTMLElement) {
    result.scrollIntoView({ block: 'start' });
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
            "removeComments" => form.options.remove_comments = true,
            "removeEmptyLines" => form.options.remove_empty_lines = true,
            "showLineNumbers" => form.options.show_line_numbers = true,
            "compress" => form.options.compress = true,
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

                        (render_workspace_form(locale, form))
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
                    button class="primary" type="submit" { (t(locale, "Generate pack", "Сформировать пакет")) }
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
                    data-source-control="true"
                    placeholder="github.com/yamadashy/repomix or yamadashy/repomix"
                    value=(form.url.as_str());
                p class="helper" {
                    (t(
                        locale,
                        "Examples: https://github.com/yamadashy/repomix or yamadashy/repomix",
                        "Примеры: https://github.com/yamadashy/repomix или yamadashy/repomix"
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
                    input type="checkbox" name="removeComments" checked[form.options.remove_comments];
                    span class="checkbox-copy" {
                        strong { (t(locale, "Remove comments", "Удалять комментарии")) }
                        span { (t(locale, "Useful when you need denser output for large repositories.", "Полезно, когда нужен более плотный вывод для больших репозиториев.")) }
                    }
                }
                label class="checkbox" {
                    input type="checkbox" name="removeEmptyLines" checked[form.options.remove_empty_lines];
                    span class="checkbox-copy" {
                        strong { (t(locale, "Remove empty lines", "Удалять пустые строки")) }
                        span { (t(locale, "Cuts visual noise in the final bundle.", "Убирает визуальный шум в итоговом пакете.")) }
                    }
                }
                label class="checkbox" {
                    input type="checkbox" name="showLineNumbers" checked[form.options.show_line_numbers];
                    span class="checkbox-copy" {
                        strong { (t(locale, "Show line numbers", "Показывать номера строк")) }
                        span { (t(locale, "Helpful when you quote snippets back into reviews or prompts.", "Удобно, когда вы вставляете фрагменты обратно в ревью или промпты.")) }
                    }
                }
                label class="checkbox" {
                    input type="checkbox" name="compress" checked[form.options.compress];
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
        section class="card result-card" id="result" {
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
