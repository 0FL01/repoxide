use chrono::{DateTime, Utc};
use maud::{html, Markup, DOCTYPE};

use crate::types::{PackOptions, PackResponse};

use super::components;

pub(crate) const RESPONSE_FRAGMENT_HEADER: &str = "x-repomix-response-fragment";

#[derive(Debug, Clone)]
pub(crate) struct WebFormState {
    pub(crate) source_kind: SourceKind,
    pub(crate) url: String,
    pub(crate) format: String,
    pub(crate) options: PackOptions,
}

impl WebFormState {
    pub(crate) fn new() -> Self {
        Self {
            source_kind: SourceKind::Url,
            url: String::new(),
            format: "xml".to_string(),
            options: PackOptions::default(),
        }
    }
}

impl Default for WebFormState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum Locale {
    En,
    Ru,
}

impl Locale {
    pub(crate) fn from_value(value: &str) -> Self {
        if value.eq_ignore_ascii_case("ru") {
            Self::Ru
        } else {
            Self::En
        }
    }

    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ru => "ru",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum SourceKind {
    Url,
    Folder,
    Zip,
}

impl SourceKind {
    pub(crate) fn from_value(value: &str) -> Option<Self> {
        match value {
            "url" => Some(Self::Url),
            "folder" => Some(Self::Folder),
            "zip" => Some(Self::Zip),
            _ => None,
        }
    }

    pub(crate) fn value(self) -> &'static str {
        match self {
            Self::Url => "url",
            Self::Folder => "folder",
            Self::Zip => "zip",
        }
    }
}

pub(crate) fn render_page(
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
                title { "Repomix" }
                meta
                    name="description"
                    content=(t(
                        locale,
                        "Pack your codebase into AI-friendly formats.",
                        "Упакуйте ваш код в AI-friendly формат.",
                    ));
                link rel="icon" type="image/svg+xml" href="/images/repomix-logo.svg";
                link rel="mask-icon" href="/images/repomix-logo.svg" color="#f97316";
                link rel="stylesheet" href="/static/repomix-home.css";
                script defer src="/static/repomix-home.js" {}
            }
            body data-locale=(locale.code()) data-has-result=(if result.is_some() || error.is_some() { "true" } else { "false" }) {
                main class="home-page" {
                    (components::home::render_home_page(locale, form, result, error))
                }
                template id="home-loading-template" {
                    (components::home::render_loading(locale))
                }
            }
        }
    }
}

pub(crate) fn render_response_root(
    locale: Locale,
    result: Option<&PackResponse>,
    error: Option<&str>,
) -> Markup {
    components::home::render_response_root(locale, result, error)
}

pub(crate) fn has_non_default_form_state(form: &WebFormState) -> bool {
    form.source_kind != SourceKind::Url
        || !form.url.trim().is_empty()
        || form.format != "xml"
        || form.options.remove_comments
        || form.options.remove_empty_lines
        || form.options.show_line_numbers
        || !form.options.file_summary
        || !form.options.directory_structure
        || form.options.output_parsable
        || form.options.compress
        || form
            .options
            .include_patterns
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        || form
            .options
            .ignore_patterns
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
}

pub(crate) fn format_timestamp(value: &str) -> String {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| {
            timestamp
                .with_timezone(&Utc)
                .format("%Y-%m-%d %H:%M UTC")
                .to_string()
        })
        .unwrap_or_else(|_| value.to_string())
}

pub(crate) fn normalize_optional_text(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub(crate) fn parse_bool_field(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "yes" | "on"
    )
}

pub(crate) fn download_file_name(response: &PackResponse) -> String {
    match response.format.as_str() {
        "xml" => "repomix-output.xml",
        "markdown" => "repomix-output.md",
        _ => "repomix-output.txt",
    }
    .to_string()
}

pub(crate) fn download_content_type(format: &str) -> &'static str {
    match format {
        "xml" => "application/xml;charset=utf-8",
        "markdown" => "text/markdown;charset=utf-8",
        _ => "text/plain;charset=utf-8",
    }
}

pub(crate) fn format_number(n: usize) -> String {
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

pub(crate) fn format_percentage(numerator: usize, denominator: usize) -> String {
    if denominator == 0 {
        "0.0%".to_string()
    } else {
        format!("{:.1}%", (numerator as f64 / denominator as f64) * 100.0)
    }
}

pub(crate) fn t(locale: Locale, en: &'static str, ru: &'static str) -> &'static str {
    match locale {
        Locale::En => en,
        Locale::Ru => ru,
    }
}
