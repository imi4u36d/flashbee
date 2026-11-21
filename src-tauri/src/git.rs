use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl CommandError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for CommandError {}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteBranchInfo {
    pub name: String,
    pub remote: String,
    pub branch: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeInfo {
    pub path: String,
    pub name: String,
    /// Checked-out branch, or `HEAD` when the worktree is detached.
    pub branch: String,
    /// Short commit hash at the worktree's HEAD.
    pub head: String,
    pub detached: bool,
    pub is_main: bool,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryInspection {
    pub path: String,
    pub name: String,
    pub current_branch: String,
    /// Local branch names, current branch first.
    pub branches: Vec<String>,
    pub remote_branches: Vec<RemoteBranchInfo>,
    pub remotes: Vec<RemoteInfo>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    /// Every working tree attached to the repository, main worktree first.
    pub worktrees: Vec<WorktreeInfo>,
}

/// Result of inspecting a folder the user wants to add. A folder is either a
/// repository itself (or inside one), or a plain directory that may contain
/// repositories further down.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryDiscovery {
    pub path: String,
    pub name: String,
    pub is_repository: bool,
    pub repositories: Vec<RepositoryInspection>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LineType {
    Context,
    Add,
    Del,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    #[serde(rename = "type")]
    pub kind: LineType,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_no: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_no: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryFile {
    pub path: String,
    pub status: ChangeStatus,
    pub language: String,
    pub staged: bool,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySnapshot {
    #[serde(flatten)]
    pub inspection: RepositoryInspection,
    pub files: Vec<RepositoryFile>,
    pub conflicts: Vec<ConflictEntry>,
    pub merge_in_progress: bool,
    pub fetched_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictEntry {
    pub path: String,
    pub kind: String,
    pub base: bool,
    pub ours: bool,
    pub theirs: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncOutcome {
    pub message: String,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PushCommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub subject: String,
    pub author: String,
    pub email: String,
    pub time: i64,
}

/// Everything the push dialog needs before the first click: the commits that
/// would land on the remote, the tags that can be pushed, and how far the
/// local branch has drifted from its remote counterpart.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PushInfo {
    pub branch: String,
    pub remote: String,
    pub remote_branch: String,
    pub upstream: Option<String>,
    pub base: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub commits: Vec<PushCommitInfo>,
    pub commits_total: u32,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitDetail {
    pub hash: String,
    pub short_hash: String,
    pub subject: String,
    pub body: String,
    pub author: String,
    pub email: String,
    pub time: i64,
    pub files: Vec<RepositoryFile>,
    pub file_count: u32,
    pub truncated: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PushOptions {
    /// Newest commit to publish. `None` pushes the current branch as-is.
    pub commit: Option<String>,
    pub remote: Option<String>,
    pub remote_branch: Option<String>,
    pub force: bool,
    pub push_tags: bool,
    /// Push only tags and leave the branch untouched.
    pub skip_commits: bool,
    /// Push a single tag instead of every tag when `push_tags` is set.
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictSide {
    Ours,
    Theirs,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ConflictSegment {
    Common {
        lines: Vec<String>,
    },
    Conflict {
        ours: Vec<String>,
        base: Vec<String>,
        theirs: Vec<String>,
        suggestion: Option<ConflictSide>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictDetail {
    pub path: String,
    pub language: String,
    pub binary: bool,
    pub ours_exists: bool,
    pub theirs_exists: bool,
    pub base_exists: bool,
    pub trailing_newline: bool,
    pub segments: Vec<ConflictSegment>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlameLine {
    pub line: u32,
    pub author: String,
    pub author_mail: String,
    pub time: i64,
    pub commit: String,
    pub summary: String,
    pub uncommitted: bool,
}

#[derive(Debug, Clone)]
struct StatusEntry {
    path: String,
    status: ChangeStatus,
    staged: bool,
    untracked: bool,
}

struct DiffSection {
    path: String,
    old_path: Option<String>,
    hunks: Vec<DiffHunk>,
}

fn language_for_path(file_path: &str) -> String {
    let name = Path::new(file_path)
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string());
    let extension = name
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_lowercase())
        .unwrap_or_default();

    let language = match extension.as_str() {
        "ts" => "typescript",
        "tsx" => "tsx",
        "js" | "mjs" | "cjs" => "javascript",
        "jsx" => "jsx",
        "css" => "css",
        "scss" => "scss",
        "less" => "less",
        "json" => "json",
        "md" | "mdx" => "markdown",
        "py" => "python",
        "go" => "go",
        "rs" => "rust",
        "toml" => "toml",
        "wgsl" => "wgsl",
        "html" | "htm" => "html",
        "yml" | "yaml" => "yaml",
        "sh" | "bash" | "zsh" => "bash",
        "sql" => "sql",
        "java" => "java",
        "kt" => "kotlin",
        "swift" => "swift",
        "c" | "h" => "c",
        "cpp" | "cc" | "hpp" => "cpp",
        "cs" => "csharp",
        "rb" => "ruby",
        "php" => "php",
        "xml" => "xml",
        "vue" => "vue",
        "svelte" => "svelte",
        _ => "text",
    };

    language.to_string()
}

fn status_from_code(code: &str) -> ChangeStatus {
    let normalized = code.replace('?', "A");
    if normalized.contains('D') {
        ChangeStatus::Deleted
    } else if normalized.contains('R') || normalized.contains('C') {
        ChangeStatus::Renamed
    } else if normalized.contains('A') {
        ChangeStatus::Added
    } else {
        ChangeStatus::Modified
    }
}

fn expand_home(input: &str) -> PathBuf {
    if input == "~" {
        return home_dir();
    }
    if let Some(rest) = input.strip_prefix("~/") {
        return home_dir().join(rest);
    }
    PathBuf::from(input)
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

struct GitOutput {
    success: bool,
    stdout: String,
    stderr: String,
}

fn git_not_found(error: std::io::Error) -> CommandError {
    if error.kind() == std::io::ErrorKind::NotFound {
        CommandError::new(
            "git_unavailable",
            "Git is not installed or not available.",
        )
    } else {
        CommandError::new("git_failed", error.to_string())
    }
}

fn run_git_capture(
    args: &[&str],
    cwd: &Path,
    envs: &[(&str, &str)],
) -> Result<GitOutput, CommandError> {
    let mut command = Command::new("git");
    command.args(args).current_dir(cwd);
    for (key, value) in envs {
        command.env(key, value);
    }

    let output = command.output().map_err(git_not_found)?;
    Ok(GitOutput {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    })
}

fn error_detail(output: &GitOutput, fallback: &str) -> String {
    if !output.stderr.is_empty() {
        output.stderr.clone()
    } else if !output.stdout.is_empty() {
        output.stdout.clone()
    } else {
        fallback.to_string()
    }
}

fn run_git(args: &[&str], cwd: &Path) -> Result<String, CommandError> {
    let output = run_git_capture(args, cwd, &[])?;
    if !output.success {
        return Err(CommandError::new(
            "git_failed",
            error_detail(&output, "The Git command failed."),
        ));
    }

    Ok(output.stdout)
}

/// Runs a command that may legitimately fail (a missing index stage, for
/// example) so callers can treat the failure as an expected absence.
fn run_git_bytes(args: &[&str], cwd: &Path) -> Result<Option<Vec<u8>>, CommandError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(git_not_found)?;

    if output.status.success() {
        Ok(Some(output.stdout))
    } else {
        Ok(None)
    }
}

/// Git network commands must never block on a terminal prompt: the app has no
/// terminal attached and the window would appear frozen.
const GIT_NETWORK_ENV: &[(&str, &str)] = &[
    ("GIT_TERMINAL_PROMPT", "0"),
    ("GIT_MERGE_AUTOEDIT", "no"),
    ("GIT_EDITOR", "true"),
];

fn validate_directory(input_path: &str) -> Result<PathBuf, CommandError> {
    if input_path.trim().is_empty() {
        return Err(CommandError::new(
            "bad_request",
            "A repository path is required.",
        ));
    }

    let requested = expand_home(input_path.trim());
    let metadata = fs::metadata(&requested).map_err(|_| {
        CommandError::new("not_found", "This folder does not exist.")
    })?;
    if !metadata.is_dir() {
        return Err(CommandError::new(
            "not_directory",
            "Select a folder, not a file.",
        ));
    }

    Ok(requested)
}

/// Rejects absolute paths and traversal so commands coming from the webview can
/// never touch files outside the repository work tree.
fn validate_relative_path(input: &str) -> Result<PathBuf, CommandError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(CommandError::new(
            "bad_request",
            "A file path is required.",
        ));
    }

    let path = Path::new(trimmed);
    if path.is_absolute() {
        return Err(CommandError::new(
            "bad_request",
            "The file path must be relative to the repository.",
        ));
    }

    for component in path.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(CommandError::new(
                "bad_request",
                "The file path must stay inside the repository.",
            ));
        }
    }

    Ok(path.to_path_buf())
}

fn repository_root(input_path: &str) -> Result<PathBuf, CommandError> {
    let requested = validate_directory(input_path)?;
    probe_repository_root(&requested)?.ok_or_else(|| {
        CommandError::new(
            "not_git",
            "This folder is not inside a Git repository.",
        )
    })
}

/// Resolves the work-tree root when `directory` sits inside a repository.
/// A plain folder (including one that only contains repositories) returns `None`.
fn probe_repository_root(directory: &Path) -> Result<Option<PathBuf>, CommandError> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(directory)
        .output()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                CommandError::new(
                    "git_unavailable",
                    "Git is not installed or not available.",
                )
            } else {
                CommandError::new("git_failed", error.to_string())
            }
        })?;

    if !output.status.success() {
        // "not a git repository" is an expected answer, not a failure.
        return Ok(None);
    }

    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if root.is_empty() {
        Ok(None)
    } else {
        Ok(Some(PathBuf::from(root)))
    }
}

/// Working trees attached to the repository. The main worktree comes first,
/// followed by every linked checkout; prunable entries (a worktree whose folder
/// is gone) and bare entries are skipped because neither has files to review.
fn read_worktrees(root_path: &Path) -> Vec<WorktreeInfo> {
    let Ok(raw) = run_git(&["worktree", "list", "--porcelain"], root_path) else {
        return Vec::new();
    };

    struct Candidate {
        path: PathBuf,
        branch: Option<String>,
        head: String,
        detached: bool,
        bare: bool,
        prunable: bool,
    }

    let mut candidates: Vec<Candidate> = Vec::new();
    for line in raw.lines() {
        if let Some(value) = line.strip_prefix("worktree ") {
            candidates.push(Candidate {
                path: PathBuf::from(value),
                branch: None,
                head: String::new(),
                detached: false,
                bare: false,
                prunable: false,
            });
            continue;
        }

        let Some(candidate) = candidates.last_mut() else {
            continue;
        };
        if let Some(value) = line.strip_prefix("HEAD ") {
            candidate.head = value.chars().take(7).collect();
        } else if let Some(value) = line.strip_prefix("branch ") {
            candidate.branch = Some(
                value
                    .strip_prefix("refs/heads/")
                    .unwrap_or(value)
                    .to_string(),
            );
        } else if line == "detached" {
            candidate.detached = true;
        } else if line == "bare" {
            candidate.bare = true;
        } else if line == "prunable" || line.starts_with("prunable ") {
            candidate.prunable = true;
        }
    }

    // Compare canonical paths so a worktree reached through a symlink (macOS
    // `/tmp` → `/private/tmp`, for instance) still matches the inspected one.
    let current = fs::canonicalize(root_path).unwrap_or_else(|_| root_path.to_path_buf());
    let mut worktrees = Vec::new();
    let mut saw_main = false;

    for candidate in candidates {
        if candidate.bare || candidate.prunable {
            continue;
        }
        let path = fs::canonicalize(&candidate.path).unwrap_or_else(|_| candidate.path.clone());
        if !path.is_dir() {
            continue;
        }

        let name = candidate
            .path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| candidate.path.to_string_lossy().to_string());
        let detached = candidate.detached || candidate.branch.is_none();
        let is_main = !saw_main;
        saw_main = true;

        worktrees.push(WorktreeInfo {
            path: path.to_string_lossy().to_string(),
            name,
            branch: candidate.branch.unwrap_or_else(|| "HEAD".to_string()),
            head: candidate.head,
            detached,
            is_main,
            is_current: path == current,
        });
    }

    worktrees
}

fn read_remotes(root_path: &Path) -> Vec<RemoteInfo> {
    let raw = run_git(&["remote", "-v"], root_path).unwrap_or_default();
    let mut remotes: Vec<RemoteInfo> = Vec::new();

    for line in raw.lines() {
        if !line.ends_with("(fetch)") {
            continue;
        }
        let Some((name, rest)) = line.split_once('\t') else {
            continue;
        };
        if remotes.iter().any(|remote| remote.name == name) {
            continue;
        }
        remotes.push(RemoteInfo {
            name: name.to_string(),
            url: rest.trim_end_matches("(fetch)").trim().to_string(),
        });
    }

    remotes.sort_by(|left, right| left.name.cmp(&right.name));
    remotes
}

fn read_remote_branches(root_path: &Path, remotes: &[RemoteInfo]) -> Vec<RemoteBranchInfo> {
    let raw = run_git(
        &[
            "for-each-ref",
            "--format=%(refname:short)%09%(symref)",
            "refs/remotes",
        ],
        root_path,
    )
    .unwrap_or_default();

    let mut branches = Vec::new();
    for line in raw.lines() {
        if line.is_empty() {
            continue;
        }
        let (name, symref) = line.split_once('\t').unwrap_or((line, ""));
        // `origin/HEAD` is a symbolic alias of a real branch, not a branch itself.
        if !symref.is_empty() || name.ends_with("/HEAD") {
            continue;
        }

        let remote = remotes
            .iter()
            .map(|candidate| candidate.name.as_str())
            .filter(|candidate| {
                name.len() > candidate.len()
                    && name.starts_with(candidate)
                    && name.as_bytes().get(candidate.len()) == Some(&b'/')
            })
            .max_by_key(|candidate| candidate.len())
            .map(str::to_string)
            .or_else(|| name.split_once('/').map(|(remote, _)| remote.to_string()));

        let Some(remote) = remote else {
            continue;
        };
        let branch = name
            .strip_prefix(&format!("{remote}/"))
            .unwrap_or(name)
            .to_string();

        branches.push(RemoteBranchInfo {
            name: name.to_string(),
            remote,
            branch,
        });
    }

    branches.sort_by(|left, right| {
        left.remote
            .cmp(&right.remote)
            .then_with(|| left.branch.cmp(&right.branch))
    });
    branches
}

fn read_upstream(root_path: &Path) -> Option<String> {
    let raw = run_git(
        &[
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
        root_path,
    )
    .ok()?;
    if raw.is_empty() {
        None
    } else {
        Some(raw)
    }
}

fn read_ahead_behind(root_path: &Path, upstream: &str) -> (u32, u32) {
    let range = format!("{upstream}...HEAD");
    let Ok(raw) = run_git(
        &["rev-list", "--left-right", "--count", range.as_str()],
        root_path,
    ) else {
        return (0, 0);
    };

    let mut parts = raw.split_whitespace();
    let behind = parts.next().and_then(|value| value.parse().ok()).unwrap_or(0);
    let ahead = parts.next().and_then(|value| value.parse().ok()).unwrap_or(0);
    (ahead, behind)
}

fn inspect_root(root_path: &Path, detailed: bool) -> Result<RepositoryInspection, CommandError> {
    let current_branch_raw = run_git(&["branch", "--show-current"], root_path).unwrap_or_default();
    let branches_raw = run_git(
        &["for-each-ref", "--format=%(refname:short)", "refs/heads"],
        root_path,
    )
    .unwrap_or_default();

    let mut local_branches: Vec<String> = branches_raw
        .lines()
        .map(|branch| branch.trim().to_string())
        .filter(|branch| !branch.is_empty())
        .collect();

    // An empty name means a detached HEAD. Keeping "HEAD" is honest and still
    // works as a diff base.
    let current_branch = if current_branch_raw.is_empty() {
        "HEAD".to_string()
    } else {
        current_branch_raw
    };

    if current_branch != "HEAD" {
        local_branches.retain(|branch| branch != &current_branch);
    }
    local_branches.sort();
    let mut branches = Vec::new();
    if current_branch != "HEAD" {
        branches.push(current_branch.clone());
    }
    branches.extend(local_branches);

    let (remote_branches, remotes, upstream, ahead, behind) = if detailed {
        let remotes = read_remotes(root_path);
        let remote_branches = read_remote_branches(root_path, &remotes);
        let upstream = read_upstream(root_path);
        let (ahead, behind) = upstream
            .as_deref()
            .map(|value| read_ahead_behind(root_path, value))
            .unwrap_or((0, 0));
        (remote_branches, remotes, upstream, ahead, behind)
    } else {
        (Vec::new(), Vec::new(), None, 0, 0)
    };

    let name = root_path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| root_path.to_string_lossy().to_string());

    // Only the full snapshot needs the worktree list; discovery stays cheap.
    let worktrees = if detailed {
        read_worktrees(root_path)
    } else {
        Vec::new()
    };

    Ok(RepositoryInspection {
        path: root_path.to_string_lossy().to_string(),
        name,
        current_branch,
        branches,
        remote_branches,
        remotes,
        upstream,
        ahead,
        behind,
        worktrees,
    })
}

pub fn inspect_repository(input_path: &str) -> Result<RepositoryInspection, CommandError> {
    let requested = validate_directory(input_path)?;
    let root = probe_repository_root(&requested)?.ok_or_else(|| {
        CommandError::new(
            "not_git",
            "This folder is not inside a Git repository.",
        )
    })?;

    inspect_root(&root, false)
}

/// Number of folder levels below the chosen folder that are scanned for
/// repositories. Immediate children are level one.
const MAX_DISCOVERY_DEPTH: usize = 4;
const MAX_SCANNED_DIRECTORIES: usize = 20_000;
const MAX_DISCOVERED_REPOSITORIES: usize = 100;

fn should_skip_directory(name: &str) -> bool {
    name.starts_with('.') || is_ignored_repository(name)
}

/// Folders that should not surface as repositories even when they happen to
/// contain a `.git` entry (build output, dependency caches, and so on).
fn is_ignored_repository(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | "node_modules"
            | "target"
            | "dist"
            | "build"
            | "out"
            | "vendor"
            | "Pods"
            | "DerivedData"
            | "__pycache__"
            | "venv"
            | "coverage"
            | "Library"
            | "Applications"
    )
}

fn scan_repositories(root: &Path) -> (Vec<RepositoryInspection>, bool) {
    let mut repositories = Vec::new();
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    let mut scanned = 0usize;
    let mut truncated = false;

    while let Some((directory, depth)) = stack.pop() {
        if repositories.len() >= MAX_DISCOVERED_REPOSITORIES {
            truncated = true;
            break;
        }

        scanned += 1;
        if scanned > MAX_SCANNED_DIRECTORIES {
            truncated = true;
            break;
        }

        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };

        let mut children: Vec<(PathBuf, String)> = entries
            .flatten()
            .filter_map(|entry| {
                let file_type = entry.file_type().ok()?;
                // Skip symlinked folders so circular links cannot trap the scan.
                if !file_type.is_dir() {
                    return None;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if is_ignored_repository(&name) {
                    return None;
                }
                Some((entry.path(), name))
            })
            .collect();
        children.sort_by(|left, right| left.0.cmp(&right.0));

        for (child, name) in children {
            if child.join(".git").exists() {
                if repositories.len() >= MAX_DISCOVERED_REPOSITORIES {
                    truncated = true;
                    break;
                }
                if let Ok(inspection) = inspect_root(&child, false) {
                    repositories.push(inspection);
                }
            } else if depth + 1 < MAX_DISCOVERY_DEPTH && !should_skip_directory(&name) {
                stack.push((child, depth + 1));
            }
        }
    }

    repositories.sort_by(|left, right| left.path.cmp(&right.path));
    (repositories, truncated)
}

pub fn discover_repositories(input_path: &str) -> Result<RepositoryDiscovery, CommandError> {
    let requested = validate_directory(input_path)?;
    let directory = fs::canonicalize(&requested).unwrap_or(requested);
    let name = directory
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| directory.to_string_lossy().to_string());

    if let Some(root) = probe_repository_root(&directory)? {
        let inspection = inspect_root(&root, false)?;
        return Ok(RepositoryDiscovery {
            path: inspection.path.clone(),
            name: inspection.name.clone(),
            is_repository: true,
            repositories: vec![inspection],
            truncated: false,
        });
    }

    let (repositories, truncated) = scan_repositories(&directory);

    Ok(RepositoryDiscovery {
        path: directory.to_string_lossy().to_string(),
        name,
        is_repository: false,
        repositories,
        truncated,
    })
}

pub fn initialize_repository(input_path: &str) -> Result<RepositoryInspection, CommandError> {
    let requested = validate_directory(input_path)?;

    // If the folder is already covered by a repository, do not create a nested one.
    if let Some(root) = probe_repository_root(&requested)? {
        return inspect_root(&root, false);
    }

    run_git(&["init"], &requested)?;

    let root = probe_repository_root(&requested)?.ok_or_else(|| {
        CommandError::new(
            "git_failed",
            "Git could not initialize this folder as a repository.",
        )
    })?;

    inspect_root(&root, false)
}

fn inspect_repository_root(
    input_path: &str,
) -> Result<(PathBuf, RepositoryInspection), CommandError> {
    let requested = validate_directory(input_path)?;
    let root = probe_repository_root(&requested)?.ok_or_else(|| {
        CommandError::new(
            "not_git",
            "This folder is not inside a Git repository.",
        )
    })?;
    let inspection = inspect_root(&root, true)?;
    Ok((root, inspection))
}

fn merge_in_progress(root_path: &Path) -> bool {
    run_git(&["rev-parse", "-q", "--verify", "MERGE_HEAD"], root_path).is_ok()
}

fn conflict_entries(root_path: &Path) -> Result<Vec<ConflictEntry>, CommandError> {
    let raw = run_git(&["ls-files", "-u", "-z"], root_path)?;
    let mut stages: BTreeMap<String, (bool, bool, bool)> = BTreeMap::new();

    for record in raw.split('\0') {
        if record.is_empty() {
            continue;
        }
        let Some((metadata, path)) = record.split_once('\t') else {
            continue;
        };
        if path.is_empty() {
            continue;
        }
        let Some(stage) = metadata.rsplit(' ').next() else {
            continue;
        };

        let entry = stages
            .entry(path.to_string())
            .or_insert((false, false, false));
        match stage {
            "1" => entry.0 = true,
            "2" => entry.1 = true,
            "3" => entry.2 = true,
            _ => {}
        }
    }

    Ok(stages
        .into_iter()
        .map(|(path, (base, ours, theirs))| ConflictEntry {
            kind: conflict_kind(base, ours, theirs).to_string(),
            path,
            base,
            ours,
            theirs,
        })
        .collect())
}

fn conflict_kind(base: bool, ours: bool, theirs: bool) -> &'static str {
    match (base, ours, theirs) {
        (true, true, true) => "bothModified",
        (false, true, true) => "bothAdded",
        (true, false, true) => "deletedByUs",
        (true, true, false) => "deletedByThem",
        (true, false, false) => "bothDeleted",
        (false, false, true) => "addedByThem",
        (false, true, false) => "addedByUs",
        _ => "modified",
    }
}

fn read_stage_bytes(
    root_path: &Path,
    stage: u8,
    relative_path: &str,
) -> Result<Option<Vec<u8>>, CommandError> {
    let spec = format!(":{stage}:{relative_path}");
    run_git_bytes(&["show", spec.as_str()], root_path)
}

fn content_lines(content: &str) -> Vec<String> {
    if content.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = content.split('\n').map(str::to_string).collect();
    if content.ends_with('\n') {
        lines.pop();
    }
    lines
}

fn push_common_segment(segments: &mut Vec<ConflictSegment>, common: &mut Vec<String>) {
    if common.is_empty() {
        return;
    }
    segments.push(ConflictSegment::Common {
        lines: std::mem::take(common),
    });
}

fn suggested_side(ours: &[String], base: &[String], theirs: &[String]) -> Option<ConflictSide> {
    if ours == base {
        Some(ConflictSide::Theirs)
    } else if theirs == base {
        Some(ConflictSide::Ours)
    } else {
        None
    }
}

fn is_conflict_marker(line: &str, marker: &str) -> bool {
    line.trim_end_matches('\r') == marker
}

fn parse_conflict_markers(merged: &str) -> Vec<ConflictSegment> {
    let lines = content_lines(merged);
    let mut segments = Vec::new();
    let mut common: Vec<String> = Vec::new();
    let mut cursor = 0;

    while cursor < lines.len() {
        let line = &lines[cursor];
        if !is_conflict_marker(line, "<<<<<<< ours") {
            common.push(line.clone());
            cursor += 1;
            continue;
        }

        push_common_segment(&mut segments, &mut common);
        cursor += 1;

        let mut ours = Vec::new();
        while cursor < lines.len()
            && !is_conflict_marker(&lines[cursor], "||||||| base")
            && !is_conflict_marker(&lines[cursor], "=======")
        {
            ours.push(lines[cursor].clone());
            cursor += 1;
        }

        let mut base = Vec::new();
        if cursor < lines.len() && is_conflict_marker(&lines[cursor], "||||||| base") {
            cursor += 1;
            while cursor < lines.len() && !is_conflict_marker(&lines[cursor], "=======") {
                base.push(lines[cursor].clone());
                cursor += 1;
            }
        }

        let mut theirs = Vec::new();
        if cursor < lines.len() && is_conflict_marker(&lines[cursor], "=======") {
            cursor += 1;
            while cursor < lines.len()
                && !is_conflict_marker(&lines[cursor], ">>>>>>> theirs")
            {
                theirs.push(lines[cursor].clone());
                cursor += 1;
            }
        }
        if cursor < lines.len() && is_conflict_marker(&lines[cursor], ">>>>>>> theirs") {
            cursor += 1;
        }

        if ours == theirs {
            segments.push(ConflictSegment::Common { lines: ours });
        } else {
            let suggestion = suggested_side(&ours, &base, &theirs);
            segments.push(ConflictSegment::Conflict {
                ours,
                base,
                theirs,
                suggestion,
            });
        }
    }

    push_common_segment(&mut segments, &mut common);
    segments
}

fn merge_conflict_markers(
    ours: &[u8],
    base: &[u8],
    theirs: &[u8],
) -> Result<String, CommandError> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or(0);
    let directory = std::env::temp_dir().join(format!(
        "flashbee-merge-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&directory)
        .map_err(|error| CommandError::new("io_failed", error.to_string()))?;

    let ours_path = directory.join("ours");
    let base_path = directory.join("base");
    let theirs_path = directory.join("theirs");

    let write_error =
        |error: std::io::Error| CommandError::new("io_failed", error.to_string());
    fs::write(&ours_path, ours).map_err(write_error)?;
    fs::write(&base_path, base).map_err(write_error)?;
    fs::write(&theirs_path, theirs).map_err(write_error)?;

    let output = Command::new("git")
        .args([
            "merge-file",
            "-p",
            "--diff3",
            "-L",
            "ours",
            "-L",
            "base",
            "-L",
            "theirs",
        ])
        .arg(&ours_path)
        .arg(&base_path)
        .arg(&theirs_path)
        .output();
    let _ = fs::remove_dir_all(&directory);

    let output = output.map_err(git_not_found)?;
    let code = output.status.code().unwrap_or(-1);
    // git merge-file uses the exit code as the number of conflicts (0-127).
    if code < 0 || code > 127 {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(CommandError::new(
            "merge_failed",
            if detail.is_empty() {
                "Git could not compare these conflict sides.".to_string()
            } else {
                detail
            },
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn read_conflict(input_path: &str, file_path: &str) -> Result<ConflictDetail, CommandError> {
    let root = repository_root(input_path)?;
    let relative = validate_relative_path(file_path)?;
    let relative_path = relative.to_string_lossy().to_string();

    let base = read_stage_bytes(&root, 1, &relative_path)?;
    let ours = read_stage_bytes(&root, 2, &relative_path)?;
    let theirs = read_stage_bytes(&root, 3, &relative_path)?;

    let binary = [&base, &ours, &theirs]
        .into_iter()
        .flatten()
        .any(|content| content.contains(&0));

    if binary {
        return Ok(ConflictDetail {
            path: relative_path.clone(),
            language: language_for_path(&relative_path),
            binary: true,
            ours_exists: ours.is_some(),
            theirs_exists: theirs.is_some(),
            base_exists: base.is_some(),
            trailing_newline: false,
            segments: Vec::new(),
        });
    }

    let base_text = base
        .as_deref()
        .map(|content| String::from_utf8_lossy(content).to_string())
        .unwrap_or_default();
    let ours_text = ours
        .as_deref()
        .map(|content| String::from_utf8_lossy(content).to_string())
        .unwrap_or_default();
    let theirs_text = theirs
        .as_deref()
        .map(|content| String::from_utf8_lossy(content).to_string())
        .unwrap_or_default();

    let merged = merge_conflict_markers(
        ours_text.as_bytes(),
        base_text.as_bytes(),
        theirs_text.as_bytes(),
    )?;

    Ok(ConflictDetail {
        path: relative_path.clone(),
        language: language_for_path(&relative_path),
        binary: false,
        ours_exists: ours.is_some(),
        theirs_exists: theirs.is_some(),
        base_exists: base.is_some(),
        trailing_newline: merged.ends_with('\n'),
        segments: parse_conflict_markers(&merged),
    })
}

pub fn save_conflict_resolution(
    input_path: &str,
    file_path: &str,
    content: &str,
) -> Result<(), CommandError> {
    let root = repository_root(input_path)?;
    let relative = validate_relative_path(file_path)?;
    let relative_path = relative.to_string_lossy().to_string();
    let target = root.join(&relative);

    fs::write(&target, content.as_bytes())
        .map_err(|error| CommandError::new("write_failed", error.to_string()))?;
    run_git(&["add", "--", relative_path.as_str()], &root)?;
    Ok(())
}

pub fn resolve_conflict_side(
    input_path: &str,
    file_path: &str,
    side: &str,
) -> Result<(), CommandError> {
    let stage = match side {
        "ours" => 2,
        "theirs" => 3,
        _ => {
            return Err(CommandError::new(
                "bad_request",
                "The conflict side must be ours or theirs.",
            ))
        }
    };

    let root = repository_root(input_path)?;
    let relative = validate_relative_path(file_path)?;
    let relative_path = relative.to_string_lossy().to_string();
    let target = root.join(&relative);

    match read_stage_bytes(&root, stage, &relative_path)? {
        Some(content) => {
            fs::write(&target, content)
                .map_err(|error| CommandError::new("write_failed", error.to_string()))?;
            run_git(&["add", "--", relative_path.as_str()], &root)?;
        }
        None => {
            // Choosing a side where the file was deleted means the resolution
            // is a deletion too.
            if target.exists() {
                fs::remove_file(&target)
                    .map_err(|error| CommandError::new("write_failed", error.to_string()))?;
            }
            run_git(
                &[
                    "rm",
                    "-f",
                    "--ignore-unmatch",
                    "--",
                    relative_path.as_str(),
                ],
                &root,
            )?;
        }
    }

    Ok(())
}

pub fn complete_merge(input_path: &str) -> Result<SyncOutcome, CommandError> {
    let root = repository_root(input_path)?;
    if !merge_in_progress(&root) {
        return Err(CommandError::new(
            "no_merge",
            "There is no merge in progress to complete.",
        ));
    }

    let output = run_git_capture(&["commit", "--no-edit"], &root, GIT_NETWORK_ENV)?;
    if !output.success {
        return Err(CommandError::new(
            "merge_failed",
            error_detail(&output, "Git could not complete the merge."),
        ));
    }

    Ok(SyncOutcome {
        message: if output.stdout.is_empty() {
            "Merge completed.".to_string()
        } else {
            output.stdout
        },
        conflicts: Vec::new(),
    })
}

pub fn abort_merge(input_path: &str) -> Result<(), CommandError> {
    let root = repository_root(input_path)?;
    if !merge_in_progress(&root) {
        return Err(CommandError::new(
            "no_merge",
            "There is no merge in progress to abort.",
        ));
    }
    run_git(&["merge", "--abort"], &root)?;
    Ok(())
}

pub fn list_remotes(input_path: &str) -> Result<Vec<RemoteInfo>, CommandError> {
    let root = repository_root(input_path)?;
    Ok(read_remotes(&root))
}

fn validate_remote_name(name: &str) -> Result<(), CommandError> {
    if name.is_empty() || name == "." || name == ".." || name.starts_with('-') {
        return Err(CommandError::new("bad_request", "Invalid remote name."));
    }
    if !name.chars().all(|character| {
        character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
    }) {
        return Err(CommandError::new("bad_request", "Invalid remote name."));
    }
    Ok(())
}

pub fn set_remote_url(
    input_path: &str,
    name: &str,
    url: &str,
) -> Result<Vec<RemoteInfo>, CommandError> {
    let name = name.trim();
    let url = url.trim();
    validate_remote_name(name)?;
    if url.is_empty() || url.starts_with('-') {
        return Err(CommandError::new(
            "bad_request",
            "A valid remote URL is required.",
        ));
    }

    let root = repository_root(input_path)?;
    let exists = read_remotes(&root).iter().any(|remote| remote.name == name);

    if exists {
        run_git(&["remote", "set-url", name, url], &root)?;
    } else {
        run_git(&["remote", "add", name, url], &root)?;
    }

    Ok(read_remotes(&root))
}

fn pull_conflict_paths(root_path: &Path) -> Result<Vec<String>, CommandError> {
    Ok(conflict_entries(root_path)?
        .into_iter()
        .map(|entry| entry.path)
        .collect())
}

pub fn pull_repository(input_path: &str) -> Result<SyncOutcome, CommandError> {
    let (root, inspection) = inspect_repository_root(input_path)?;

    if inspection.remotes.is_empty() {
        return Err(CommandError::new(
            "no_remote",
            "This repository has no remote configured.",
        ));
    }
    if inspection.upstream.is_none() {
        return Err(CommandError::new(
            "no_upstream",
            "The current branch has no upstream branch.",
        ));
    }

    let output = run_git_capture(
        &["pull", "--no-rebase", "--no-edit"],
        &root,
        GIT_NETWORK_ENV,
    )?;
    let conflicts = pull_conflict_paths(&root)?;

    if output.success || !conflicts.is_empty() {
        Ok(SyncOutcome {
            message: error_detail(&output, "Already up to date."),
            conflicts,
        })
    } else {
        Err(CommandError::new(
            "pull_failed",
            error_detail(&output, "Git could not pull from the remote."),
        ))
    }
}

fn preferred_remote(inspection: &RepositoryInspection) -> String {
    inspection
        .remotes
        .iter()
        .find(|remote| remote.name == "origin")
        .or_else(|| inspection.remotes.first())
        .map(|remote| remote.name.clone())
        .unwrap_or_else(|| "origin".to_string())
}

fn split_remote_ref(value: &str) -> Option<(String, String)> {
    let (remote, branch) = value.split_once('/')?;
    if remote.is_empty() || branch.is_empty() {
        return None;
    }
    Some((remote.to_string(), branch.to_string()))
}

fn remote_tracking_ref_exists(root: &Path, remote: &str, branch: &str) -> bool {
    let reference = format!("refs/remotes/{remote}/{branch}");
    run_git(
        &["rev-parse", "--verify", "--quiet", reference.as_str()],
        root,
    )
    .is_ok()
}

/// The ref the local branch is compared against. Upstream wins; otherwise use
/// the remote branch with the same name, the remote HEAD, or its default
/// branch. `None` means the remote is empty and every commit is new.
fn push_base(
    root: &Path,
    upstream: Option<&str>,
    remote: &str,
    remote_branch: &str,
) -> Option<String> {
    if let Some(upstream) = upstream {
        return Some(upstream.to_string());
    }
    if remote_tracking_ref_exists(root, remote, remote_branch) {
        return Some(format!("{remote}/{remote_branch}"));
    }
    if remote_tracking_ref_exists(root, remote, "HEAD") {
        return Some(format!("{remote}/HEAD"));
    }
    for fallback in ["main", "master"] {
        if remote_tracking_ref_exists(root, remote, fallback) {
            return Some(format!("{remote}/{fallback}"));
        }
    }
    None
}

fn read_push_commits(
    root: &Path,
    base: Option<&str>,
    limit: u32,
) -> Result<(Vec<PushCommitInfo>, u32), CommandError> {
    let range = base.map(|value| format!("{value}..HEAD"));
    let count_target = range.as_deref().unwrap_or("HEAD");
    let total = run_git(&["rev-list", "--count", count_target], root)
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
        .unwrap_or(0);

    let limit_value = limit.to_string();
    let mut args = vec![
        "log",
        "--format=%H%x00%h%x00%an%x00%ae%x00%at%x00%s%x00%x1e",
        "-n",
        limit_value.as_str(),
    ];
    if let Some(range) = range.as_deref() {
        args.push(range);
    } else {
        args.push("HEAD");
    }

    let raw = run_git(&args, root)?;
    let mut commits = Vec::new();
    for record in raw.split('\x1e') {
        let record = record.trim_start_matches(|value| value == '\n' || value == '\r');
        if record.is_empty() {
            continue;
        }
        let mut fields = record.split('\0');
        let hash = fields.next().unwrap_or("").trim();
        if hash.is_empty() {
            continue;
        }
        commits.push(PushCommitInfo {
            hash: hash.to_string(),
            short_hash: fields.next().unwrap_or("").trim().to_string(),
            author: fields.next().unwrap_or("").trim().to_string(),
            email: fields.next().unwrap_or("").trim().to_string(),
            time: fields.next().unwrap_or("").trim().parse().unwrap_or(0),
            subject: fields.next().unwrap_or("").trim().to_string(),
        });
    }

    Ok((commits, total))
}

pub fn push_info(input_path: &str) -> Result<PushInfo, CommandError> {
    let (root, inspection) = inspect_repository_root(input_path)?;

    if inspection.remotes.is_empty() {
        return Err(CommandError::new(
            "no_remote",
            "This repository has no remote configured.",
        ));
    }
    if inspection.current_branch.is_empty() || inspection.current_branch == "HEAD" {
        return Err(CommandError::new(
            "detached_head",
            "A detached HEAD cannot be pushed.",
        ));
    }

    let upstream_parts = inspection.upstream.as_deref().and_then(split_remote_ref);
    let (remote, remote_branch) = match upstream_parts {
        Some((remote, branch)) => (remote, branch),
        None => (
            preferred_remote(&inspection),
            inspection.current_branch.clone(),
        ),
    };
    let base = push_base(
        &root,
        inspection.upstream.as_deref(),
        &remote,
        &remote_branch,
    );
    let (ahead, behind) = base
        .as_deref()
        .map(|value| read_ahead_behind(&root, value))
        .unwrap_or((0, 0));
    let (commits, commits_total) = read_push_commits(&root, base.as_deref(), 500)?;
    let tags = run_git(&["tag", "--list"], &root)
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect();

    Ok(PushInfo {
        branch: inspection.current_branch,
        remote,
        remote_branch,
        upstream: inspection.upstream,
        base,
        ahead,
        behind,
        commits,
        commits_total,
        tags,
    })
}

fn is_revision_token(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed.len() <= 64
        && trimmed.chars().all(|character| character.is_ascii_hexdigit())
}

fn read_first_parent(root: &Path, revision: &str) -> Option<String> {
    let raw = run_git(&["rev-list", "--parents", "-n", "1", revision], root).ok()?;
    raw.split_whitespace().nth(1).map(str::to_string)
}

fn read_commit_statuses(
    root: &Path,
    revision: &str,
    parent: Option<&str>,
) -> HashMap<String, ChangeStatus> {
    let raw = if let Some(parent) = parent {
        run_git(
            &[
                "-c",
                "core.quotePath=false",
                "diff",
                "--name-status",
                "--find-renames",
                parent,
                revision,
            ],
            root,
        )
    } else {
        run_git(
            &[
                "-c",
                "core.quotePath=false",
                "show",
                "--format=",
                "--name-status",
                "--find-renames",
                revision,
            ],
            root,
        )
    }
    .unwrap_or_default();

    let mut statuses = HashMap::new();
    for line in raw.lines() {
        let mut fields = line.split('\t');
        let Some(code) = fields.next() else {
            continue;
        };
        let Some(first_path) = fields.next() else {
            continue;
        };
        let second_path = fields.next();
        let status = match code.chars().next() {
            Some('A') | Some('C') => ChangeStatus::Added,
            Some('D') => ChangeStatus::Deleted,
            Some('R') => ChangeStatus::Renamed,
            _ => ChangeStatus::Modified,
        };
        let path = if matches!(status, ChangeStatus::Renamed) {
            second_path.unwrap_or(first_path)
        } else {
            first_path
        };
        statuses.insert(path.to_string(), status);
    }
    statuses
}

pub fn commit_detail(input_path: &str, revision: &str) -> Result<CommitDetail, CommandError> {
    let revision = revision.trim();
    if !is_revision_token(revision) {
        return Err(CommandError::new(
            "bad_revision",
            "The selected commit is not valid.",
        ));
    }
    let root = repository_root(input_path)?;

    let meta = run_git(
        &[
            "show",
            "-s",
            "--format=%H%x00%h%x00%an%x00%ae%x00%at%x00%s%x00%b",
            revision,
        ],
        &root,
    )?;
    let mut fields = meta.trim_start_matches('\n').split('\0');
    let hash = fields.next().unwrap_or("").trim().to_string();
    if hash.is_empty() {
        return Err(CommandError::new(
            "bad_revision",
            "The selected commit could not be read.",
        ));
    }
    let short_hash = fields.next().unwrap_or("").trim().to_string();
    let author = fields.next().unwrap_or("").trim().to_string();
    let email = fields.next().unwrap_or("").trim().to_string();
    let time = fields.next().unwrap_or("").trim().parse().unwrap_or(0);
    let subject = fields.next().unwrap_or("").trim().to_string();
    let body = fields.next().unwrap_or("").trim().to_string();

    let parent = read_first_parent(&root, &hash);
    let statuses = read_commit_statuses(&root, &hash, parent.as_deref());
    let diff = if let Some(parent) = parent.as_deref() {
        run_git(
            &[
                "-c",
                "core.quotePath=false",
                "diff",
                parent,
                hash.as_str(),
                "--no-color",
                "--no-ext-diff",
                "--unified=3",
                "--find-renames",
            ],
            &root,
        )
    } else {
        run_git(
            &[
                "-c",
                "core.quotePath=false",
                "show",
                "--format=",
                "--no-color",
                "--no-ext-diff",
                "--unified=3",
                "--find-renames",
                hash.as_str(),
            ],
            &root,
        )
    }
    .unwrap_or_default();

    let mut files: Vec<RepositoryFile> = parse_diff_sections(&diff)
        .into_iter()
        .map(|section| {
            let status = statuses
                .get(section.path.as_str())
                .cloned()
                .unwrap_or(ChangeStatus::Modified);
            RepositoryFile {
                path: section.path.clone(),
                status,
                language: language_for_path(&section.path),
                staged: false,
                hunks: section.hunks,
            }
        })
        .collect();
    files.sort_by(|left, right| left.path.cmp(&right.path));

    let file_count = files.len() as u32;
    const MAX_COMMIT_FILES: usize = 120;
    let truncated = files.len() > MAX_COMMIT_FILES;
    files.truncate(MAX_COMMIT_FILES);

    Ok(CommitDetail {
        hash,
        short_hash,
        subject,
        body,
        author,
        email,
        time,
        files,
        file_count,
        truncated,
    })
}

pub fn push_repository(
    input_path: &str,
    options: Option<PushOptions>,
) -> Result<SyncOutcome, CommandError> {
    let (root, inspection) = inspect_repository_root(input_path)?;

    if inspection.remotes.is_empty() {
        return Err(CommandError::new(
            "no_remote",
            "This repository has no remote configured.",
        ));
    }
    if inspection.current_branch.is_empty() || inspection.current_branch == "HEAD" {
        return Err(CommandError::new(
            "detached_head",
            "A detached HEAD cannot be pushed.",
        ));
    }

    let options = options.unwrap_or_default();
    let upstream_parts = inspection.upstream.as_deref().and_then(split_remote_ref);
    let remote = options
        .remote
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .filter(|name| inspection.remotes.iter().any(|entry| entry.name == *name))
        .map(str::to_string)
        .or_else(|| upstream_parts.as_ref().map(|(name, _)| name.clone()))
        .unwrap_or_else(|| preferred_remote(&inspection));
    let remote_branch = options
        .remote_branch
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .or_else(|| upstream_parts.as_ref().map(|(_, branch)| branch.clone()))
        .unwrap_or_else(|| inspection.current_branch.clone());

    let selected_commit = options
        .commit
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if let Some(commit) = selected_commit {
        if !options.skip_commits {
            if !is_revision_token(commit) {
                return Err(CommandError::new(
                    "bad_revision",
                    "The selected commit is not valid.",
                ));
            }
            let ancestor = run_git_capture(
                &["merge-base", "--is-ancestor", commit, "HEAD"],
                &root,
                &[],
            )?;
            if !ancestor.success {
                return Err(CommandError::new(
                    "bad_revision",
                    "The selected commit is not part of the current branch.",
                ));
            }
        }
    }

    let tag = if options.push_tags {
        options
            .tag
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
    } else {
        None
    };
    if options.push_tags {
        if let Some(tag) = tag {
            let reference = format!("refs/tags/{tag}");
            let valid = run_git_capture(
                &["check-ref-format", reference.as_str()],
                &root,
                &[],
            )?
            .success;
            if !valid {
                return Err(CommandError::new(
                    "bad_tag",
                    "The selected tag is not valid.",
                ));
            }
        }
    }

    let mut args: Vec<String> = vec!["push".to_string()];
    if options.force {
        args.push("--force-with-lease".to_string());
    }
    if options.push_tags && tag.is_none() {
        args.push("--tags".to_string());
    }

    let mut pushes_refs = false;
    let mut remote_selected = false;

    if !options.skip_commits {
        if let Some(commit) = selected_commit {
            args.push(remote.clone());
            args.push(format!("{commit}:refs/heads/{remote_branch}"));
        } else if inspection.upstream.is_none() {
            args.push("-u".to_string());
            args.push(remote.clone());
            args.push(inspection.current_branch.clone());
        } else {
            args.push(remote.clone());
        }
        remote_selected = true;
        pushes_refs = true;
    }

    if let Some(tag) = tag {
        if !remote_selected {
            args.push(remote.clone());
        }
        args.push(format!("refs/tags/{tag}"));
        pushes_refs = true;
    } else if options.push_tags {
        if !remote_selected {
            args.push(remote.clone());
        }
        pushes_refs = true;
    }

    if !pushes_refs {
        return Ok(SyncOutcome {
            message: "Nothing to push.".to_string(),
            conflicts: Vec::new(),
        });
    }

    let git_args: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = run_git_capture(&git_args, &root, GIT_NETWORK_ENV)?;
    if !output.success {
        return Err(CommandError::new(
            "push_failed",
            error_detail(&output, "Git could not push to the remote."),
        ));
    }

    // `sha:branch` refspecs cannot set upstream tracking themselves. Link the
    // branch once the remote counterpart exists so the next pull/push just works.
    if !options.skip_commits
        && inspection.upstream.is_none()
        && remote_branch == inspection.current_branch
    {
        let tracking = format!("--set-upstream-to={remote}/{remote_branch}");
        let _ = run_git_capture(&["branch", tracking.as_str()], &root, &[]);
    }

    Ok(SyncOutcome {
        message: error_detail(&output, "Everything up-to-date."),
        conflicts: Vec::new(),
    })
}

fn read_status_entries(root: &Path) -> Result<Vec<StatusEntry>, CommandError> {
    let output = run_git(
        &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
        root,
    )?;
    if output.is_empty() {
        return Ok(Vec::new());
    }

    let mut fields = output.split('\0');
    let mut entries = Vec::new();

    while let Some(value) = fields.next() {
        if value.is_empty() {
            continue;
        }
        if value.len() < 3 {
            continue;
        }
        let code = &value[..2];
        let first_path = value.get(3..).unwrap_or("").to_string();
        let mut file_path = first_path.clone();

        if code.contains('R') || code.contains('C') {
            if let Some(second_path) = fields.next() {
                if !second_path.is_empty() {
                    file_path = second_path.to_string();
                }
            }
        }

        let untracked = code == "??";
        let mut chars = code.chars();
        let index_status = chars.next().unwrap_or(' ');
        let worktree_status = chars.next().unwrap_or(' ');
        let staged = !untracked
            && index_status != ' '
            && index_status != '?'
            && worktree_status == ' ';

        entries.push(StatusEntry {
            path: file_path,
            status: status_from_code(code),
            staged,
            untracked,
        });
    }

    Ok(entries)
}

fn strip_diff_prefix(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed == "/dev/null" {
        return None;
    }
    let stripped = trimmed
        .strip_prefix("a/")
        .or_else(|| trimmed.strip_prefix("b/"))
        .unwrap_or(trimmed);
    Some(stripped.to_string())
}

fn parse_hunk_start(header: &str) -> Option<(u32, u32)> {
    let rest = header.strip_prefix("@@ -")?;
    let plus_index = rest.find(" +")?;
    let old_part = &rest[..plus_index];
    let after_plus = &rest[plus_index + 2..];
    let end = after_plus.find(" @@")?;
    let new_part = &after_plus[..end];
    let old_start = old_part.split(',').next()?.parse::<u32>().ok()?;
    let new_start = new_part.split(',').next()?.parse::<u32>().ok()?;
    Some((old_start, new_start))
}

fn parse_hunks_from_diff(lines: &[String]) -> Vec<DiffHunk> {
    let mut hunks: Vec<DiffHunk> = Vec::new();
    let mut current: Option<usize> = None;
    let mut old_no: u32 = 0;
    let mut new_no: u32 = 0;

    for line in lines {
        if line.starts_with("@@ ") {
            if let Some((old_start, new_start)) = parse_hunk_start(line) {
                old_no = old_start;
                new_no = new_start;
                hunks.push(DiffHunk {
                    header: line.clone(),
                    lines: Vec::new(),
                });
                current = Some(hunks.len() - 1);
            }
            continue;
        }

        let Some(index) = current else {
            continue;
        };
        if line.starts_with('\\') {
            continue;
        }
        let hunk = &mut hunks[index];

        if let Some(content) = line.strip_prefix('+') {
            hunk.lines.push(DiffLine {
                kind: LineType::Add,
                content: content.to_string(),
                old_no: None,
                new_no: Some(new_no),
            });
            new_no += 1;
            continue;
        }

        if let Some(content) = line.strip_prefix('-') {
            hunk.lines.push(DiffLine {
                kind: LineType::Del,
                content: content.to_string(),
                old_no: Some(old_no),
                new_no: None,
            });
            old_no += 1;
            continue;
        }

        if line.starts_with(' ') || line.is_empty() {
            let content = line.get(1..).unwrap_or("").to_string();
            hunk.lines.push(DiffLine {
                kind: LineType::Context,
                content,
                old_no: Some(old_no),
                new_no: Some(new_no),
            });
            old_no += 1;
            new_no += 1;
        }
    }

    hunks
}

fn parse_diff_sections(diff: &str) -> Vec<DiffSection> {
    let mut sections = Vec::new();
    let mut current_lines: Vec<String> = Vec::new();
    let mut current_old_path: Option<String> = None;
    let mut current_new_path: Option<String> = None;

    fn flush(
        sections: &mut Vec<DiffSection>,
        lines: &mut Vec<String>,
        old_path: &mut Option<String>,
        new_path: &mut Option<String>,
    ) {
        if new_path.is_none() && old_path.is_none() {
            lines.clear();
            return;
        }
        let Some(file_path) = new_path.clone().or_else(|| old_path.clone()) else {
            lines.clear();
            *old_path = None;
            *new_path = None;
            return;
        };
        let hunks = parse_hunks_from_diff(lines);
        if !hunks.is_empty() {
            sections.push(DiffSection {
                path: file_path.clone(),
                old_path: old_path.clone().filter(|path| path != &file_path),
                hunks,
            });
        }
        lines.clear();
        *old_path = None;
        *new_path = None;
    }

    for line in diff.split('\n') {
        if line.starts_with("diff --git ") {
            flush(
                &mut sections,
                &mut current_lines,
                &mut current_old_path,
                &mut current_new_path,
            );
            continue;
        }
        if let Some(value) = line.strip_prefix("--- ") {
            current_old_path = strip_diff_prefix(value);
            continue;
        }
        if let Some(value) = line.strip_prefix("+++ ") {
            current_new_path = strip_diff_prefix(value);
            continue;
        }
        current_lines.push(line.to_string());
    }

    flush(
        &mut sections,
        &mut current_lines,
        &mut current_old_path,
        &mut current_new_path,
    );

    sections
}

fn read_untracked_file(root: &Path, file_path: &str) -> Option<RepositoryFile> {
    let absolute_path = root.join(file_path);
    let metadata = fs::metadata(&absolute_path).ok()?;
    if !metadata.is_file() || metadata.len() > 512 * 1024 {
        return None;
    }

    let content = fs::read_to_string(&absolute_path).ok()?;
    if content.contains('\0') {
        return None;
    }

    let mut raw_lines: Vec<&str> = content.split('\n').collect();
    if raw_lines.last() == Some(&"") {
        raw_lines.pop();
    }
    let limited = &raw_lines[..raw_lines.len().min(2000)];

    let lines: Vec<DiffLine> = limited
        .iter()
        .enumerate()
        .map(|(index, line)| DiffLine {
            kind: LineType::Add,
            content: (*line).to_string(),
            old_no: None,
            new_no: Some((index + 1) as u32),
        })
        .collect();

    let header = if lines.is_empty() {
        "@@ -0,0 +0,0 @@".to_string()
    } else {
        format!("@@ -0,0 +1,{} @@", lines.len())
    };

    Some(RepositoryFile {
        path: file_path.to_string(),
        status: ChangeStatus::Added,
        language: language_for_path(file_path),
        staged: false,
        hunks: vec![DiffHunk { header, lines }],
    })
}

pub fn repository_snapshot(
    input_path: &str,
    base: Option<&str>,
) -> Result<RepositorySnapshot, CommandError> {
    let requested_dir = validate_directory(input_path)?;
    let root = probe_repository_root(&requested_dir)?.ok_or_else(|| {
        CommandError::new(
            "not_git",
            "This folder is not inside a Git repository.",
        )
    })?;
    let inspection = inspect_root(&root, true)?;
    let status_entries = read_status_entries(&root)?;
    let status_by_path: HashMap<&str, &StatusEntry> = status_entries
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect();

    let requested_base = base.unwrap_or("").trim();
    let known_branch = inspection
        .branches
        .iter()
        .any(|branch| branch == requested_base)
        || inspection
            .remote_branches
            .iter()
            .any(|branch| branch.name == requested_base);
    let diff_base = if requested_base.is_empty() || !known_branch {
        "HEAD"
    } else {
        requested_base
    };

    let mut files_by_path: HashMap<String, RepositoryFile> = HashMap::new();

    let diff = run_git(
        &[
            "-c",
            "core.quotePath=false",
            "diff",
            diff_base,
            "--no-color",
            "--no-ext-diff",
            "--unified=3",
            "--find-renames",
        ],
        &root,
    )
    .or_else(|_| {
        run_git(
            &[
                "-c",
                "core.quotePath=false",
                "diff",
                "--cached",
                "--no-color",
                "--no-ext-diff",
                "--unified=3",
                "--find-renames",
            ],
            &root,
        )
    })
    .unwrap_or_default();

    for section in parse_diff_sections(&diff) {
        let status_entry = status_by_path
            .get(section.path.as_str())
            .copied()
            .or_else(|| {
                section
                    .old_path
                    .as_deref()
                    .and_then(|old_path| status_by_path.get(old_path).copied())
            });

        let language = language_for_path(&section.path);
        files_by_path.insert(
            section.path.clone(),
            RepositoryFile {
                path: section.path,
                status: status_entry
                    .map(|entry| entry.status.clone())
                    .unwrap_or(ChangeStatus::Modified),
                language,
                staged: status_entry.map(|entry| entry.staged).unwrap_or(false),
                hunks: section.hunks,
            },
        );
    }

    for entry in &status_entries {
        if !entry.untracked || files_by_path.contains_key(&entry.path) {
            continue;
        }
        if let Some(file) = read_untracked_file(&root, &entry.path) {
            files_by_path.insert(file.path.clone(), file);
        }
    }

    let mut files: Vec<RepositoryFile> = files_by_path.into_values().collect();
    files.sort_by(|left, right| left.path.cmp(&right.path));
    files.truncate(120);

    Ok(RepositorySnapshot {
        inspection,
        files,
        conflicts: conflict_entries(&root)?,
        merge_in_progress: merge_in_progress(&root),
        fetched_at: chrono::Utc::now().to_rfc3339(),
    })
}

fn blame_header(line: &str) -> Option<(&str, u32)> {
    let mut parts = line.split(' ');
    let sha = parts.next()?;
    if !matches!(sha.len(), 40 | 64) || !sha.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    parts.next()?;
    let final_line = parts.next()?.parse::<u32>().ok()?;
    Some((sha, final_line))
}

fn parse_blame(output: &str) -> Vec<BlameLine> {
    let mut entries: Vec<BlameLine> = Vec::new();
    let mut current: Option<BlameLine> = None;
    let mut seen: std::collections::HashSet<u32> = std::collections::HashSet::new();

    for line in output.split('\n') {
        if line.starts_with('\t') {
            if let Some(entry) = current.take() {
                if seen.insert(entry.line) {
                    entries.push(entry);
                }
            }
            continue;
        }

        if let Some((sha, final_line)) = blame_header(line) {
            if let Some(entry) = current.take() {
                if seen.insert(entry.line) {
                    entries.push(entry);
                }
            }

            let uncommitted = sha.bytes().all(|byte| byte == b'0');
            current = Some(BlameLine {
                line: final_line,
                author: String::new(),
                author_mail: String::new(),
                time: 0,
                commit: sha.to_string(),
                summary: String::new(),
                uncommitted,
            });
            continue;
        }

        let Some(entry) = current.as_mut() else {
            continue;
        };

        if let Some(value) = line.strip_prefix("author ") {
            entry.author = value.to_string();
        } else if let Some(value) = line.strip_prefix("author-mail ") {
            entry.author_mail = value
                .trim()
                .trim_start_matches('<')
                .trim_end_matches('>')
                .to_string();
        } else if let Some(value) = line.strip_prefix("author-time ") {
            entry.time = value.trim().parse::<i64>().unwrap_or(0);
        } else if let Some(value) = line.strip_prefix("summary ") {
            entry.summary = value.to_string();
        }
    }

    if let Some(entry) = current.take() {
        if seen.insert(entry.line) {
            entries.push(entry);
        }
    }

    entries.sort_by_key(|entry| entry.line);
    entries
}

pub fn blame_file(
    input_path: &str,
    file_path: &str,
    revision: Option<&str>,
    lines: Option<Vec<u32>>,
) -> Result<Vec<BlameLine>, CommandError> {
    let relative_path = file_path.trim();
    if relative_path.is_empty() {
        return Err(CommandError::new(
            "bad_request",
            "A file path is required.",
        ));
    }

    let revision = revision.map(str::trim).filter(|value| !value.is_empty());
    if matches!(revision, Some(value) if value.starts_with('-')) {
        return Err(CommandError::new("bad_request", "Invalid revision."));
    }

    let root = expand_home(input_path.trim());
    let mut args: Vec<&str> = vec!["-c", "core.quotePath=false", "blame", "--line-porcelain"];
    if let Some(value) = revision {
        args.push(value);
    }
    args.push("--");
    args.push(relative_path);

    let output = run_git(&args, &root)?;
    let mut entries = parse_blame(&output);

    if let Some(requested) = lines {
        if !requested.is_empty() {
            let wanted: std::collections::HashSet<u32> = requested.into_iter().collect();
            entries.retain(|entry| wanted.contains(&entry.line));
        }
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::{
        commit_detail, complete_merge, conflict_entries, conflict_kind, discover_repositories,
        initialize_repository, inspect_root, list_remotes, parse_blame, parse_conflict_markers,
        pull_repository, push_info, push_repository, read_conflict, resolve_conflict_side,
        save_conflict_resolution, scan_repositories, set_remote_url, should_skip_directory,
        suggested_side, ConflictSegment, ConflictSide, PushOptions,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn git_in(directory: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(directory)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn configure_user(directory: &Path) {
        git_in(directory, &["config", "user.name", "FlashBee Test"]);
        git_in(directory, &["config", "user.email", "test@flashbee.local"]);
    }

    fn commit_file(directory: &Path, file: &str, contents: &str, message: &str) {
        fs::write(directory.join(file), contents).unwrap();
        git_in(directory, &["add", file]);
        git_in(directory, &["commit", "-m", message]);
    }

    fn temp_root(label: &str) -> PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "flashbee-{label}-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn skips_heavyweight_and_hidden_directories() {
        assert!(should_skip_directory("node_modules"));
        assert!(should_skip_directory(".git"));
        assert!(should_skip_directory("target"));
        assert!(!should_skip_directory("projects"));
    }

    #[test]
    fn discovers_nested_repositories_and_skips_ignored_folders() {
        let root = temp_root("discovery");
        fs::create_dir_all(root.join("first/.git")).unwrap();
        fs::create_dir_all(root.join("nested/second")).unwrap();
        fs::write(root.join("nested/second/.git"), "gitdir: elsewhere").unwrap();
        fs::create_dir_all(root.join("node_modules/ignored/.git")).unwrap();
        fs::create_dir_all(root.join(".hidden/ignored/.git")).unwrap();
        fs::create_dir_all(root.join(".dotfiles/.git")).unwrap();
        fs::create_dir_all(root.join("nested/plain")).unwrap();

        let (repositories, truncated) = scan_repositories(&root);
        let names: Vec<&str> = repositories
            .iter()
            .map(|repository| repository.name.as_str())
            .collect();

        assert_eq!(names, vec![".dotfiles", "first", "second"]);
        assert!(!truncated);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn initializes_a_plain_folder_as_a_repository() {
        let root = temp_root("init");
        let project = root.join("project");
        fs::create_dir_all(&project).unwrap();

        let inspection = initialize_repository(&project.to_string_lossy()).unwrap();

        assert!(project.join(".git").exists());
        assert_eq!(inspection.name, "project");
        assert!(!inspection.current_branch.is_empty());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn discovers_repositories_below_a_plain_folder() {
        let root = temp_root("folder");
        let child = root.join("child");
        fs::create_dir_all(&child).unwrap();
        assert!(Command::new("git")
            .arg("init")
            .arg(&child)
            .output()
            .unwrap()
            .status
            .success());

        let discovery = discover_repositories(&root.to_string_lossy()).unwrap();
        assert!(!discovery.is_repository);
        assert_eq!(discovery.repositories.len(), 1);
        assert_eq!(discovery.repositories[0].name, "child");

        let child_discovery = discover_repositories(&child.to_string_lossy()).unwrap();
        assert!(child_discovery.is_repository);
        assert_eq!(child_discovery.repositories.len(), 1);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn reports_linked_worktrees_and_skips_missing_ones() {
        let root = temp_root("worktrees");
        let project = root.join("project");
        fs::create_dir_all(&project).unwrap();
        git_in(&project, &["init"]);
        configure_user(&project);
        commit_file(&project, "README.md", "hello\n", "initial");

        let checkout = root.join("checkout");
        git_in(
            &project,
            &[
                "worktree",
                "add",
                "-b",
                "feature",
                checkout.to_string_lossy().as_ref(),
            ],
        );

        let inspection = inspect_root(&project, true).unwrap();
        assert_eq!(inspection.worktrees.len(), 2);
        assert!(inspection.worktrees[0].is_main);
        assert!(inspection.worktrees[0].is_current);

        let linked = inspection
            .worktrees
            .iter()
            .find(|worktree| worktree.name == "checkout")
            .expect("linked worktree should be listed");
        assert_eq!(linked.branch, "feature");
        assert!(!linked.is_main);
        assert!(!linked.is_current);
        assert!(!linked.detached);
        assert!(!linked.head.is_empty());

        // A deleted worktree folder must not leave a dead entry behind.
        fs::remove_dir_all(&checkout).unwrap();
        let inspection = inspect_root(&project, true).unwrap();
        assert_eq!(inspection.worktrees.len(), 1);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn parses_porcelain_blame_entries() {
        let output = concat!(
            "9a849d1da78ae11d8fea2581ed98cd786ea0b464 31 30 1\n",
            "author wangzhuo\n",
            "author-mail <wxwzmail@gmail.com>\n",
            "author-time 1795157239\n",
            "summary feat: build multi-repository Git viewer\n",
            "filename src/components/DiffViewer.tsx\n",
            "\t  onModeChange: (mode: DiffMode) => void\n",
            "0000000000000000000000000000000000000000 32 31 1\n",
            "author Not Committed Yet\n",
            "author-mail <not.committed.yet>\n",
            "author-time 1795157999\n",
            "summary Version of src/components/DiffViewer.tsx from the working tree\n",
            "filename src/components/DiffViewer.tsx\n",
            "\t+ added line\n"
        );

        let entries = parse_blame(output);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].line, 30);
        assert_eq!(entries[0].author, "wangzhuo");
        assert_eq!(entries[0].author_mail, "wxwzmail@gmail.com");
        assert_eq!(entries[0].time, 1795157239);
        assert!(!entries[0].uncommitted);
        assert!(entries[1].uncommitted);
        assert_eq!(entries[1].summary, "Version of src/components/DiffViewer.tsx from the working tree");
    }

    #[test]
    fn parses_diff3_conflict_segments() {
        let merged = concat!(
            "first\n",
            "<<<<<<< ours\n",
            "left line\n",
            "||||||| base\n",
            "base line\n",
            "=======\n",
            "right line\n",
            ">>>>>>> theirs\n",
            "last\n"
        );

        let segments = parse_conflict_markers(merged);
        assert_eq!(segments.len(), 3);

        match &segments[0] {
            ConflictSegment::Common { lines } => assert_eq!(lines, &vec!["first".to_string()]),
            _ => panic!("expected common segment"),
        }
        match &segments[1] {
            ConflictSegment::Conflict {
                ours,
                base,
                theirs,
                suggestion,
            } => {
                assert_eq!(ours, &vec!["left line".to_string()]);
                assert_eq!(base, &vec!["base line".to_string()]);
                assert_eq!(theirs, &vec!["right line".to_string()]);
                assert_eq!(*suggestion, None);
            }
            _ => panic!("expected conflict segment"),
        }
        match &segments[2] {
            ConflictSegment::Common { lines } => assert_eq!(lines, &vec!["last".to_string()]),
            _ => panic!("expected common segment"),
        }
    }

    #[test]
    fn suggests_the_side_that_actually_changed() {
        let base = vec!["a".to_string()];
        assert_eq!(
            suggested_side(&base, &base, &["b".to_string()]),
            Some(ConflictSide::Theirs)
        );
        assert_eq!(
            suggested_side(&["b".to_string()], &base, &base),
            Some(ConflictSide::Ours)
        );
        assert_eq!(
            suggested_side(&["b".to_string()], &base, &["c".to_string()]),
            None
        );
    }

    #[test]
    fn treats_identical_sides_as_common_content() {
        let merged = concat!(
            "<<<<<<< ours\n",
            "same\n",
            "||||||| base\n",
            "old\n",
            "=======\n",
            "same\n",
            ">>>>>>> theirs\n"
        );

        let segments = parse_conflict_markers(merged);
        assert_eq!(segments.len(), 1);
        match &segments[0] {
            ConflictSegment::Common { lines } => assert_eq!(lines, &vec!["same".to_string()]),
            _ => panic!("expected common segment"),
        }
    }

    #[test]
    fn classifies_unmerged_index_stages() {
        assert_eq!(conflict_kind(true, true, true), "bothModified");
        assert_eq!(conflict_kind(false, true, true), "bothAdded");
        assert_eq!(conflict_kind(true, false, true), "deletedByUs");
        assert_eq!(conflict_kind(true, true, false), "deletedByThem");
    }

    #[test]
    fn pushes_pulls_and_resolves_a_real_conflict() {
        let root = temp_root("sync");
        let remote = root.join("origin.git");
        let repository = root.join("repository");
        let other = root.join("other");
        fs::create_dir_all(&remote).unwrap();
        fs::create_dir_all(&repository).unwrap();

        git_in(&remote, &["init", "--bare", "-b", "main"]);
        git_in(&repository, &["init", "-b", "main"]);
        configure_user(&repository);
        commit_file(&repository, "conflict.txt", "first\nbase\nlast\n", "base");
        git_in(
            &repository,
            &["remote", "add", "origin", remote.to_str().unwrap()],
        );

        let remotes = list_remotes(repository.to_str().unwrap()).unwrap();
        assert_eq!(remotes.len(), 1);
        assert_eq!(remotes[0].name, "origin");

        // No upstream yet: push establishes it and uploads main.
        let pushed = push_repository(repository.to_str().unwrap(), None).unwrap();
        assert!(pushed.conflicts.is_empty());
        let inspection = inspect_root(&repository, true).unwrap();
        assert_eq!(inspection.upstream.as_deref(), Some("origin/main"));
        assert!(inspection
            .remote_branches
            .iter()
            .any(|branch| branch.name == "origin/main"));
        assert_eq!((inspection.ahead, inspection.behind), (0, 0));

        // Another clone moves the remote forward.
        git_in(
            &root,
            &[
                "clone",
                "-b",
                "main",
                remote.to_str().unwrap(),
                other.to_str().unwrap(),
            ],
        );
        configure_user(&other);
        commit_file(&other, "readme.txt", "remote update\n", "remote update");
        git_in(&other, &["push", "origin", "main"]);

        let pulled = pull_repository(repository.to_str().unwrap()).unwrap();
        assert!(pulled.conflicts.is_empty());
        assert_eq!(
            fs::read_to_string(repository.join("readme.txt")).unwrap(),
            "remote update\n"
        );

        // Both sides now change the same line: pull must stop on a conflict.
        commit_file(&other, "conflict.txt", "first\ntheirs\nlast\n", "theirs");
        git_in(&other, &["push", "origin", "main"]);
        commit_file(&repository, "conflict.txt", "first\nours\nlast\n", "ours");

        let conflicted = pull_repository(repository.to_str().unwrap()).unwrap();
        assert_eq!(conflicted.conflicts, vec!["conflict.txt".to_string()]);

        let entries = conflict_entries(&repository).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].kind, "bothModified");

        let detail = read_conflict(repository.to_str().unwrap(), "conflict.txt").unwrap();
        assert!(!detail.binary);
        assert!(detail.ours_exists && detail.theirs_exists && detail.base_exists);
        let conflict = detail
            .segments
            .iter()
            .find_map(|segment| match segment {
                ConflictSegment::Conflict {
                    ours,
                    base,
                    theirs,
                    suggestion,
                } => Some((ours, base, theirs, suggestion)),
                _ => None,
            })
            .expect("conflict block");
        assert_eq!(conflict.0, &vec!["ours".to_string()]);
        assert_eq!(conflict.1, &vec!["base".to_string()]);
        assert_eq!(conflict.2, &vec!["theirs".to_string()]);
        assert_eq!(*conflict.3, None);

        save_conflict_resolution(
            repository.to_str().unwrap(),
            "conflict.txt",
            "first\nmerged\nlast\n",
        )
        .unwrap();
        assert!(conflict_entries(&repository).unwrap().is_empty());

        let completed = complete_merge(repository.to_str().unwrap()).unwrap();
        assert!(completed.conflicts.is_empty());
        assert_eq!(
            fs::read_to_string(repository.join("conflict.txt")).unwrap(),
            "first\nmerged\nlast\n"
        );
        assert!(!repository.join(".git/MERGE_HEAD").exists());

        // Updating an existing remote URL is idempotent and visible.
        let updated = set_remote_url(
            repository.to_str().unwrap(),
            "origin",
            remote.to_str().unwrap(),
        )
        .unwrap();
        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].url, remote.to_str().unwrap());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn ignores_pull_without_an_upstream() {
        let root = temp_root("no-upstream");
        let repository = root.join("repository");
        fs::create_dir_all(&repository).unwrap();
        git_in(&repository, &["init", "-b", "main"]);
        configure_user(&repository);
        commit_file(&repository, "file.txt", "content\n", "initial");

        let error = pull_repository(repository.to_str().unwrap()).unwrap_err();
        assert_eq!(error.code, "no_remote");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn resolves_a_delete_modify_conflict_by_deleting_the_file() {
        let root = temp_root("delete-conflict");
        let repository = root.join("repository");
        fs::create_dir_all(&repository).unwrap();
        git_in(&repository, &["init", "-b", "main"]);
        configure_user(&repository);
        commit_file(&repository, "file.txt", "content\n", "initial");

        git_in(&repository, &["checkout", "-b", "remove"]);
        git_in(&repository, &["rm", "file.txt"]);
        git_in(&repository, &["commit", "-m", "remove file"]);
        git_in(&repository, &["checkout", "main"]);
        commit_file(&repository, "file.txt", "changed\n", "change file");

        let merge = Command::new("git")
            .args(["merge", "remove"])
            .current_dir(&repository)
            .output()
            .unwrap();
        assert!(!merge.status.success());

        let entries = conflict_entries(&repository).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].kind, "deletedByThem");
        assert!(!entries[0].theirs);

        let detail = read_conflict(repository.to_str().unwrap(), "file.txt").unwrap();
        assert!(detail.ours_exists);
        assert!(!detail.theirs_exists);

        // Choosing the deleted side must remove the file, not leave an empty one.
        resolve_conflict_side(repository.to_str().unwrap(), "file.txt", "theirs").unwrap();
        assert!(conflict_entries(&repository).unwrap().is_empty());
        assert!(!repository.join("file.txt").exists());
        complete_merge(repository.to_str().unwrap()).unwrap();

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn describes_and_selectively_pushes_commits_and_tags() {
        let root = temp_root("push-dialog");
        let remote = root.join("origin.git");
        let repository = root.join("repository");
        fs::create_dir_all(&remote).unwrap();
        fs::create_dir_all(&repository).unwrap();

        git_in(&remote, &["init", "--bare", "-b", "main"]);
        git_in(&repository, &["init", "-b", "main"]);
        configure_user(&repository);
        commit_file(&repository, "base.txt", "base\n", "base");
        git_in(
            &repository,
            &["remote", "add", "origin", remote.to_str().unwrap()],
        );
        git_in(&repository, &["push", "-u", "origin", "main"]);

        commit_file(&repository, "second.txt", "second\n", "second");
        commit_file(&repository, "third.txt", "third\n", "third");
        git_in(&repository, &["tag", "v1"]);

        let path = repository.to_str().unwrap();
        let info = push_info(path).unwrap();
        assert_eq!(info.remote, "origin");
        assert_eq!(info.remote_branch, "main");
        assert_eq!(info.ahead, 2);
        assert_eq!(info.commits.len(), 2);
        assert_eq!(info.commits[0].subject, "third");
        assert!(info.tags.iter().any(|tag| tag == "v1"));

        let detail = commit_detail(path, &info.commits[0].hash).unwrap();
        assert_eq!(detail.subject, "third");
        assert!(detail
            .files
            .iter()
            .any(|file| file.path == "third.txt"));

        let remote_head = git_in(&remote, &["rev-parse", "main"]);

        // Pushing only a tag must leave the remote branch untouched.
        push_repository(
            path,
            Some(PushOptions {
                push_tags: true,
                skip_commits: true,
                tag: Some("v1".to_string()),
                ..PushOptions::default()
            }),
        )
        .unwrap();
        assert_eq!(git_in(&remote, &["rev-parse", "main"]), remote_head);
        assert_eq!(
            git_in(&remote, &["rev-parse", "refs/tags/v1"]),
            git_in(&repository, &["rev-parse", "refs/tags/v1"])
        );

        // Pushing the older selected commit moves the remote to that commit.
        let selected = info.commits[1].hash.clone();
        push_repository(
            path,
            Some(PushOptions {
                commit: Some(selected.clone()),
                ..PushOptions::default()
            }),
        )
        .unwrap();
        assert_eq!(git_in(&remote, &["rev-parse", "main"]), selected);

        let remaining = push_info(path).unwrap();
        assert_eq!(remaining.commits.len(), 1);
        assert_eq!(remaining.commits[0].subject, "third");

        // A normal push publishes the tip and clears the ahead count.
        push_repository(path, None).unwrap();
        assert_eq!(
            git_in(&remote, &["rev-parse", "main"]),
            git_in(&repository, &["rev-parse", "HEAD"])
        );
        assert_eq!(push_info(path).unwrap().ahead, 0);

        fs::remove_dir_all(&root).unwrap();
    }
}
