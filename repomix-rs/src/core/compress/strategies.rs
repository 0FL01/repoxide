/// Trait representing a parsing strategy for a specific language family
pub trait LanguageStrategy: Send + Sync {
    /// Extract function/method signature (up to opening brace, arrow, or colon)
    fn extract_signature(&self, lines: &[&str], start_row: usize, end_row: usize) -> Option<String>;

    /// Extract class/interface declaration (just the header, not the body)
    fn extract_declaration(&self, lines: &[&str], start_row: usize, end_row: usize) -> Option<String>;
}

/// Strategy for C-style languages (Rust, C, C++, Java, JS/TS, Go, etc.)
/// Also works for languages that use braces {} or similar structures.
pub struct CStyleStrategy;

impl LanguageStrategy for CStyleStrategy {
    fn extract_signature(&self, lines: &[&str], start_row: usize, end_row: usize) -> Option<String> {
        let mut result_lines: Vec<&str> = Vec::new();

        for i in start_row..=end_row.min(lines.len().saturating_sub(1)) {
            let line = lines[i];
            result_lines.push(line);

            let trimmed = line.trim();

            // Check for function signature end patterns
            if trimmed.contains('{') {
                // Remove everything after the opening brace
                let last_idx = result_lines.len() - 1;
                if let Some(brace_pos) = result_lines[last_idx].find('{') {
                    let mut modified = result_lines[last_idx][..brace_pos].to_string();
                    modified = modified.trim_end().to_string();
                    if !modified.is_empty() {
                        return Some(
                            result_lines[..last_idx]
                                .iter()
                                .chain(std::iter::once(&modified.as_str()))
                                .copied()
                                .collect::<Vec<_>>()
                                .join("\n"),
                        );
                    }
                }
                break;
            }

            // Arrow function end
            if trimmed.ends_with("=>") || trimmed.ends_with("-> {") {
                let last_idx = result_lines.len() - 1;
                let modified = result_lines[last_idx]
                    .replace("=> {", "")
                    .replace("=>", "")
                    .replace("-> {", "")
                    .trim_end()
                    .to_string();
                if !modified.is_empty() {
                    return Some(
                        result_lines[..last_idx]
                            .iter()
                            .chain(std::iter::once(&modified.as_str()))
                            .copied()
                            .collect::<Vec<_>>()
                            .join("\n"),
                    );
                }
                break;
            }

            // Semicolon ends a signature (for abstract methods, type definitions)
            if trimmed.ends_with(';') {
                break;
            }

            // For Rust-like syntax
            if trimmed.ends_with("where") || trimmed.contains("where ") {
                continue; // Include where clauses
            }
        }

        if result_lines.is_empty() {
            return None;
        }

        Some(result_lines.join("\n"))
    }

    fn extract_declaration(&self, lines: &[&str], start_row: usize, end_row: usize) -> Option<String> {
        let mut result_lines: Vec<String> = Vec::new();

        for i in start_row..=end_row.min(lines.len().saturating_sub(1)) {
            let line = lines[i];

            // Check for opening brace
            if line.contains('{') {
                // Take content before the brace
                if let Some(brace_pos) = line.find('{') {
                    let before_brace = line[..brace_pos].trim();
                    if !before_brace.is_empty() {
                        result_lines.push(before_brace.to_string());
                    }
                }
                break;
            }

            result_lines.push(line.to_string());

            // Check for extends/implements on next line
            let trimmed = line.trim();
            if i == start_row
                && i + 1 <= end_row
                && i + 1 < lines.len()
                && (lines[i + 1].trim().starts_with("extends")
                    || lines[i + 1].trim().starts_with("implements")
                    || lines[i + 1].trim().starts_with("where"))
            {
                continue;
            }

            // For languages ending declarations with colon (Python classes) -- kept for compatibility for now
            if trimmed.ends_with(':') {
                break;
            }
        }

        if result_lines.is_empty() {
            return None;
        }

        Some(result_lines.join("\n").trim().to_string())
    }
}

/// Strategy for Python (indentation-based, colon-terminated blocks)
pub struct PythonStrategy;

impl LanguageStrategy for PythonStrategy {
    fn extract_signature(&self, lines: &[&str], start_row: usize, end_row: usize) -> Option<String> {
        let mut result_lines: Vec<&str> = Vec::new();

        for i in start_row..=end_row.min(lines.len().saturating_sub(1)) {
            let line = lines[i];
            result_lines.push(line);

            let trimmed = line.trim();


            // Check for colon that ends the signature
            // We need to handle comments, e.g. "def foo(): # comment"
            // But usually tree-sitter range should cover what we need.
            // Since we are working with raw lines within the range provided by tree-sitter,
            // we look for the line that has a colon at the end (ignoring comments).
            
            // Simple check: does it end with ':'?
            if trimmed.ends_with(':') {
                 return Some(result_lines.join("\n"));
            }

            // Check if it ends with ':' followed by comment
            if let Some(comment_start) = trimmed.find('#') {
                let code_part = trimmed[..comment_start].trim();
                if code_part.ends_with(':') {
                     return Some(result_lines.join("\n"));
                }
            }
        }

        // If we exhausted the range without finding a colon, return what we have
        // (though likely it means the range was incomplete or something else)
        if result_lines.is_empty() {
            None
        } else {
            Some(result_lines.join("\n"))
        }
    }

    fn extract_declaration(&self, lines: &[&str], start_row: usize, end_row: usize) -> Option<String> {
        // For Python classes, it's the same logic as functions: "class Foo(Bar):"
        self.extract_signature(lines, start_row, end_row)
    }
}
