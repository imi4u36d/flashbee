use flashbee_git::{ChangeStatus, DiffHunk, DiffLine, LineType, RepositoryFile};

#[derive(Clone, Copy, Debug, Default)]
pub struct FileStats {
    pub additions: usize,
    pub deletions: usize,
}

impl FileStats {
    pub fn total(self) -> usize {
        self.additions + self.deletions
    }
}

pub fn file_stats(file: &RepositoryFile) -> FileStats {
    let mut stats = FileStats::default();
    for hunk in &file.hunks {
        for line in &hunk.lines {
            match line.kind {
                LineType::Add => stats.additions += 1,
                LineType::Del => stats.deletions += 1,
                LineType::Context => {}
            }
        }
    }
    stats
}

#[derive(Clone)]
pub struct SplitRow {
    pub left: Option<DiffLine>,
    pub right: Option<DiffLine>,
}

pub enum DiffRow {
    Hunk { index: usize, hunk: DiffHunk },
    Split(SplitRow),
}

pub fn build_split_rows(hunks: &[DiffHunk]) -> Vec<DiffRow> {
    let mut rows = Vec::new();
    for (hunk_index, hunk) in hunks.iter().enumerate() {
        rows.push(DiffRow::Hunk {
            index: hunk_index,
            hunk: hunk.clone(),
        });
        let mut cursor = 0usize;
        while cursor < hunk.lines.len() {
            let line = &hunk.lines[cursor];
            if line.kind == LineType::Context {
                rows.push(DiffRow::Split(SplitRow {
                    left: Some(line.clone()),
                    right: Some(line.clone()),
                }));
                cursor += 1;
                continue;
            }

            let mut deleted = Vec::new();
            let mut added = Vec::new();
            while cursor < hunk.lines.len() && hunk.lines[cursor].kind == LineType::Del {
                deleted.push(hunk.lines[cursor].clone());
                cursor += 1;
            }
            while cursor < hunk.lines.len() && hunk.lines[cursor].kind == LineType::Add {
                added.push(hunk.lines[cursor].clone());
                cursor += 1;
            }
            if deleted.is_empty() && added.is_empty() {
                cursor += 1;
                continue;
            }
            let depth = deleted.len().max(added.len());
            for index in 0..depth {
                rows.push(DiffRow::Split(SplitRow {
                    left: deleted.get(index).cloned(),
                    right: added.get(index).cloned(),
                }));
            }
        }
    }
    rows
}

pub fn path_parts(path: &str) -> (&str, &str) {
    match path.rsplit_once('/') {
        Some((dir, name)) => (name, dir),
        None => (path, ""),
    }
}

pub fn language_label(language: &str) -> String {
    match language {
        "typescript" => "TypeScript".to_string(),
        "tsx" => "TSX".to_string(),
        "javascript" => "JavaScript".to_string(),
        "jsx" => "JSX".to_string(),
        "rust" => "Rust".to_string(),
        "python" => "Python".to_string(),
        "go" => "Go".to_string(),
        "css" => "CSS".to_string(),
        "scss" => "SCSS".to_string(),
        "json" => "JSON".to_string(),
        "markdown" => "Markdown".to_string(),
        "yaml" => "YAML".to_string(),
        "toml" => "TOML".to_string(),
        "html" => "HTML".to_string(),
        "shell" | "bash" | "zsh" => "Shell".to_string(),
        "sql" => "SQL".to_string(),
        "swift" => "Swift".to_string(),
        "kotlin" => "Kotlin".to_string(),
        "java" => "Java".to_string(),
        "text" => "Text".to_string(),
        other => other.to_uppercase(),
    }
}

pub fn status_letter(status: ChangeStatus) -> &'static str {
    match status {
        ChangeStatus::Modified => "M",
        ChangeStatus::Added => "A",
        ChangeStatus::Deleted => "D",
        ChangeStatus::Renamed => "R",
    }
}

pub fn status_key(status: ChangeStatus) -> &'static str {
    match status {
        ChangeStatus::Modified => "modified",
        ChangeStatus::Added => "added",
        ChangeStatus::Deleted => "deleted",
        ChangeStatus::Renamed => "renamed",
    }
}

pub fn is_image(path: &str) -> bool {
    let extension = path
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .unwrap_or_default();
    matches!(
        extension.as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "tiff" | "avif"
    )
}

pub fn is_generated(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".min.js")
        || lower.ends_with(".min.css")
        || lower.ends_with(".map")
        || lower.ends_with(".lock")
        || lower == "package-lock.json"
        || lower == "pnpm-lock.yaml"
        || lower == "yarn.lock"
        || lower == "cargo.lock"
        || lower == "composer.lock"
        || lower == "poetry.lock"
        || path.starts_with("dist/")
        || path.starts_with("build/")
        || path.starts_with("target/")
        || path.starts_with("node_modules/")
        || path.starts_with("vendor/")
}

pub fn whitespace_only(file: &RepositoryFile) -> bool {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    for hunk in &file.hunks {
        for line in &hunk.lines {
            match line.kind {
                LineType::Add => added.push(line.content.trim().to_string()),
                LineType::Del => removed.push(line.content.trim().to_string()),
                LineType::Context => {}
            }
        }
    }
    !added.is_empty() && added == removed
}

/// A content-derived key used to keep review progress stable when the working
/// tree refreshes, while invalidating it when the diff actually changes.
pub fn review_fingerprint(file: &RepositoryFile) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    let mut feed = |value: &str| {
        for byte in value.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x100000001b3);
    };
    feed(&file.id);
    feed(&format!("{:?}", file.status));
    for hunk in &file.hunks {
        feed(&hunk.header);
        for line in &hunk.lines {
            feed(match line.kind {
                LineType::Add => "+",
                LineType::Del => "-",
                LineType::Context => " ",
            });
            feed(&line.content);
        }
    }
    format!("{hash:016x}")
}

const MAX_WORD_TOKENS: usize = 500;

fn tokenize(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut current_is_word = false;
    for ch in value.chars() {
        let is_word = ch.is_alphanumeric() || ch == '_';
        if current.is_empty() || is_word == current_is_word {
            current.push(ch);
            current_is_word = is_word;
        } else {
            tokens.push(std::mem::take(&mut current));
            current.push(ch);
            current_is_word = is_word;
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    if tokens.len() > MAX_WORD_TOKENS {
        tokens.clear();
        tokens.push(value.to_string());
    }
    tokens
}

#[derive(Clone)]
pub struct WordSegment {
    pub text: String,
    pub changed: bool,
}

/// Pairs two lines and marks the word runs that actually differ. Both results
/// concatenate back to the original input.
pub fn word_diff(old: &str, new: &str) -> (Vec<WordSegment>, Vec<WordSegment>) {
    let old_tokens = tokenize(old);
    let new_tokens = tokenize(new);
    let n = old_tokens.len();
    let m = new_tokens.len();

    if n == 0 || m == 0 || n.saturating_mul(m) > 250_000 {
        return (
            vec![WordSegment {
                text: old.to_string(),
                changed: !old.is_empty(),
            }],
            vec![WordSegment {
                text: new.to_string(),
                changed: !new.is_empty(),
            }],
        );
    }

    // Longest common subsequence over word tokens.
    let mut table = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            table[i][j] = if old_tokens[i] == new_tokens[j] {
                table[i + 1][j + 1] + 1
            } else {
                table[i + 1][j].max(table[i][j + 1])
            };
        }
    }

    let mut old_segments: Vec<WordSegment> = Vec::new();
    let mut new_segments: Vec<WordSegment> = Vec::new();
    let mut i = 0usize;
    let mut j = 0usize;
    let push_old = |segments: &mut Vec<WordSegment>, text: &str, changed: bool| {
        if text.is_empty() {
            return;
        }
        if let Some(last) = segments.last_mut() {
            if last.changed == changed {
                last.text.push_str(text);
                return;
            }
        }
        segments.push(WordSegment {
            text: text.to_string(),
            changed,
        });
    };

    while i < n && j < m {
        if old_tokens[i] == new_tokens[j] {
            push_old(&mut old_segments, &old_tokens[i], false);
            push_old(&mut new_segments, &new_tokens[j], false);
            i += 1;
            j += 1;
        } else if table[i + 1][j] >= table[i][j + 1] {
            push_old(&mut old_segments, &old_tokens[i], true);
            i += 1;
        } else {
            push_old(&mut new_segments, &new_tokens[j], true);
            j += 1;
        }
    }
    while i < n {
        push_old(&mut old_segments, &old_tokens[i], true);
        i += 1;
    }
    while j < m {
        push_old(&mut new_segments, &new_tokens[j], true);
        j += 1;
    }

    (old_segments, new_segments)
}
