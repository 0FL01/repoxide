use axum::{
    extract::{Multipart, Path, State},
    http::{
        header::{CONTENT_TYPE, HOST, ORIGIN, REFERER},
        HeaderMap, StatusCode,
    },
    response::{Html, IntoResponse, Response},
};
use std::{path::Component, path::PathBuf, sync::Arc};
use uuid::Uuid;

use crate::{
    state::AppState,
    views::home::{
        normalize_optional_text, parse_bool_field, render_page, render_response_root, Locale,
        SourceKind, WebFormState, RESPONSE_FRAGMENT_HEADER,
    },
};

use super::pack::{build_folder_upload, execute_pack_job, PackJob, PackSource};

const APP_CSS: &str = include_str!("../../static/repoxide-home.css");
const APP_JS: &str = include_str!("../../static/repoxide-home.js");
const REPOXIDE_LOGO_SVG: &str = include_str!("../../../client/src/public/images/repoxide-logo.svg");

pub async fn index() -> Html<String> {
    Html(render_page(Locale::En, &WebFormState::new(), None, None).into_string())
}

pub async fn index_ru() -> Html<String> {
    Html(render_page(Locale::Ru, &WebFormState::new(), None, None).into_string())
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

pub async fn home_css() -> Response {
    ([(CONTENT_TYPE, "text/css; charset=utf-8")], APP_CSS).into_response()
}

pub async fn home_js() -> Response {
    (
        [(CONTENT_TYPE, "application/javascript; charset=utf-8")],
        APP_JS,
    )
        .into_response()
}

pub async fn repoxide_logo_svg() -> Response {
    (
        [(CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
        REPOXIDE_LOGO_SVG,
    )
        .into_response()
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
    let wants_fragment = headers
        .get(RESPONSE_FRAGMENT_HEADER)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"));

    if !is_same_origin_form_request(&headers) {
        let form = WebFormState::new();
        return render_response(
            StatusCode::FORBIDDEN,
            Locale::En,
            &form,
            None,
            Some("Cross-origin form submissions are not allowed. Use the same-origin web UI or /api/pack."),
            wants_fragment,
        );
    }

    let (locale, form, job) = match parse_web_form(multipart).await {
        Ok(parsed) => parsed,
        Err((locale, form, message, status)) => {
            return render_response(status, locale, &form, None, Some(&message), wants_fragment);
        }
    };

    match execute_pack_job(state, job).await {
        Ok(result) => render_response(
            StatusCode::OK,
            locale,
            &form,
            Some(&result),
            None,
            wants_fragment,
        ),
        Err(err) => render_response(
            err.status,
            locale,
            &form,
            None,
            Some(&err.message),
            wants_fragment,
        ),
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
    std::env::var_os("REPOXIDE_SCHEMA_DIR")
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
    let mut upload_id: Option<String> = None;
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
            "uploadId" => {
                upload_id = Some(field.text().await.map_err(|e| {
                    (
                        locale,
                        form.clone(),
                        format!(
                            "{}: {e}",
                            t(
                                locale,
                                "Failed to read uploadId",
                                "Не удалось прочитать uploadId"
                            )
                        ),
                        StatusCode::BAD_REQUEST,
                    )
                })?);
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
            if let Some(raw_upload_id) = upload_id {
                let upload_id = Uuid::parse_str(raw_upload_id.trim()).map_err(|_| {
                    (
                        locale,
                        form.clone(),
                        t(
                            locale,
                            "Invalid uploadId format",
                            "Некорректный формат uploadId",
                        )
                        .to_string(),
                        StatusCode::BAD_REQUEST,
                    )
                })?;
                PackSource::ChunkedUpload { upload_id }
            } else {
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
        }
        SourceKind::Folder => {
            let files = build_folder_upload(folder_manifest, folder_files)
                .map_err(|err| (locale, form.clone(), err.message, err.status))?;
            PackSource::FolderUpload { files }
        }
    };

    Ok((
        locale,
        form.clone(),
        PackJob {
            format: form.format.clone(),
            options: form.options.clone(),
            source,
        },
    ))
}

fn render_response(
    status: StatusCode,
    locale: Locale,
    form: &WebFormState,
    result: Option<&crate::types::PackResponse>,
    error: Option<&str>,
    wants_fragment: bool,
) -> Response {
    if wants_fragment {
        return (
            status,
            Html(render_response_root(locale, result, error).into_string()),
        )
            .into_response();
    }

    (
        status,
        Html(render_page(locale, form, result, error).into_string()),
    )
        .into_response()
}

fn t(locale: Locale, en: &'static str, ru: &'static str) -> &'static str {
    crate::views::home::t(locale, en, ru)
}
