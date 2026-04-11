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
  --bg: #f5f7fb;
  --panel: rgba(255, 255, 255, 0.9);
  --panel-strong: #ffffff;
  --border: rgba(15, 23, 42, 0.12);
  --text: #0f172a;
  --muted: #475569;
  --brand: #2563eb;
  --brand-strong: #1d4ed8;
  --success: #166534;
  --danger: #b91c1c;
  --shadow: 0 18px 45px rgba(15, 23, 42, 0.08);
}

@media (prefers-color-scheme: dark) {
  :root {
    --bg: #0b1120;
    --panel: rgba(15, 23, 42, 0.88);
    --panel-strong: #111827;
    --border: rgba(148, 163, 184, 0.2);
    --text: #e5eefc;
    --muted: #a5b4cc;
    --brand: #60a5fa;
    --brand-strong: #93c5fd;
    --success: #86efac;
    --danger: #fca5a5;
    --shadow: 0 18px 45px rgba(2, 6, 23, 0.45);
  }
}

* { box-sizing: border-box; }
body {
  margin: 0;
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background:
    radial-gradient(circle at top left, rgba(37, 99, 235, 0.14), transparent 34%),
    radial-gradient(circle at top right, rgba(59, 130, 246, 0.1), transparent 28%),
    var(--bg);
  color: var(--text);
}

a { color: inherit; }

.page {
  max-width: 1200px;
  margin: 0 auto;
  padding: 32px 20px 48px;
}

.hero {
  display: grid;
  gap: 18px;
  margin-bottom: 28px;
}

.badge {
  width: fit-content;
  padding: 6px 10px;
  border-radius: 999px;
  background: rgba(37, 99, 235, 0.12);
  color: var(--brand);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.hero h1 {
  margin: 0;
  font-size: clamp(2.4rem, 4vw, 4rem);
  line-height: 1.05;
}

.hero p {
  margin: 0;
  max-width: 800px;
  color: var(--muted);
  font-size: 1.05rem;
  line-height: 1.6;
}

.locale-switcher {
  display: inline-flex;
  width: fit-content;
  gap: 8px;
  padding: 6px;
  border-radius: 999px;
  background: var(--panel);
  border: 1px solid var(--border);
  box-shadow: var(--shadow);
}

.locale-link {
  padding: 8px 12px;
  border-radius: 999px;
  text-decoration: none;
  color: var(--muted);
  font-weight: 600;
}

.locale-link.active {
  background: var(--brand);
  color: white;
}

.stack {
  display: grid;
  gap: 20px;
}

.card {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 24px;
  padding: 24px;
  box-shadow: var(--shadow);
  backdrop-filter: blur(12px);
}

.card h2,
.card h3 {
  margin: 0 0 10px;
}

.card p {
  color: var(--muted);
}

.notice {
  border-left: 4px solid var(--danger);
}

.notice h2,
.notice p,
.notice li {
  color: var(--danger);
}

.result-header,
.form-header {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.meta-grid,
.forms-grid,
.summary-grid,
.checkbox-grid {
  display: grid;
  gap: 16px;
}

.forms-grid {
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
}

.summary-grid {
  margin-top: 18px;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
}

.summary-item,
.meta-item {
  padding: 16px;
  border-radius: 18px;
  background: var(--panel-strong);
  border: 1px solid var(--border);
}

.summary-item strong,
.meta-item strong {
  display: block;
  margin-bottom: 6px;
  font-size: 0.9rem;
  color: var(--muted);
}

.summary-item span,
.meta-item span {
  font-size: 1.1rem;
  font-weight: 700;
}

.field,
.options-grid {
  display: grid;
  gap: 10px;
}

.field {
  margin-bottom: 14px;
}

label {
  font-weight: 700;
}

input[type="text"],
input[type="url"],
select,
textarea,
input[type="file"] {
  width: 100%;
  border-radius: 16px;
  border: 1px solid var(--border);
  background: var(--panel-strong);
  color: var(--text);
  padding: 12px 14px;
  font: inherit;
}

textarea {
  min-height: 380px;
  resize: vertical;
  font-family: "SFMono-Regular", "Cascadia Code", "JetBrains Mono", monospace;
  line-height: 1.5;
}

.checkbox-grid {
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
}

.checkbox {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px 14px;
  border-radius: 16px;
  border: 1px solid var(--border);
  background: var(--panel-strong);
}

.checkbox input {
  margin-top: 4px;
}

.form-actions,
.result-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-top: 16px;
}

button,
.button-link {
  appearance: none;
  border: none;
  cursor: pointer;
  border-radius: 16px;
  padding: 12px 16px;
  font: inherit;
  font-weight: 700;
  text-decoration: none;
}

button.primary,
.button-link.primary {
  background: var(--brand);
  color: white;
}

button.primary:hover,
.button-link.primary:hover {
  background: var(--brand-strong);
}

button.secondary,
.button-link.secondary {
  background: var(--panel-strong);
  color: var(--text);
  border: 1px solid var(--border);
}

.helper {
  margin: 0;
  font-size: 0.92rem;
  color: var(--muted);
}

table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 18px;
  overflow: hidden;
  border-radius: 18px;
  border: 1px solid var(--border);
}

th,
td {
  padding: 12px 14px;
  border-bottom: 1px solid var(--border);
  text-align: left;
}

thead {
  background: rgba(37, 99, 235, 0.08);
}

tbody tr:last-child td {
  border-bottom: none;
}

.footer {
  margin-top: 24px;
  color: var(--muted);
  text-align: center;
  font-size: 0.95rem;
}
"#;

const APP_JS: &str = r#"
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

    if (!value) {
      return;
    }

    await navigator.clipboard.writeText(value);
    const original = copyButton.textContent;
    copyButton.textContent = copyButton.dataset.copiedLabel || 'Copied';
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
    url: String,
    format: String,
    options: PackOptions,
}

impl WebFormState {
    fn new() -> Self {
        Self {
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

#[derive(Debug, Clone, Copy)]
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
                    header class="hero" {
                        div class="badge" { (t(locale, "Rust web frontend", "Rust веб-фронт")) }
                        h1 { (t(locale, "Pack repositories without the Node frontend", "Упаковывайте репозитории без Node-фронта")) }
                        p {
                            (t(
                                locale,
                                "The active web UI now ships directly from the Rust Axum server. Use repository URLs, ZIP uploads, or browser folder uploads and get AI-ready output immediately.",
                                "Активный веб-интерфейс теперь отдается напрямую из Rust Axum-сервера. Используйте URL репозитория, ZIP-файлы или загрузку папки из браузера и сразу получайте AI-ready результат."
                            ))
                        }
                        nav class="locale-switcher" aria-label=(t(locale, "Language switcher", "Переключатель языка")) {
                            a href="/en" class=(locale_class(locale, Locale::En)) { "EN" }
                            a href="/ru" class=(locale_class(locale, Locale::Ru)) { "RU" }
                        }
                    }

                    div class="stack" {
                        @if let Some(error_message) = error {
                            section class="card notice" {
                                h2 { (t(locale, "Request failed", "Запрос завершился ошибкой")) }
                                p { (error_message) }
                            }
                        }

                        @if let Some(response) = result {
                            (render_result(locale, response))
                        }

                        section class="forms-grid" {
                            (render_url_form(locale, form))
                            (render_zip_form(locale, form))
                            (render_folder_form(locale, form))
                        }
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

fn render_url_form(locale: Locale, form: &WebFormState) -> Markup {
    html! {
        section class="card" {
            div class="form-header" {
                h2 { (t(locale, "Pack repository URL", "Упаковать репозиторий по URL")) }
                span class="badge" { "URL" }
            }
            p { (t(locale, "Clone a public repository and pack it on the server.", "Клонируйте публичный репозиторий и упакуйте его на сервере.")) }
            form method="post" action="/pack" enctype="multipart/form-data" {
                input type="hidden" name="locale" value=(locale.code());
                input type="hidden" name="sourceKind" value="url";
                div class="field" {
                    label for="url-source" { (t(locale, "Repository URL or shorthand", "URL репозитория или shorthand")) }
                    input
                        id="url-source"
                        type="text"
                        inputmode="url"
                        autocapitalize="off"
                        spellcheck="false"
                        name="url"
                        placeholder="https://github.com/yamadashy/repomix or yamadashy/repomix"
                        value=(form.url.as_str());
                }
                (render_shared_options(locale, form, "url"))
                div class="form-actions" {
                    button class="primary" type="submit" { (t(locale, "Pack repository", "Упаковать репозиторий")) }
                }
            }
        }
    }
}

fn render_zip_form(locale: Locale, form: &WebFormState) -> Markup {
    html! {
        section class="card" {
            div class="form-header" {
                h2 { (t(locale, "Upload ZIP archive", "Загрузить ZIP-архив")) }
                span class="badge" { "ZIP" }
            }
            p { (t(locale, "Upload an archive generated locally and pack it directly in the Rust backend.", "Загрузите локально подготовленный архив и упакуйте его напрямую в Rust backend.")) }
            form method="post" action="/pack" enctype="multipart/form-data" {
                input type="hidden" name="locale" value=(locale.code());
                input type="hidden" name="sourceKind" value="zip";
                div class="field" {
                    label for="zip-file" { (t(locale, "ZIP file", "ZIP-файл")) }
                    input id="zip-file" type="file" name="file" accept=".zip,application/zip";
                    p class="helper" { (t(locale, "Use ZIP uploads for exact folder structures or larger local inputs.", "Используйте ZIP-загрузку для точной структуры папок или более крупных локальных данных.")) }
                }
                (render_shared_options(locale, form, "zip"))
                div class="form-actions" {
                    button class="primary" type="submit" { (t(locale, "Pack archive", "Упаковать архив")) }
                }
            }
        }
    }
}

fn render_folder_form(locale: Locale, form: &WebFormState) -> Markup {
    html! {
        section class="card" {
            div class="form-header" {
                h2 { (t(locale, "Upload a folder", "Загрузить папку")) }
                span class="badge" { (t(locale, "Browser paths", "Пути браузера")) }
            }
            p { (t(locale, "Select a directory and the browser will send its relative paths to the Rust backend.", "Выберите директорию, и браузер передаст ее относительные пути в Rust backend.")) }
            form method="post" action="/pack" enctype="multipart/form-data" data-folder-form="true" {
                input type="hidden" name="locale" value=(locale.code());
                input type="hidden" name="sourceKind" value="folder";
                input type="hidden" name="folderManifest" value="";
                div class="field" {
                    label for="folder-files" { (t(locale, "Folder contents", "Содержимое папки")) }
                    input
                        id="folder-files"
                        type="file"
                        name="folderFiles"
                        multiple="multiple"
                        webkitdirectory="webkitdirectory"
                        directory="directory";
                    p class="helper" { (t(locale, "Requires browser support for directory uploads and relative path metadata.", "Требуется поддержка загрузки директорий и относительных путей в браузере.")) }
                }
                (render_shared_options(locale, form, "folder"))
                div class="form-actions" {
                    button class="primary" type="submit" { (t(locale, "Pack folder", "Упаковать папку")) }
                }
            }
        }
    }
}

fn render_shared_options(locale: Locale, form: &WebFormState, prefix: &str) -> Markup {
    let format_id = format!("{prefix}-format");
    let include_id = format!("{prefix}-include");
    let ignore_id = format!("{prefix}-ignore");

    html! {
        div class="options-grid" {
            div class="field" {
                label for=(format_id.as_str()) { (t(locale, "Output format", "Формат вывода")) }
                select id=(format_id.as_str()) name="format" {
                    option value="xml" selected[form.format == "xml"] { "XML" }
                    option value="markdown" selected[form.format == "markdown"] { "Markdown" }
                    option value="plain" selected[form.format == "plain"] { (t(locale, "Plain text", "Обычный текст")) }
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
            }
            div class="field" {
                label for=(ignore_id.as_str()) { (t(locale, "Ignore patterns", "Ignore-паттерны")) }
                input
                    id=(ignore_id.as_str())
                    type="text"
                    name="ignorePatterns"
                    placeholder="target/**,node_modules/**"
                    value=(form.options.ignore_patterns.as_deref().unwrap_or(""));
            }
            div class="checkbox-grid" {
                label class="checkbox" {
                    input type="checkbox" name="removeComments" checked[form.options.remove_comments];
                    span { (t(locale, "Remove comments", "Удалять комментарии")) }
                }
                label class="checkbox" {
                    input type="checkbox" name="removeEmptyLines" checked[form.options.remove_empty_lines];
                    span { (t(locale, "Remove empty lines", "Удалять пустые строки")) }
                }
                label class="checkbox" {
                    input type="checkbox" name="showLineNumbers" checked[form.options.show_line_numbers];
                    span { (t(locale, "Show line numbers", "Показывать номера строк")) }
                }
                label class="checkbox" {
                    input type="checkbox" name="compress" checked[form.options.compress];
                    span { (t(locale, "Enable compression", "Включить сжатие")) }
                }
            }
        }
    }
}

fn render_result(locale: Locale, response: &PackResponse) -> Markup {
    let download_name = download_file_name(response);
    let download_type = download_content_type(&response.format);

    html! {
        section class="card" id="result" {
            div class="result-header" {
                div {
                    h2 { (t(locale, "Packed output", "Готовый результат")) }
                    p { (t(locale, "Review, copy, or download the generated bundle below.", "Просмотрите, скопируйте или скачайте сгенерированный пакет ниже.")) }
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

            div class="summary-grid" {
                div class="summary-item" {
                    strong { (t(locale, "Repository", "Репозиторий")) }
                    span { (&response.metadata.repository) }
                }
                div class="summary-item" {
                    strong { (t(locale, "Format", "Формат")) }
                    span { (&response.format) }
                }
                div class="summary-item" {
                    strong { (t(locale, "Generated at", "Сгенерировано")) }
                    span { (&response.metadata.timestamp) }
                }
                @if let Some(summary) = &response.metadata.summary {
                    div class="summary-item" {
                        strong { (t(locale, "Files", "Файлы")) }
                        span { (summary.total_files) }
                    }
                    div class="summary-item" {
                        strong { (t(locale, "Characters", "Символы")) }
                        span { (summary.total_characters) }
                    }
                    div class="summary-item" {
                        strong { (t(locale, "Tokens", "Токены")) }
                        span { (summary.total_tokens) }
                    }
                }
            }

            div class="field" style="margin-top: 18px;" {
                label for="result-output" { (t(locale, "Output", "Вывод")) }
                textarea id="result-output" readonly { (&response.content) }
            }

            @if let Some(top_files) = &response.metadata.top_files {
                @if !top_files.is_empty() {
                    h3 { (t(locale, "Top files", "Топ файлов")) }
                    table {
                        thead {
                            tr {
                                th { (t(locale, "Path", "Путь")) }
                                th { (t(locale, "Characters", "Символы")) }
                                th { (t(locale, "Tokens", "Токены")) }
                            }
                        }
                        tbody {
                            @for file in top_files {
                                tr {
                                    td { (&file.path) }
                                    td { (file.char_count) }
                                    td { (file.token_count) }
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

fn t(locale: Locale, en: &'static str, ru: &'static str) -> &'static str {
    match locale {
        Locale::En => en,
        Locale::Ru => ru,
    }
}
