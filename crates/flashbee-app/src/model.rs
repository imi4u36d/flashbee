use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use flashbee_git::{
    ConflictEntry, RemoteBranchInfo, RemoteInfo, RepositoryFile, RepositorySnapshot, WorktreeInfo,
};
use serde::{Deserialize, Serialize};

use crate::i18n::{Lang, LocalePreference};
use crate::theme::{ResolvedTheme, ThemeMode};

pub const SIDEBAR_WIDTH_MIN: f32 = 240.0;
pub const SIDEBAR_WIDTH_MAX: f32 = 480.0;
pub const DEFAULT_SIDEBAR_WIDTH: f32 = 292.0;
pub const UI_SIZE_MIN: f32 = 13.0;
pub const UI_SIZE_MAX: f32 = 18.0;
pub const CODE_SIZE_MIN: f32 = 11.0;
pub const CODE_SIZE_MAX: f32 = 20.0;

pub fn config_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.flashbee.app")
}

fn read_json<T: for<'de> Deserialize<'de>>(name: &str) -> Option<T> {
    let path = config_dir().join(name);
    let contents = fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}

fn write_json<T: Serialize>(name: &str, value: &T) -> Result<(), String> {
    let directory = config_dir();
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let path = directory.join(name);
    let temporary = directory.join(format!("{name}.tmp"));
    let contents = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    fs::write(&temporary, contents).map_err(|error| error.to_string())?;
    fs::rename(&temporary, &path).map_err(|error| error.to_string())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub version: u32,
    pub theme: ThemeMode,
    pub locale: LocalePreference,
    pub ui_size: f32,
    pub code_size: f32,
    pub auto_fetch: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 4,
            theme: ThemeMode::System,
            locale: LocalePreference::System,
            ui_size: 15.0,
            code_size: 14.0,
            auto_fetch: true,
        }
    }
}

impl Settings {
    pub fn lang(&self) -> Lang {
        Lang::from_preference(self.locale)
    }

    pub fn resolved_theme(&self, system_dark: bool) -> ResolvedTheme {
        match self.theme {
            ThemeMode::Light => ResolvedTheme::Light,
            ThemeMode::Dark => ResolvedTheme::Dark,
            ThemeMode::System => {
                if system_dark {
                    ResolvedTheme::Dark
                } else {
                    ResolvedTheme::Light
                }
            }
        }
    }

    pub fn clamp(mut self) -> Self {
        self.ui_size = self.ui_size.clamp(UI_SIZE_MIN, UI_SIZE_MAX);
        self.code_size = self.code_size.clamp(CODE_SIZE_MIN, CODE_SIZE_MAX);
        self
    }
}

pub fn load_settings() -> Settings {
    read_json::<Settings>("settings.json")
        .unwrap_or_default()
        .clamp()
}

pub fn save_settings(settings: &Settings) {
    let _ = write_json("settings.json", settings);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LayoutState {
    pub version: u32,
    pub sidebar_width: f32,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            version: 1,
            sidebar_width: DEFAULT_SIDEBAR_WIDTH,
        }
    }
}

pub fn load_layout() -> LayoutState {
    let mut layout = read_json::<LayoutState>("layout.json").unwrap_or_default();
    layout.sidebar_width = if layout.sidebar_width <= 0.0 {
        DEFAULT_SIDEBAR_WIDTH
    } else {
        layout
            .sidebar_width
            .clamp(SIDEBAR_WIDTH_MIN, SIDEBAR_WIDTH_MAX)
    };
    layout
}

pub fn save_layout(layout: &LayoutState) {
    let _ = write_json("layout.json", layout);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SavedWindowState {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub maximized: bool,
}

impl Default for SavedWindowState {
    fn default() -> Self {
        Self {
            width: 1360.0,
            height: 880.0,
            x: 0.0,
            y: 0.0,
            maximized: false,
        }
    }
}

pub fn load_window_state() -> SavedWindowState {
    read_json::<SavedWindowState>("window-state.json").unwrap_or_default()
}

pub fn save_window_state(state: &SavedWindowState) {
    let _ = write_json("window-state.json", state);
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ReviewStore {
    pub version: u32,
    /// repository path -> branch key -> file id -> diff fingerprint
    pub repos: HashMap<String, HashMap<String, HashMap<String, String>>>,
}

impl ReviewStore {
    pub fn load() -> Self {
        read_json::<ReviewStore>("reviews.json").unwrap_or_default()
    }

    pub fn save(&self) {
        let _ = write_json("reviews.json", self);
    }

    pub fn reviewed(&self, repo: &str, branch: &str, file_id: &str, fingerprint: &str) -> bool {
        self.repos
            .get(repo)
            .and_then(|branches| branches.get(branch))
            .and_then(|files| files.get(file_id))
            .is_some_and(|value| value == fingerprint)
    }

    pub fn set_reviewed(
        &mut self,
        repo: &str,
        branch: &str,
        file_id: &str,
        fingerprint: &str,
        reviewed: bool,
    ) {
        let branches = self.repos.entry(repo.to_string()).or_default();
        let files = branches.entry(branch.to_string()).or_default();
        if reviewed {
            files.insert(file_id.to_string(), fingerprint.to_string());
        } else {
            files.remove(file_id);
        }
    }

    pub fn clear_branch(&mut self, repo: &str, branch: &str) {
        if let Some(branches) = self.repos.get_mut(repo) {
            branches.remove(branch);
        }
    }

    pub fn reviewed_count(&self, repo: &str, branch: &str) -> usize {
        self.repos
            .get(repo)
            .and_then(|branches| branches.get(branch))
            .map(|files| files.len())
            .unwrap_or(0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectGroup {
    pub id: String,
    pub name: String,
    pub path: String,
    pub collapsed: bool,
}

impl Default for ProjectGroup {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            path: String::new(),
            collapsed: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub branch: String,
    pub current_branch: String,
    pub ahead: u32,
    pub behind: u32,
    pub accent: String,
    pub accent_soft: String,
    pub updated: String,
    pub branches: Vec<String>,
    pub remote_branches: Vec<RemoteBranchInfo>,
    pub remotes: Vec<RemoteInfo>,
    pub upstream: Option<String>,
    pub source: String,
    pub group_id: Option<String>,
    pub worktree_of: Option<String>,
    pub worktree_head: Option<String>,
    pub hydrated: bool,
    #[serde(skip)]
    pub files: Vec<RepositoryFile>,
    #[serde(skip)]
    pub conflicts: Vec<ConflictEntry>,
    #[serde(skip)]
    pub merge_in_progress: bool,
    #[serde(skip)]
    pub loading: bool,
    #[serde(skip)]
    pub last_error: Option<String>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            path: String::new(),
            branch: "main".to_string(),
            current_branch: "main".to_string(),
            ahead: 0,
            behind: 0,
            accent: "#f6bd60".to_string(),
            accent_soft: "rgba(246, 189, 96, 0.12)".to_string(),
            updated: "now".to_string(),
            branches: Vec::new(),
            remote_branches: Vec::new(),
            remotes: Vec::new(),
            upstream: None,
            source: "local".to_string(),
            group_id: None,
            worktree_of: None,
            worktree_head: None,
            hydrated: false,
            files: Vec::new(),
            conflicts: Vec::new(),
            merge_in_progress: false,
            loading: false,
            last_error: None,
        }
    }
}

impl Project {
    pub fn is_local(&self) -> bool {
        self.source == "local"
    }

    pub fn is_worktree(&self) -> bool {
        self.worktree_of.is_some()
    }

    pub fn staged_files(&self) -> Vec<&RepositoryFile> {
        self.files.iter().filter(|file| file.staged).collect()
    }

    pub fn unstaged_files(&self) -> Vec<&RepositoryFile> {
        self.files.iter().filter(|file| !file.staged).collect()
    }

    pub fn changed_count(&self) -> usize {
        self.files.len()
    }

    pub fn apply_snapshot(&mut self, snapshot: RepositorySnapshot) {
        let RepositorySnapshot {
            inspection,
            files,
            conflicts,
            merge_in_progress,
            ..
        } = snapshot;
        self.files = files;
        self.conflicts = conflicts;
        self.merge_in_progress = merge_in_progress;
        self.branches = inspection.branches;
        self.remote_branches = inspection.remote_branches;
        self.remotes = inspection.remotes;
        self.upstream = inspection.upstream;
        self.ahead = inspection.ahead;
        self.behind = inspection.behind;
        self.current_branch = inspection.current_branch.clone();
        if !self.branches.iter().any(|branch| branch == &self.branch) {
            self.branch = inspection.current_branch;
        }
        self.hydrated = true;
        self.loading = false;
        self.last_error = None;
        self.updated = "now".to_string();
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub projects: Vec<Project>,
    pub groups: Vec<ProjectGroup>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorkspaceState {
    pub version: u32,
    pub workspaces: Vec<Workspace>,
    pub active_workspace_id: String,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self {
            version: 2,
            workspaces: vec![Workspace {
                id: "personal".to_string(),
                name: "Personal workspace".to_string(),
                projects: Vec::new(),
                groups: Vec::new(),
            }],
            active_workspace_id: "personal".to_string(),
        }
    }
}

impl WorkspaceState {
    pub fn active(&self) -> Option<&Workspace> {
        self.workspaces
            .iter()
            .find(|workspace| workspace.id == self.active_workspace_id)
            .or_else(|| self.workspaces.first())
    }

    pub fn active_mut(&mut self) -> Option<&mut Workspace> {
        let id = self.active_workspace_id.clone();
        if self.workspaces.iter().any(|workspace| workspace.id == id) {
            self.workspaces
                .iter_mut()
                .find(|workspace| workspace.id == id)
        } else {
            self.workspaces.first_mut()
        }
    }

    pub fn sanitize(&mut self) {
        if self.workspaces.is_empty() {
            *self = Self::default();
            return;
        }
        if !self
            .workspaces
            .iter()
            .any(|workspace| workspace.id == self.active_workspace_id)
        {
            self.active_workspace_id = self.workspaces[0].id.clone();
        }
        for workspace in &mut self.workspaces {
            let group_ids: Vec<String> = workspace
                .groups
                .iter()
                .map(|group| group.id.clone())
                .collect();
            for project in &mut workspace.projects {
                if let Some(group_id) = project.group_id.clone() {
                    if !group_ids.contains(&group_id) {
                        project.group_id = None;
                    }
                }
            }
        }
    }
}

pub fn load_workspaces() -> WorkspaceState {
    let mut state = read_json::<WorkspaceState>("workspaces.json").unwrap_or_default();
    state.sanitize();
    state
}

pub fn save_workspaces(state: &WorkspaceState) {
    let mut persistable = state.clone();
    persistable.version = 2;
    for workspace in &mut persistable.workspaces {
        workspace
            .projects
            .retain(|project| project.worktree_of.is_none());
        for project in &mut workspace.projects {
            project.files.clear();
            project.conflicts.clear();
            project.merge_in_progress = false;
            project.loading = false;
        }
    }
    let _ = write_json("workspaces.json", &persistable);
}

pub const PROJECT_ACCENTS: [(&str, &str); 6] = [
    ("#f6bd60", "rgba(246, 189, 96, 0.12)"),
    ("#64d2e8", "rgba(100, 210, 232, 0.11)"),
    ("#b49cff", "rgba(180, 156, 255, 0.11)"),
    ("#83d4a4", "rgba(131, 212, 164, 0.11)"),
    ("#ff9f76", "rgba(255, 159, 118, 0.11)"),
    ("#8fc7ff", "rgba(143, 199, 255, 0.11)"),
];

pub fn random_id(prefix: &str) -> String {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    let noise = (stamp as u32) ^ std::process::id();
    format!("{prefix}-{stamp:x}-{noise:x}")
}

pub fn repository_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

pub fn create_project(
    name: &str,
    path: &str,
    branch: &str,
    existing_count: usize,
    group_id: Option<String>,
) -> Project {
    let (accent, accent_soft) = PROJECT_ACCENTS[existing_count % PROJECT_ACCENTS.len()];
    Project {
        id: random_id("repo"),
        name: name.to_string(),
        path: path.to_string(),
        branch: branch.to_string(),
        current_branch: branch.to_string(),
        accent: accent.to_string(),
        accent_soft: accent_soft.to_string(),
        branches: vec![branch.to_string()],
        group_id,
        ..Project::default()
    }
}

pub fn create_group(name: &str, path: &str) -> ProjectGroup {
    ProjectGroup {
        id: random_id("group"),
        name: name.to_string(),
        path: path.to_string(),
        collapsed: false,
    }
}

pub fn create_workspace(name: &str) -> Workspace {
    Workspace {
        id: random_id("workspace"),
        name: name.trim().to_string(),
        projects: Vec::new(),
        groups: Vec::new(),
    }
}

/// Derived worktree cards live only in memory: the parent snapshot recreates
/// them on every launch, so persisting them would resurrect dead checkouts.
pub fn merge_worktrees(
    projects: Vec<Project>,
    parent: &Project,
    worktrees: &[WorktreeInfo],
) -> Vec<Project> {
    let mut next: Vec<Project> = projects
        .into_iter()
        .filter(|project| project.worktree_of.as_deref() != Some(parent.path.as_str()))
        .collect();
    for (index, worktree) in worktrees.iter().enumerate() {
        if worktree.is_main {
            continue;
        }
        let id = format!("{}::{}", parent.id, worktree.path);
        let (accent, accent_soft) = PROJECT_ACCENTS[(index + 2) % PROJECT_ACCENTS.len()];
        next.push(Project {
            id,
            name: worktree.name.clone(),
            path: worktree.path.clone(),
            branch: worktree.branch.clone(),
            current_branch: worktree.branch.clone(),
            accent: accent.to_string(),
            accent_soft: accent_soft.to_string(),
            branches: vec![worktree.branch.clone()],
            source: "local".to_string(),
            worktree_of: Some(parent.path.clone()),
            worktree_head: worktree.detached.then(|| worktree.head.clone()),
            hydrated: false,
            ..Project::default()
        });
    }
    next
}

pub fn workspace_label(workspace: &Workspace, lang: Lang) -> String {
    match workspace.id.as_str() {
        "personal" => crate::i18n::t(lang, "sidebar.personalWorkspace").to_string(),
        "sample" => crate::i18n::t(lang, "sidebar.sampleWorkspace").to_string(),
        _ => workspace.name.clone(),
    }
}

pub fn clamp_sidebar_width(width: f32) -> f32 {
    width.clamp(SIDEBAR_WIDTH_MIN, SIDEBAR_WIDTH_MAX)
}
