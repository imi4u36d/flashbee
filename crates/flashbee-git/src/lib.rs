use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Write as _;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
    /// Optional machine-readable companion, such as the worktree path that
    /// owns a branch or the branch that failed to merge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl CommandError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for CommandError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteBranchInfo {
    pub name: String,
    pub remote: String,
    pub branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
    /// Stable identity for selection: `staged:<path>`, `unstaged:<path>`, or
    /// `untracked:<path>`. A partially staged file appears twice.
    pub id: String,
    pub path: String,
    /// Previous path for renames and deletions.
    pub old_path: Option<String>,
    pub status: ChangeStatus,
    pub language: String,
    /// The change shown is between the index and the comparison base.
    pub staged: bool,
    /// The change shown is between the working tree and the index.
    pub unstaged: bool,
    pub untracked: bool,
    pub binary: bool,
    /// The file is a Git LFS pointer rather than its real content.
    pub lfs: bool,
    /// The diff was shortened before being sent to the UI.
    pub truncated: bool,
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
pub struct CommitOutcome {
    pub hash: String,
    pub short_hash: String,
    pub subject: String,
    pub files_changed: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchSwitchOutcome {
    pub branch: String,
    /// Uncommitted tracked changes were moved into a stash first.
    pub stashed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub remote: Option<String>,
    pub upstream: Option<String>,
    /// Commits on this branch that are not on HEAD.
    pub ahead: u32,
    /// Commits on HEAD that are not on this branch.
    pub behind: u32,
    pub short_commit: String,
    pub subject: String,
    pub time: i64,
    /// Checkout that currently has this branch checked out, if any.
    pub worktree_path: Option<String>,
    /// The branch is already contained in the current HEAD.
    pub merged: bool,
}

#[derive(Debug, Clone)]
pub struct WorktreeAddOptions {
    pub path: String,
    pub branch: Option<String>,
    pub new_branch: Option<String>,
    pub start_point: Option<String>,
    pub detach: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeSummary {
    #[serde(flatten)]
    pub info: WorktreeInfo,
    pub changed: u32,
    pub conflicts: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StashEntry {
    pub index: u32,
    pub reference: String,
    pub message: String,
    pub branch: String,
    pub time: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogCommit {
    pub hash: String,
    pub short_hash: String,
    pub parents: Vec<String>,
    pub author: String,
    pub email: String,
    pub time: i64,
    pub subject: String,
    pub refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareDiff {
    pub base: String,
    pub target: String,
    pub files: Vec<RepositoryFile>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSearchHit {
    pub path: String,
    pub line: u32,
    pub preview: String,
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
    old_path: Option<String>,
    index_status: char,
    worktree_status: char,
    untracked: bool,
}

impl StatusEntry {
    fn staged_status(&self) -> Option<ChangeStatus> {
        if self.untracked {
            return None;
        }
        status_from_char(self.index_status)
    }

    fn unstaged_status(&self) -> Option<ChangeStatus> {
        if self.untracked {
            return Some(ChangeStatus::Added);
        }
        status_from_char(self.worktree_status)
    }
}

#[derive(Debug, Clone)]
struct DiffSection {
    path: String,
    old_path: Option<String>,
    binary: bool,
    hunks: Vec<DiffHunk>,
}

/// A diff section kept as raw lines so a single hunk can be rebuilt into a
/// patch for `git apply`.
#[derive(Debug, Clone)]
struct RawHunk {
    header: String,
    lines: Vec<String>,
}

#[derive(Debug, Clone)]
struct RawSection {
    new_path: Option<String>,
    old_path: Option<String>,
    header_lines: Vec<String>,
    hunks: Vec<RawHunk>,
    binary: bool,
}

impl RawSection {
    fn path(&self) -> Option<&str> {
        self.new_path.as_deref().or(self.old_path.as_deref())
    }
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

fn status_from_char(code: char) -> Option<ChangeStatus> {
    match code {
        'A' | '?' => Some(ChangeStatus::Added),
        'D' => Some(ChangeStatus::Deleted),
        'R' | 'C' => Some(ChangeStatus::Renamed),
        'M' | 'T' | 'U' => Some(ChangeStatus::Modified),
        _ => None,
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
        CommandError::new("git_unavailable", "Git is not installed or not available.")
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

fn run_git_with_input(args: &[&str], cwd: &Path, input: &str) -> Result<GitOutput, CommandError> {
    let mut child = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(git_not_found)?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .map_err(|error| CommandError::new("git_failed", error.to_string()))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|error| CommandError::new("git_failed", error.to_string()))?;
    Ok(GitOutput {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    })
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
    let metadata = fs::metadata(&requested)
        .map_err(|_| CommandError::new("not_found", "This folder does not exist."))?;
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
        return Err(CommandError::new("bad_request", "A file path is required."));
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
    probe_repository_root(&requested)?
        .ok_or_else(|| CommandError::new("not_git", "This folder is not inside a Git repository."))
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
                CommandError::new("git_unavailable", "Git is not installed or not available.")
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
            "--format=%(refname)%09%(symref)",
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
        let (full_ref, symref) = line.split_once('\t').unwrap_or((line, ""));
        // `origin/HEAD` is a symbolic alias of a real branch, not a branch itself.
        if !symref.is_empty() || full_ref.ends_with("/HEAD") {
            continue;
        }
        // Build the display name from the full ref: abbreviated names become
        // `remotes/origin/main` once a local branch shadows `origin/main`.
        let Some(name) = full_ref.strip_prefix("refs/remotes/") else {
            continue;
        };
        let Some((remote, branch)) = split_remote_branch(name, remotes) else {
            continue;
        };

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

/// The remote-tracking branch the current branch follows. Resolved from the
/// branch config rather than from Git's abbreviated names: those turn into
/// `remotes/origin/main` as soon as a local branch shadows `origin/main`, and
/// splitting that string by hand invents a remote named `remotes`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct UpstreamTracking {
    remote: String,
    branch: String,
}

impl UpstreamTracking {
    /// Unambiguous ref used for revision ranges.
    fn reference(&self) -> String {
        format!("refs/remotes/{}/{}", self.remote, self.branch)
    }

    fn display(&self) -> String {
        format!("{}/{}", self.remote, self.branch)
    }
}

fn read_upstream(root_path: &Path, remotes: &[RemoteInfo]) -> Option<UpstreamTracking> {
    let branch = run_git(&["branch", "--show-current"], root_path)
        .unwrap_or_default()
        .trim()
        .to_string();
    if !branch.is_empty() {
        let remote = run_git(
            &["config", "--get", &format!("branch.{branch}.remote")],
            root_path,
        )
        .unwrap_or_default();
        let merge = run_git(
            &["config", "--get", &format!("branch.{branch}.merge")],
            root_path,
        )
        .unwrap_or_default();
        // `.` means the branch tracks another local branch, not a remote.
        if !remote.is_empty()
            && remote != "."
            && let Some(tracked) = merge.trim().strip_prefix("refs/heads/")
            && !tracked.is_empty()
        {
            return Some(UpstreamTracking {
                remote,
                branch: tracked.to_string(),
            });
        }
    }

    // Fall back to Git's resolution when the config is missing or unexpected.
    // `--symbolic-full-name` keeps the ref unambiguous.
    let full_ref = run_git(
        &["rev-parse", "--symbolic-full-name", "@{upstream}"],
        root_path,
    )
    .ok()?;
    let name = full_ref.trim().strip_prefix("refs/remotes/")?;
    let (remote, branch) = split_remote_branch(name, remotes)?;
    Some(UpstreamTracking { remote, branch })
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
    let behind = parts
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let ahead = parts
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    (ahead, behind)
}

fn inspect_root(root_path: &Path, detailed: bool) -> Result<RepositoryInspection, CommandError> {
    let current_branch_raw = run_git(&["branch", "--show-current"], root_path).unwrap_or_default();
    let branches_raw = run_git(
        &["for-each-ref", "--format=%(refname)", "refs/heads"],
        root_path,
    )
    .unwrap_or_default();

    // Strip the full ref prefix ourselves so a branch named `origin/main`
    // stays `origin/main` instead of collapsing into `heads/origin/main`.
    let mut local_branches: Vec<String> = branches_raw
        .lines()
        .map(str::trim)
        .filter_map(|branch| branch.strip_prefix("refs/heads/"))
        .map(str::to_string)
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
        let tracking = read_upstream(root_path, &remotes);
        let (ahead, behind) = tracking
            .as_ref()
            .map(|tracking| read_ahead_behind(root_path, &tracking.reference()))
            .unwrap_or((0, 0));
        (
            remote_branches,
            remotes,
            tracking.map(|tracking| tracking.display()),
            ahead,
            behind,
        )
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
        CommandError::new("not_git", "This folder is not inside a Git repository.")
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
        CommandError::new("not_git", "This folder is not inside a Git repository.")
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
            while cursor < lines.len() && !is_conflict_marker(&lines[cursor], ">>>>>>> theirs") {
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

fn merge_conflict_markers(ours: &[u8], base: &[u8], theirs: &[u8]) -> Result<String, CommandError> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or(0);
    let directory =
        std::env::temp_dir().join(format!("flashbee-merge-{}-{stamp}", std::process::id()));
    fs::create_dir_all(&directory)
        .map_err(|error| CommandError::new("io_failed", error.to_string()))?;

    let ours_path = directory.join("ours");
    let base_path = directory.join("base");
    let theirs_path = directory.join("theirs");

    let write_error = |error: std::io::Error| CommandError::new("io_failed", error.to_string());
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
            ));
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
                &["rm", "-f", "--ignore-unmatch", "--", relative_path.as_str()],
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
    if !name
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
    {
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

/// Splits `origin/main` — or Git's ambiguous abbreviation
/// `remotes/origin/main` — into a configured remote and its branch. Longer
/// remote names win so a remote such as `team/fork` is not cut down to `team`.
fn split_remote_branch(value: &str, remotes: &[RemoteInfo]) -> Option<(String, String)> {
    let value = value.trim();
    let value = value.strip_prefix("refs/remotes/").unwrap_or(value);
    if value.is_empty() {
        return None;
    }
    if let Some(parts) = known_remote_branch(value, remotes) {
        return Some(parts);
    }
    let ambiguous = value.strip_prefix("remotes/")?;
    known_remote_branch(ambiguous, remotes).or_else(|| {
        ambiguous
            .split_once('/')
            .filter(|(remote, branch)| !remote.is_empty() && !branch.is_empty())
            .map(|(remote, branch)| (remote.to_string(), branch.to_string()))
    })
}

fn known_remote_branch(value: &str, remotes: &[RemoteInfo]) -> Option<(String, String)> {
    let remote = remotes
        .iter()
        .map(|remote| remote.name.as_str())
        .filter(|name| {
            value.len() > name.len()
                && value.starts_with(name)
                && value.as_bytes().get(name.len()) == Some(&b'/')
        })
        .max_by_key(|name| name.len())?;
    let branch = &value[remote.len() + 1..];
    if branch.is_empty() {
        return None;
    }
    Some((remote.to_string(), branch.to_string()))
}

/// Turns a full tracking ref into the short form shown in the UI.
fn display_tracking_ref(value: &str) -> String {
    value
        .strip_prefix("refs/remotes/")
        .or_else(|| value.strip_prefix("refs/heads/"))
        .unwrap_or(value)
        .to_string()
}

fn remote_tracking_ref_exists(root: &Path, remote: &str, branch: &str) -> bool {
    let reference = format!("refs/remotes/{remote}/{branch}");
    run_git(
        &["rev-parse", "--verify", "--quiet", reference.as_str()],
        root,
    )
    .is_ok()
}

/// The ref the local branch is compared against. The upstream ref wins;
/// otherwise use the remote branch with the same name, the remote HEAD, or its
/// default branch. `None` means the remote is empty and every commit is new.
fn push_base(
    root: &Path,
    upstream_ref: Option<&str>,
    remote: &str,
    remote_branch: &str,
) -> Option<String> {
    if let Some(upstream_ref) = upstream_ref {
        return Some(upstream_ref.to_string());
    }
    if remote_tracking_ref_exists(root, remote, remote_branch) {
        return Some(format!("refs/remotes/{remote}/{remote_branch}"));
    }
    if remote_tracking_ref_exists(root, remote, "HEAD") {
        return Some(format!("refs/remotes/{remote}/HEAD"));
    }
    for fallback in ["main", "master"] {
        if remote_tracking_ref_exists(root, remote, fallback) {
            return Some(format!("refs/remotes/{remote}/{fallback}"));
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

    let upstream_parts = inspection
        .upstream
        .as_deref()
        .and_then(|value| split_remote_branch(value, &inspection.remotes));
    let (remote, remote_branch) = upstream_parts.clone().unwrap_or_else(|| {
        (
            preferred_remote(&inspection),
            inspection.current_branch.clone(),
        )
    });
    // Compare against the full tracking ref: the abbreviated form may resolve
    // to a local branch that happens to share the name.
    let upstream_ref = upstream_parts
        .as_ref()
        .map(|(remote, branch)| format!("refs/remotes/{remote}/{branch}"));
    let base = push_base(&root, upstream_ref.as_deref(), &remote, &remote_branch);
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
        && trimmed
            .chars()
            .all(|character| character.is_ascii_hexdigit())
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
                .unwrap_or_else(|| infer_status(&section));
            let path = section.path.clone();
            let language = language_for_path(&path);
            RepositoryFile {
                id: path.clone(),
                path,
                old_path: section.old_path,
                status,
                language,
                staged: false,
                unstaged: false,
                untracked: false,
                binary: section.binary,
                lfs: worktree_file_is_lfs(&root, &section.path),
                truncated: false,
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
    let upstream_parts = inspection
        .upstream
        .as_deref()
        .and_then(|value| split_remote_branch(value, &inspection.remotes));
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
            let ancestor =
                run_git_capture(&["merge-base", "--is-ancestor", commit, "HEAD"], &root, &[])?;
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
            let valid =
                run_git_capture(&["check-ref-format", reference.as_str()], &root, &[])?.success;
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
        let tracking = format!("--set-upstream-to=refs/remotes/{remote}/{remote_branch}");
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
        if value.is_empty() || value.len() < 4 {
            continue;
        }
        let code = &value[..2];
        let mut chars = code.chars();
        let index_status = chars.next().unwrap_or(' ');
        let worktree_status = chars.next().unwrap_or(' ');
        let first_path = value.get(3..).unwrap_or("").to_string();
        let mut file_path = first_path.clone();
        let mut old_path = None;

        if code.contains('R') || code.contains('C') {
            if let Some(second_path) = fields.next() {
                if !second_path.is_empty() {
                    // Porcelain v1 emits the original path first in -z mode.
                    old_path = Some(first_path);
                    file_path = second_path.to_string();
                }
            }
        }

        entries.push(StatusEntry {
            path: file_path,
            old_path,
            index_status,
            worktree_status,
            untracked: code == "??",
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

fn parse_raw_sections(diff: &str) -> Vec<RawSection> {
    let mut sections: Vec<RawSection> = Vec::new();
    let mut current: Option<RawSection> = None;
    let mut current_hunk: Option<RawHunk> = None;

    fn flush_hunk(section: &mut Option<RawSection>, hunk: &mut Option<RawHunk>) {
        if let (Some(section), Some(hunk)) = (section.as_mut(), hunk.take()) {
            section.hunks.push(hunk);
        }
    }

    fn flush_section(
        sections: &mut Vec<RawSection>,
        section: &mut Option<RawSection>,
        hunk: &mut Option<RawHunk>,
    ) {
        flush_hunk(section, hunk);
        if let Some(section) = section.take() {
            if section.path().is_some() {
                sections.push(section);
            }
        }
    }

    for line in diff.split('\n') {
        if line.starts_with("diff --git ") {
            flush_section(&mut sections, &mut current, &mut current_hunk);
            current = Some(RawSection {
                new_path: None,
                old_path: None,
                header_lines: vec![line.to_string()],
                hunks: Vec::new(),
                binary: false,
            });
            continue;
        }

        if line.starts_with("diff --cc ") || line.starts_with("diff --combined ") {
            // Combined (unmerged) diffs are not line-addressable; the conflict
            // resolver reads the index stages instead.
            flush_section(&mut sections, &mut current, &mut current_hunk);
            continue;
        }

        let Some(section) = current.as_mut() else {
            continue;
        };

        if line.starts_with("@@ ") {
            flush_hunk(&mut current, &mut current_hunk);
            current_hunk = Some(RawHunk {
                header: line.to_string(),
                lines: vec![line.to_string()],
            });
            continue;
        }

        if let Some(hunk) = current_hunk.as_mut() {
            hunk.lines.push(line.to_string());
            continue;
        }

        section.header_lines.push(line.to_string());
        if line.starts_with("Binary files ") || line.starts_with("GIT binary patch") {
            section.binary = true;
        }
        if let Some(value) = line.strip_prefix("--- ") {
            section.old_path = strip_diff_prefix(value);
        }
        if let Some(value) = line.strip_prefix("+++ ") {
            section.new_path = strip_diff_prefix(value);
        }
    }

    flush_section(&mut sections, &mut current, &mut current_hunk);
    sections
}

fn parse_diff_sections(diff: &str) -> Vec<DiffSection> {
    parse_raw_sections(diff)
        .into_iter()
        .filter_map(|raw| {
            let path = raw.path()?.to_string();
            let old_path = raw.old_path.filter(|value| value != &path);
            let lines: Vec<String> = raw
                .hunks
                .iter()
                .flat_map(|hunk| hunk.lines.iter().cloned())
                .collect();
            let hunks = parse_hunks_from_diff(&lines);
            if hunks.is_empty() && !raw.binary {
                return None;
            }
            Some(DiffSection {
                path,
                old_path,
                binary: raw.binary,
                hunks,
            })
        })
        .collect()
}

/// Rebuilds a single hunk into a patch that `git apply` can consume.
fn hunk_patch(diff: &str, file_path: &str, hunk_header: &str) -> Option<String> {
    let section = parse_raw_sections(diff)
        .into_iter()
        .find(|section| section.path() == Some(file_path))?;
    let hunk = section
        .hunks
        .iter()
        .find(|hunk| hunk.header == hunk_header)?;
    if section.binary {
        return None;
    }
    let mut patch = section.header_lines.join("\n");
    if !patch.ends_with('\n') {
        patch.push('\n');
    }
    patch.push_str(&hunk.lines.join("\n"));
    patch.push('\n');
    Some(patch)
}

fn is_lfs_pointer(content: &str) -> bool {
    content
        .lines()
        .next()
        .is_some_and(|line| line.trim() == "version https://git-lfs.github.com/spec/v1")
}

fn read_untracked_file(root: &Path, file_path: &str) -> Option<RepositoryFile> {
    let absolute_path = root.join(file_path);
    let metadata = fs::metadata(&absolute_path).ok()?;
    if !metadata.is_file() {
        return None;
    }

    const MAX_BYTES: u64 = 512 * 1024;
    let oversized = metadata.len() > MAX_BYTES;
    let bytes = if oversized {
        use std::io::Read as _;
        let mut file = fs::File::open(&absolute_path).ok()?;
        let mut sample = vec![0u8; 8192];
        let read = file.read(&mut sample).ok()?;
        sample.truncate(read);
        sample
    } else {
        fs::read(&absolute_path).ok()?
    };

    let binary = bytes.iter().take(8192).any(|byte| *byte == 0);
    let content = if binary {
        None
    } else {
        String::from_utf8(bytes).ok()
    };
    let lfs = content.as_deref().is_some_and(is_lfs_pointer);

    let mut hunks = Vec::new();
    let mut truncated = oversized;
    if let Some(content) = content.as_deref().filter(|_| !oversized) {
        let mut raw_lines: Vec<&str> = content.split('\n').collect();
        if raw_lines.last() == Some(&"") {
            raw_lines.pop();
        }
        if raw_lines.len() > 2000 {
            raw_lines.truncate(2000);
            truncated = true;
        }
        let lines: Vec<DiffLine> = raw_lines
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
        hunks.push(DiffHunk { header, lines });
    }

    Some(RepositoryFile {
        id: format!("untracked:{file_path}"),
        path: file_path.to_string(),
        old_path: None,
        status: ChangeStatus::Added,
        language: language_for_path(file_path),
        staged: false,
        unstaged: true,
        untracked: true,
        binary,
        lfs,
        truncated,
        hunks,
    })
}

fn has_head(root: &Path) -> bool {
    run_git(&["rev-parse", "--verify", "--quiet", "HEAD"], root).is_ok()
}

fn infer_status(section: &DiffSection) -> ChangeStatus {
    if section.old_path.is_some() && section.old_path.as_deref() != Some(&section.path) {
        return ChangeStatus::Renamed;
    }
    let mut additions = 0usize;
    let mut deletions = 0usize;
    for hunk in &section.hunks {
        for line in &hunk.lines {
            match line.kind {
                LineType::Add => additions += 1,
                LineType::Del => deletions += 1,
                LineType::Context => {}
            }
        }
    }
    if additions > 0 && deletions == 0 {
        ChangeStatus::Added
    } else if deletions > 0 && additions == 0 {
        ChangeStatus::Deleted
    } else {
        ChangeStatus::Modified
    }
}

fn worktree_file_is_lfs(root: &Path, file_path: &str) -> bool {
    let path = root.join(file_path);
    match fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() && metadata.len() <= 4096 => fs::read_to_string(path)
            .map(|content| is_lfs_pointer(&content))
            .unwrap_or(false),
        _ => false,
    }
}

fn repository_file_from_section(
    root: &Path,
    section: DiffSection,
    status: ChangeStatus,
    staged: bool,
    unstaged: bool,
    untracked: bool,
) -> RepositoryFile {
    let id_kind = if staged {
        "staged"
    } else if untracked {
        "untracked"
    } else {
        "unstaged"
    };
    let lfs = !untracked && worktree_file_is_lfs(root, &section.path);
    RepositoryFile {
        id: format!("{id_kind}:{}", section.path),
        language: language_for_path(&section.path),
        path: section.path,
        old_path: section.old_path,
        status,
        staged,
        unstaged,
        untracked,
        binary: section.binary,
        lfs,
        truncated: false,
        hunks: section.hunks,
    }
}

pub fn repository_snapshot(
    input_path: &str,
    base: Option<&str>,
) -> Result<RepositorySnapshot, CommandError> {
    let requested_dir = validate_directory(input_path)?;
    let root = probe_repository_root(&requested_dir)?.ok_or_else(|| {
        CommandError::new("not_git", "This folder is not inside a Git repository.")
    })?;
    let inspection = inspect_root(&root, true)?;
    let status_entries = read_status_entries(&root)?;
    let status_by_path: HashMap<&str, &StatusEntry> = status_entries
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect();
    let conflicts = conflict_entries(&root)?;
    let conflict_paths: std::collections::HashSet<&str> =
        conflicts.iter().map(|entry| entry.path.as_str()).collect();

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
        if has_head(&root) {
            Some("HEAD".to_string())
        } else {
            None
        }
    } else if requested_base == "HEAD" && !has_head(&root) {
        None
    } else {
        Some(requested_base.to_string())
    };

    let mut staged_args = vec![
        "-c".to_string(),
        "core.quotePath=false".to_string(),
        "diff".to_string(),
        "--cached".to_string(),
    ];
    if let Some(base) = diff_base.as_deref() {
        staged_args.push(base.to_string());
    }
    for value in [
        "--no-color",
        "--no-ext-diff",
        "--unified=3",
        "--find-renames",
    ] {
        staged_args.push(value.to_string());
    }
    let staged_args_ref: Vec<&str> = staged_args.iter().map(String::as_str).collect();
    let staged_diff = run_git(&staged_args_ref, &root).unwrap_or_default();

    let unstaged_diff = run_git(
        &[
            "-c",
            "core.quotePath=false",
            "diff",
            "--no-color",
            "--no-ext-diff",
            "--unified=3",
            "--find-renames",
        ],
        &root,
    )
    .unwrap_or_default();

    let mut files: Vec<RepositoryFile> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for section in parse_diff_sections(&staged_diff) {
        let entry = status_by_path
            .get(section.path.as_str())
            .copied()
            .or_else(|| {
                section
                    .old_path
                    .as_deref()
                    .and_then(|old_path| status_by_path.get(old_path).copied())
            });
        let status = entry
            .and_then(StatusEntry::staged_status)
            .unwrap_or_else(|| infer_status(&section));
        let mut file = repository_file_from_section(&root, section, status, true, false, false);
        if file.old_path.is_none() {
            file.old_path = entry.and_then(|entry| entry.old_path.clone());
        }
        seen.insert(file.id.clone());
        files.push(file);
    }

    for section in parse_diff_sections(&unstaged_diff) {
        if conflict_paths.contains(section.path.as_str()) {
            continue;
        }
        let entry = status_by_path
            .get(section.path.as_str())
            .copied()
            .or_else(|| {
                section
                    .old_path
                    .as_deref()
                    .and_then(|old_path| status_by_path.get(old_path).copied())
            });
        let status = entry
            .and_then(StatusEntry::unstaged_status)
            .unwrap_or_else(|| infer_status(&section));
        let mut file = repository_file_from_section(&root, section, status, false, true, false);
        if file.old_path.is_none() {
            file.old_path = entry.and_then(|entry| entry.old_path.clone());
        }
        seen.insert(file.id.clone());
        files.push(file);
    }

    for entry in &status_entries {
        if !entry.untracked || conflict_paths.contains(entry.path.as_str()) {
            continue;
        }
        let id = format!("untracked:{}", entry.path);
        if seen.contains(&id) {
            continue;
        }
        if let Some(file) = read_untracked_file(&root, &entry.path) {
            seen.insert(file.id.clone());
            files.push(file);
        }
    }

    files.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| right.staged.cmp(&left.staged))
    });
    const MAX_FILES: usize = 400;
    files.truncate(MAX_FILES);

    Ok(RepositorySnapshot {
        inspection,
        files,
        conflicts,
        merge_in_progress: merge_in_progress(&root),
        fetched_at: chrono::Utc::now().to_rfc3339(),
    })
}

fn validate_path_list(file_paths: &[String]) -> Result<Vec<String>, CommandError> {
    if file_paths.is_empty() {
        return Err(CommandError::new("bad_request", "A file path is required."));
    }

    let mut paths = Vec::new();
    for file_path in file_paths {
        let relative = validate_relative_path(file_path)?;
        let value = relative.to_string_lossy().to_string();
        if !paths.contains(&value) {
            paths.push(value);
        }
    }
    Ok(paths)
}

fn run_git_args(root: &Path, args: Vec<String>) -> Result<GitOutput, CommandError> {
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_git_capture(&refs, root, &[])
}

fn path_in_head(root: &Path, file_path: &str) -> bool {
    run_git(&["cat-file", "-e", &format!("HEAD:{file_path}")], root).is_ok()
}

pub fn stage_files(input_path: &str, file_paths: &[String]) -> Result<(), CommandError> {
    let root = repository_root(input_path)?;
    let paths = validate_path_list(file_paths)?;
    let mut args = vec!["add".to_string(), "-A".to_string(), "--".to_string()];
    args.extend(paths);
    let output = run_git_args(&root, args)?;
    if !output.success {
        return Err(CommandError::new(
            "stage_failed",
            error_detail(&output, "Git could not stage these files."),
        ));
    }
    Ok(())
}

pub fn unstage_files(input_path: &str, file_paths: &[String]) -> Result<(), CommandError> {
    let root = repository_root(input_path)?;
    let paths = validate_path_list(file_paths)?;
    let mut args = vec![
        "restore".to_string(),
        "--staged".to_string(),
        "--".to_string(),
    ];
    args.extend(paths.clone());
    let output = run_git_args(&root, args.clone())?;
    if output.success {
        return Ok(());
    }

    if has_head(&root) {
        return Err(CommandError::new(
            "unstage_failed",
            error_detail(&output, "Git could not unstage these files."),
        ));
    }

    // On a branch without commits there is no HEAD to restore from; dropping
    // the new entries from the index is the equivalent operation.
    let mut fallback = vec![
        "rm".to_string(),
        "--cached".to_string(),
        "-f".to_string(),
        "--".to_string(),
    ];
    fallback.extend(paths);
    let fallback_output = run_git_args(&root, fallback)?;
    if !fallback_output.success {
        return Err(CommandError::new(
            "unstage_failed",
            error_detail(&fallback_output, "Git could not unstage these files."),
        ));
    }
    Ok(())
}

fn remove_untracked_file(root: &Path, file_path: &str) -> Result<(), CommandError> {
    let relative = validate_relative_path(file_path)?;
    let canonical_root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let target = root.join(&relative);
    let parent = target.parent().unwrap_or(root);
    let canonical_parent = fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
    if !canonical_parent.starts_with(&canonical_root) {
        return Err(CommandError::new(
            "bad_request",
            "The file path must stay inside the repository.",
        ));
    }
    fs::remove_file(&target).map_err(|error| {
        CommandError::new(
            "discard_failed",
            format!("Could not remove this file: {error}"),
        )
    })
}

pub fn discard_file(
    input_path: &str,
    file_path: &str,
    old_path: Option<&str>,
    untracked: bool,
    staged: bool,
) -> Result<(), CommandError> {
    let root = repository_root(input_path)?;
    let mut file_paths = vec![file_path.to_string()];
    if let Some(old_path) = old_path.filter(|value| !value.is_empty()) {
        file_paths.push(old_path.to_string());
    }
    let paths = validate_path_list(&file_paths)?;

    if untracked {
        remove_untracked_file(&root, file_path)?;
        return Ok(());
    }

    if staged {
        if !has_head(&root) {
            let mut args = vec![
                "rm".to_string(),
                "--cached".to_string(),
                "-f".to_string(),
                "--".to_string(),
            ];
            args.extend(paths);
            let output = run_git_args(&root, args)?;
            if !output.success {
                return Err(CommandError::new(
                    "discard_failed",
                    error_detail(&output, "Git could not discard these changes."),
                ));
            }
            for path in &file_paths {
                let _ = remove_untracked_file(&root, path);
            }
            return Ok(());
        }

        let mut successes = 0usize;
        let mut failure = None;
        for path in &paths {
            let output = run_git_args(
                &root,
                vec![
                    "restore".to_string(),
                    "--source=HEAD".to_string(),
                    "--staged".to_string(),
                    "--worktree".to_string(),
                    "--".to_string(),
                    path.clone(),
                ],
            )?;
            if output.success {
                successes += 1;
            } else {
                failure = Some(error_detail(
                    &output,
                    "Git could not discard these changes.",
                ));
            }
        }
        if successes == 0 {
            return Err(CommandError::new(
                "discard_failed",
                failure.unwrap_or_else(|| "Git could not discard these changes.".to_string()),
            ));
        }
        // A staged rename leaves the new path untracked after restoring the
        // old one; discarding the change means removing it as well.
        if !path_in_head(&root, file_path) && old_path.is_some_and(|old| old != file_path) {
            if root.join(file_path).exists() {
                remove_untracked_file(&root, file_path)?;
            }
        }
        return Ok(());
    }

    let mut args = vec![
        "restore".to_string(),
        "--worktree".to_string(),
        "--".to_string(),
    ];
    args.extend(paths);
    let output = run_git_args(&root, args)?;
    if !output.success {
        return Err(CommandError::new(
            "discard_failed",
            error_detail(&output, "Git could not discard these changes."),
        ));
    }
    Ok(())
}

fn validate_hunk_header(header: &str) -> Result<(), CommandError> {
    if !header.starts_with("@@ ")
        || header.contains('\n')
        || header.contains('\r')
        || header.len() > 512
    {
        return Err(CommandError::new(
            "bad_request",
            "The selected hunk is not valid.",
        ));
    }
    Ok(())
}

fn diff_for_file(root: &Path, file_path: &str, cached: bool) -> Result<String, CommandError> {
    let mut args = vec![
        "-c".to_string(),
        "core.quotePath=false".to_string(),
        "diff".to_string(),
    ];
    if cached {
        args.push("--cached".to_string());
    }
    for value in [
        "--no-color",
        "--no-ext-diff",
        "--unified=3",
        "--find-renames",
        "--",
    ] {
        args.push(value.to_string());
    }
    args.push(file_path.to_string());
    let output = run_git_args(root, args)?;
    if !output.success {
        return Err(CommandError::new(
            "diff_failed",
            error_detail(&output, "Could not read this file's diff."),
        ));
    }
    Ok(output.stdout)
}

fn apply_patch(root: &Path, patch: &str, cached: bool, reverse: bool) -> Result<(), CommandError> {
    let mut args = vec!["apply".to_string()];
    if cached {
        args.push("--cached".to_string());
    }
    if reverse {
        args.push("--reverse".to_string());
    }
    for value in ["--whitespace=nowarn", "--recount", "-"] {
        args.push(value.to_string());
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = run_git_with_input(&refs, root, patch)?;
    if !output.success {
        return Err(CommandError::new(
            "patch_failed",
            error_detail(&output, "Git could not apply this hunk."),
        ));
    }
    Ok(())
}

fn hunk_operation(
    input_path: &str,
    file_path: &str,
    hunk_header: &str,
    source_cached: bool,
    reverse: bool,
    cached: bool,
) -> Result<(), CommandError> {
    validate_hunk_header(hunk_header)?;
    let root = repository_root(input_path)?;
    let relative = validate_relative_path(file_path)?;
    let relative = relative.to_string_lossy().to_string();
    let diff = diff_for_file(&root, &relative, source_cached)?;
    let patch = hunk_patch(&diff, &relative, hunk_header).ok_or_else(|| {
        CommandError::new(
            "hunk_not_found",
            "This hunk is no longer part of the diff. Refresh and try again.",
        )
    })?;
    apply_patch(&root, &patch, cached, reverse)
}

/// Stages one hunk from the working tree into the index.
pub fn stage_hunk(
    input_path: &str,
    file_path: &str,
    hunk_header: &str,
) -> Result<(), CommandError> {
    hunk_operation(input_path, file_path, hunk_header, false, false, true)
}

/// Removes one staged hunk from the index, leaving the working tree untouched.
pub fn unstage_hunk(
    input_path: &str,
    file_path: &str,
    hunk_header: &str,
) -> Result<(), CommandError> {
    hunk_operation(input_path, file_path, hunk_header, true, true, true)
}

/// Reverts one unstaged hunk in the working tree. This is destructive and the
/// UI must confirm it first.
pub fn discard_hunk(
    input_path: &str,
    file_path: &str,
    hunk_header: &str,
) -> Result<(), CommandError> {
    hunk_operation(input_path, file_path, hunk_header, false, true, false)
}

pub fn commit_repository(
    input_path: &str,
    message: &str,
    amend: bool,
) -> Result<CommitOutcome, CommandError> {
    let root = repository_root(input_path)?;
    let message = message.trim();
    if message.is_empty() {
        return Err(CommandError::new(
            "empty_message",
            "Write a commit message first.",
        ));
    }

    if !amend {
        let staged = run_git_capture(&["diff", "--cached", "--quiet"], &root, &[])?;
        if staged.success {
            return Err(CommandError::new(
                "nothing_staged",
                "Stage at least one change before committing.",
            ));
        }
    }

    let mut args = vec!["commit".to_string(), "-m".to_string(), message.to_string()];
    if amend {
        args.push("--amend".to_string());
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = run_git_capture(&refs, &root, GIT_NETWORK_ENV)?;
    if !output.success {
        return Err(CommandError::new(
            "commit_failed",
            error_detail(&output, "Git could not create this commit."),
        ));
    }

    let meta = run_git(&["show", "-s", "--format=%H%x00%h%x00%s", "HEAD"], &root)?;
    let mut fields = meta.trim().split('\0');
    let hash = fields.next().unwrap_or("").trim().to_string();
    let short_hash = fields.next().unwrap_or("").trim().to_string();
    let subject = fields.next().unwrap_or("").trim().to_string();
    let files_changed = run_git(&["show", "--format=", "--name-only", "-z", "HEAD"], &root)
        .map(|raw| {
            raw.split('\0')
                .filter(|value| !value.trim().is_empty())
                .count() as u32
        })
        .unwrap_or(0);

    Ok(CommitOutcome {
        hash,
        short_hash,
        subject,
        files_changed,
    })
}

pub fn fetch_repository(
    input_path: &str,
    remote: Option<&str>,
) -> Result<SyncOutcome, CommandError> {
    let (root, inspection) = inspect_repository_root(input_path)?;
    if inspection.remotes.is_empty() {
        return Err(CommandError::new(
            "no_remote",
            "This repository has no remote configured.",
        ));
    }

    let mut args = vec!["fetch".to_string(), "--prune".to_string()];
    if let Some(remote) = remote.map(str::trim).filter(|value| !value.is_empty()) {
        validate_remote_name(remote)?;
        if !inspection.remotes.iter().any(|entry| entry.name == remote) {
            return Err(CommandError::new(
                "unknown_remote",
                "That remote is not configured for this repository.",
            ));
        }
        args.push(remote.to_string());
    } else {
        args.push("--all".to_string());
    }

    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = run_git_capture(&refs, &root, GIT_NETWORK_ENV)?;
    if !output.success {
        return Err(CommandError::new(
            "fetch_failed",
            error_detail(&output, "Git could not fetch from the remote."),
        ));
    }

    let message = if output.stderr.is_empty() {
        "Fetched the latest refs from the remote.".to_string()
    } else {
        output.stderr
    };
    Ok(SyncOutcome {
        message,
        conflicts: Vec::new(),
    })
}

fn validate_branch_name(root: &Path, branch: &str) -> Result<String, CommandError> {
    let branch = branch.trim();
    if branch.is_empty() || branch.len() > 200 || branch.starts_with('-') {
        return Err(CommandError::new(
            "bad_branch",
            "Enter a valid branch name.",
        ));
    }
    let output = run_git_capture(&["check-ref-format", "--branch", branch], root, &[])?;
    if !output.success {
        return Err(CommandError::new(
            "bad_branch",
            "Enter a valid branch name.",
        ));
    }
    Ok(branch.to_string())
}

fn stash_push(root: &Path, message: &str, include_untracked: bool) -> Result<bool, CommandError> {
    let mut args = vec!["stash".to_string(), "push".to_string()];
    if !message.trim().is_empty() {
        args.push("-m".to_string());
        args.push(message.trim().to_string());
    }
    if include_untracked {
        args.push("--include-untracked".to_string());
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = run_git_capture(&refs, root, &[])?;
    if !output.success {
        return Err(CommandError::new(
            "stash_failed",
            error_detail(&output, "Git could not stash these changes."),
        ));
    }
    let combined = format!("{}{}", output.stdout, output.stderr);
    Ok(!combined.contains("No local changes to save"))
}

pub fn switch_branch(
    input_path: &str,
    branch: &str,
    create: bool,
    stash: bool,
) -> Result<BranchSwitchOutcome, CommandError> {
    let (root, inspection) = inspect_repository_root(input_path)?;
    let branch = validate_branch_name(&root, branch)?;

    let local = inspection.branches.iter().any(|entry| entry == &branch);
    let remote = inspection
        .remote_branches
        .iter()
        .find(|entry| entry.name == branch)
        .cloned();

    if create && local {
        return Err(CommandError::new(
            "branch_exists",
            "A branch with this name already exists.",
        ));
    }
    if !create && !local && remote.is_none() {
        return Err(CommandError::new(
            "unknown_branch",
            "This branch does not exist.",
        ));
    }

    if !create {
        let current = run_git(&["branch", "--show-current"], &root).unwrap_or_default();
        if current.trim() == branch {
            return Ok(BranchSwitchOutcome {
                branch,
                stashed: false,
            });
        }
        if let Some(owner) = read_worktrees(&root).into_iter().find(|worktree| {
            !worktree.is_current && !worktree.detached && worktree.branch == branch
        }) {
            return Err(CommandError::new(
                "branch_checked_out",
                format!(
                    "This branch is already checked out in another worktree: {}",
                    owner.path
                ),
            )
            .with_detail(owner.path));
        }
    }

    let status = run_git_capture(
        &["status", "--porcelain", "--untracked-files=no"],
        &root,
        &[],
    )?;
    let dirty = !status.stdout.trim().is_empty();

    if dirty && !create && !stash {
        return Err(CommandError::new(
            "dirty_worktree",
            "Commit or stash your changes before switching branches.",
        ));
    }

    let mut stashed = false;
    if dirty && stash && !create {
        let message = format!("FlashBee: changes before switching to {branch}");
        stashed = stash_push(&root, &message, false)?;
    }

    let mut args: Vec<String> = vec!["checkout".to_string()];
    if create {
        args.push("-b".to_string());
        args.push(branch.clone());
    } else if local {
        args.push(branch.clone());
    } else if let Some(remote) = remote.as_ref() {
        args.push("-b".to_string());
        args.push(branch.clone());
        args.push("--track".to_string());
        // Track the full ref so a local branch with the same name cannot
        // shadow the remote-tracking branch.
        args.push(format!("refs/remotes/{}", remote.name));
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = run_git_capture(&refs, &root, GIT_NETWORK_ENV)?;
    if !output.success {
        if stashed {
            let _ = run_git_capture(&["stash", "pop"], &root, &[]);
        }
        let message = error_detail(&output, "Git could not switch branches.");
        let lowered = message.to_lowercase();
        if lowered.contains("already checked out") || lowered.contains("already used by worktree") {
            let detail = message
                .split("at '")
                .nth(1)
                .and_then(|value| value.split('\'').next())
                .map(str::to_string);
            let mut error = CommandError::new(
                "branch_checked_out",
                format!("This branch is already checked out in another worktree. {message}"),
            );
            if let Some(detail) = detail {
                error = error.with_detail(detail);
            }
            return Err(error);
        }
        return Err(CommandError::new("switch_failed", message));
    }

    Ok(BranchSwitchOutcome { branch, stashed })
}

fn branch_worktree_owner(root: &Path, branch: &str) -> Option<String> {
    read_worktrees(root)
        .into_iter()
        .find(|worktree| !worktree.detached && worktree.branch == branch)
        .map(|worktree| worktree.path)
}

fn ref_is_ancestor(root: &Path, ancestor: &str, descendant: &str) -> bool {
    run_git_capture(
        &["merge-base", "--is-ancestor", ancestor, descendant],
        root,
        &[],
    )
    .map(|output| output.success)
    .unwrap_or(false)
}

fn compare_ref_counts(root: &Path, base: &str, tip: &str) -> (u32, u32) {
    let range = format!("{base}...{tip}");
    let Ok(raw) = run_git(
        &["rev-list", "--left-right", "--count", range.as_str()],
        root,
    ) else {
        return (0, 0);
    };
    let mut parts = raw.split_whitespace();
    let base_only = parts
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let tip_only = parts
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    // (ahead, behind) relative to `base`.
    (tip_only, base_only)
}

pub fn list_branch_info(input_path: &str) -> Result<Vec<BranchInfo>, CommandError> {
    let root = repository_root(input_path)?;
    let remotes = read_remotes(&root);
    let current = run_git(&["branch", "--show-current"], &root)
        .unwrap_or_default()
        .trim()
        .to_string();
    let worktrees = read_worktrees(&root);
    let worktree_by_branch: HashMap<String, String> = worktrees
        .iter()
        .filter(|worktree| !worktree.detached)
        .map(|worktree| (worktree.branch.clone(), worktree.path.clone()))
        .collect();

    let raw = run_git(
        &[
            "-c",
            "core.quotePath=false",
            "for-each-ref",
            "--format=%(refname)%00%(objectname:short)%00%(committerdate:unix)%00%(contents:subject)%00%(upstream)%00%(symref)%00",
            "refs/heads",
            "refs/remotes",
        ],
        &root,
    )?;

    let mut branches = Vec::new();
    for record in raw.lines() {
        let record = record.trim_end_matches('\r');
        if record.is_empty() {
            continue;
        }
        let mut fields = record.split('\0');
        let full_ref = fields.next().unwrap_or("").trim();
        let is_remote = full_ref.starts_with("refs/remotes/");
        // Derive the display name ourselves: Git abbreviates a ref that shares
        // its name with a local branch to `remotes/origin/main`.
        let name = if is_remote {
            full_ref.strip_prefix("refs/remotes/")
        } else {
            full_ref.strip_prefix("refs/heads/")
        }
        .unwrap_or(full_ref)
        .to_string();
        if name.is_empty() {
            continue;
        }
        let short_commit = fields.next().unwrap_or("").trim().to_string();
        let time = fields.next().unwrap_or("").trim().parse().unwrap_or(0);
        let subject = fields.next().unwrap_or("").trim().to_string();
        let upstream = fields
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(display_tracking_ref);
        let symref = fields.next().unwrap_or("").trim();
        if is_remote && (!symref.is_empty() || name.ends_with("/HEAD")) {
            continue;
        }
        let is_current = !is_remote && name == current;
        let (ahead, behind) = if is_current || is_remote {
            (0, 0)
        } else {
            compare_ref_counts(&root, "HEAD", full_ref)
        };
        let merged = !is_remote && (is_current || ref_is_ancestor(&root, full_ref, "HEAD"));
        let remote = if is_remote {
            split_remote_branch(&name, &remotes).map(|(remote, _)| remote)
        } else {
            None
        };
        branches.push(BranchInfo {
            name: name.to_string(),
            is_current,
            is_remote,
            remote,
            upstream,
            ahead,
            behind,
            short_commit,
            subject,
            time,
            worktree_path: worktree_by_branch.get(&name).cloned(),
            merged,
        });
    }

    branches.sort_by(|left, right| {
        left.is_remote
            .cmp(&right.is_remote)
            .then_with(|| right.is_current.cmp(&left.is_current))
            .then_with(|| left.name.cmp(&right.name))
    });
    Ok(branches)
}

pub fn create_branch(
    input_path: &str,
    name: &str,
    start_point: Option<&str>,
    checkout: bool,
) -> Result<BranchSwitchOutcome, CommandError> {
    let (root, inspection) = inspect_repository_root(input_path)?;
    let name = validate_branch_name(&root, name)?;
    if inspection.branches.iter().any(|branch| branch == &name) {
        return Err(CommandError::new(
            "branch_exists",
            "A branch with this name already exists.",
        ));
    }

    let start = start_point.unwrap_or("HEAD").trim();
    if start.is_empty() || start.starts_with('-') || start.len() > 200 {
        return Err(CommandError::new(
            "bad_start_point",
            "Choose a valid starting point for the branch.",
        ));
    }
    let start_hash = resolve_commit_revision(&root, Some(start))?;
    let tracking = inspection
        .remote_branches
        .iter()
        .find(|branch| branch.name == start)
        .cloned();
    // Fully-qualified: `origin/main` alone may name a local branch instead.
    let tracking_ref = tracking
        .as_ref()
        .map(|tracking| format!("refs/remotes/{}", tracking.name));

    let output = if checkout {
        let mut args = vec!["switch", "-c", name.as_str()];
        if let Some(tracking_ref) = tracking_ref.as_deref() {
            args.push("--track");
            args.push(tracking_ref);
        } else {
            args.push(start_hash.as_str());
        }
        run_git_capture(&args, &root, GIT_NETWORK_ENV)?
    } else if let Some(tracking_ref) = tracking_ref.as_deref() {
        run_git_capture(
            &["branch", "--track", name.as_str(), tracking_ref],
            &root,
            &[],
        )?
    } else {
        run_git_capture(&["branch", name.as_str(), start_hash.as_str()], &root, &[])?
    };

    if !output.success {
        return Err(CommandError::new(
            if checkout {
                "switch_failed"
            } else {
                "branch_create_failed"
            },
            error_detail(&output, "Git could not create this branch."),
        ));
    }
    Ok(BranchSwitchOutcome {
        branch: name,
        stashed: false,
    })
}

pub fn rename_branch(input_path: &str, old_name: &str, new_name: &str) -> Result<(), CommandError> {
    let (root, inspection) = inspect_repository_root(input_path)?;
    let old_name = validate_branch_name(&root, old_name)?;
    let new_name = validate_branch_name(&root, new_name)?;
    if old_name == new_name {
        return Ok(());
    }
    if !inspection.branches.iter().any(|branch| branch == &old_name) {
        return Err(CommandError::new(
            "unknown_branch",
            "This branch does not exist.",
        ));
    }
    if inspection.branches.iter().any(|branch| branch == &new_name) {
        return Err(CommandError::new(
            "branch_exists",
            "A branch with this name already exists.",
        ));
    }
    if let Some(owner) = branch_worktree_owner(&root, &old_name) {
        let current = run_git(&["branch", "--show-current"], &root).unwrap_or_default();
        if !current.trim().is_empty() && current.trim() == old_name {
            // Renaming the current branch is allowed.
        } else if Path::new(&owner) != root {
            return Err(CommandError::new(
                "branch_checked_out",
                format!("This branch is checked out in another worktree: {owner}"),
            )
            .with_detail(owner));
        }
    }
    let output = run_git_capture(
        &["branch", "-m", old_name.as_str(), new_name.as_str()],
        &root,
        &[],
    )?;
    if !output.success {
        return Err(CommandError::new(
            "branch_rename_failed",
            error_detail(&output, "Git could not rename this branch."),
        ));
    }
    Ok(())
}

pub fn delete_branch(input_path: &str, name: &str, force: bool) -> Result<(), CommandError> {
    let (root, inspection) = inspect_repository_root(input_path)?;
    let name = validate_branch_name(&root, name)?;
    if !inspection.branches.iter().any(|branch| branch == &name) {
        return Err(CommandError::new(
            "unknown_branch",
            "This branch does not exist.",
        ));
    }
    let current = run_git(&["branch", "--show-current"], &root).unwrap_or_default();
    if current.trim() == name {
        return Err(CommandError::new(
            "cannot_delete_current",
            "Switch away from this branch before deleting it.",
        ));
    }
    if let Some(owner) = branch_worktree_owner(&root, &name) {
        return Err(CommandError::new(
            "branch_checked_out",
            format!("This branch is checked out in another worktree: {owner}"),
        )
        .with_detail(owner));
    }

    let output = run_git_capture(
        &["branch", if force { "-D" } else { "-d" }, name.as_str()],
        &root,
        &[],
    )?;
    if !output.success {
        let message = error_detail(&output, "Git could not delete this branch.");
        if !force
            && (message.to_lowercase().contains("not fully merged")
                || message.to_lowercase().contains("not merged"))
        {
            return Err(CommandError::new(
                "branch_not_merged",
                "This branch has commits that are not merged. Delete it with force if you are sure.",
            )
            .with_detail(name));
        }
        return Err(CommandError::new("branch_delete_failed", message));
    }
    Ok(())
}

pub fn merge_branch(input_path: &str, source: &str) -> Result<SyncOutcome, CommandError> {
    let root = repository_root(input_path)?;
    let current = run_git(&["branch", "--show-current"], &root).unwrap_or_default();
    let current = current.trim().to_string();
    if current.is_empty() || current == "HEAD" {
        return Err(CommandError::new(
            "detached_head",
            "A detached HEAD cannot merge a branch.",
        ));
    }
    let source = source.trim();
    if source.is_empty() || source == current {
        return Err(CommandError::new(
            "bad_request",
            "Choose a different branch to merge.",
        ));
    }
    let source_hash = resolve_commit_revision(&root, Some(source))?;
    let output = run_git_capture(
        &["merge", "--no-edit", source_hash.as_str()],
        &root,
        GIT_NETWORK_ENV,
    )?;
    let conflicts = pull_conflict_paths(&root)?;
    if !output.success && conflicts.is_empty() {
        return Err(CommandError::new(
            "merge_failed",
            error_detail(&output, "Git could not merge this branch."),
        ));
    }
    Ok(SyncOutcome {
        message: error_detail(&output, "Merge completed."),
        conflicts,
    })
}

pub fn set_branch_upstream(
    input_path: &str,
    branch: &str,
    upstream: &str,
) -> Result<(), CommandError> {
    let (root, inspection) = inspect_repository_root(input_path)?;
    let branch = validate_branch_name(&root, branch)?;
    if !inspection.branches.iter().any(|entry| entry == &branch) {
        return Err(CommandError::new(
            "unknown_branch",
            "This branch does not exist.",
        ));
    }
    let upstream = upstream.trim();
    if upstream.is_empty() || upstream.starts_with('-') {
        return Err(CommandError::new(
            "bad_request",
            "Choose a valid upstream branch.",
        ));
    }
    resolve_commit_revision(&root, Some(upstream))?;
    let tracking = format!("--set-upstream-to={upstream}");
    let output = run_git_capture(&["branch", tracking.as_str(), branch.as_str()], &root, &[])?;
    if !output.success {
        return Err(CommandError::new(
            "upstream_failed",
            error_detail(&output, "Git could not set the upstream branch."),
        ));
    }
    Ok(())
}

pub fn list_worktrees(input_path: &str) -> Result<Vec<WorktreeSummary>, CommandError> {
    let root = repository_root(input_path)?;
    let mut summaries = Vec::new();
    for worktree in read_worktrees(&root) {
        let path = PathBuf::from(&worktree.path);
        let (changed, conflicts) = if path.is_dir() {
            let changed = read_status_entries(&path)
                .map(|entries| entries.len() as u32)
                .unwrap_or(0);
            let conflicts = conflict_entries(&path)
                .map(|entries| entries.len() as u32)
                .unwrap_or(0);
            (changed, conflicts)
        } else {
            (0, 0)
        };
        summaries.push(WorktreeSummary {
            info: worktree,
            changed,
            conflicts,
        });
    }
    Ok(summaries)
}

pub fn add_worktree(input_path: &str, options: WorktreeAddOptions) -> Result<(), CommandError> {
    let (root, inspection) = inspect_repository_root(input_path)?;
    if options.branch.is_some() && options.new_branch.is_some() {
        return Err(CommandError::new(
            "bad_request",
            "Choose either an existing branch or a new branch.",
        ));
    }
    let target = expand_home(options.path.trim());
    if !target.is_absolute() {
        return Err(CommandError::new(
            "bad_request",
            "The worktree path must be absolute.",
        ));
    }
    if target == root {
        return Err(CommandError::new(
            "bad_request",
            "A worktree cannot reuse the main checkout path.",
        ));
    }

    let mut args: Vec<String> = vec!["worktree".to_string(), "add".to_string()];
    let mut checkout_ref = None;
    if let Some(new_branch) = options.new_branch.as_deref() {
        let new_branch = validate_branch_name(&root, new_branch)?;
        if inspection
            .branches
            .iter()
            .any(|branch| branch == &new_branch)
        {
            return Err(CommandError::new(
                "branch_exists",
                "A branch with this name already exists.",
            ));
        }
        let start = options.start_point.as_deref().unwrap_or("HEAD").trim();
        let start_hash = resolve_commit_revision(&root, Some(start))?;
        args.push("-b".to_string());
        args.push(new_branch);
        args.push(target.to_string_lossy().to_string());
        args.push(start_hash);
    } else if let Some(branch) = options.branch.as_deref() {
        let branch = validate_branch_name(&root, branch)?;
        if !inspection.branches.iter().any(|entry| entry == &branch) {
            return Err(CommandError::new(
                "unknown_branch",
                "This branch does not exist.",
            ));
        }
        if let Some(owner) = branch_worktree_owner(&root, &branch) {
            return Err(CommandError::new(
                "branch_checked_out",
                format!("This branch is already checked out in a worktree: {owner}"),
            )
            .with_detail(owner));
        }
        checkout_ref = Some(branch.clone());
        args.push(target.to_string_lossy().to_string());
        args.push(branch);
    } else if options.detach {
        args.push("--detach".to_string());
        args.push(target.to_string_lossy().to_string());
        let start = options.start_point.as_deref().unwrap_or("HEAD").trim();
        args.push(resolve_commit_revision(&root, Some(start))?);
    } else {
        return Err(CommandError::new(
            "bad_request",
            "Choose a branch or detach the worktree.",
        ));
    }

    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = run_git_capture(&refs, &root, &[])?;
    if !output.success {
        let message = error_detail(&output, "Git could not create this worktree.");
        if message.to_lowercase().contains("already checked out")
            || message.to_lowercase().contains("already used by worktree")
        {
            return Err(CommandError::new("branch_checked_out", message)
                .with_detail(checkout_ref.unwrap_or_default()));
        }
        return Err(CommandError::new("worktree_add_failed", message));
    }
    Ok(())
}

pub fn remove_worktree(input_path: &str, target: &str, force: bool) -> Result<(), CommandError> {
    let root = repository_root(input_path)?;
    let target_path = PathBuf::from(expand_home(target.trim()));
    let canonical_target = fs::canonicalize(&target_path).unwrap_or_else(|_| target_path.clone());
    let worktrees = read_worktrees(&root);
    let Some(worktree) = worktrees.iter().find(|worktree| {
        fs::canonicalize(&worktree.path).unwrap_or_else(|_| PathBuf::from(&worktree.path))
            == canonical_target
    }) else {
        return Err(CommandError::new(
            "unknown_worktree",
            "This worktree is not attached to the repository.",
        ));
    };
    if worktree.is_main {
        return Err(CommandError::new(
            "cannot_remove_main",
            "The main checkout cannot be removed.",
        ));
    }
    if worktree.is_current {
        return Err(CommandError::new(
            "cannot_remove_current",
            "Switch to another checkout before removing this worktree.",
        ));
    }

    let mut args = vec!["worktree", "remove"];
    if force {
        args.push("--force");
    }
    args.push(worktree.path.as_str());
    let output = run_git_capture(&args, &root, &[])?;
    if !output.success {
        let message = error_detail(&output, "Git could not remove this worktree.");
        if !force {
            return Err(CommandError::new(
                "worktree_dirty",
                "This worktree has local changes. Remove it with force only if you want to discard them.",
            )
            .with_detail(worktree.path.clone()));
        }
        return Err(CommandError::new("worktree_remove_failed", message));
    }
    Ok(())
}

pub fn prune_worktrees(input_path: &str) -> Result<u32, CommandError> {
    let root = repository_root(input_path)?;
    let before = run_git(&["worktree", "list", "--porcelain"], &root).unwrap_or_default();
    let count = before
        .lines()
        .filter(|line| line.starts_with("prunable"))
        .count() as u32;
    let output = run_git_capture(&["worktree", "prune"], &root, &[])?;
    if !output.success {
        return Err(CommandError::new(
            "worktree_prune_failed",
            error_detail(&output, "Git could not prune worktrees."),
        ));
    }
    Ok(count)
}

fn parse_stash_line(line: &str) -> Option<StashEntry> {
    let mut fields = line.split('\0');
    let reference = fields.next()?.trim().to_string();
    let raw_message = fields.next()?.trim().to_string();
    let time = fields.next()?.trim().parse().unwrap_or(0);
    if reference.is_empty() {
        return None;
    }
    let index = reference
        .strip_prefix("stash@{")?
        .strip_suffix('}')?
        .parse()
        .ok()?;
    let subject = raw_message
        .strip_prefix("WIP on ")
        .or_else(|| raw_message.strip_prefix("On "))
        .unwrap_or(&raw_message);
    let (branch, message) = match subject.split_once(':') {
        Some((branch, message)) => (branch.trim().to_string(), message.trim().to_string()),
        None => (String::new(), subject.trim().to_string()),
    };
    Some(StashEntry {
        index,
        reference,
        message,
        branch,
        time,
    })
}

pub fn stash_list(input_path: &str) -> Result<Vec<StashEntry>, CommandError> {
    let root = repository_root(input_path)?;
    let raw = run_git(
        &["stash", "list", "--format=%gd%x00%gs%x00%ct%x00%x1e"],
        &root,
    )?;
    Ok(raw
        .split('\x1e')
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter_map(parse_stash_line)
        .collect())
}

pub fn stash_save(
    input_path: &str,
    message: &str,
    include_untracked: bool,
) -> Result<bool, CommandError> {
    let root = repository_root(input_path)?;
    stash_push(&root, message, include_untracked)
}

pub fn stash_apply(input_path: &str, index: u32) -> Result<SyncOutcome, CommandError> {
    let root = repository_root(input_path)?;
    let stashes = stash_list(input_path)?;
    if !stashes.iter().any(|entry| entry.index == index) {
        return Err(CommandError::new(
            "unknown_stash",
            "This stash no longer exists.",
        ));
    }
    let reference = format!("stash@{{{index}}}");
    let output = run_git_capture(&["stash", "apply", reference.as_str()], &root, &[])?;
    let conflicts = pull_conflict_paths(&root)?;
    if !output.success && conflicts.is_empty() {
        return Err(CommandError::new(
            "stash_failed",
            error_detail(&output, "Git could not restore this stash."),
        ));
    }
    Ok(SyncOutcome {
        message: error_detail(&output, "Stash applied."),
        conflicts,
    })
}

pub fn stash_drop(input_path: &str, index: u32) -> Result<(), CommandError> {
    let root = repository_root(input_path)?;
    let stashes = stash_list(input_path)?;
    if !stashes.iter().any(|entry| entry.index == index) {
        return Err(CommandError::new(
            "unknown_stash",
            "This stash no longer exists.",
        ));
    }
    let reference = format!("stash@{{{index}}}");
    let output = run_git_capture(&["stash", "drop", reference.as_str()], &root, &[])?;
    if !output.success {
        return Err(CommandError::new(
            "stash_failed",
            error_detail(&output, "Git could not drop this stash."),
        ));
    }
    Ok(())
}

fn resolve_commit_revision(root: &Path, revision: Option<&str>) -> Result<String, CommandError> {
    let revision = revision.unwrap_or("HEAD").trim();
    if revision.is_empty() || revision.len() > 200 || revision.starts_with('-') {
        return Err(CommandError::new(
            "bad_revision",
            "The selected revision is not valid.",
        ));
    }
    let target = format!("{revision}^{{commit}}");
    let output = run_git_capture(
        &["rev-parse", "--verify", "--quiet", target.as_str()],
        root,
        &[],
    )?;
    let hash = output.stdout.trim();
    if !output.success || hash.is_empty() {
        return Err(CommandError::new(
            "bad_revision",
            "The selected revision could not be resolved.",
        ));
    }
    Ok(hash.to_string())
}

pub fn commit_log(
    input_path: &str,
    revision: Option<&str>,
    file_path: Option<&str>,
    limit: u32,
    skip: u32,
) -> Result<Vec<LogCommit>, CommandError> {
    let root = repository_root(input_path)?;
    let hash = resolve_commit_revision(&root, revision)?;
    let limit = limit.clamp(1, 500);
    let skip = skip.min(5000);
    let mut args = vec![
        "log".to_string(),
        "--date-order".to_string(),
        "--format=%H%x00%h%x00%P%x00%an%x00%ae%x00%ct%x00%s%x00%D%x00%x1e".to_string(),
        "-n".to_string(),
        limit.to_string(),
    ];
    if skip > 0 {
        args.push("--skip".to_string());
        args.push(skip.to_string());
    }
    if let Some(file_path) = file_path.map(str::trim).filter(|value| !value.is_empty()) {
        let relative = validate_relative_path(file_path)?;
        args.push("--follow".to_string());
        args.push(hash);
        args.push("--".to_string());
        args.push(relative.to_string_lossy().to_string());
    } else {
        args.push(hash);
    }

    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let raw = run_git(&refs, &root)?;
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
        let short_hash = fields.next().unwrap_or("").trim().to_string();
        let parents = fields
            .next()
            .unwrap_or("")
            .split_whitespace()
            .map(str::to_string)
            .collect();
        let author = fields.next().unwrap_or("").trim().to_string();
        let email = fields.next().unwrap_or("").trim().to_string();
        let time = fields.next().unwrap_or("").trim().parse().unwrap_or(0);
        let subject = fields.next().unwrap_or("").trim().to_string();
        let refs = fields
            .next()
            .unwrap_or("")
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect();
        commits.push(LogCommit {
            hash: hash.to_string(),
            short_hash,
            parents,
            author,
            email,
            time,
            subject,
            refs,
        });
    }
    Ok(commits)
}

pub fn compare_refs(
    input_path: &str,
    base: &str,
    target: &str,
) -> Result<CompareDiff, CommandError> {
    let root = repository_root(input_path)?;
    let base_hash = resolve_commit_revision(&root, Some(base))?;
    let target_hash = resolve_commit_revision(&root, Some(target))?;
    let statuses = read_commit_statuses(&root, &target_hash, Some(&base_hash));
    let diff = run_git(
        &[
            "-c",
            "core.quotePath=false",
            "diff",
            base_hash.as_str(),
            target_hash.as_str(),
            "--no-color",
            "--no-ext-diff",
            "--unified=3",
            "--find-renames",
        ],
        &root,
    )
    .unwrap_or_default();

    let files = parse_diff_sections(&diff)
        .into_iter()
        .map(|section| {
            let status = statuses
                .get(section.path.as_str())
                .cloned()
                .unwrap_or_else(|| infer_status(&section));
            let path = section.path.clone();
            RepositoryFile {
                id: path.clone(),
                path,
                old_path: section.old_path,
                status,
                language: language_for_path(&section.path),
                staged: false,
                unstaged: false,
                untracked: false,
                binary: section.binary,
                lfs: false,
                truncated: false,
                hunks: section.hunks,
            }
        })
        .collect();

    Ok(CompareDiff {
        base: base_hash,
        target: target_hash,
        files,
    })
}

pub fn search_commits(
    input_path: &str,
    query: &str,
    limit: u32,
) -> Result<Vec<LogCommit>, CommandError> {
    let root = repository_root(input_path)?;
    let query = query.trim();
    if query.is_empty() || query.len() > 200 || query.contains('\0') {
        return Ok(Vec::new());
    }
    let limit = limit.clamp(1, 100);
    let raw = run_git(
        &[
            "log",
            "--all",
            "--date-order",
            "--fixed-strings",
            "-i",
            &format!("--grep={query}"),
            "--format=%H%x00%h%x00%P%x00%an%x00%ae%x00%ct%x00%s%x00%D%x00%x1e",
            "-n",
            &limit.to_string(),
        ],
        &root,
    )?;
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
        commits.push(LogCommit {
            hash: hash.to_string(),
            short_hash: fields.next().unwrap_or("").trim().to_string(),
            parents: fields
                .next()
                .unwrap_or("")
                .split_whitespace()
                .map(str::to_string)
                .collect(),
            author: fields.next().unwrap_or("").trim().to_string(),
            email: fields.next().unwrap_or("").trim().to_string(),
            time: fields.next().unwrap_or("").trim().parse().unwrap_or(0),
            subject: fields.next().unwrap_or("").trim().to_string(),
            refs: fields
                .next()
                .unwrap_or("")
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect(),
        });
    }
    Ok(commits)
}

fn ripgrep_search(
    root: &Path,
    query: &str,
    seen: &mut std::collections::HashSet<String>,
    limit: usize,
) -> Option<Vec<FileSearchHit>> {
    let output = Command::new("rg")
        .args([
            "--no-heading",
            "--line-number",
            "--color",
            "never",
            "--fixed-strings",
            "--max-count",
            "5",
            "--max-filesize",
            "1M",
            "--glob",
            "!.git",
            "--",
            query,
        ])
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() && output.stdout.is_empty() {
        // Exit code 1 means "no matches"; anything else may still have output.
        let code = output.status.code().unwrap_or(-1);
        if code == 1 {
            return Some(Vec::new());
        }
        return None;
    }
    let mut hits = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if hits.len() >= limit {
            break;
        }
        let mut parts = line.splitn(3, ':');
        let Some(path) = parts.next() else {
            continue;
        };
        let Some(line_no) = parts.next().and_then(|value| value.parse::<u32>().ok()) else {
            continue;
        };
        let preview: String = parts
            .next()
            .unwrap_or("")
            .trim()
            .chars()
            .take(200)
            .collect();
        if seen.insert(path.to_string()) {
            hits.push(FileSearchHit {
                path: path.to_string(),
                line: line_no,
                preview,
            });
        }
    }
    Some(hits)
}

pub fn search_files(
    input_path: &str,
    query: &str,
    limit: u32,
) -> Result<Vec<FileSearchHit>, CommandError> {
    let root = repository_root(input_path)?;
    let query = query.trim();
    if query.is_empty() || query.len() > 200 || query.contains('\0') {
        return Ok(Vec::new());
    }
    let limit = limit.clamp(1, 200) as usize;
    let normalized = query.to_lowercase();
    let mut hits: Vec<FileSearchHit> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    if let Some(found) = ripgrep_search(&root, query, &mut seen, limit) {
        hits = found;
    } else {
        let output = run_git_capture(
            &["grep", "--no-color", "-n", "-I", "-F", "-e", query],
            &root,
            &[],
        )?;
        if output.success {
            for line in output.stdout.lines() {
                if hits.len() >= limit {
                    break;
                }
                let mut parts = line.splitn(3, ':');
                let Some(path) = parts.next() else {
                    continue;
                };
                let Some(line_no) = parts.next().and_then(|value| value.parse::<u32>().ok()) else {
                    continue;
                };
                let preview = parts.next().unwrap_or("").trim();
                let preview: String = preview.chars().take(200).collect();
                if seen.insert(path.to_string()) {
                    hits.push(FileSearchHit {
                        path: path.to_string(),
                        line: line_no,
                        preview,
                    });
                }
            }
        }
    }

    for args in [
        vec!["ls-files", "-z"],
        vec!["ls-files", "--others", "--exclude-standard", "-z"],
    ] {
        if hits.len() >= limit {
            break;
        }
        let raw = run_git(&args, &root).unwrap_or_default();
        for path in raw.split('\0').filter(|value| !value.is_empty()) {
            if hits.len() >= limit {
                break;
            }
            if path.to_lowercase().contains(&normalized) && seen.insert(path.to_string()) {
                hits.push(FileSearchHit {
                    path: path.to_string(),
                    line: 0,
                    preview: String::new(),
                });
            }
        }
    }

    Ok(hits)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSide {
    Head,
    Index,
    Worktree,
}

/// Returns the raw bytes of one side of a file. Used by the image diff viewer,
/// where neither syntax highlighting nor line diffs apply.
pub fn read_file_side(
    input_path: &str,
    file_path: &str,
    side: FileSide,
) -> Result<Option<Vec<u8>>, CommandError> {
    let root = repository_root(input_path)?;
    let relative = validate_relative_path(file_path)?;
    let relative = relative.to_string_lossy().to_string();
    match side {
        FileSide::Worktree => {
            let bytes = fs::read(root.join(&relative))
                .map_err(|error| CommandError::new("read_failed", error.to_string()))?;
            Ok(Some(bytes))
        }
        FileSide::Head => {
            let spec = format!("HEAD:{relative}");
            run_git_bytes(&["show", spec.as_str()], &root)
        }
        FileSide::Index => {
            let spec = format!(":{relative}");
            run_git_bytes(&["show", spec.as_str()], &root)
        }
    }
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
        return Err(CommandError::new("bad_request", "A file path is required."));
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
        ChangeStatus, ConflictSegment, ConflictSide, FileSide, PushOptions, WorktreeAddOptions,
        abort_merge, add_worktree, commit_detail, commit_log, commit_repository, compare_refs,
        complete_merge, conflict_entries, conflict_kind, create_branch, delete_branch,
        discard_file, discard_hunk, discover_repositories, fetch_repository, initialize_repository,
        inspect_root, list_branch_info, list_remotes, list_worktrees, merge_branch, parse_blame,
        parse_conflict_markers, prune_worktrees, pull_repository, push_info, push_repository,
        read_conflict, read_file_side, remove_worktree, rename_branch, repository_snapshot,
        resolve_conflict_side, save_conflict_resolution, scan_repositories, search_commits,
        search_files, set_remote_url, should_skip_directory, stage_files, stage_hunk, stash_apply,
        stash_drop, stash_list, suggested_side, switch_branch, unstage_files, unstage_hunk,
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
        let root =
            std::env::temp_dir().join(format!("flashbee-{label}-{}-{stamp}", std::process::id()));
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
        assert!(
            Command::new("git")
                .arg("init")
                .arg(&child)
                .output()
                .unwrap()
                .status
                .success()
        );

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
        assert_eq!(
            entries[1].summary,
            "Version of src/components/DiffViewer.tsx from the working tree"
        );
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
        assert!(
            inspection
                .remote_branches
                .iter()
                .any(|branch| branch.name == "origin/main")
        );
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
        assert!(detail.files.iter().any(|file| file.path == "third.txt"));

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

    #[test]
    fn resolves_push_target_when_a_local_branch_shadows_the_remote() {
        let root = temp_root("shadowed-upstream");
        let remote = root.join("origin.git");
        let repository = root.join("repository");
        fs::create_dir_all(&remote).unwrap();
        fs::create_dir_all(&repository).unwrap();

        git_in(&remote, &["init", "--bare", "-b", "main"]);
        git_in(&repository, &["init", "-b", "main"]);
        configure_user(&repository);
        commit_file(&repository, "readme.txt", "first\n", "base");
        git_in(
            &repository,
            &["remote", "add", "origin", remote.to_str().unwrap()],
        );
        git_in(&repository, &["push", "-u", "origin", "main"]);

        // A local branch sharing the remote-tracking name makes every
        // abbreviated `origin/main` ref ambiguous. Git then reports the
        // upstream as `remotes/origin/main`, which must not become a remote
        // called `remotes`.
        git_in(&repository, &["branch", "origin/main", "main"]);

        let path = repository.to_str().unwrap();
        let info = push_info(path).unwrap();
        assert_eq!(info.remote, "origin");
        assert_eq!(info.remote_branch, "main");
        assert_eq!(info.upstream.as_deref(), Some("origin/main"));

        commit_file(&repository, "readme.txt", "first\nsecond\n", "second");
        let info = push_info(path).unwrap();
        assert_eq!((info.ahead, info.behind), (1, 0));
        assert_eq!(info.commits.len(), 1);

        push_repository(path, None).unwrap();
        assert_eq!(
            git_in(&remote, &["rev-parse", "main"]),
            git_in(&repository, &["rev-parse", "HEAD"])
        );
        assert_eq!(push_info(path).unwrap().ahead, 0);

        // The branch manager lists both the local shadow and the remote
        // branch under their real names.
        let branches = list_branch_info(path).unwrap();
        assert!(
            branches
                .iter()
                .any(|branch| branch.name == "origin/main" && !branch.is_remote)
        );
        let tracking = branches
            .iter()
            .find(|branch| branch.name == "origin/main" && branch.is_remote)
            .unwrap();
        assert_eq!(tracking.remote.as_deref(), Some("origin"));

        let inspection = inspect_root(&repository, true).unwrap();
        assert_eq!(inspection.upstream.as_deref(), Some("origin/main"));
        assert!(
            inspection
                .branches
                .iter()
                .any(|branch| branch == "origin/main")
        );

        fs::remove_dir_all(&root).unwrap();
    }

    fn init_repo(label: &str) -> (PathBuf, PathBuf) {
        let root = temp_root(label);
        let repo = root.join("repo");
        fs::create_dir_all(&repo).unwrap();
        git_in(&repo, &["init", "-b", "main"]);
        configure_user(&repo);
        (root, repo)
    }

    fn file_id<'a>(
        snapshot: &'a crate::RepositorySnapshot,
        id: &str,
    ) -> Option<&'a crate::RepositoryFile> {
        snapshot.files.iter().find(|file| file.id == id)
    }

    #[test]
    fn splits_staged_and_unstaged_changes() {
        let (root, repo) = init_repo("split");
        commit_file(&repo, "app.txt", "one\ntwo\nthree\n", "initial");
        let path = repo.to_string_lossy().to_string();

        fs::write(repo.join("app.txt"), "one\nTWO\nthree\n").unwrap();
        let snapshot = repository_snapshot(&path, None).unwrap();
        let file = file_id(&snapshot, "unstaged:app.txt").unwrap();
        assert!(!file.staged);
        assert!(file.unstaged);
        assert!(!file.untracked);
        assert_eq!(file.status, ChangeStatus::Modified);
        assert_eq!(file.hunks.len(), 1);

        stage_files(&path, &["app.txt".to_string()]).unwrap();
        let snapshot = repository_snapshot(&path, None).unwrap();
        assert!(file_id(&snapshot, "staged:app.txt").is_some());
        assert!(file_id(&snapshot, "unstaged:app.txt").is_none());

        // Staged and unstaged changes to the same file appear as two entries.
        fs::write(repo.join("app.txt"), "one\nTWO\nTHREE\n").unwrap();
        let snapshot = repository_snapshot(&path, None).unwrap();
        let staged = file_id(&snapshot, "staged:app.txt").unwrap();
        let unstaged = file_id(&snapshot, "unstaged:app.txt").unwrap();
        assert!(staged.staged && !staged.unstaged);
        assert!(!unstaged.staged && unstaged.unstaged);
        assert_eq!(unstaged.hunks.len(), 1);

        unstage_files(&path, &["app.txt".to_string()]).unwrap();
        let snapshot = repository_snapshot(&path, None).unwrap();
        assert!(file_id(&snapshot, "staged:app.txt").is_none());
        assert!(file_id(&snapshot, "unstaged:app.txt").is_some());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn stages_unstages_and_discards_individual_hunks() {
        let (root, repo) = init_repo("hunks");
        let original = (1..=40)
            .map(|line| format!("line {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        commit_file(&repo, "long.txt", &(original.clone() + "\n"), "initial");
        let mut changed: Vec<String> = original.lines().map(str::to_string).collect();
        changed[1] = "line 2 changed".to_string();
        changed[34] = "line 35 changed".to_string();
        fs::write(repo.join("long.txt"), changed.join("\n") + "\n").unwrap();
        let path = repo.to_string_lossy().to_string();

        let snapshot = repository_snapshot(&path, None).unwrap();
        let file = file_id(&snapshot, "unstaged:long.txt").unwrap();
        assert_eq!(file.hunks.len(), 2);
        let first_header = file.hunks[0].header.clone();

        stage_hunk(&path, "long.txt", &first_header).unwrap();
        let snapshot = repository_snapshot(&path, None).unwrap();
        let staged = file_id(&snapshot, "staged:long.txt").unwrap();
        let unstaged = file_id(&snapshot, "unstaged:long.txt").unwrap();
        assert_eq!(staged.hunks.len(), 1);
        assert_eq!(unstaged.hunks.len(), 1);
        assert!(git_in(&repo, &["diff", "--cached"]).contains("line 2 changed"));
        assert!(!git_in(&repo, &["diff", "--cached"]).contains("line 35 changed"));

        let staged_header = staged.hunks[0].header.clone();
        unstage_hunk(&path, "long.txt", &staged_header).unwrap();
        let snapshot = repository_snapshot(&path, None).unwrap();
        assert!(file_id(&snapshot, "staged:long.txt").is_none());
        let unstaged = file_id(&snapshot, "unstaged:long.txt").unwrap();
        assert_eq!(unstaged.hunks.len(), 2);

        let second_header = unstaged.hunks[1].header.clone();
        discard_hunk(&path, "long.txt", &second_header).unwrap();
        let content = fs::read_to_string(repo.join("long.txt")).unwrap();
        assert!(content.contains("line 2 changed"));
        assert!(content.contains("line 35\n"));
        assert!(!content.contains("line 35 changed"));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn commits_and_amends_staged_changes() {
        let (root, repo) = init_repo("commit");
        commit_file(&repo, "app.txt", "one\n", "initial");
        let path = repo.to_string_lossy().to_string();

        fs::write(repo.join("app.txt"), "one\ntwo\n").unwrap();
        stage_files(&path, &["app.txt".to_string()]).unwrap();
        let outcome = commit_repository(&path, "add second line", false).unwrap();
        assert_eq!(outcome.subject, "add second line");
        assert_eq!(outcome.files_changed, 1);
        assert_eq!(
            git_in(&repo, &["log", "-1", "--format=%s"]),
            "add second line"
        );

        fs::write(repo.join("app.txt"), "one\ntwo\nthree\n").unwrap();
        stage_files(&path, &["app.txt".to_string()]).unwrap();
        let amended = commit_repository(&path, "add two lines", true).unwrap();
        assert_eq!(amended.subject, "add two lines");
        assert_eq!(git_in(&repo, &["rev-list", "--count", "HEAD"]), "2");

        let nothing = commit_repository(&path, "nothing", false).unwrap_err();
        assert_eq!(nothing.code, "nothing_staged");

        let empty = commit_repository(&path, "  ", false).unwrap_err();
        assert_eq!(empty.code, "empty_message");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn discards_tracked_and_untracked_changes() {
        let (root, repo) = init_repo("discard");
        commit_file(&repo, "app.txt", "one\n", "initial");
        let path = repo.to_string_lossy().to_string();

        fs::write(repo.join("app.txt"), "one\ntwo\n").unwrap();
        discard_file(&path, "app.txt", None, false, false).unwrap();
        assert_eq!(fs::read_to_string(repo.join("app.txt")).unwrap(), "one\n");

        fs::write(repo.join("app.txt"), "one\nthree\n").unwrap();
        stage_files(&path, &["app.txt".to_string()]).unwrap();
        discard_file(&path, "app.txt", None, false, true).unwrap();
        assert_eq!(fs::read_to_string(repo.join("app.txt")).unwrap(), "one\n");
        assert!(git_in(&repo, &["diff", "--cached"]).is_empty());

        fs::write(repo.join("new.txt"), "new\n").unwrap();
        discard_file(&path, "new.txt", None, true, false).unwrap();
        assert!(!repo.join("new.txt").exists());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn switches_creates_and_stashes_branches() {
        let (root, repo) = init_repo("branches");
        commit_file(&repo, "app.txt", "one\n", "initial");
        let path = repo.to_string_lossy().to_string();

        let created = switch_branch(&path, "feature", true, false).unwrap();
        assert_eq!(created.branch, "feature");
        assert!(!created.stashed);
        assert_eq!(git_in(&repo, &["branch", "--show-current"]), "feature");
        git_in(&repo, &["checkout", "main"]);

        fs::write(repo.join("app.txt"), "one\nwork in progress\n").unwrap();
        let blocked = switch_branch(&path, "feature", false, false).unwrap_err();
        assert_eq!(blocked.code, "dirty_worktree");

        let switched = switch_branch(&path, "feature", false, true).unwrap();
        assert!(switched.stashed);
        assert_eq!(git_in(&repo, &["branch", "--show-current"]), "feature");
        assert_eq!(fs::read_to_string(repo.join("app.txt")).unwrap(), "one\n");
        let stashes = stash_list(&path).unwrap();
        assert_eq!(stashes.len(), 1);

        let outcome = stash_apply(&path, 0).unwrap();
        assert!(outcome.conflicts.is_empty());
        assert!(
            fs::read_to_string(repo.join("app.txt"))
                .unwrap()
                .contains("work in progress")
        );
        stash_drop(&path, 0).unwrap();
        assert!(stash_list(&path).unwrap().is_empty());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn fetches_new_commits_from_a_remote() {
        let (root, repo) = init_repo("fetch");
        let remote = root.join("remote.git");
        fs::create_dir_all(&remote).unwrap();
        git_in(&remote, &["init", "--bare"]);
        let path = repo.to_string_lossy().to_string();
        commit_file(&repo, "app.txt", "one\n", "initial");
        set_remote_url(&path, "origin", &remote.to_string_lossy()).unwrap();
        push_repository(&path, None).unwrap();

        let other = root.join("other");
        git_in(
            &root,
            &["clone", remote.to_str().unwrap(), other.to_str().unwrap()],
        );
        configure_user(&other);
        commit_file(&other, "app.txt", "one\ntwo\n", "remote change");
        git_in(&other, &["push", "origin", "main"]);

        let before = repository_snapshot(&path, None).unwrap();
        assert_eq!(before.inspection.behind, 0);
        assert!(before.inspection.upstream.is_some());
        fetch_repository(&path, None).unwrap();
        let after = repository_snapshot(&path, None).unwrap();
        assert_eq!(after.inspection.behind, 1);
        assert_eq!(after.inspection.ahead, 0);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn reads_history_compares_refs_and_searches() {
        let (root, repo) = init_repo("history");
        let path = repo.to_string_lossy().to_string();
        commit_file(&repo, "app.txt", "alpha\n", "first commit");
        commit_file(&repo, "app.txt", "alpha\nbeta\n", "second change");
        commit_file(&repo, "other.txt", "gamma\n", "third change");

        let log = commit_log(&path, None, None, 2, 0).unwrap();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].subject, "third change");
        assert_eq!(log[1].subject, "second change");
        assert!(!log[0].parents.is_empty());

        let file_log = commit_log(&path, None, Some("app.txt"), 10, 0).unwrap();
        assert_eq!(file_log.len(), 2);
        assert!(
            file_log
                .iter()
                .all(|commit| commit.subject != "third change")
        );

        let matches = search_commits(&path, "second", 10).unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].subject, "second change");

        let file_hits = search_files(&path, "gamma", 10).unwrap();
        assert!(file_hits.iter().any(|hit| hit.path == "other.txt"));

        let comparison = compare_refs(&path, "HEAD~2", "HEAD").unwrap();
        assert!(comparison.files.iter().any(|file| file.path == "app.txt"));
        assert!(comparison.files.iter().any(|file| file.path == "other.txt"));

        fs::remove_dir_all(&root).unwrap();
    }
    #[test]
    fn reads_file_sides_for_image_diffs() {
        let (root, repo) = init_repo("file-sides");
        let path = repo.to_string_lossy().to_string();
        let original = [0u8, 1, 2, 3, 4, 5];
        fs::write(repo.join("icon.bin"), original).unwrap();
        git_in(&repo, &["add", "icon.bin"]);
        git_in(&repo, &["commit", "-m", "add icon"]);

        assert_eq!(
            read_file_side(&path, "icon.bin", FileSide::Head).unwrap(),
            Some(original.to_vec())
        );
        assert_eq!(
            read_file_side(&path, "icon.bin", FileSide::Index).unwrap(),
            Some(original.to_vec())
        );
        assert_eq!(
            read_file_side(&path, "icon.bin", FileSide::Worktree).unwrap(),
            Some(original.to_vec())
        );

        let changed = [0u8, 9, 8, 7];
        fs::write(repo.join("icon.bin"), changed).unwrap();
        assert_eq!(
            read_file_side(&path, "icon.bin", FileSide::Index).unwrap(),
            Some(original.to_vec())
        );
        assert_eq!(
            read_file_side(&path, "icon.bin", FileSide::Worktree).unwrap(),
            Some(changed.to_vec())
        );

        stage_files(&path, &["icon.bin".to_string()]).unwrap();
        assert_eq!(
            read_file_side(&path, "icon.bin", FileSide::Head).unwrap(),
            Some(original.to_vec())
        );
        assert_eq!(
            read_file_side(&path, "icon.bin", FileSide::Index).unwrap(),
            Some(changed.to_vec())
        );

        fs::remove_dir_all(&root).unwrap();
    }
    #[test]
    fn lists_branches_with_worktree_owners() {
        let (root, repo) = init_repo("branch-list");
        let path = repo.to_string_lossy().to_string();
        commit_file(&repo, "app.txt", "one\n", "initial");
        git_in(&repo, &["branch", "feature"]);
        let worktree_path = root.join("feature-checkout");
        add_worktree(
            &path,
            WorktreeAddOptions {
                path: worktree_path.to_string_lossy().to_string(),
                branch: Some("feature".to_string()),
                new_branch: None,
                start_point: None,
                detach: false,
            },
        )
        .unwrap();

        let branches = list_branch_info(&path).unwrap();
        let main = branches
            .iter()
            .find(|branch| branch.name == "main")
            .unwrap();
        let feature = branches
            .iter()
            .find(|branch| branch.name == "feature")
            .unwrap();
        assert!(main.is_current);
        assert!(!feature.is_current);
        assert!(feature.merged);
        assert!(
            feature
                .worktree_path
                .as_deref()
                .is_some_and(|path| path.ends_with("feature-checkout"))
        );

        remove_worktree(&path, &worktree_path.to_string_lossy(), false).unwrap();
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn creates_renames_and_deletes_branches() {
        let (root, repo) = init_repo("branch-crud");
        let path = repo.to_string_lossy().to_string();
        commit_file(&repo, "app.txt", "one\n", "initial");

        create_branch(&path, "feature", None, false).unwrap();
        assert!(
            list_branch_info(&path)
                .unwrap()
                .iter()
                .any(|branch| branch.name == "feature")
        );
        rename_branch(&path, "feature", "feature-renamed").unwrap();
        let branches = list_branch_info(&path).unwrap();
        assert!(!branches.iter().any(|branch| branch.name == "feature"));
        assert!(
            branches
                .iter()
                .any(|branch| branch.name == "feature-renamed")
        );

        // A branch with unmerged commits cannot be deleted without force.
        switch_branch(&path, "feature-renamed", false, false).unwrap();
        commit_file(&repo, "app.txt", "one\ntwo\n", "unmerged work");
        switch_branch(&path, "main", false, false).unwrap();
        let blocked = delete_branch(&path, "feature-renamed", false).unwrap_err();
        assert_eq!(blocked.code, "branch_not_merged");
        delete_branch(&path, "feature-renamed", true).unwrap();
        assert!(
            !list_branch_info(&path)
                .unwrap()
                .iter()
                .any(|branch| branch.name == "feature-renamed")
        );

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn merges_branches_and_reports_conflicts() {
        let (root, repo) = init_repo("branch-merge");
        let path = repo.to_string_lossy().to_string();
        commit_file(&repo, "app.txt", "one\n", "initial");
        git_in(&repo, &["branch", "feature"]);
        switch_branch(&path, "feature", false, false).unwrap();
        commit_file(&repo, "app.txt", "feature\n", "feature change");
        switch_branch(&path, "main", false, false).unwrap();
        commit_file(&repo, "app.txt", "main\n", "main change");

        let outcome = merge_branch(&path, "feature").unwrap();
        assert_eq!(outcome.conflicts.len(), 1);
        assert!(!conflict_entries(&repo).unwrap().is_empty());
        abort_merge(&path).unwrap();
        assert!(conflict_entries(&repo).unwrap().is_empty());

        // Fast-forward merge after resetting the conflicting change.
        fs::write(repo.join("app.txt"), "one\n").unwrap();
        git_in(&repo, &["commit", "-am", "revert main change"]);
        let outcome = merge_branch(&path, "feature").unwrap();
        assert!(outcome.conflicts.is_empty());
        assert_eq!(git_in(&repo, &["show", "HEAD:app.txt"]), "feature");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn adds_removes_and_prunes_worktrees() {
        let (root, repo) = init_repo("worktree-crud");
        let path = repo.to_string_lossy().to_string();
        commit_file(&repo, "app.txt", "one\n", "initial");
        let worktree_path = root.join("new-checkout");
        add_worktree(
            &path,
            WorktreeAddOptions {
                path: worktree_path.to_string_lossy().to_string(),
                branch: None,
                new_branch: Some("feature".to_string()),
                start_point: Some("HEAD".to_string()),
                detach: false,
            },
        )
        .unwrap();
        let summaries = list_worktrees(&path).unwrap();
        assert!(
            summaries
                .iter()
                .any(|summary| summary.info.branch == "feature" && summary.changed == 0)
        );

        fs::write(worktree_path.join("dirty.txt"), "dirty\n").unwrap();
        let dirty = remove_worktree(&path, &worktree_path.to_string_lossy(), false).unwrap_err();
        assert_eq!(dirty.code, "worktree_dirty");
        remove_worktree(&path, &worktree_path.to_string_lossy(), true).unwrap();
        assert!(!worktree_path.exists());

        // A deleted worktree directory is pruned from the worktree list.
        let prunable_path = root.join("prunable-checkout");
        add_worktree(
            &path,
            WorktreeAddOptions {
                path: prunable_path.to_string_lossy().to_string(),
                branch: None,
                new_branch: Some("prunable".to_string()),
                start_point: Some("HEAD".to_string()),
                detach: false,
            },
        )
        .unwrap();
        fs::remove_dir_all(&prunable_path).unwrap();
        assert_eq!(prune_worktrees(&path).unwrap(), 1);

        fs::remove_dir_all(&root).unwrap();
    }
}
