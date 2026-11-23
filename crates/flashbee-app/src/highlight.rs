use gpui::Hsla;

use crate::theme::Theme;

#[derive(Clone, Debug)]
pub struct Span {
    pub text: String,
    pub color: Hsla,
    pub italic: bool,
}

impl Span {
    fn new(text: impl Into<String>, color: Hsla, italic: bool) -> Self {
        Self {
            text: text.into(),
            color,
            italic,
        }
    }
}

fn keywords(language: &str) -> &'static [&'static str] {
    match language {
        "rust" => &[
            "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
            "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
            "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super",
            "trait", "true", "type", "unsafe", "use", "where", "while", "yield",
        ],
        "typescript" | "tsx" | "javascript" | "jsx" => &[
            "as",
            "async",
            "await",
            "break",
            "case",
            "catch",
            "class",
            "const",
            "continue",
            "debugger",
            "default",
            "delete",
            "do",
            "else",
            "export",
            "extends",
            "false",
            "finally",
            "for",
            "from",
            "function",
            "get",
            "if",
            "implements",
            "import",
            "in",
            "instanceof",
            "interface",
            "let",
            "new",
            "null",
            "of",
            "private",
            "protected",
            "public",
            "readonly",
            "return",
            "set",
            "static",
            "super",
            "switch",
            "this",
            "throw",
            "true",
            "try",
            "type",
            "typeof",
            "undefined",
            "var",
            "void",
            "while",
            "with",
            "yield",
        ],
        "python" => &[
            "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del",
            "elif", "else", "except", "False", "finally", "for", "from", "global", "if", "import",
            "in", "is", "lambda", "None", "nonlocal", "not", "or", "pass", "raise", "return",
            "True", "try", "while", "with", "yield",
        ],
        "go" => &[
            "break",
            "case",
            "chan",
            "const",
            "continue",
            "default",
            "defer",
            "else",
            "fallthrough",
            "for",
            "func",
            "go",
            "goto",
            "if",
            "import",
            "interface",
            "map",
            "package",
            "range",
            "return",
            "select",
            "struct",
            "switch",
            "type",
            "var",
        ],
        "css" | "scss" | "less" => &["important", "media", "supports", "keyframes", "from", "to"],
        "bash" | "sh" | "zsh" => &[
            "case", "do", "done", "elif", "else", "esac", "fi", "for", "function", "if", "in",
            "select", "then", "until", "while", "export", "local", "return",
        ],
        "sql" => &[
            "select", "from", "where", "join", "left", "right", "inner", "outer", "on", "group",
            "by", "order", "having", "limit", "insert", "into", "values", "update", "set",
            "delete", "create", "table", "drop", "alter", "and", "or", "not", "null", "as",
        ],
        "toml" => &["true", "false"],
        "yaml" => &["true", "false", "null", "yes", "no"],
        _ => &[],
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_' || ch == '$'
}

fn is_ident(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '$'
}

fn line_comment(language: &str) -> Option<&'static str> {
    match language {
        "python" | "bash" | "sh" | "zsh" | "toml" | "yaml" | "makefile" | "ruby" => Some("#"),
        "sql" => Some("--"),
        "lua" => Some("--"),
        "html" | "xml" | "vue" | "svelte" => Some("<!--"),
        _ => Some("//"),
    }
}

pub fn highlight_line(line: &str, language: &str, theme: Theme) -> Vec<Span> {
    if language == "markdown" {
        return highlight_markdown(line, theme);
    }

    let chars: Vec<char> = line.chars().collect();
    let mut spans: Vec<Span> = Vec::new();
    let mut i = 0usize;
    let comment_prefix = line_comment(language);
    let keyword_list = keywords(language);
    let json_like = matches!(language, "json" | "jsonc");

    while i < chars.len() {
        let ch = chars[i];

        if ch.is_whitespace() {
            let start = i;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            push(
                &mut spans,
                chars[start..i].iter().collect(),
                theme.text,
                false,
            );
            continue;
        }

        if let Some(prefix) = comment_prefix {
            let remaining: String = chars[i..].iter().collect();
            if remaining.starts_with(prefix)
                && (prefix == "<!--" || i == 0 || !chars[i - 1].is_alphanumeric())
            {
                push(&mut spans, remaining, theme.dim, true);
                break;
            }
        }

        if ch == '"' || ch == '\'' || ch == '`' {
            let quote = ch;
            let start = i;
            i += 1;
            let mut escaped = false;
            while i < chars.len() {
                let current = chars[i];
                i += 1;
                if escaped {
                    escaped = false;
                    continue;
                }
                if current == '\\' {
                    escaped = true;
                    continue;
                }
                if current == quote {
                    break;
                }
            }
            let text: String = chars[start..i].iter().collect();
            let mut color = theme.add;
            if json_like || matches!(language, "yaml" | "toml") {
                let mut next = i;
                while next < chars.len() && chars[next].is_whitespace() {
                    next += 1;
                }
                if next < chars.len() && chars[next] == ':' {
                    color = theme.blue;
                }
            }
            push(&mut spans, text, color, false);
            continue;
        }

        if ch.is_ascii_digit() {
            let start = i;
            i += 1;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric() || matches!(chars[i], '.' | '_' | 'x' | 'X'))
            {
                i += 1;
            }
            push(
                &mut spans,
                chars[start..i].iter().collect(),
                theme.amber,
                false,
            );
            continue;
        }

        if is_ident_start(ch) {
            let start = i;
            i += 1;
            while i < chars.len() && is_ident(chars[i]) {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let mut color = theme.text;
            let mut italic = false;
            if keyword_list.contains(&word.as_str()) {
                color = theme.violet;
            } else if word == "true"
                || word == "false"
                || word == "null"
                || word == "None"
                || word == "nil"
            {
                color = theme.violet;
            } else {
                let mut next = i;
                while next < chars.len() && chars[next].is_whitespace() {
                    next += 1;
                }
                if next < chars.len() && chars[next] == '(' {
                    color = theme.blue;
                } else if word
                    .chars()
                    .next()
                    .is_some_and(|value| value.is_uppercase())
                {
                    color = theme.blue;
                }
            }
            if matches!(language, "json" | "yaml" | "toml") && word == "null" {
                italic = true;
            }
            push(&mut spans, word, color, italic);
            continue;
        }

        let color = match ch {
            '{' | '}' | '(' | ')' | '[' | ']' | '<' | '>' | ';' | ',' | ':' | '=' | '+' | '-'
            | '*' | '/' | '%' | '&' | '|' | '!' | '?' | '.' => theme.muted,
            _ => theme.text,
        };
        push(&mut spans, ch.to_string(), color, false);
        i += 1;
    }

    spans
}

fn highlight_markdown(line: &str, theme: Theme) -> Vec<Span> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        return vec![Span::new(line, theme.amber, false)];
    }
    if trimmed.starts_with('>') {
        return vec![Span::new(line, theme.muted, true)];
    }
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        let indent = line.len() - trimmed.len();
        let mut spans = vec![Span::new(&line[..indent], theme.text, false)];
        spans.push(Span::new(&trimmed[..2], theme.amber, false));
        spans.push(Span::new(&trimmed[2..], theme.text, false));
        return spans;
    }
    if trimmed.starts_with("```") {
        return vec![Span::new(line, theme.violet, false)];
    }
    let mut spans = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find('`') {
        if start > 0 {
            spans.push(Span::new(&rest[..start], theme.text, false));
        }
        let after = &rest[start + 1..];
        if let Some(end) = after.find('`') {
            spans.push(Span::new(&rest[start..start + end + 2], theme.add, false));
            rest = &after[end + 1..];
        } else {
            spans.push(Span::new(&rest[start..], theme.text, false));
            rest = "";
            break;
        }
    }
    if !rest.is_empty() {
        spans.push(Span::new(rest, theme.text, false));
    }
    spans
}

fn push(spans: &mut Vec<Span>, text: String, color: Hsla, italic: bool) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = spans.last_mut() {
        if last.color == color && last.italic == italic {
            last.text.push_str(&text);
            return;
        }
    }
    spans.push(Span::new(text, color, italic));
}
