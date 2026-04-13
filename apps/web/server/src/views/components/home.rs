use maud::{html, Markup, PreEscaped};

use crate::types::{FileInfo, PackResponse};

use super::super::home::{
    download_content_type, download_file_name, format_number, format_percentage, format_timestamp,
    has_non_default_form_state, t, Locale, SourceKind, WebFormState,
};

pub(crate) fn render_home_page(
    locale: Locale,
    form: &WebFormState,
    result: Option<&PackResponse>,
    error: Option<&str>,
) -> Markup {
    html! {
        section class="home-hero" {
            div class="home-hero__topbar" {
                (render_theme_switcher(locale))
            }
            h1 class="home-hero__title" { "Repomix" }
            p class="home-hero__description" {
                (t(locale, "Pack your codebase into ", "Упакуйте ваш код в "))
                span class="home-hero__description-accent" { "AI-friendly" }
                (t(locale, " formats", " формат"))
            }
        }

        section class="home-shell" {
            form
                class="try-it-container"
                method="post"
                action="/pack"
                enctype="multipart/form-data"
                novalidate
                data-home-form
                data-locale=(locale.code())
            {
                input type="hidden" name="locale" value=(locale.code());
                input
                    type="hidden"
                    name="sourceKind"
                    value=(form.source_kind.value())
                    data-source-kind-input;
                input type="hidden" name="format" value=(form.format.as_str()) data-format-input;

                div class="input-row" {
                    div class="tab-container" role="tablist" aria-label=(t(locale, "Source mode", "Режим источника")) {
                        (render_mode_tab(form.source_kind, SourceKind::Url, t(locale, "Repository URL", "URL репозитория"), LINK_ICON))
                        (render_mode_tab(form.source_kind, SourceKind::Folder, t(locale, "Folder upload", "Загрузка папки"), FOLDER_OPEN_ICON))
                        (render_mode_tab(form.source_kind, SourceKind::Zip, t(locale, "ZIP upload", "Загрузка ZIP"), FOLDER_ARCHIVE_ICON))
                    }

                    div class="input-field" {
                        (render_url_source_panel(locale, form))
                        (render_folder_source_panel(locale, form.source_kind == SourceKind::Folder))
                        (render_zip_source_panel(locale, form.source_kind == SourceKind::Zip))
                    }

                    div class="pack-button-wrapper" {
                        button
                            type="submit"
                            class="pack-button"
                            data-pack-button
                            aria-label=(t(locale, "Pack repository", "Упаковать репозиторий"))
                        {
                            span class="pack-button__text pack-button__text--normal" data-pack-button-label {
                                (t(locale, "Pack", "Pack"))
                            }
                            span class="pack-button__text pack-button__text--hover" {
                                (t(locale, "Cancel", "Отмена"))
                            }
                            span class="pack-button__icon" data-pack-button-icon {
                                (icon(PACK_ICON))
                            }
                        }

                        div
                            class=(if has_non_default_form_state(form) { "tooltip-container" } else { "tooltip-container is-hidden" })
                            data-reset-tooltip
                        {
                            button
                                class="reset-button"
                                type="button"
                                data-reset-button
                                aria-label=(t(locale, "Reset all options", "Сбросить все настройки"))
                            {
                                (icon(ROTATE_CCW_ICON))
                            }
                            div class="tooltip-content" {
                                (t(
                                    locale,
                                    "Reset all options to default values",
                                    "Сбросить все параметры к значениям по умолчанию",
                                ))
                                div class="tooltip-arrow" {}
                            }
                        }
                    }
                }

                (render_pack_options(locale, form))

                div id="home-response-root" class="response-root" data-response-root {
                    (render_response_root(locale, result, error))
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
    if let Some(error) = error {
        render_error(locale, error)
    } else if let Some(result) = result {
        render_result(locale, result)
    } else {
        html! {}
    }
}

pub(crate) fn render_loading(locale: Locale) -> Markup {
    html! {
        div class="result-viewer result-viewer--loading" {
            div class="loading-state" {
                span class="loading-spinner" aria-hidden="true" {}
                p class="loading-title" { (t(locale, "Processing repository...", "Обработка репозитория...")) }
            }
        }
    }
}

fn render_theme_switcher(locale: Locale) -> Markup {
    html! {
        div class="theme-switcher" data-theme-switcher {
            span class="theme-switcher__label" {
                (t(locale, "Theme", "Тема"))
            }
            div class="theme-toggle" role="group" aria-label=(t(locale, "Color theme", "Цветовая тема")) {
                (render_theme_toggle_button("system", t(locale, "System", "Системная"), true))
                (render_theme_toggle_button("light", t(locale, "Light", "Светлая"), false))
                (render_theme_toggle_button("dark", t(locale, "Dark", "Тёмная"), false))
            }
        }
    }
}

fn render_theme_toggle_button(value: &'static str, label: &'static str, active: bool) -> Markup {
    html! {
        button
            type="button"
            class=(if active { "theme-toggle__button is-active" } else { "theme-toggle__button" })
            data-theme-option=(value)
            aria-pressed=(if active { "true" } else { "false" })
            title=(label)
        {
            (label)
        }
    }
}

fn render_mode_tab(
    active: SourceKind,
    kind: SourceKind,
    label: &'static str,
    icon_svg: &'static str,
) -> Markup {
    html! {
        button
            type="button"
            class=(if active == kind { "mode-tab is-active" } else { "mode-tab" })
            data-mode-tab=(kind.value())
            role="tab"
            aria-selected=(if active == kind { "true" } else { "false" })
            aria-label=(label)
            title=(label)
        {
            span class="mode-tab__icon" aria-hidden="true" { (icon(icon_svg)) }
            span class="sr-only" { (label) }
        }
    }
}

fn render_url_source_panel(locale: Locale, form: &WebFormState) -> Markup {
    let is_active = form.source_kind == SourceKind::Url;

    html! {
        div
            class=(if is_active { "source-panel is-active" } else { "source-panel" })
            data-mode-panel="url"
            aria-hidden=(if is_active { "false" } else { "true" })
        {
            div class="input-group" {
                input
                    type="text"
                    class="repository-input"
                    name="url"
                    value=(form.url.as_str())
                    placeholder="GitHub repository URL or user/repo (e.g., yamadashy/repomix)"
                    autocomplete="on"
                    aria-label=(t(locale, "GitHub repository URL", "URL репозитория GitHub"))
                    data-url-input;
                div class="url-warning is-hidden" data-url-warning {
                    span class="warning-icon" aria-hidden="true" { (icon(ALERT_TRIANGLE_ICON)) }
                    span {
                        (t(
                            locale,
                            "Please enter a valid GitHub repository URL (e.g., yamadashy/repomix)",
                            "Введите корректный URL репозитория GitHub (например, yamadashy/repomix)",
                        ))
                    }
                }
            }
        }
    }
}

fn render_zip_source_panel(locale: Locale, is_active: bool) -> Markup {
    render_upload_source_panel(
        locale,
        is_active,
        SourceKind::Zip,
        t(
            locale,
            "Drop your ZIP file here or click to browse (max 50MB)",
            "Перетащите ZIP-файл или нажмите для выбора (макс. 50 МБ)",
        ),
        t(locale, "Selected:", "Выбрано:"),
        FOLDER_ARCHIVE_ICON,
        "zip",
        false,
        ".zip",
    )
}

fn render_folder_source_panel(locale: Locale, is_active: bool) -> Markup {
    render_upload_source_panel(
        locale,
        is_active,
        SourceKind::Folder,
        t(
            locale,
            "Drop your folder here or click to browse (max 50MB)",
            "Перетащите папку или нажмите для выбора (макс. 50 МБ)",
        ),
        t(locale, "Selected:", "Выбрано:"),
        FOLDER_OPEN_ICON,
        "folder",
        true,
        "",
    )
}

fn render_upload_source_panel(
    locale: Locale,
    is_active: bool,
    kind: SourceKind,
    placeholder: &'static str,
    selected_prefix: &'static str,
    icon_svg: &'static str,
    input_kind: &'static str,
    is_directory: bool,
    accept: &'static str,
) -> Markup {
    html! {
        div
            class=(if is_active { "source-panel is-active" } else { "source-panel" })
            data-mode-panel=(kind.value())
            aria-hidden=(if is_active { "false" } else { "true" })
        {
            div
                class="upload-container"
                data-upload-zone=(input_kind)
                tabindex="0"
                role="button"
                aria-label=(placeholder)
            {
                input
                    type="file"
                    class="hidden-input"
                    data-upload-input=(input_kind)
                    accept=(accept)
                    multiple[is_directory]
                    webkitdirectory[is_directory];
                div class="upload-content" {
                    span class="upload-icon" aria-hidden="true" { (icon(icon_svg)) }
                    div class="upload-text" {
                        p class="upload-placeholder" data-upload-placeholder { (placeholder) }
                        p class="upload-selection is-hidden" data-upload-selection {
                            span class="upload-selection__prefix" { (selected_prefix) " " }
                            span data-upload-selection-name {}
                            button
                                type="button"
                                class="clear-button"
                                data-clear-selection=(input_kind)
                                aria-label=(t(locale, "Clear selection", "Очистить выбор"))
                            { "×" }
                        }
                        p class="upload-error is-hidden" data-upload-error {}
                    }
                }
            }
        }
    }
}

fn render_pack_options(locale: Locale, form: &WebFormState) -> Markup {
    let include_patterns = form.options.include_patterns.as_deref().unwrap_or("");
    let ignore_patterns = form.options.ignore_patterns.as_deref().unwrap_or("");

    html! {
        div class="options-container" {
            div class="left-column" {
                div class="option-section" {
                    p class="option-label" { (t(locale, "Output Format", "Формат вывода")) }
                    div class="format-buttons" role="radiogroup" aria-label=(t(locale, "Output format", "Формат вывода")) {
                        (render_format_button(form, "xml", "XML"))
                        (render_format_button(form, "markdown", "Markdown"))
                        (render_format_button(form, "plain", "Plain"))
                    }
                }

                div class="option-section" {
                    p class="option-label" {
                        (t(locale, "Include Patterns (using glob patterns)", "Include-паттерны (с glob-шаблонами)"))
                    }
                    input
                        type="text"
                        class="pattern-input"
                        name="includePatterns"
                        value=(include_patterns)
                        placeholder=(t(locale, "Comma-separated patterns to include. e.g., src/**/*.ts", "Паттерны через запятую для включения. Например, src/**/*.ts"))
                        aria-label=(t(locale, "Include patterns", "Include-паттерны"))
                        data-include-input;
                }

                div class="option-section" {
                    p class="option-label" { (t(locale, "Ignore Patterns", "Ignore-паттерны")) }
                    input
                        type="text"
                        class="pattern-input"
                        name="ignorePatterns"
                        value=(ignore_patterns)
                        placeholder=(t(locale, "Comma-separated patterns to ignore. e.g., **/*.test.ts,README.md", "Паттерны через запятую для исключения. Например, **/*.test.ts,README.md"))
                        aria-label=(t(locale, "Ignore patterns", "Ignore-паттерны"))
                        data-ignore-input;
                }
            }

            div class="right-column" {
                div class="option-section" {
                    p class="option-label" { (t(locale, "Output Format Options", "Параметры формата вывода")) }
                    div class="checkbox-group" {
                        (render_checkbox("fileSummary", form.options.file_summary, t(locale, "Include File Summary", "Добавить сводку по файлам"), None))
                        (render_checkbox("directoryStructure", form.options.directory_structure, t(locale, "Include Directory Structure", "Добавить структуру директорий"), None))
                        (render_checkbox("showLineNumbers", form.options.show_line_numbers, t(locale, "Show Line Numbers", "Показывать номера строк"), None))
                        (render_checkbox("outputParsable", form.options.output_parsable, t(locale, "Output Parsable Format", "Вывод в парсируемом формате"), Some(t(locale, "Whether to escape the output based on the chosen style schema. Note that this can increase token count.", "Экранировать вывод в соответствии с выбранной схемой. Это может увеличить число токенов."))))
                    }
                }

                div class="option-section" {
                    p class="option-label" { (t(locale, "File Processing Options", "Параметры обработки файлов")) }
                    div class="checkbox-group" {
                        (render_checkbox("compress", form.options.compress, t(locale, "Compress Code", "Сжать код"), Some(t(locale, "Utilize Tree-sitter to extract essential code signatures and structure while removing implementation details, significantly reducing token usage.", "Использовать Tree-sitter для извлечения сигнатур и структуры кода без деталей реализации, чтобы существенно снизить число токенов."))))
                        (render_checkbox("removeComments", form.options.remove_comments, t(locale, "Remove Comments", "Удалить комментарии"), None))
                        (render_checkbox("removeEmptyLines", form.options.remove_empty_lines, t(locale, "Remove Empty Lines", "Удалить пустые строки"), None))
                    }
                }
            }
        }
    }
}

fn render_format_button(form: &WebFormState, value: &'static str, label: &'static str) -> Markup {
    html! {
        button
            type="button"
            class=(if form.format == value { "format-button is-active" } else { "format-button" })
            data-format-button=(value)
            aria-pressed=(if form.format == value { "true" } else { "false" })
        {
            (label)
        }
    }
}

fn render_checkbox(
    name: &'static str,
    checked: bool,
    label: &'static str,
    tooltip: Option<&'static str>,
) -> Markup {
    html! {
        label class="checkbox-label" {
            input
                type="checkbox"
                class="checkbox-input"
                name=(name)
                value="true"
                checked[checked]
                data-option-checkbox;
            @if let Some(tooltip_text) = tooltip {
                div class="option-with-tooltip" {
                    span { (label) }
                    div class="tooltip-container" {
                        span class="help-icon" aria-hidden="true" { (icon(HELP_CIRCLE_ICON)) }
                        div class="tooltip-content tooltip-content--wide" {
                            (tooltip_text)
                            div class="tooltip-arrow" {}
                        }
                    }
                }
            } @else {
                span { (label) }
            }
        }
    }
}

fn render_error(locale: Locale, error: &str) -> Markup {
    html! {
        div class="result-viewer" {
            div class="result-error" {
                div class="result-error__icon" aria-hidden="true" { (icon(ALERT_TRIANGLE_ICON)) }
                h2 class="result-error__title" { (t(locale, "Unable to pack repository", "Не удалось упаковать репозиторий")) }
                p class="result-error__message" { (error) }
            }
        }
    }
}

fn render_result(locale: Locale, response: &PackResponse) -> Markup {
    let has_file_selection = response
        .metadata
        .all_files
        .as_ref()
        .is_some_and(|files| !files.is_empty());

    html! {
        div class="result-viewer" data-result-viewer {
            @if has_file_selection {
                div class="tab-navigation" role="tablist" aria-label=(t(locale, "Result views", "Вкладки результата")) {
                    button
                        type="button"
                        class="tab-button is-active"
                        data-result-tab="result"
                        aria-selected="true"
                    {
                        (t(locale, "Result", "Результат"))
                    }
                    button
                        type="button"
                        class="tab-button"
                        data-result-tab="files"
                        aria-selected="false"
                    {
                        (t(locale, "File Selection", "Выбор файлов"))
                    }
                }
            }

            div class="result-panel is-active" data-result-panel="result" {
                (render_result_content(locale, response))
            }

            @if let Some(all_files) = response.metadata.all_files.as_ref().filter(|files| !files.is_empty()) {
                div class="result-panel" data-result-panel="files" {
                    (render_file_selection(locale, all_files))
                }
            }
        }
    }
}

fn render_result_content(locale: Locale, response: &PackResponse) -> Markup {
    let formatted_timestamp = format_timestamp(&response.metadata.timestamp);

    html! {
        div class="result-content" {
            aside class="metadata-panel" {
                div class="metadata-section" {
                    h3 {
                        span class="metadata-section__icon" aria-hidden="true" { (icon(GIT_FORK_ICON)) }
                        (t(locale, "Repository Info", "Информация о репозитории"))
                    }
                    dl {
                        dt { (t(locale, "Repository", "Репозиторий")) }
                        dd { (&response.metadata.repository) }
                        dt { (t(locale, "Generated At", "Сгенерировано")) }
                        dd { (formatted_timestamp) }
                        dt { (t(locale, "Format", "Формат")) }
                        dd { (&response.format) }
                    }
                }

                @if let Some(summary) = &response.metadata.summary {
                    div class="metadata-section" {
                        h3 {
                            span class="metadata-section__icon" aria-hidden="true" { (icon(PACKAGE_SEARCH_ICON)) }
                            (t(locale, "Pack Summary", "Сводка pack"))
                        }
                        dl {
                            dt { (t(locale, "Total Files", "Всего файлов")) }
                            dd { (format_number(summary.total_files)) " " span class="unit" { (t(locale, "files", "файлов")) } }
                            dt { (t(locale, "Total Tokens", "Всего токенов")) }
                            dd { (format_number(summary.total_tokens)) " " span class="unit" { (t(locale, "tokens", "токенов")) } }
                            dt { (t(locale, "Total Size", "Общий размер")) }
                            dd { (format_number(summary.total_characters)) " " span class="unit" { (t(locale, "chars", "символов")) } }
                        }
                    }
                }

                @if let (Some(summary), Some(top_files)) = (&response.metadata.summary, &response.metadata.top_files) {
                    div class="metadata-section" {
                        h3 {
                            span class="metadata-section__icon" aria-hidden="true" { (icon(BAR_CHART_ICON)) }
                            (t(locale, "Top Files", "Топ файлов"))
                        }
                        ol class="top-files-list" {
                            @for file in top_files {
                                li {
                                    div class="top-files-list__path" { (&file.path) }
                                    div class="top-files-list__stats" {
                                        (format_number(file.token_count)) " " span class="unit" { (t(locale, "tokens", "токенов")) }
                                        span class="separator-unit" { "|" }
                                        (format_number(file.char_count)) " " span class="unit" { (t(locale, "chars", "символов")) }
                                        span class="separator-unit" { "|" }
                                        (format_percentage(file.token_count, summary.total_tokens))
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div class="output-panel" {
                div class="output-actions" {
                    button
                        type="button"
                        class="action-button"
                        data-copy-output
                        aria-label=(t(locale, "Copy output", "Скопировать вывод"))
                    {
                        span class="action-button__icon" aria-hidden="true" { (icon(COPY_ICON)) }
                        span data-copy-label data-default=(t(locale, "Copy", "Копировать")) data-success=(t(locale, "Copied!", "Скопировано!")) {
                            (t(locale, "Copy", "Копировать"))
                        }
                    }
                    button
                        type="button"
                        class="action-button"
                        data-download-output
                        data-download-name=(download_file_name(response))
                        data-download-type=(download_content_type(&response.format))
                        aria-label=(t(locale, "Download output", "Скачать вывод"))
                    {
                        span class="action-button__icon" aria-hidden="true" { (icon(DOWNLOAD_ICON)) }
                        span { (t(locale, "Download", "Скачать")) }
                    }
                }

                div class="output-viewer" {
                    pre class="output-code" data-result-output { (&response.content) }
                }
            }
        }
    }
}

fn render_file_selection(locale: Locale, files: &[FileInfo]) -> Markup {
    let total_tokens = files.iter().map(|file| file.token_count).sum::<usize>();

    html! {
        div class="file-selection-container" data-file-selection {
            div class="file-selection-header" {
                h3 class="file-selection-title" {
                    span class="file-selection-title__icon" aria-hidden="true" { (icon(FILE_TEXT_ICON)) }
                    (t(locale, "File Selection", "Выбор файлов"))
                }
                div class="file-selection-actions" {
                    button type="button" class="action-button action-button--secondary" data-select-all {
                        (t(locale, "Select All", "Выбрать все"))
                    }
                    button type="button" class="action-button action-button--secondary" data-deselect-all {
                        (t(locale, "Deselect All", "Снять выбор"))
                    }
                    button type="button" class="action-button action-button--primary" data-repack-selected {
                        span class="action-button__label" data-repack-label data-default=(t(locale, "Re-pack Selected", "Перепаковать выбранное")) data-loading=(t(locale, "Re-packing...", "Перепаковка...")) {
                            (t(locale, "Re-pack Selected", "Перепаковать выбранное"))
                        }
                    }
                }
            }

            div
                class="file-selection-stats"
                data-total-count=(files.len())
                data-total-tokens=(total_tokens)
            {
                span class="stat-item" {
                    span data-selected-count { (files.len()) }
                    " " (t(locale, "of", "из")) " "
                    span data-total-count-label { (files.len()) }
                    " " (t(locale, "files selected", "файлов выбрано"))
                }
                span class="stat-separator" { "|" }
                span class="stat-item" {
                    span data-selected-tokens { (format_number(total_tokens)) }
                    " " (t(locale, "tokens", "токенов"))
                    " ("
                    span data-selected-percent { "100.0%" }
                    ")"
                }
            }

            div class="file-list-container" {
                table class="file-table" aria-label=(t(locale, "File selection table", "Таблица выбора файлов")) {
                    thead {
                        tr {
                            th class="checkbox-column" {
                                input type="checkbox" class="header-checkbox" checked data-file-master-toggle aria-label=(t(locale, "Select or deselect all files", "Выбрать или снять выбор со всех файлов"));
                            }
                            th class="file-path-column" { (t(locale, "File Path", "Путь к файлу")) }
                            th class="tokens-column" { (t(locale, "Tokens", "Токены")) }
                        }
                    }
                    tbody {
                        @for file in files {
                            tr class="file-row file-row-selected" data-file-row {
                                td class="checkbox-cell" {
                                    input
                                        type="checkbox"
                                        class="file-checkbox"
                                        checked
                                        data-file-checkbox
                                        data-file-path=(file.path.as_str())
                                        data-token-count=(file.token_count)
                                        aria-label=(file.path.as_str());
                                }
                                td class="file-path-cell" {
                                    span class="file-path" { (&file.path) }
                                }
                                td class="tokens-cell" {
                                    span class="file-tokens" { (format_number(file.token_count)) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn icon(svg: &'static str) -> Markup {
    html! { (PreEscaped(svg)) }
}

const LINK_ICON: &str = r#"<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'><path d='M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07L11.8 5'/><path d='M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07L12.2 19'/></svg>"#;
const FOLDER_OPEN_ICON: &str = r#"<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'><path d='M6 14h8l2-2h6'/><path d='M4 20a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2v1'/></svg>"#;
const FOLDER_ARCHIVE_ICON: &str = r#"<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'><path d='M20 7V5a2 2 0 0 0-2-2H6l-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2'/><path d='M12 12v6'/><path d='M9 15h6'/><path d='M10 7h4'/></svg>"#;
const ROTATE_CCW_ICON: &str = r#"<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'><path d='M3 2v6h6'/><path d='M3 8a9 9 0 1 0 3-6.7L3 4'/></svg>"#;
const PACK_ICON: &str = r#"<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'><path d='m16 16 2 2 4-4'/><path d='M21 12V7a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 7v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l2-1.14'/><path d='m7.5 4.27 9 5.15'/><polyline points='3.29 6.91 12 12.01 20.71 6.91'/><line x1='12' y1='22.08' x2='12' y2='12'/></svg>"#;
const ALERT_TRIANGLE_ICON: &str = r#"<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'><path d='m10.29 3.86-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.71-3.14l-8-14a2 2 0 0 0-3.42 0Z'/><line x1='12' y1='9' x2='12' y2='13'/><line x1='12' y1='17' x2='12.01' y2='17'/></svg>"#;
const HELP_CIRCLE_ICON: &str = r#"<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'><circle cx='12' cy='12' r='10'/><path d='M9.09 9a3 3 0 1 1 5.82 1c0 2-3 3-3 3'/><path d='M12 17h.01'/></svg>"#;
const COPY_ICON: &str = r#"<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'><rect x='9' y='9' width='13' height='13' rx='2' ry='2'/><path d='M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1'/></svg>"#;
const DOWNLOAD_ICON: &str = r#"<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'><path d='M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4'/><polyline points='7 10 12 15 17 10'/><line x1='12' y1='15' x2='12' y2='3'/></svg>"#;
const GIT_FORK_ICON: &str = r#"<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'><circle cx='12' cy='18' r='3'/><circle cx='6' cy='6' r='3'/><circle cx='18' cy='6' r='3'/><path d='M18 9a9 9 0 0 1-9 9'/><path d='M6 9a9 9 0 0 0 9 9'/></svg>"#;
const PACKAGE_SEARCH_ICON: &str = r#"<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'><path d='m16 16 5 5'/><circle cx='11.5' cy='11.5' r='5.5'/><path d='M7.5 5.5 12 3l4.5 2.5v5L12 13 7.5 10.5z'/></svg>"#;
const BAR_CHART_ICON: &str = r#"<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'><line x1='12' y1='20' x2='12' y2='10'/><line x1='18' y1='20' x2='18' y2='4'/><line x1='6' y1='20' x2='6' y2='16'/></svg>"#;
const FILE_TEXT_ICON: &str = r#"<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'><path d='M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z'/><path d='M14 2v6h6'/><path d='M16 13H8'/><path d='M16 17H8'/><path d='M10 9H8'/></svg>"#;
