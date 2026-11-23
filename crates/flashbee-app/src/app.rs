use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use flashbee_git::{
    BranchInfo, CommandError, CommitDetail, CompareDiff, ConflictDetail, ConflictSegment,
    ConflictSide, LogCommit, PushInfo, RepositoryFile, RepositorySnapshot, StashEntry,
    WorktreeAddOptions, WorktreeSummary,
};
use gpui::{
    AnyElement, Context, Entity, FocusHandle, Focusable, Pixels, Point, Render, SharedString,
    Window, WindowAppearance, actions, div, prelude::*, px,
};

use crate::diff;
use crate::i18n::{self, Lang};
use crate::input::TextInput;
use crate::model::{
    self, LayoutState, Project, ReviewStore, SavedWindowState, Settings, WorkspaceState,
    create_project, create_workspace, load_layout, load_settings, load_window_state,
    load_workspaces, merge_worktrees, save_layout, save_settings, save_window_state,
    save_workspaces, workspace_label,
};
use crate::theme::{GlobalTheme, ResolvedTheme, Theme, ThemeMode};
use crate::ui;

actions!(
    flashbee,
    [
        RefreshRepository,
        OpenRepository,
        NewWorkspace,
        ToggleSidebar,
        OpenPalette,
        SplitDiffAction,
        UnifiedDiffAction,
        OpenSettings,
        NextFileAction,
        PreviousFileAction,
        OpenCommitAction,
        FetchRepository,
        OpenHistoryView,
        OpenOverviewView,
        ToggleReviewedAction,
        StageAllAction,
        CloseOverlayAction,
        ToggleDiffLayoutAction,
        CopyPrDescriptionAction,
        OpenBranchesAction,
    ]
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MainView {
    Changes,
    Overview,
    History,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffMode {
    Split,
    Unified,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Error,
    Info,
}

#[derive(Clone)]
pub struct Toast {
    pub id: u64,
    pub text: String,
    pub kind: ToastKind,
}

pub struct CommitDialogState {
    pub input: Entity<TextInput>,
    pub amend: bool,
    pub push_after: bool,
    pub busy: bool,
    pub error: Option<String>,
}

pub struct BranchDialogState {
    pub input: Entity<TextInput>,
    pub busy: bool,
    pub error: Option<String>,
}

pub struct StashDialogState {
    pub input: Entity<TextInput>,
    pub include_untracked: bool,
    pub busy: bool,
    pub error: Option<String>,
    pub entries: Vec<StashEntry>,
    pub loading: bool,
}

#[derive(Clone)]
pub struct DiscoveredRepo {
    pub name: String,
    pub path: String,
    pub selected: bool,
}

pub struct AddRepoState {
    pub input: Entity<TextInput>,
    pub scanning: bool,
    pub results: Vec<DiscoveredRepo>,
    pub error: Option<String>,
    pub manual_note: Option<String>,
}

#[derive(Clone)]
pub struct ConfirmState {
    pub title: String,
    pub description: String,
    pub confirm_label: String,
    pub action: ConfirmAction,
}

#[derive(Clone)]
pub enum ConfirmAction {
    DiscardFile {
        project_id: String,
        file_id: String,
    },
    DiscardHunk {
        project_id: String,
        file_id: String,
        header: String,
    },
    DropStash {
        project_id: String,
        index: u32,
    },
    StashAndSwitch {
        project_id: String,
        branch: String,
    },
    RemoveProject {
        project_id: String,
    },
    RemoveWorkspace {
        workspace_id: String,
    },
    AbortMerge {
        project_id: String,
    },
    DeleteBranch {
        project_id: String,
        name: String,
        force: bool,
    },
    MergeBranch {
        project_id: String,
        source: String,
    },
    RemoveWorktree {
        project_id: String,
        path: String,
        force: bool,
    },
}

pub struct PushState {
    pub info: Option<PushInfo>,
    pub selected: HashSet<String>,
    pub force: bool,
    pub tags: bool,
    pub busy: bool,
    pub loading: bool,
    pub error: Option<String>,
    pub skip_commits: bool,
}

pub struct RemoteDialogState {
    pub name_input: Entity<TextInput>,
    pub url_input: Entity<TextInput>,
    pub editing: Option<String>,
    pub busy: bool,
    pub error: Option<String>,
    pub remotes: Vec<flashbee_git::RemoteInfo>,
    pub loading: bool,
}

pub struct WorkspaceDialogState {
    pub input: Entity<TextInput>,
    pub error: Option<String>,
}

pub struct HistoryState {
    pub log: Vec<LogCommit>,
    pub selected: Option<String>,
    pub detail: Option<CommitDetail>,
    pub detail_file: Option<String>,
    pub busy: bool,
    pub loading_detail: bool,
    pub error: Option<String>,
    pub file_filter: Option<String>,
    pub skip: u32,
    pub has_more: bool,
    pub compare_base: Entity<TextInput>,
    pub compare_target: Entity<TextInput>,
    pub compare: Option<CompareDiff>,
    pub compare_file: Option<String>,
    pub compare_busy: bool,
    pub show_compare: bool,
}

pub struct SearchResults {
    pub commits: Vec<(String, String, LogCommit)>,
    pub code: Vec<(String, String, u32, String)>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlameSide {
    Old,
    New,
}

#[derive(Clone)]
pub struct SidebarDrag;

struct DragPreview;

impl Render for DragPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size(px(1.)).opacity(0.0)
    }
}

#[derive(Clone)]
pub struct BlameHover {
    pub file_id: String,
    pub side: BlameSide,
    pub line: u32,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone)]
pub struct BranchInfoCache {
    pub project_path: String,
    pub branches: Vec<BranchInfo>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BranchTab {
    Branches,
    Worktrees,
}

pub struct BranchManagerState {
    pub project_id: String,
    pub tab: BranchTab,
    pub branches: Vec<BranchInfo>,
    pub worktrees: Vec<WorktreeSummary>,
    pub loading: bool,
    pub busy: bool,
    pub error: Option<String>,
    pub filter_input: Entity<TextInput>,
    pub create_name: Entity<TextInput>,
    pub create_start: Entity<TextInput>,
    pub create_switch: bool,
    pub rename_target: Option<String>,
    pub rename_input: Option<Entity<TextInput>>,
    pub worktree_branch: Entity<TextInput>,
    pub worktree_path: Entity<TextInput>,
    pub worktree_start: Entity<TextInput>,
    pub worktree_new_branch: bool,
}

pub struct ImageDiff {
    pub fingerprint: String,
    pub old_path: Option<PathBuf>,
    pub new_path: Option<PathBuf>,
    pub old_size: u64,
    pub new_size: u64,
}

pub struct SearchState {
    pub input: Entity<TextInput>,
    pub query: String,
    pub generation: u64,
    pub busy: bool,
    pub results: SearchResults,
}

pub struct FlashBeeApp {
    pub state: WorkspaceState,
    pub settings: Settings,
    pub layout: LayoutState,
    pub system_dark: bool,
    pub active_project_id: String,
    pub selected_by_project: HashMap<String, String>,
    pub expanded_project_id: Option<String>,
    pub mode: DiffMode,
    pub view: MainView,
    pub sidebar_collapsed: bool,
    pub resizing_sidebar: bool,
    pub toasts: Vec<Toast>,
    pub next_toast_id: u64,
    pub loading: HashSet<String>,
    pub folding_groups: HashSet<String>,
    pub settings_open: bool,
    pub conflict_open: bool,
    pub conflict_file: Option<String>,
    pub conflict_detail: Option<ConflictDetail>,
    pub conflict_choices: Vec<Option<ConflictSide>>,
    pub conflict_busy: bool,
    pub conflict_loading: bool,
    pub conflict_error: Option<String>,
    pub commit_dialog: Option<CommitDialogState>,
    pub branch_dialog: Option<BranchDialogState>,
    pub stash_dialog: Option<StashDialogState>,
    pub add_repo: Option<AddRepoState>,
    pub confirm: Option<ConfirmState>,
    pub push: Option<PushState>,
    pub remote_dialog: Option<RemoteDialogState>,
    pub workspace_dialog: Option<WorkspaceDialogState>,
    pub history: Option<HistoryState>,
    pub search: Option<SearchState>,
    pub reviews: ReviewStore,
    pub image_diffs: HashMap<String, ImageDiff>,
    pub image_loading: HashSet<String>,
    pub blame_cache: HashMap<String, flashbee_git::BlameLine>,
    pub blame_loading: HashSet<String>,
    pub blame_hover: Option<BlameHover>,
    pub blame_generation: u64,
    pub branch_info_cache: Option<BranchInfoCache>,
    pub branch_manager: Option<BranchManagerState>,
    pub pending_worktree_path: Option<String>,
    pub pending_focus: Option<Entity<TextInput>>,
    pub pending_main_focus: bool,
    pub main_focus: FocusHandle,
    pub branch_menu_project: Option<String>,
    pub repo_menu_project: Option<String>,
    pub workspace_menu_open: bool,
    pub filter_unreviewed: bool,
    pub hide_generated: bool,
    pub hide_whitespace: bool,
    pub file_filter: String,
    pub filter_input: Entity<TextInput>,
    pub file_filter_focus: FocusHandle,
    pub last_window_bounds: Option<(f32, f32, f32, f32)>,
    pub window_maximized: bool,
    pub last_auto_fetch: Instant,
    pub toast_timer_generation: u64,
}

impl FlashBeeApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut state = load_workspaces();
        state.sanitize();
        let settings = load_settings();
        let layout = load_layout();
        let active_project_id = state
            .active()
            .and_then(|workspace| {
                workspace
                    .projects
                    .iter()
                    .find(|project| project.worktree_of.is_none())
                    .or_else(|| workspace.projects.first())
            })
            .map(|project| project.id.clone())
            .unwrap_or_default();
        let expanded_project_id = if active_project_id.is_empty() {
            None
        } else {
            Some(active_project_id.clone())
        };
        let reviews = ReviewStore::load();
        let filter_lang = settings.lang();
        let filter_input =
            cx.new(|cx| TextInput::new(cx, i18n::t(filter_lang, "sidebar.filterPlaceholder"), ""));
        let filter_observer = filter_input.clone();
        let system_dark = true;
        let window_state = load_window_state();

        let mut app = Self {
            state,
            settings,
            layout,
            system_dark,
            active_project_id,
            selected_by_project: HashMap::new(),
            expanded_project_id,
            mode: DiffMode::Split,
            view: MainView::Changes,
            sidebar_collapsed: false,
            resizing_sidebar: false,
            toasts: Vec::new(),
            next_toast_id: 1,
            loading: HashSet::new(),
            folding_groups: HashSet::new(),
            settings_open: false,
            conflict_open: false,
            conflict_file: None,
            conflict_detail: None,
            conflict_choices: Vec::new(),
            conflict_busy: false,
            conflict_loading: false,
            conflict_error: None,
            commit_dialog: None,
            branch_dialog: None,
            stash_dialog: None,
            add_repo: None,
            confirm: None,
            push: None,
            remote_dialog: None,
            workspace_dialog: None,
            history: None,
            search: None,
            reviews,
            image_diffs: HashMap::new(),
            image_loading: HashSet::new(),
            blame_cache: HashMap::new(),
            blame_loading: HashSet::new(),
            blame_hover: None,
            blame_generation: 0,
            branch_info_cache: None,
            branch_manager: None,
            pending_worktree_path: None,
            pending_focus: None,
            pending_main_focus: true,
            main_focus: cx.focus_handle(),
            branch_menu_project: None,
            repo_menu_project: None,
            workspace_menu_open: false,
            filter_unreviewed: false,
            hide_generated: false,
            hide_whitespace: false,
            file_filter: String::new(),
            filter_input,
            file_filter_focus: cx.focus_handle(),
            last_window_bounds: Some((
                window_state.x,
                window_state.y,
                window_state.width,
                window_state.height,
            )),
            window_maximized: window_state.maximized,
            last_auto_fetch: Instant::now(),
            toast_timer_generation: 0,
        };

        let theme = Theme::for_theme(app.resolved_theme());
        cx.set_global(GlobalTheme(theme));
        cx.observe(&filter_observer, |app, input, cx| {
            let value = input.read(cx).content().to_string();
            if app.file_filter != value {
                app.file_filter = value;
                cx.notify();
            }
        })
        .detach();
        app.start_loading(cx);
        app.start_background_loop(cx);
        app
    }

    pub fn install_window_observers(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.system_dark = matches!(
            window.appearance(),
            WindowAppearance::Dark | WindowAppearance::VibrantDark
        );
        cx.set_global(GlobalTheme(Theme::for_theme(self.resolved_theme())));

        cx.observe_window_appearance(window, |app, window, cx| {
            let dark = matches!(
                window.appearance(),
                WindowAppearance::Dark | WindowAppearance::VibrantDark
            );
            if app.system_dark != dark {
                app.system_dark = dark;
                if app.settings.theme == ThemeMode::System {
                    cx.set_global(GlobalTheme(Theme::for_theme(app.resolved_theme())));
                }
                cx.notify();
            }
        })
        .detach();

        cx.observe_window_activation(window, |app, window, cx| {
            if window.is_window_active() {
                app.refresh_active(cx);
                app.maybe_auto_fetch(cx);
            }
        })
        .detach();

        cx.observe_window_bounds(window, |app, window, _cx| {
            app.remember_window_bounds(window);
        })
        .detach();

        cx.on_app_quit(|app, _cx| {
            app.persist_workspaces();
            if let Some((x, y, width, height)) = app.last_window_bounds {
                save_window_state(&SavedWindowState {
                    width,
                    height,
                    x,
                    y,
                    maximized: app.window_maximized,
                });
            }
            async {}
        })
        .detach();
    }

    pub fn resolved_theme(&self) -> ResolvedTheme {
        self.settings.resolved_theme(self.system_dark)
    }

    pub fn theme(&self) -> Theme {
        Theme::for_theme(self.resolved_theme())
    }

    pub fn lang(&self) -> Lang {
        self.settings.lang()
    }

    pub fn t(&self, key: &str) -> &'static str {
        i18n::t(self.lang(), key)
    }

    pub fn active_project(&self) -> Option<&Project> {
        let workspace = self.state.active()?;
        workspace
            .projects
            .iter()
            .find(|project| project.id == self.active_project_id)
            .or_else(|| workspace.projects.first())
    }

    pub fn active_project_mut(&mut self) -> Option<&mut Project> {
        let active_id = self.active_project_id.clone();
        let workspace = self.state.active_mut()?;
        let index = workspace
            .projects
            .iter()
            .position(|project| project.id == active_id)
            .or(if workspace.projects.is_empty() {
                None
            } else {
                Some(0)
            })?;
        workspace.projects.get_mut(index)
    }

    fn project_mut(&mut self, project_id: &str) -> Option<&mut Project> {
        self.state
            .active_mut()?
            .projects
            .iter_mut()
            .find(|project| project.id == project_id)
    }

    pub fn project(&self, project_id: &str) -> Option<&Project> {
        self.state
            .active()?
            .projects
            .iter()
            .find(|project| project.id == project_id)
    }

    pub fn active_file(&self) -> Option<&RepositoryFile> {
        let project = self.active_project()?;
        let selected = self.selected_by_project.get(&project.id);
        selected
            .and_then(|id| project.files.iter().find(|file| &file.id == id))
            .or_else(|| project.files.first())
    }

    pub fn file(&self, project_id: &str, file_id: &str) -> Option<&RepositoryFile> {
        self.project(project_id)?
            .files
            .iter()
            .find(|file| file.id == file_id)
    }

    pub fn reviewed(&self, project: &Project, file: &RepositoryFile) -> bool {
        self.reviews.reviewed(
            &project.path,
            &project.branch,
            &file.id,
            &diff::review_fingerprint(file),
        )
    }

    pub fn review_counts(&self, project: &Project) -> (usize, usize) {
        let total = project.files.len();
        let done = project
            .files
            .iter()
            .filter(|file| self.reviewed(project, file))
            .count();
        (done, total)
    }

    pub fn visible_files<'a>(&'a self, project: &'a Project) -> Vec<&'a RepositoryFile> {
        project
            .files
            .iter()
            .filter(|file| {
                if self.filter_unreviewed && self.reviewed(project, file) {
                    return false;
                }
                if self.hide_generated && diff::is_generated(&file.path) {
                    return false;
                }
                if self.hide_whitespace && diff::whitespace_only(file) {
                    return false;
                }
                if !self.file_filter.trim().is_empty() {
                    let filter = self.file_filter.to_lowercase();
                    if !file.path.to_lowercase().contains(&filter) {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    pub fn select_file(&mut self, project_id: &str, file_id: &str, cx: &mut Context<Self>) {
        self.reveal_group(project_id);
        self.active_project_id = project_id.to_string();
        self.expanded_project_id = Some(project_id.to_string());
        self.selected_by_project
            .insert(project_id.to_string(), file_id.to_string());
        self.view = MainView::Changes;
        cx.notify();
    }

    pub fn activate_project(&mut self, project_id: &str, cx: &mut Context<Self>) {
        if self.active_project_id == project_id {
            self.expanded_project_id = if self.expanded_project_id.as_deref() == Some(project_id) {
                None
            } else {
                Some(project_id.to_string())
            };
            cx.notify();
            return;
        }
        self.active_project_id = project_id.to_string();
        self.expanded_project_id = Some(project_id.to_string());
        self.view = MainView::Changes;
        self.conflict_open = self
            .project(project_id)
            .is_some_and(|project| !project.conflicts.is_empty() || project.merge_in_progress);
        self.branch_menu_project = None;
        self.repo_menu_project = None;
        if self
            .project(project_id)
            .is_some_and(|project| !project.hydrated)
        {
            self.load_project(project_id.to_string(), true, cx);
        }
        cx.notify();
    }

    fn reveal_group(&mut self, project_id: &str) {
        let Some(project) = self.project(project_id) else {
            return;
        };
        if let Some(group_id) = project.group_id.clone() {
            self.folding_groups.remove(&group_id);
        }
    }

    pub fn go_to_file(&mut self, offset: isize, cx: &mut Context<Self>) {
        let Some(project) = self.active_project() else {
            return;
        };
        let files = self.visible_files(project);
        if files.is_empty() {
            return;
        }
        let current = self
            .selected_by_project
            .get(&project.id)
            .cloned()
            .unwrap_or_default();
        let index = files
            .iter()
            .position(|file| file.id == current)
            .unwrap_or(0);
        let next = (index as isize + offset).clamp(0, files.len() as isize - 1) as usize;
        let id = files[next].id.clone();
        let project_id = project.id.clone();
        self.select_file(&project_id, &id, cx);
    }

    pub fn toast(&mut self, text: impl Into<String>, kind: ToastKind, cx: &mut Context<Self>) {
        let id = self.next_toast_id;
        self.next_toast_id += 1;
        self.toasts.push(Toast {
            id,
            text: text.into(),
            kind,
        });
        if self.toasts.len() > 3 {
            self.toasts.remove(0);
        }
        self.toast_timer_generation += 1;
        let generation = self.toast_timer_generation;
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(2800))
                .await;
            this.update(cx, |app, cx| {
                if app.toast_timer_generation == generation {
                    app.toasts.clear();
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    pub fn toast_error(&mut self, error: &CommandError, cx: &mut Context<Self>) {
        let message = error
            .message
            .lines()
            .next()
            .unwrap_or(&error.message)
            .to_string();
        self.toast(message, ToastKind::Error, cx);
    }

    pub fn persist_workspaces(&self) {
        save_workspaces(&self.state);
    }

    pub fn persist_settings(&self) {
        save_settings(&self.settings);
    }

    pub fn persist_layout(&self) {
        save_layout(&self.layout);
    }

    pub fn remember_window_bounds(&mut self, window: &Window) {
        let bounds = window.bounds();
        self.last_window_bounds = Some((
            bounds.origin.x.into(),
            bounds.origin.y.into(),
            bounds.size.width.into(),
            bounds.size.height.into(),
        ));
        self.window_maximized = window.is_maximized();
    }

    fn start_loading(&mut self, cx: &mut Context<Self>) {
        let mut targets: Vec<(String, String, String)> = self
            .state
            .active()
            .map(|workspace| {
                workspace
                    .projects
                    .iter()
                    .filter(|project| project.is_local() && !project.is_worktree())
                    .map(|project| {
                        (
                            project.id.clone(),
                            project.path.clone(),
                            project.branch.clone(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        targets.sort_by_key(|(id, _, _)| if id == &self.active_project_id { 0 } else { 1 });
        for (id, _, _) in targets {
            self.load_project(id, false, cx);
        }
    }

    fn start_background_loop(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_secs(240))
                    .await;
                let should_run = this
                    .update(cx, |app, cx| {
                        app.maybe_auto_fetch(cx);
                        true
                    })
                    .unwrap_or(false);
                if !should_run {
                    break;
                }
            }
        })
        .detach();
    }

    fn maybe_auto_fetch(&mut self, cx: &mut Context<Self>) {
        if !self.settings.auto_fetch || self.last_auto_fetch.elapsed() < Duration::from_secs(120) {
            return;
        }
        self.last_auto_fetch = Instant::now();
        self.fetch_all(true, cx);
    }

    pub fn load_project(&mut self, project_id: String, force: bool, cx: &mut Context<Self>) {
        let Some((path, branch)) = self
            .project(&project_id)
            .map(|project| (project.path.clone(), project.branch.clone()))
        else {
            return;
        };
        if force {
            self.loading.remove(&project_id);
        }
        if !self.loading.insert(project_id.clone()) {
            return;
        }
        if let Some(project) = self.project_mut(&project_id) {
            project.loading = true;
        }
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(
                    async move { flashbee_git::repository_snapshot(&path, Some(&branch)) },
                )
                .await;
            this.update(cx, |app, cx| {
                app.apply_snapshot_result(&project_id, result, cx);
            })
            .ok();
        })
        .detach();
    }

    fn apply_snapshot_result(
        &mut self,
        project_id: &str,
        result: Result<RepositorySnapshot, CommandError>,
        cx: &mut Context<Self>,
    ) {
        self.loading.remove(project_id);
        let mut children: Vec<String> = Vec::new();
        let mut selection_update: Option<Option<String>> = None;
        let mut error_message: Option<String> = None;

        if let Some(workspace) = self.state.active_mut() {
            if let Some(index) = workspace
                .projects
                .iter()
                .position(|project| project.id == project_id)
            {
                match result {
                    Ok(snapshot) => {
                        let worktrees = snapshot.inspection.worktrees.clone();
                        let is_parent = workspace.projects[index].worktree_of.is_none();
                        let project_id_string = workspace.projects[index].id.clone();
                        let previous_selection = self
                            .selected_by_project
                            .get(project_id)
                            .cloned()
                            .unwrap_or_default();
                        workspace.projects[index].apply_snapshot(snapshot);
                        workspace.projects[index].files.sort_by(|left, right| {
                            diff::is_generated(&left.path)
                                .cmp(&diff::is_generated(&right.path))
                                .then_with(|| right.staged.cmp(&left.staged))
                                .then_with(|| left.path.cmp(&right.path))
                        });
                        let parent = workspace.projects[index].clone();
                        if is_parent {
                            children = worktrees
                                .iter()
                                .filter(|worktree| !worktree.is_main)
                                .map(|worktree| format!("{project_id_string}::{}", worktree.path))
                                .collect();
                            workspace.projects =
                                merge_worktrees(workspace.projects.clone(), &parent, &worktrees);
                        }
                        if let Some(project) = workspace
                            .projects
                            .iter()
                            .find(|project| project.id == project_id)
                        {
                            if !project
                                .files
                                .iter()
                                .any(|file| file.id == previous_selection)
                            {
                                let previous_path = previous_selection
                                    .split_once(':')
                                    .map(|(_, path)| path)
                                    .unwrap_or(previous_selection.as_str());
                                let replacement = project
                                    .files
                                    .iter()
                                    .find(|file| file.path == previous_path)
                                    .map(|file| file.id.clone())
                                    .or_else(|| project.files.first().map(|file| file.id.clone()));
                                selection_update = Some(replacement);
                            }
                        }
                    }
                    Err(error) => {
                        workspace.projects[index].loading = false;
                        workspace.projects[index].last_error = Some(error.message.clone());
                        error_message = Some(error.message);
                    }
                }
            }
        }

        if let Some(selection) = selection_update {
            match selection {
                Some(file_id) => {
                    self.selected_by_project
                        .insert(project_id.to_string(), file_id);
                }
                None => {
                    self.selected_by_project.remove(project_id);
                }
            }
        }
        if let Some(message) = error_message {
            self.toast(message, ToastKind::Error, cx);
        }
        for child in children {
            if self
                .project(&child)
                .is_some_and(|project| !project.hydrated)
            {
                self.load_project(child, false, cx);
            }
        }
        if self.active_project_id == project_id
            && self
                .project(project_id)
                .is_some_and(|project| !project.conflicts.is_empty() || project.merge_in_progress)
        {
            if self.conflict_file.is_none() {
                if let Some(path) = self
                    .project(project_id)
                    .and_then(|project| project.conflicts.first())
                    .map(|conflict| conflict.path.clone())
                {
                    self.load_conflict(path, cx);
                }
            }
            self.conflict_open = true;
        }
        if let Some(target) = self.pending_worktree_path.clone() {
            let normalized = target.trim_end_matches('/');
            let found = self.state.active().and_then(|workspace| {
                workspace
                    .projects
                    .iter()
                    .find(|project| project.path.trim_end_matches('/') == normalized)
                    .map(|project| project.id.clone())
            });
            if let Some(id) = found {
                self.pending_worktree_path = None;
                self.activate_project(&id, cx);
            }
        }
        cx.notify();
    }

    pub fn refresh_active(&mut self, cx: &mut Context<Self>) {
        if let Some(project) = self.active_project() {
            if project.is_local() {
                let id = project.id.clone();
                self.load_project(id, true, cx);
            }
        }
    }

    pub fn refresh_project(&mut self, project_id: &str, cx: &mut Context<Self>) {
        self.load_project(project_id.to_string(), true, cx);
    }

    pub fn refresh_all(&mut self, cx: &mut Context<Self>) {
        let ids: Vec<String> = self
            .state
            .active()
            .map(|workspace| {
                workspace
                    .projects
                    .iter()
                    .filter(|project| project.is_local())
                    .map(|project| project.id.clone())
                    .collect()
            })
            .unwrap_or_default();
        for id in ids {
            self.load_project(id, true, cx);
        }
    }

    pub fn fetch_active(&mut self, cx: &mut Context<Self>) {
        let Some(project) = self.active_project() else {
            return;
        };
        if !project.is_local() {
            return;
        }
        let path = project.path.clone();
        let id = project.id.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::fetch_repository(&path, None) })
                .await;
            this.update(cx, |app, cx| match result {
                Ok(_) => {
                    app.toast(app.t("repo.fetched"), ToastKind::Success, cx);
                    app.load_project(id, true, cx);
                }
                Err(error) if error.code == "no_remote" => {
                    app.toast(app.t("repo.noRemote"), ToastKind::Info, cx);
                }
                Err(error) => app.toast_error(&error, cx),
            })
            .ok();
        })
        .detach();
    }

    pub fn fetch_all(&mut self, quiet: bool, cx: &mut Context<Self>) {
        let targets: Vec<(String, String)> = self
            .state
            .active()
            .map(|workspace| {
                workspace
                    .projects
                    .iter()
                    .filter(|project| {
                        project.is_local() && !project.is_worktree() && !project.remotes.is_empty()
                    })
                    .map(|project| (project.id.clone(), project.path.clone()))
                    .collect()
            })
            .unwrap_or_default();
        if targets.is_empty() {
            if !quiet {
                self.toast(self.t("repo.noRemote"), ToastKind::Info, cx);
            }
            return;
        }
        let total = targets.len();
        self.toast(
            i18n::fill(self.t("sidebar.fetchAll"), &[]),
            ToastKind::Info,
            cx,
        );
        cx.spawn(async move |this, cx| {
            let mut done = 0usize;
            for (id, path) in targets {
                let result = cx
                    .background_spawn(async move { flashbee_git::fetch_repository(&path, None) })
                    .await;
                if result.is_ok() {
                    done += 1;
                }
                this.update(cx, |app, cx| {
                    app.load_project(id, true, cx);
                })
                .ok();
            }
            this.update(cx, |app, cx| {
                if !quiet {
                    let message = if done == total {
                        i18n::fill(
                            app.t("toast.fetchAllDone"),
                            &[("count", &total.to_string())],
                        )
                    } else {
                        i18n::fill(
                            app.t("toast.fetchAllPartial"),
                            &[("done", &done.to_string()), ("total", &total.to_string())],
                        )
                    };
                    app.toast(message, ToastKind::Success, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn pull_active(&mut self, cx: &mut Context<Self>) {
        let Some(project) = self.active_project() else {
            return;
        };
        if !project.is_local() || !project.conflicts.is_empty() || project.merge_in_progress {
            return;
        }
        let path = project.path.clone();
        let id = project.id.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::pull_repository(&path) })
                .await;
            this.update(cx, |app, cx| {
                match result {
                    Ok(outcome) => {
                        if outcome.conflicts.is_empty() {
                            app.toast(app.t("repo.pulled"), ToastKind::Success, cx);
                        } else {
                            let count = outcome.conflicts.len().to_string();
                            let message =
                                i18n::fill(app.t("repo.pullConflicts"), &[("count", &count)]);
                            app.toast(message, ToastKind::Info, cx);
                        }
                    }
                    Err(error) if error.code == "no_remote" => {
                        app.toast(app.t("repo.noRemote"), ToastKind::Info, cx);
                    }
                    Err(error) if error.code == "no_upstream" => {
                        app.toast(app.t("repo.noUpstream"), ToastKind::Info, cx);
                    }
                    Err(error) => app.toast_error(&error, cx),
                }
                app.load_project(id, true, cx);
            })
            .ok();
        })
        .detach();
    }

    pub fn open_push(&mut self, cx: &mut Context<Self>) {
        let Some(project) = self.active_project() else {
            return;
        };
        if !project.is_local() {
            return;
        }
        let path = project.path.clone();
        self.push = Some(PushState {
            info: None,
            selected: HashSet::new(),
            force: false,
            tags: false,
            busy: false,
            loading: true,
            error: None,
            skip_commits: false,
        });
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::push_info(&path) })
                .await;
            this.update(cx, |app, cx| app.finish_push_info(result, cx))
                .ok();
        })
        .detach();
    }

    fn finish_push_info(&mut self, result: Result<PushInfo, CommandError>, cx: &mut Context<Self>) {
        let Some(push) = self.push.as_mut() else {
            return;
        };
        push.loading = false;
        match result {
            Ok(info) => {
                push.selected = info
                    .commits
                    .iter()
                    .map(|commit| commit.hash.clone())
                    .collect();
                push.info = Some(info);
            }
            Err(error) => {
                push.error = Some(error.message);
            }
        }
        cx.notify();
    }

    pub fn execute_push(&mut self, cx: &mut Context<Self>) {
        let Some(project) = self.active_project() else {
            return;
        };
        let Some(push) = self.push.as_ref() else {
            return;
        };
        if push.busy {
            return;
        }
        let path = project.path.clone();
        let id = project.id.clone();
        let force = push.force;
        let tags = push.tags;
        let skip_commits = push.skip_commits;
        let selected = push.selected.clone();
        let info = push.info.clone();
        let options = flashbee_git::PushOptions {
            commit: info.as_ref().and_then(|info| {
                info.commits
                    .iter()
                    .find(|commit| selected.contains(&commit.hash))
                    .map(|commit| commit.hash.clone())
            }),
            force,
            push_tags: tags,
            skip_commits,
            ..flashbee_git::PushOptions::default()
        };
        if let Some(push) = self.push.as_mut() {
            push.busy = true;
            push.error = None;
        }
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(
                    async move { flashbee_git::push_repository(&path, Some(options)) },
                )
                .await;
            this.update(cx, |app, cx| match result {
                Ok(_) => {
                    app.push = None;
                    app.toast(app.t("repo.pushed"), ToastKind::Success, cx);
                    app.load_project(id, true, cx);
                }
                Err(error) => {
                    if let Some(push) = app.push.as_mut() {
                        push.busy = false;
                        push.error = Some(error.message);
                    }
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn stage_file(&mut self, project_id: &str, file_id: &str, cx: &mut Context<Self>) {
        let Some(project) = self.project(project_id) else {
            return;
        };
        let Some(file) = project.files.iter().find(|file| file.id == file_id) else {
            return;
        };
        let path = project.path.clone();
        let mut paths = vec![file.path.clone()];
        if let Some(old_path) = file.old_path.clone() {
            paths.push(old_path);
        }
        let project_id = project_id.to_string();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::stage_files(&path, &paths) })
                .await;
            this.update(cx, |app, cx| {
                if let Err(error) = result {
                    app.toast_error(&error, cx);
                } else {
                    app.load_project(project_id, true, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn unstage_file(&mut self, project_id: &str, file_id: &str, cx: &mut Context<Self>) {
        let Some(project) = self.project(project_id) else {
            return;
        };
        let Some(file) = project.files.iter().find(|file| file.id == file_id) else {
            return;
        };
        let path = project.path.clone();
        let mut paths = vec![file.path.clone()];
        if let Some(old_path) = file.old_path.clone() {
            paths.push(old_path);
        }
        let project_id = project_id.to_string();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::unstage_files(&path, &paths) })
                .await;
            this.update(cx, |app, cx| {
                if let Err(error) = result {
                    app.toast_error(&error, cx);
                } else {
                    app.load_project(project_id, true, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn stage_all(&mut self, cx: &mut Context<Self>) {
        let Some(project) = self.active_project() else {
            return;
        };
        let path = project.path.clone();
        let paths: Vec<String> = project
            .files
            .iter()
            .filter(|file| !file.staged)
            .flat_map(|file| {
                let mut values = vec![file.path.clone()];
                if let Some(old_path) = file.old_path.clone() {
                    values.push(old_path);
                }
                values
            })
            .collect();
        if paths.is_empty() {
            return;
        }
        let id = project.id.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::stage_files(&path, &paths) })
                .await;
            this.update(cx, |app, cx| {
                if let Err(error) = result {
                    app.toast_error(&error, cx);
                } else {
                    app.load_project(id, true, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn unstage_all(&mut self, cx: &mut Context<Self>) {
        let Some(project) = self.active_project() else {
            return;
        };
        let path = project.path.clone();
        let paths: Vec<String> = project
            .files
            .iter()
            .filter(|file| file.staged)
            .flat_map(|file| {
                let mut values = vec![file.path.clone()];
                if let Some(old_path) = file.old_path.clone() {
                    values.push(old_path);
                }
                values
            })
            .collect();
        if paths.is_empty() {
            return;
        }
        let id = project.id.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::unstage_files(&path, &paths) })
                .await;
            this.update(cx, |app, cx| {
                if let Err(error) = result {
                    app.toast_error(&error, cx);
                } else {
                    app.load_project(id, true, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn request_discard_file(
        &mut self,
        project_id: &str,
        file_id: &str,
        cx: &mut Context<Self>,
    ) {
        let Some(file) = self.file(project_id, file_id).cloned() else {
            return;
        };
        let name = file.path.clone();
        self.confirm = Some(ConfirmState {
            title: self.t("diff.discardTitle").to_string(),
            description: i18n::fill(self.t("diff.discardDescription"), &[("name", &name)]),
            confirm_label: self.t("diff.discard").to_string(),
            action: ConfirmAction::DiscardFile {
                project_id: project_id.to_string(),
                file_id: file_id.to_string(),
            },
        });
        cx.notify();
    }

    fn discard_file(&mut self, project_id: &str, file_id: &str, cx: &mut Context<Self>) {
        let Some(project) = self.project(project_id) else {
            return;
        };
        let Some(file) = project.files.iter().find(|file| file.id == file_id) else {
            return;
        };
        let path = project.path.clone();
        let file_path = file.path.clone();
        let old_path = file.old_path.clone();
        let untracked = file.untracked;
        let staged = file.staged;
        let project_id = project_id.to_string();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    flashbee_git::discard_file(
                        &path,
                        &file_path,
                        old_path.as_deref(),
                        untracked,
                        staged,
                    )
                })
                .await;
            this.update(cx, |app, cx| {
                if let Err(error) = result {
                    app.toast_error(&error, cx);
                } else {
                    app.toast(app.t("diff.discarded"), ToastKind::Success, cx);
                    app.load_project(project_id, true, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn stage_hunk(
        &mut self,
        project_id: &str,
        file_id: &str,
        header: &str,
        cx: &mut Context<Self>,
    ) {
        let Some(project) = self.project(project_id) else {
            return;
        };
        let Some(file) = project.files.iter().find(|file| file.id == file_id) else {
            return;
        };
        let path = project.path.clone();
        let file_path = file.path.clone();
        let header = header.to_string();
        let project_id = project_id.to_string();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(
                    async move { flashbee_git::stage_hunk(&path, &file_path, &header) },
                )
                .await;
            this.update(cx, |app, cx| {
                if let Err(error) = result {
                    app.toast_error(&error, cx);
                } else {
                    app.load_project(project_id, true, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn unstage_hunk(
        &mut self,
        project_id: &str,
        file_id: &str,
        header: &str,
        cx: &mut Context<Self>,
    ) {
        let Some(project) = self.project(project_id) else {
            return;
        };
        let Some(file) = project.files.iter().find(|file| file.id == file_id) else {
            return;
        };
        let path = project.path.clone();
        let file_path = file.path.clone();
        let header = header.to_string();
        let project_id = project_id.to_string();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(
                    async move { flashbee_git::unstage_hunk(&path, &file_path, &header) },
                )
                .await;
            this.update(cx, |app, cx| {
                if let Err(error) = result {
                    app.toast_error(&error, cx);
                } else {
                    app.load_project(project_id, true, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn request_discard_hunk(
        &mut self,
        project_id: &str,
        file_id: &str,
        header: &str,
        cx: &mut Context<Self>,
    ) {
        let Some(file) = self.file(project_id, file_id) else {
            return;
        };
        let name = file.path.clone();
        self.confirm = Some(ConfirmState {
            title: self.t("diff.discardTitle").to_string(),
            description: i18n::fill(self.t("diff.discardHunkDescription"), &[("name", &name)]),
            confirm_label: self.t("diff.discard").to_string(),
            action: ConfirmAction::DiscardHunk {
                project_id: project_id.to_string(),
                file_id: file_id.to_string(),
                header: header.to_string(),
            },
        });
        cx.notify();
    }

    fn discard_hunk(
        &mut self,
        project_id: &str,
        file_id: &str,
        header: &str,
        cx: &mut Context<Self>,
    ) {
        let Some(project) = self.project(project_id) else {
            return;
        };
        let Some(file) = project.files.iter().find(|file| file.id == file_id) else {
            return;
        };
        let path = project.path.clone();
        let file_path = file.path.clone();
        let header = header.to_string();
        let project_id = project_id.to_string();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(
                    async move { flashbee_git::discard_hunk(&path, &file_path, &header) },
                )
                .await;
            this.update(cx, |app, cx| {
                if let Err(error) = result {
                    app.toast_error(&error, cx);
                } else {
                    app.toast(app.t("diff.discarded"), ToastKind::Success, cx);
                    app.load_project(project_id, true, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn copy_pr_description(&mut self, cx: &mut Context<Self>) {
        let Some(project) = self.active_project() else {
            return;
        };
        let mut markdown = format!(
            "## Changes on `{}`\n\n### Summary\n",
            project.current_branch
        );
        for file in &project.files {
            let stats = diff::file_stats(file);
            markdown.push_str(&format!(
                "- **{}** `{}` (+{} −{})\n",
                diff::status_letter(file.status),
                file.path,
                stats.additions,
                stats.deletions
            ));
        }
        if let Some(file) = self.active_file() {
            markdown.push_str(&format!("\n### `{}`\n\n```diff\n", file.path));
            for hunk in &file.hunks {
                markdown.push_str(&hunk.header);
                markdown.push('\n');
                for line in &hunk.lines {
                    let prefix = match line.kind {
                        flashbee_git::LineType::Add => "+",
                        flashbee_git::LineType::Del => "-",
                        flashbee_git::LineType::Context => " ",
                    };
                    markdown.push_str(prefix);
                    markdown.push_str(&line.content);
                    markdown.push('\n');
                }
            }
            markdown.push_str("```\n");
        }
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(markdown));
        self.toast(self.t("diff.copyMarkdownDone"), ToastKind::Success, cx);
    }

    pub fn toggle_reviewed(&mut self, project_id: &str, file_id: &str, cx: &mut Context<Self>) {
        let Some((path, branch, fingerprint, id)) = self.project(project_id).and_then(|project| {
            project
                .files
                .iter()
                .find(|file| file.id == file_id)
                .map(|file| {
                    (
                        project.path.clone(),
                        project.branch.clone(),
                        diff::review_fingerprint(file),
                        file.id.clone(),
                    )
                })
        }) else {
            return;
        };
        let currently = self.reviews.reviewed(&path, &branch, &id, &fingerprint);
        self.reviews
            .set_reviewed(&path, &branch, &id, &fingerprint, !currently);
        self.reviews.save();
        cx.notify();
    }

    pub fn reset_review(&mut self, cx: &mut Context<Self>) {
        if let Some(project) = self.active_project() {
            let path = project.path.clone();
            let branch = project.branch.clone();
            self.reviews.clear_branch(&path, &branch);
            self.reviews.save();
            self.toast(self.t("toast.reviewReset"), ToastKind::Info, cx);
        }
    }

    pub fn open_commit(&mut self, cx: &mut Context<Self>) {
        let Some(project) = self.active_project() else {
            return;
        };
        if !project.is_local() {
            return;
        }
        let staged = project.files.iter().filter(|file| file.staged).count();
        if staged == 0 {
            self.toast(self.t("commit.nothingStaged"), ToastKind::Info, cx);
            return;
        }
        let entity = cx.entity();
        let submit_entity = entity.clone();
        let escape_entity = entity;
        let placeholder = self.t("commit.messagePlaceholder").to_string();
        let input = cx.new(|cx| {
            TextInput::new(cx, placeholder, "")
                .with_submit(move |_window, cx| {
                    submit_entity.update(cx, |app, cx| app.submit_commit(cx));
                })
                .with_escape(move |_window, cx| {
                    escape_entity.update(cx, |app, cx| {
                        app.commit_dialog = None;
                        app.pending_main_focus = true;
                        cx.notify();
                    });
                })
        });
        self.pending_focus = Some(input.clone());
        self.commit_dialog = Some(CommitDialogState {
            input,
            amend: false,
            push_after: false,
            busy: false,
            error: None,
        });
        self.branch_menu_project = None;
        cx.notify();
    }

    pub fn submit_commit(&mut self, cx: &mut Context<Self>) {
        let Some(dialog) = self.commit_dialog.as_ref() else {
            return;
        };
        if dialog.busy {
            return;
        }
        let message = dialog.input.read(cx).content().trim().to_string();
        if message.is_empty() {
            let message = self.t("commit.messageRequired").to_string();
            if let Some(dialog) = self.commit_dialog.as_mut() {
                dialog.error = Some(message);
            }
            cx.notify();
            return;
        }
        let Some(project) = self.active_project() else {
            return;
        };
        let path = project.path.clone();
        let id = project.id.clone();
        let amend = dialog.amend;
        let push_after = dialog.push_after;
        if let Some(dialog) = self.commit_dialog.as_mut() {
            dialog.busy = true;
            dialog.error = None;
        }
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(
                    async move { flashbee_git::commit_repository(&path, &message, amend) },
                )
                .await;
            this.update(cx, |app, cx| match result {
                Ok(outcome) => {
                    app.commit_dialog = None;
                    app.pending_main_focus = true;
                    let message = i18n::fill(
                        app.t("commit.committed"),
                        &[
                            ("hash", &outcome.short_hash),
                            ("count", &outcome.files_changed.to_string()),
                        ],
                    );
                    app.toast(message, ToastKind::Success, cx);
                    app.load_project(id, true, cx);
                    if push_after {
                        app.open_push(cx);
                    }
                }
                Err(error) => {
                    if let Some(dialog) = app.commit_dialog.as_mut() {
                        dialog.busy = false;
                        dialog.error = Some(error.message);
                    }
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn open_branch_dialog(&mut self, cx: &mut Context<Self>) {
        let entity = cx.entity();
        let submit_entity = entity.clone();
        let escape_entity = entity;
        let placeholder = self.t("branch.namePlaceholder").to_string();
        let input = cx.new(|cx| {
            TextInput::new(cx, placeholder, "")
                .with_submit(move |_window, cx| {
                    submit_entity.update(cx, |app, cx| app.submit_new_branch(cx));
                })
                .with_escape(move |_window, cx| {
                    escape_entity.update(cx, |app, cx| {
                        app.branch_dialog = None;
                        app.pending_main_focus = true;
                        cx.notify();
                    });
                })
        });
        self.pending_focus = Some(input.clone());
        self.branch_dialog = Some(BranchDialogState {
            input,
            busy: false,
            error: None,
        });
        self.branch_menu_project = None;
        cx.notify();
    }

    pub fn submit_new_branch(&mut self, cx: &mut Context<Self>) {
        let Some(dialog) = self.branch_dialog.as_ref() else {
            return;
        };
        if dialog.busy {
            return;
        }
        let branch = dialog.input.read(cx).content().trim().to_string();
        if branch.is_empty() {
            let message = self.t("branch.nameRequired").to_string();
            if let Some(dialog) = self.branch_dialog.as_mut() {
                dialog.error = Some(message);
            }
            cx.notify();
            return;
        }
        let Some(project) = self.active_project() else {
            return;
        };
        let path = project.path.clone();
        let id = project.id.clone();
        if let Some(dialog) = self.branch_dialog.as_mut() {
            dialog.busy = true;
            dialog.error = None;
        }
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    flashbee_git::switch_branch(&path, &branch, true, false)
                })
                .await;
            this.update(cx, |app, cx| match result {
                Ok(outcome) => {
                    app.branch_dialog = None;
                    app.pending_main_focus = true;
                    if let Some(project) = app.project_mut(&id) {
                        project.branch = outcome.branch.clone();
                        project.current_branch = outcome.branch.clone();
                    }
                    app.persist_workspaces();
                    let message = i18n::fill(app.t("branch.created"), &[("name", &outcome.branch)]);
                    app.toast(message, ToastKind::Success, cx);
                    app.load_project(id, true, cx);
                }
                Err(error) => {
                    let message = if error.code == "branch_exists" {
                        app.t("branch.nameExists").to_string()
                    } else if error.code == "bad_branch" {
                        app.t("branch.nameInvalid").to_string()
                    } else {
                        error.message
                    };
                    if let Some(dialog) = app.branch_dialog.as_mut() {
                        dialog.busy = false;
                        dialog.error = Some(message);
                    }
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn switch_branch(&mut self, project_id: &str, branch: &str, cx: &mut Context<Self>) {
        let Some(project) = self.project(project_id) else {
            return;
        };
        let path = project.path.clone();
        let project_id = project_id.to_string();
        let branch = branch.to_string();
        let branch_for_call = branch.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    flashbee_git::switch_branch(&path, &branch_for_call, false, false)
                })
                .await;
            this.update(cx, |app, cx| match result {
                Ok(outcome) => {
                    app.branch_menu_project = None;
                    if let Some(project) = app.project_mut(&project_id) {
                        project.branch = outcome.branch.clone();
                        project.current_branch = outcome.branch.clone();
                    }
                    app.persist_workspaces();
                    let message =
                        i18n::fill(app.t("branch.switched"), &[("name", &outcome.branch)]);
                    app.toast(message, ToastKind::Success, cx);
                    app.load_project(project_id, true, cx);
                    app.load_branch_manager(cx);
                }
                Err(error) if error.code == "dirty_worktree" => {
                    app.branch_menu_project = None;
                    let name = app
                        .project(&project_id)
                        .map(|project| project.name.clone())
                        .unwrap_or_default();
                    let count = app
                        .project(&project_id)
                        .map(|project| project.files.len())
                        .unwrap_or(0);
                    app.confirm = Some(ConfirmState {
                        title: app.t("branch.dirtyTitle").to_string(),
                        description: i18n::fill(
                            app.t("branch.dirtyDescription"),
                            &[("name", &name), ("count", &count.to_string())],
                        ),
                        confirm_label: app.t("branch.stashAndSwitch").to_string(),
                        action: ConfirmAction::StashAndSwitch {
                            project_id: project_id.clone(),
                            branch: branch.clone(),
                        },
                    });
                    cx.notify();
                }
                Err(error) => {
                    let message =
                        i18n::fill(app.t("branch.switchFailed"), &[("message", &error.message)]);
                    app.toast(message, ToastKind::Error, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn stash_and_switch(&mut self, project_id: &str, branch: &str, cx: &mut Context<Self>) {
        let Some(project) = self.project(project_id) else {
            return;
        };
        let path = project.path.clone();
        let project_id = project_id.to_string();
        let branch = branch.to_string();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    flashbee_git::switch_branch(&path, &branch, false, true)
                })
                .await;
            this.update(cx, |app, cx| match result {
                Ok(outcome) => {
                    if let Some(project) = app.project_mut(&project_id) {
                        project.branch = outcome.branch.clone();
                        project.current_branch = outcome.branch.clone();
                    }
                    app.persist_workspaces();
                    let message =
                        i18n::fill(app.t("branch.switched"), &[("name", &outcome.branch)]);
                    app.toast(message, ToastKind::Success, cx);
                    if outcome.stashed {
                        app.toast(app.t("branch.stashed"), ToastKind::Info, cx);
                    }
                    app.load_project(project_id, true, cx);
                    app.load_branch_manager(cx);
                }
                Err(error) => app.toast_error(&error, cx),
            })
            .ok();
        })
        .detach();
    }

    pub fn open_stash(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self
            .active_project()
            .filter(|project| project.is_local())
            .map(|project| project.path.clone())
        else {
            return;
        };
        let entity = cx.entity();
        let submit_entity = entity.clone();
        let escape_entity = entity;
        let placeholder = self.t("stash.messagePlaceholder").to_string();
        let input = cx.new(|cx| {
            TextInput::new(cx, placeholder, "")
                .with_submit(move |_window, cx| {
                    submit_entity.update(cx, |app, cx| app.save_stash(cx));
                })
                .with_escape(move |_window, cx| {
                    escape_entity.update(cx, |app, cx| {
                        app.stash_dialog = None;
                        app.pending_main_focus = true;
                        cx.notify();
                    });
                })
        });
        self.pending_focus = Some(input.clone());
        self.stash_dialog = Some(StashDialogState {
            input,
            include_untracked: false,
            busy: false,
            error: None,
            entries: Vec::new(),
            loading: true,
        });
        self.repo_menu_project = None;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::stash_list(&path) })
                .await;
            this.update(cx, |app, cx| {
                if let Some(dialog) = app.stash_dialog.as_mut() {
                    dialog.loading = false;
                    match result {
                        Ok(entries) => dialog.entries = entries,
                        Err(error) => dialog.error = Some(error.message),
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    pub fn save_stash(&mut self, cx: &mut Context<Self>) {
        let Some(dialog) = self.stash_dialog.as_ref() else {
            return;
        };
        if dialog.busy {
            return;
        }
        let message = dialog.input.read(cx).content().trim().to_string();
        let include_untracked = dialog.include_untracked;
        let Some(project) = self.active_project() else {
            return;
        };
        let path = project.path.clone();
        let id = project.id.clone();
        if let Some(dialog) = self.stash_dialog.as_mut() {
            dialog.busy = true;
            dialog.error = None;
        }
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    flashbee_git::stash_save(&path, &message, include_untracked)
                })
                .await;
            this.update(cx, |app, cx| match result {
                Ok(created) => {
                    if let Some(dialog) = app.stash_dialog.as_mut() {
                        dialog.busy = false;
                    }
                    if created {
                        app.toast(app.t("stash.saved"), ToastKind::Success, cx);
                        app.load_project(id, true, cx);
                        app.reload_stashes(cx);
                    } else {
                        app.toast(app.t("stash.nothing"), ToastKind::Info, cx);
                    }
                }
                Err(error) => {
                    if let Some(dialog) = app.stash_dialog.as_mut() {
                        dialog.busy = false;
                        dialog.error = Some(error.message);
                    }
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn reload_stashes(&mut self, cx: &mut Context<Self>) {
        let Some(project) = self.active_project() else {
            return;
        };
        let path = project.path.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::stash_list(&path) })
                .await;
            this.update(cx, |app, cx| {
                if let Some(dialog) = app.stash_dialog.as_mut() {
                    match result {
                        Ok(entries) => dialog.entries = entries,
                        Err(error) => dialog.error = Some(error.message),
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn apply_stash(&mut self, index: u32, cx: &mut Context<Self>) {
        let Some(project) = self.active_project() else {
            return;
        };
        let path = project.path.clone();
        let id = project.id.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::stash_apply(&path, index) })
                .await;
            this.update(cx, |app, cx| match result {
                Ok(outcome) => {
                    if outcome.conflicts.is_empty() {
                        app.toast(app.t("stash.applied"), ToastKind::Success, cx);
                    } else {
                        let count = outcome.conflicts.len().to_string();
                        let message = i18n::fill(app.t("repo.pullConflicts"), &[("count", &count)]);
                        app.toast(message, ToastKind::Info, cx);
                    }
                    app.reload_stashes(cx);
                    app.load_project(id, true, cx);
                }
                Err(error) => app.toast_error(&error, cx),
            })
            .ok();
        })
        .detach();
    }

    pub fn request_drop_stash(&mut self, index: u32, cx: &mut Context<Self>) {
        self.confirm = Some(ConfirmState {
            title: self.t("stash.drop").to_string(),
            description: format!("stash@{{{index}}}"),
            confirm_label: self.t("stash.drop").to_string(),
            action: ConfirmAction::DropStash {
                project_id: self.active_project_id.clone(),
                index,
            },
        });
        cx.notify();
    }

    fn drop_stash(&mut self, project_id: &str, index: u32, cx: &mut Context<Self>) {
        let Some(project) = self.project(project_id) else {
            return;
        };
        let path = project.path.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::stash_drop(&path, index) })
                .await;
            this.update(cx, |app, cx| {
                if let Err(error) = result {
                    app.toast_error(&error, cx);
                } else {
                    app.toast(app.t("stash.dropped"), ToastKind::Success, cx);
                    app.reload_stashes(cx);
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn open_remote_dialog(&mut self, cx: &mut Context<Self>) {
        let Some(project) = self.active_project() else {
            return;
        };
        if !project.is_local() {
            return;
        }
        let entity = cx.entity();
        let escape_entity = entity.clone();
        let name_placeholder = self.t("remote.nameLabel").to_string();
        let url_placeholder = self.t("remote.urlLabel").to_string();
        let name_input = cx.new(|cx| {
            TextInput::new(cx, name_placeholder, "origin").with_escape({
                let escape_entity = escape_entity.clone();
                move |_window, cx| {
                    escape_entity.update(cx, |app, cx| {
                        app.remote_dialog = None;
                        app.pending_main_focus = true;
                        cx.notify();
                    });
                }
            })
        });
        let url_input = cx.new(|cx| TextInput::new(cx, url_placeholder, ""));
        let remotes = project.remotes.clone();
        self.pending_focus = Some(url_input.clone());
        self.remote_dialog = Some(RemoteDialogState {
            name_input,
            url_input,
            editing: None,
            busy: false,
            error: None,
            remotes,
            loading: false,
        });
        self.repo_menu_project = None;
        let _ = entity;
        cx.notify();
    }

    pub fn select_remote(&mut self, name: String, cx: &mut Context<Self>) {
        let Some(dialog) = self.remote_dialog.as_ref() else {
            return;
        };
        let url = dialog
            .remotes
            .iter()
            .find(|remote| remote.name == name)
            .map(|remote| remote.url.clone())
            .unwrap_or_default();
        let name_input = dialog.name_input.clone();
        let url_input = dialog.url_input.clone();
        name_input.update(cx, |input, cx| input.set_content(name.clone(), cx));
        url_input.update(cx, |input, cx| input.set_content(url, cx));
        if let Some(dialog) = self.remote_dialog.as_mut() {
            dialog.editing = Some(name);
            dialog.error = None;
        }
        cx.notify();
    }

    pub fn save_remote(&mut self, cx: &mut Context<Self>) {
        let Some(dialog) = self.remote_dialog.as_ref() else {
            return;
        };
        if dialog.busy {
            return;
        }
        let name = dialog.name_input.read(cx).content().trim().to_string();
        let url = dialog.url_input.read(cx).content().trim().to_string();
        let Some(project) = self.active_project() else {
            return;
        };
        let path = project.path.clone();
        if name.is_empty() {
            let message = self.t("remote.nameLabel").to_string();
            if let Some(dialog) = self.remote_dialog.as_mut() {
                dialog.error = Some(message);
            }
            cx.notify();
            return;
        }
        if url.is_empty() {
            let message = self.t("remote.urlRequired").to_string();
            if let Some(dialog) = self.remote_dialog.as_mut() {
                dialog.error = Some(message);
            }
            cx.notify();
            return;
        }
        if let Some(dialog) = self.remote_dialog.as_mut() {
            dialog.busy = true;
            dialog.error = None;
        }
        cx.notify();
        let name_for_task = name.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    flashbee_git::set_remote_url(&path, &name_for_task, &url)
                })
                .await;
            this.update(cx, |app, cx| match result {
                Ok(remotes) => {
                    if let Some(dialog) = app.remote_dialog.as_mut() {
                        dialog.remotes = remotes;
                        dialog.busy = false;
                        dialog.editing = Some(name.clone());
                    }
                    app.toast(app.t("repo.remoteSaved"), ToastKind::Success, cx);
                    if let Some(project) = app.active_project().cloned() {
                        app.load_project(project.id, true, cx);
                    }
                }
                Err(error) => {
                    if let Some(dialog) = app.remote_dialog.as_mut() {
                        dialog.busy = false;
                        dialog.error = Some(error.message);
                    }
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn load_branch_info_cache(&mut self, project_id: String, cx: &mut Context<Self>) {
        let Some(project) = self.project(&project_id) else {
            return;
        };
        let path = project.path.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::list_branch_info(&path) })
                .await;
            this.update(cx, |app, cx| {
                if let Ok(branches) = result {
                    if let Some(project) = app.project(&project_id) {
                        app.branch_info_cache = Some(BranchInfoCache {
                            project_path: project.path.clone(),
                            branches,
                        });
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn open_branch_manager(
        &mut self,
        project_id: String,
        tab: BranchTab,
        cx: &mut Context<Self>,
    ) {
        let Some(project) = self.project(&project_id).cloned() else {
            return;
        };
        if !project.is_local() {
            return;
        }
        let current = project.current_branch.clone();
        let suggested_worktree_branch = {
            let base = if current.trim().is_empty() {
                "worktree".to_string()
            } else {
                format!("{}-worktree", current)
            };
            let mut candidate = base.clone();
            let mut index = 2;
            while project.branches.iter().any(|branch| branch == &candidate) {
                candidate = format!("{base}-{index}");
                index += 1;
            }
            candidate
        };
        let suggested_name = format!(
            "{}-{}",
            project.name,
            suggested_worktree_branch.replace(['/', '\\'], "-")
        );
        let suggested_path = PathBuf::from(&project.path)
            .parent()
            .map(|parent| parent.join(&suggested_name))
            .unwrap_or_else(|| PathBuf::from(&suggested_name))
            .to_string_lossy()
            .to_string();

        let filter_input =
            cx.new(|cx| TextInput::new(cx, i18n::t(self.lang(), "branch.filterPlaceholder"), ""));
        let create_name =
            cx.new(|cx| TextInput::new(cx, i18n::t(self.lang(), "branch.namePlaceholder"), ""));
        let create_start =
            cx.new(|cx| TextInput::new(cx, self.t("branch.startPlaceholder"), current.clone()));
        let worktree_branch = cx.new(|cx| {
            TextInput::new(
                cx,
                self.t("worktree.branchPlaceholder"),
                suggested_worktree_branch.clone(),
            )
        });
        let worktree_path =
            cx.new(|cx| TextInput::new(cx, self.t("worktree.pathPlaceholder"), suggested_path));
        let worktree_start =
            cx.new(|cx| TextInput::new(cx, self.t("worktree.startPlaceholder"), current.clone()));

        self.pending_focus = Some(filter_input.clone());
        self.branch_manager = Some(BranchManagerState {
            project_id,
            tab,
            branches: Vec::new(),
            worktrees: Vec::new(),
            loading: false,
            busy: false,
            error: None,
            filter_input,
            create_name,
            create_start,
            create_switch: true,
            rename_target: None,
            rename_input: None,
            worktree_branch,
            worktree_path,
            worktree_start,
            worktree_new_branch: true,
        });
        self.branch_menu_project = None;
        self.repo_menu_project = None;
        self.load_branch_manager(cx);
        cx.notify();
    }

    pub fn load_branch_manager(&mut self, cx: &mut Context<Self>) {
        let Some(manager) = self.branch_manager.as_ref() else {
            return;
        };
        let Some(project) = self.project(&manager.project_id) else {
            return;
        };
        let path = project.path.clone();
        let project_id = manager.project_id.clone();
        if let Some(manager) = self.branch_manager.as_mut() {
            manager.loading = true;
            manager.error = None;
        }
        cx.notify();
        cx.spawn(async move |this, cx| {
            let branch_path = path.clone();
            let branch_result = cx
                .background_spawn(async move { flashbee_git::list_branch_info(&branch_path) })
                .await;
            let worktree_path = path.clone();
            let worktree_result = cx
                .background_spawn(async move { flashbee_git::list_worktrees(&worktree_path) })
                .await;
            this.update(cx, |app, cx| {
                let mut cache_branches = None;
                let mut manager_error = None;
                {
                    let Some(manager) = app.branch_manager.as_mut() else {
                        return;
                    };
                    if manager.project_id != project_id {
                        return;
                    }
                    manager.loading = false;
                    match branch_result {
                        Ok(branches) => {
                            cache_branches = Some(branches.clone());
                            manager.branches = branches;
                        }
                        Err(error) => manager_error = Some(error.message),
                    }
                    match worktree_result {
                        Ok(worktrees) => manager.worktrees = worktrees,
                        Err(error) => manager_error = Some(error.message),
                    }
                    if let Some(error) = manager_error {
                        manager.error = Some(error);
                    }
                }
                if let Some(branches) = cache_branches {
                    if let Some(project) = app.project(&project_id) {
                        app.branch_info_cache = Some(BranchInfoCache {
                            project_path: project.path.clone(),
                            branches,
                        });
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn manager_project(&self) -> Option<Project> {
        let manager = self.branch_manager.as_ref()?;
        self.project(&manager.project_id).cloned()
    }

    fn workspace_root_project_id(&self, project_id: &str) -> String {
        let Some(project) = self.project(project_id) else {
            return project_id.to_string();
        };
        if let Some(root_path) = project.worktree_of.as_deref() {
            if let Some(root) = self.state.active().and_then(|workspace| {
                workspace
                    .projects
                    .iter()
                    .find(|entry| entry.path == root_path)
            }) {
                return root.id.clone();
            }
        }
        project_id.to_string()
    }

    pub fn set_branch_manager_tab(&mut self, tab: BranchTab, cx: &mut Context<Self>) {
        if let Some(manager) = self.branch_manager.as_mut() {
            manager.tab = tab;
        }
        cx.notify();
    }

    pub fn set_create_switch(&mut self, value: bool, cx: &mut Context<Self>) {
        if let Some(manager) = self.branch_manager.as_mut() {
            manager.create_switch = value;
        }
        cx.notify();
    }

    pub fn create_branch_from_manager(&mut self, cx: &mut Context<Self>) {
        let Some(manager) = self.branch_manager.as_ref() else {
            return;
        };
        if manager.busy {
            return;
        }
        let name = manager.create_name.read(cx).content().trim().to_string();
        let start = manager.create_start.read(cx).content().trim().to_string();
        let switch_after = manager.create_switch;
        let project_id = manager.project_id.clone();
        if name.is_empty() {
            let message = self.t("branch.nameRequired").to_string();
            if let Some(manager) = self.branch_manager.as_mut() {
                manager.error = Some(message);
            }
            cx.notify();
            return;
        }
        let Some(path) = self
            .project(&project_id)
            .map(|project| project.path.clone())
        else {
            return;
        };
        if let Some(manager) = self.branch_manager.as_mut() {
            manager.busy = true;
            manager.error = None;
        }
        cx.notify();
        cx.spawn(async move |this, cx| {
            let start = if start.is_empty() {
                "HEAD".to_string()
            } else {
                start
            };
            let result = cx
                .background_spawn(async move {
                    flashbee_git::create_branch(&path, &name, Some(&start), false)
                })
                .await;
            this.update(cx, |app, cx| {
                if let Some(manager) = app.branch_manager.as_mut() {
                    manager.busy = false;
                }
                match result {
                    Ok(outcome) => {
                        let message =
                            i18n::fill(app.t("branch.created"), &[("name", &outcome.branch)]);
                        app.toast(message, ToastKind::Success, cx);
                        if let Some(manager) = app.branch_manager.as_mut() {
                            manager.create_name.update(cx, |input, cx| input.clear(cx));
                        }
                        app.load_branch_manager(cx);
                        app.load_project(project_id.clone(), true, cx);
                        if switch_after {
                            app.switch_branch(&project_id, &outcome.branch, cx);
                        }
                    }
                    Err(error) => {
                        if let Some(manager) = app.branch_manager.as_mut() {
                            manager.error = Some(error.message);
                        }
                        cx.notify();
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn begin_rename_branch(&mut self, name: String, cx: &mut Context<Self>) {
        let Some(manager) = self.branch_manager.as_ref() else {
            return;
        };
        let input = cx.new(|cx| TextInput::new(cx, self.t("branch.namePlaceholder"), name.clone()));
        self.pending_focus = Some(input.clone());
        if let Some(manager) = self.branch_manager.as_mut() {
            manager.rename_target = Some(name);
            manager.rename_input = Some(input);
        }
        cx.notify();
    }

    pub fn cancel_rename_branch(&mut self, cx: &mut Context<Self>) {
        if let Some(manager) = self.branch_manager.as_mut() {
            manager.rename_target = None;
            manager.rename_input = None;
        }
        self.pending_main_focus = true;
        cx.notify();
    }

    pub fn submit_rename_branch(&mut self, cx: &mut Context<Self>) {
        let Some(manager) = self.branch_manager.as_ref() else {
            return;
        };
        let Some(old_name) = manager.rename_target.clone() else {
            return;
        };
        let Some(input) = manager.rename_input.clone() else {
            return;
        };
        let new_name = input.read(cx).content().trim().to_string();
        if new_name.is_empty() {
            return;
        }
        let project_id = manager.project_id.clone();
        let Some(project) = self.project(&project_id) else {
            return;
        };
        let path = project.path.clone();
        if let Some(manager) = self.branch_manager.as_mut() {
            manager.busy = true;
            manager.error = None;
        }
        cx.notify();
        let old_name_for_task = old_name.clone();
        let new_name_for_task = new_name.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    flashbee_git::rename_branch(&path, &old_name_for_task, &new_name_for_task)
                })
                .await;
            this.update(cx, |app, cx| {
                if let Some(manager) = app.branch_manager.as_mut() {
                    manager.busy = false;
                    manager.rename_target = None;
                    manager.rename_input = None;
                }
                match result {
                    Ok(()) => {
                        if let Some(project) = app.project_mut(&project_id) {
                            if project.current_branch == old_name {
                                project.current_branch = new_name.clone();
                            }
                            if project.branch == old_name {
                                project.branch = new_name.clone();
                            }
                        }
                        app.branch_info_cache = None;
                        app.toast(
                            i18n::fill(app.t("branch.renamed"), &[("name", &new_name)]),
                            ToastKind::Success,
                            cx,
                        );
                        app.load_branch_manager(cx);
                        app.load_project(project_id, true, cx);
                    }
                    Err(error) => {
                        if let Some(manager) = app.branch_manager.as_mut() {
                            manager.error = Some(error.message);
                        }
                        cx.notify();
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn request_delete_branch(&mut self, name: String, force: bool, cx: &mut Context<Self>) {
        let Some(manager) = self.branch_manager.as_ref() else {
            return;
        };
        let project_id = manager.project_id.clone();
        let description = if force {
            i18n::fill(
                self.t("branch.deleteUnmergedDescription"),
                &[("name", &name)],
            )
        } else {
            i18n::fill(self.t("branch.deleteDescription"), &[("name", &name)])
        };
        self.confirm = Some(ConfirmState {
            title: self.t("branch.deleteTitle").to_string(),
            description,
            confirm_label: self.t("branch.delete").to_string(),
            action: ConfirmAction::DeleteBranch {
                project_id,
                name,
                force,
            },
        });
        cx.notify();
    }

    fn delete_branch(&mut self, project_id: &str, name: &str, force: bool, cx: &mut Context<Self>) {
        let Some(project) = self.project(project_id) else {
            return;
        };
        let path = project.path.clone();
        let project_id = project_id.to_string();
        let name = name.to_string();
        let name_for_task = name.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    flashbee_git::delete_branch(&path, &name_for_task, force)
                })
                .await;
            this.update(cx, |app, cx| match result {
                Ok(()) => {
                    let message = i18n::fill(app.t("branch.deleted"), &[("name", &name)]);
                    app.toast(message, ToastKind::Success, cx);
                    app.branch_info_cache = None;
                    app.load_branch_manager(cx);
                    app.load_project(project_id, true, cx);
                }
                Err(error) => {
                    if error.code == "branch_not_merged" {
                        app.request_delete_branch(name.clone(), true, cx);
                    } else if let Some(manager) = app.branch_manager.as_mut() {
                        manager.error = Some(error.message);
                        cx.notify();
                    } else {
                        app.toast_error(&error, cx);
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn request_merge_branch(&mut self, source: String, cx: &mut Context<Self>) {
        let Some(manager) = self.branch_manager.as_ref() else {
            return;
        };
        let project_id = manager.project_id.clone();
        let Some(project) = self.project(&project_id) else {
            return;
        };
        let current = project.current_branch.clone();
        self.confirm = Some(ConfirmState {
            title: self.t("branch.mergeTitle").to_string(),
            description: i18n::fill(
                self.t("branch.mergeDescription"),
                &[("source", &source), ("current", &current)],
            ),
            confirm_label: self.t("branch.merge").to_string(),
            action: ConfirmAction::MergeBranch { project_id, source },
        });
        cx.notify();
    }

    fn merge_branch(&mut self, project_id: &str, source: &str, cx: &mut Context<Self>) {
        let Some(project) = self.project(project_id) else {
            return;
        };
        let path = project.path.clone();
        let project_id = project_id.to_string();
        let source = source.to_string();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::merge_branch(&path, &source) })
                .await;
            this.update(cx, |app, cx| match result {
                Ok(outcome) => {
                    app.branch_manager = None;
                    if outcome.conflicts.is_empty() {
                        app.toast(app.t("branch.merged"), ToastKind::Success, cx);
                    } else {
                        let count = outcome.conflicts.len().to_string();
                        app.toast(
                            i18n::fill(app.t("branch.mergeConflicts"), &[("count", &count)]),
                            ToastKind::Info,
                            cx,
                        );
                    }
                    app.branch_info_cache = None;
                    if let Some(project) = app.project(&project_id) {
                        let id = project.id.clone();
                        app.load_project(id, true, cx);
                    }
                    app.conflict_open = !outcome.conflicts.is_empty();
                    cx.notify();
                }
                Err(error) => {
                    if let Some(manager) = app.branch_manager.as_mut() {
                        manager.error = Some(error.message);
                        cx.notify();
                    } else {
                        app.toast_error(&error, cx);
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn add_worktree_from_manager(&mut self, cx: &mut Context<Self>) {
        let Some(manager) = self.branch_manager.as_ref() else {
            return;
        };
        if manager.busy {
            return;
        }
        let branch = manager
            .worktree_branch
            .read(cx)
            .content()
            .trim()
            .to_string();
        let target = manager.worktree_path.read(cx).content().trim().to_string();
        let start = manager.worktree_start.read(cx).content().trim().to_string();
        let new_branch = manager.worktree_new_branch;
        let project_id = manager.project_id.clone();
        if branch.is_empty() {
            let message = self.t("worktree.branchRequired").to_string();
            if let Some(manager) = self.branch_manager.as_mut() {
                manager.error = Some(message);
            }
            cx.notify();
            return;
        }
        if target.is_empty() {
            let message = self.t("worktree.pathRequired").to_string();
            if let Some(manager) = self.branch_manager.as_mut() {
                manager.error = Some(message);
            }
            cx.notify();
            return;
        }
        let Some(path) = self
            .project(&project_id)
            .map(|project| project.path.clone())
        else {
            return;
        };
        if let Some(manager) = self.branch_manager.as_mut() {
            manager.busy = true;
            manager.error = None;
        }
        cx.notify();
        let options = WorktreeAddOptions {
            path: target.clone(),
            branch: if new_branch {
                None
            } else {
                Some(branch.clone())
            },
            new_branch: if new_branch {
                Some(branch.clone())
            } else {
                None
            },
            start_point: if new_branch {
                Some(if start.is_empty() {
                    "HEAD".to_string()
                } else {
                    start
                })
            } else {
                None
            },
            detach: false,
        };
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::add_worktree(&path, options) })
                .await;
            this.update(cx, |app, cx| {
                if let Some(manager) = app.branch_manager.as_mut() {
                    manager.busy = false;
                }
                match result {
                    Ok(()) => {
                        app.pending_worktree_path = Some(target.clone());
                        let message = i18n::fill(
                            app.t("worktree.created"),
                            &[("branch", &branch), ("path", &target)],
                        );
                        app.toast(message, ToastKind::Success, cx);
                        app.branch_manager = None;
                        app.branch_info_cache = None;
                        let root_id = app.workspace_root_project_id(&project_id);
                        app.load_project(root_id, true, cx);
                        cx.notify();
                    }
                    Err(error) => {
                        if let Some(manager) = app.branch_manager.as_mut() {
                            manager.error = Some(error.message);
                            cx.notify();
                        } else {
                            app.toast_error(&error, cx);
                        }
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn request_remove_worktree(&mut self, path: String, force: bool, cx: &mut Context<Self>) {
        let Some(manager) = self.branch_manager.as_ref() else {
            return;
        };
        let project_id = manager.project_id.clone();
        let description = if force {
            i18n::fill(
                self.t("worktree.removeDirtyDescription"),
                &[("path", &path)],
            )
        } else {
            i18n::fill(self.t("worktree.removeDescription"), &[("path", &path)])
        };
        self.confirm = Some(ConfirmState {
            title: self.t("worktree.removeTitle").to_string(),
            description,
            confirm_label: self.t("worktree.remove").to_string(),
            action: ConfirmAction::RemoveWorktree {
                project_id,
                path,
                force,
            },
        });
        cx.notify();
    }

    fn remove_worktree(
        &mut self,
        project_id: &str,
        target: &str,
        force: bool,
        cx: &mut Context<Self>,
    ) {
        let Some(project) = self.project(project_id) else {
            return;
        };
        let path = project.path.clone();
        let project_id = project_id.to_string();
        let target = target.to_string();
        let target_for_task = target.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    flashbee_git::remove_worktree(&path, &target_for_task, force)
                })
                .await;
            this.update(cx, |app, cx| match result {
                Ok(()) => {
                    app.toast(app.t("worktree.removed"), ToastKind::Success, cx);
                    app.load_branch_manager(cx);
                    let root_id = app.workspace_root_project_id(&project_id);
                    app.load_project(root_id, true, cx);
                }
                Err(error) if error.code == "worktree_dirty" && !force => {
                    app.request_remove_worktree(target.clone(), true, cx);
                }
                Err(error) => {
                    if let Some(manager) = app.branch_manager.as_mut() {
                        manager.error = Some(error.message);
                        cx.notify();
                    } else {
                        app.toast_error(&error, cx);
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn prune_worktrees(&mut self, cx: &mut Context<Self>) {
        let Some(manager) = self.branch_manager.as_ref() else {
            return;
        };
        let project_id = manager.project_id.clone();
        let Some(path) = self
            .project(&project_id)
            .map(|project| project.path.clone())
        else {
            return;
        };
        if let Some(manager) = self.branch_manager.as_mut() {
            manager.busy = true;
        }
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::prune_worktrees(&path) })
                .await;
            this.update(cx, |app, cx| {
                if let Some(manager) = app.branch_manager.as_mut() {
                    manager.busy = false;
                }
                match result {
                    Ok(count) => {
                        app.toast(
                            i18n::fill(app.t("worktree.pruned"), &[("count", &count.to_string())]),
                            ToastKind::Info,
                            cx,
                        );
                        app.load_branch_manager(cx);
                        let root_id = app.workspace_root_project_id(&project_id);
                        app.load_project(root_id, true, cx);
                    }
                    Err(error) => {
                        if let Some(manager) = app.branch_manager.as_mut() {
                            manager.error = Some(error.message);
                            cx.notify();
                        }
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn open_worktree_path(&mut self, path: &str, cx: &mut Context<Self>) {
        let normalized = path.trim_end_matches('/');
        let found = self
            .state
            .active()
            .and_then(|workspace| {
                workspace
                    .projects
                    .iter()
                    .find(|project| project.path.trim_end_matches('/') == normalized)
            })
            .cloned();
        if let Some(project) = found {
            self.branch_menu_project = None;
            self.branch_manager = None;
            self.activate_project(&project.id, cx);
            return;
        }
        self.pending_worktree_path = Some(path.to_string());
        self.branch_menu_project = None;
        self.branch_manager = None;
        if let Some(project) = self.active_project() {
            let root_id = self.workspace_root_project_id(&project.id);
            self.load_project(root_id, true, cx);
        }
        self.toast(
            i18n::fill(self.t("worktree.loading"), &[("path", &path)]),
            ToastKind::Info,
            cx,
        );
    }

    pub fn open_settings(&mut self, cx: &mut Context<Self>) {
        self.settings_open = true;
        self.branch_menu_project = None;
        self.repo_menu_project = None;
        self.workspace_menu_open = false;
        self.pending_main_focus = false;
        cx.notify();
    }

    pub fn close_settings(&mut self, cx: &mut Context<Self>) {
        self.settings_open = false;
        self.pending_main_focus = true;
        cx.notify();
    }

    pub fn update_settings(&mut self, cx: &mut Context<Self>, update: impl FnOnce(&mut Settings)) {
        update(&mut self.settings);
        self.settings = self.settings.clone().clamp();
        let theme = Theme::for_theme(self.resolved_theme());
        cx.set_global(GlobalTheme(theme));
        self.persist_settings();
        cx.notify();
    }

    pub fn open_add_repo(&mut self, cx: &mut Context<Self>) {
        let entity = cx.entity();
        let submit_entity = entity.clone();
        let escape_entity = entity;
        let placeholder = self.t("repo.pathPlaceholder").to_string();
        let input = cx.new(|cx| {
            TextInput::new(cx, placeholder, "")
                .with_submit(move |_window, cx| {
                    submit_entity.update(cx, |app, cx| app.inspect_add_repo_path(cx));
                })
                .with_escape(move |_window, cx| {
                    escape_entity.update(cx, |app, cx| {
                        app.add_repo = None;
                        app.pending_main_focus = true;
                        cx.notify();
                    });
                })
        });
        self.pending_focus = Some(input.clone());
        self.add_repo = Some(AddRepoState {
            input,
            scanning: false,
            results: Vec::new(),
            error: None,
            manual_note: None,
        });
        self.repo_menu_project = None;
        cx.notify();
    }

    pub fn browse_repo_folder(&mut self, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(self.t("repo.chooseFolder").into()),
        });
        cx.spawn(async move |this, cx| {
            let result = receiver.await;
            this.update(cx, |app, cx| match result {
                Ok(Ok(Some(paths))) => {
                    if let Some(path) = paths.first() {
                        app.add_repo_from_path(path.clone(), cx);
                    }
                }
                Ok(Ok(None)) => {}
                Ok(Err(error)) => app.toast(error.to_string(), ToastKind::Error, cx),
                Err(error) => app.toast(error.to_string(), ToastKind::Error, cx),
            })
            .ok();
        })
        .detach();
    }

    pub fn inspect_add_repo_path(&mut self, cx: &mut Context<Self>) {
        let Some(dialog) = self.add_repo.as_ref() else {
            return;
        };
        let path = dialog.input.read(cx).content().trim().to_string();
        if path.is_empty() {
            let message = self.t("repo.pathRequired").to_string();
            if let Some(dialog) = self.add_repo.as_mut() {
                dialog.error = Some(message);
            }
            cx.notify();
            return;
        }
        self.add_repo_from_path(PathBuf::from(path), cx);
    }

    fn add_repo_from_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let path_string = path.to_string_lossy().to_string();
        if self
            .state
            .active()
            .is_some_and(|workspace| workspace.projects.iter().any(|p| p.path == path_string))
        {
            self.toast(self.t("repo.pathExists"), ToastKind::Info, cx);
            return;
        }
        if let Some(dialog) = self.add_repo.as_mut() {
            dialog.scanning = true;
            dialog.error = None;
            dialog.results.clear();
            dialog.manual_note = Some(path_string.clone());
        }
        cx.notify();
        cx.spawn(async move |this, cx| {
            let inspect_path = path_string.clone();
            let inspection = cx
                .background_spawn(async move { flashbee_git::inspect_repository(&inspect_path) })
                .await;
            match inspection {
                Ok(info) => {
                    this.update(cx, |app, cx| {
                        app.finish_add_repo(vec![(info.name, info.path)], cx)
                    })
                    .ok();
                }
                Err(error) if error.code == "not_git" => {
                    let discovery_path = path_string.clone();
                    let discovery = cx
                        .background_spawn(async move {
                            flashbee_git::discover_repositories(&discovery_path)
                        })
                        .await;
                    this.update(cx, |app, cx| match discovery {
                        Ok(found) => {
                            let repos: Vec<(String, String)> = found
                                .repositories
                                .into_iter()
                                .map(|repository| (repository.name, repository.path))
                                .collect();
                            app.finish_add_repo(repos, cx);
                        }
                        Err(error) => app.finish_add_repo_error(error, cx),
                    })
                    .ok();
                }
                Err(error) => {
                    this.update(cx, |app, cx| app.finish_add_repo_error(error, cx))
                        .ok();
                }
            }
        })
        .detach();
    }

    fn finish_add_repo(&mut self, repositories: Vec<(String, String)>, cx: &mut Context<Self>) {
        let empty_message = self.t("repo.groupNotFound").to_string();
        let Some(dialog) = self.add_repo.as_mut() else {
            return;
        };
        dialog.scanning = false;
        if repositories.is_empty() {
            dialog.error = Some(empty_message);
        } else if repositories.len() == 1 {
            let (name, path) = repositories.into_iter().next().unwrap();
            self.add_repository(&name, &path, cx);
            self.add_repo = None;
        } else {
            dialog.results = repositories
                .into_iter()
                .map(|(name, path)| DiscoveredRepo {
                    name,
                    path,
                    selected: true,
                })
                .collect();
        }
        cx.notify();
    }

    fn finish_add_repo_error(&mut self, error: CommandError, cx: &mut Context<Self>) {
        let message = match error.code.as_str() {
            "not_git" => self.t("repo.notGit").to_string(),
            "not_found" => self.t("repo.notFound").to_string(),
            "not_directory" => self.t("repo.notDirectory").to_string(),
            "git_unavailable" => self.t("repo.gitUnavailable").to_string(),
            _ => error.message,
        };
        let Some(dialog) = self.add_repo.as_mut() else {
            return;
        };
        dialog.scanning = false;
        dialog.error = Some(message);
        cx.notify();
    }

    pub fn add_selected_repositories(&mut self, cx: &mut Context<Self>) {
        let Some(dialog) = self.add_repo.as_ref() else {
            return;
        };
        let selected: Vec<(String, String)> = dialog
            .results
            .iter()
            .filter(|repo| repo.selected)
            .map(|repo| (repo.name.clone(), repo.path.clone()))
            .collect();
        if selected.is_empty() {
            return;
        }
        let group_path = dialog
            .manual_note
            .clone()
            .filter(|path| selected.iter().any(|(_, repo)| repo != path));
        let mut group_id = None;
        if let Some(parent_path) = group_path.as_deref() {
            let group_name = crate::model::repository_name(parent_path);
            if let Some(workspace) = self.state.active_mut() {
                let existing = workspace
                    .groups
                    .iter()
                    .find(|group| group.path == parent_path)
                    .map(|group| group.id.clone());
                group_id = existing.or_else(|| {
                    let group = crate::model::create_group(&group_name, parent_path);
                    let id = group.id.clone();
                    workspace.groups.push(group);
                    Some(id)
                });
            }
        }
        for (name, path) in selected {
            self.add_repository_in_group(&name, &path, group_id.clone(), cx);
        }
        self.add_repo = None;
        self.pending_main_focus = true;
        cx.notify();
    }

    fn add_repository_in_group(
        &mut self,
        name: &str,
        path: &str,
        group_id: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(workspace) = self.state.active_mut() else {
            return;
        };
        let project = create_project(name, path, "main", workspace.projects.len(), group_id);
        let id = project.id.clone();
        workspace.projects.push(project);
        self.active_project_id = id.clone();
        self.expanded_project_id = Some(id.clone());
        self.selected_by_project.remove(&id);
        self.persist_workspaces();
        let message = i18n::fill(self.t("repo.created"), &[("name", name)]);
        self.toast(message, ToastKind::Success, cx);
        self.load_project(id, false, cx);
    }

    pub fn add_repository(&mut self, name: &str, path: &str, cx: &mut Context<Self>) {
        let Some(workspace) = self.state.active_mut() else {
            return;
        };
        let project = create_project(name, path, "main", workspace.projects.len(), None);
        let id = project.id.clone();
        workspace.projects.push(project);
        self.active_project_id = id.clone();
        self.expanded_project_id = Some(id.clone());
        self.selected_by_project.remove(&id);
        self.persist_workspaces();
        let message = i18n::fill(self.t("repo.created"), &[("name", name)]);
        self.toast(message, ToastKind::Success, cx);
        self.load_project(id, false, cx);
    }

    pub fn remove_project(&mut self, project_id: &str, cx: &mut Context<Self>) {
        let Some(workspace) = self.state.active_mut() else {
            return;
        };
        let Some(project) = workspace
            .projects
            .iter()
            .find(|project| project.id == project_id)
            .cloned()
        else {
            return;
        };
        workspace.projects.retain(|entry| {
            entry.id != project_id && entry.worktree_of.as_deref() != Some(project.path.as_str())
        });
        if self.active_project_id == project_id {
            self.active_project_id = workspace
                .projects
                .first()
                .map(|project| project.id.clone())
                .unwrap_or_default();
            self.expanded_project_id = if self.active_project_id.is_empty() {
                None
            } else {
                Some(self.active_project_id.clone())
            };
        }
        self.selected_by_project.remove(project_id);
        self.persist_workspaces();
        let message = i18n::fill(self.t("repo.removed"), &[("name", &project.name)]);
        self.toast(message, ToastKind::Info, cx);
        cx.notify();
    }

    pub fn create_workspace(&mut self, name: &str, cx: &mut Context<Self>) {
        let workspace = create_workspace(name);
        let id = workspace.id.clone();
        let label = name.trim().to_string();
        self.state.workspaces.push(workspace);
        self.state.active_workspace_id = id;
        self.active_project_id.clear();
        self.expanded_project_id = None;
        self.view = MainView::Changes;
        self.persist_workspaces();
        let message = i18n::fill(self.t("workspace.created"), &[("name", &label)]);
        self.toast(message, ToastKind::Success, cx);
        cx.notify();
    }

    pub fn switch_workspace(&mut self, workspace_id: &str, cx: &mut Context<Self>) {
        if !self
            .state
            .workspaces
            .iter()
            .any(|workspace| workspace.id == workspace_id)
        {
            return;
        }
        self.state.active_workspace_id = workspace_id.to_string();
        let first = self
            .state
            .active()
            .and_then(|workspace| workspace.projects.first())
            .map(|project| project.id.clone())
            .unwrap_or_default();
        self.active_project_id = first.clone();
        self.expanded_project_id = if first.is_empty() { None } else { Some(first) };
        self.view = MainView::Changes;
        self.workspace_menu_open = false;
        self.persist_workspaces();
        self.start_loading(cx);
        cx.notify();
    }

    pub fn request_remove_project(&mut self, project_id: &str, cx: &mut Context<Self>) {
        let Some(project) = self.project(project_id) else {
            return;
        };
        if project.is_worktree() {
            return;
        }
        let name = project.name.clone();
        self.repo_menu_project = None;
        self.confirm = Some(ConfirmState {
            title: self.t("repo.removeTitle").to_string(),
            description: i18n::fill(self.t("repo.removeDescription"), &[("name", &name)]),
            confirm_label: self.t("repo.remove").to_string(),
            action: ConfirmAction::RemoveProject {
                project_id: project_id.to_string(),
            },
        });
        cx.notify();
    }

    pub fn confirm_action(&mut self, cx: &mut Context<Self>) {
        let Some(confirm) = self.confirm.take() else {
            return;
        };
        match confirm.action {
            ConfirmAction::DiscardFile {
                project_id,
                file_id,
            } => self.discard_file(&project_id, &file_id, cx),
            ConfirmAction::DiscardHunk {
                project_id,
                file_id,
                header,
            } => self.discard_hunk(&project_id, &file_id, &header, cx),
            ConfirmAction::DropStash { project_id, index } => {
                self.drop_stash(&project_id, index, cx)
            }
            ConfirmAction::StashAndSwitch { project_id, branch } => {
                self.stash_and_switch(&project_id, &branch, cx)
            }
            ConfirmAction::RemoveProject { project_id } => self.remove_project(&project_id, cx),
            ConfirmAction::RemoveWorkspace { workspace_id } => {
                self.remove_workspace(&workspace_id, cx)
            }
            ConfirmAction::AbortMerge { project_id } => self.abort_merge(&project_id, cx),
            ConfirmAction::DeleteBranch {
                project_id,
                name,
                force,
            } => self.delete_branch(&project_id, &name, force, cx),
            ConfirmAction::MergeBranch { project_id, source } => {
                self.merge_branch(&project_id, &source, cx)
            }
            ConfirmAction::RemoveWorktree {
                project_id,
                path,
                force,
            } => self.remove_worktree(&project_id, &path, force, cx),
        }
        self.pending_main_focus = true;
        cx.notify();
    }

    fn remove_workspace(&mut self, workspace_id: &str, cx: &mut Context<Self>) {
        if self.state.workspaces.len() <= 1 {
            self.toast(self.t("sidebar.keepOneWorkspace"), ToastKind::Info, cx);
            return;
        }
        let Some(workspace) = self
            .state
            .workspaces
            .iter()
            .find(|workspace| workspace.id == workspace_id)
            .cloned()
        else {
            return;
        };
        self.state
            .workspaces
            .retain(|entry| entry.id != workspace_id);
        if self.state.active_workspace_id == workspace_id {
            if let Some(first) = self.state.workspaces.first() {
                self.state.active_workspace_id = first.id.clone();
                self.active_project_id = first
                    .projects
                    .first()
                    .map(|project| project.id.clone())
                    .unwrap_or_default();
                self.expanded_project_id = if self.active_project_id.is_empty() {
                    None
                } else {
                    Some(self.active_project_id.clone())
                };
            }
        }
        self.persist_workspaces();
        let label = workspace_label(&workspace, self.lang());
        let message = i18n::fill(self.t("workspace.deleted"), &[("name", &label)]);
        self.toast(message, ToastKind::Info, cx);
        cx.notify();
    }

    pub fn request_remove_workspace(&mut self, cx: &mut Context<Self>) {
        let Some(workspace) = self.state.active() else {
            return;
        };
        if self.state.workspaces.len() <= 1 {
            self.toast(self.t("sidebar.keepOneWorkspace"), ToastKind::Info, cx);
            return;
        }
        let workspace_id = workspace.id.clone();
        let label = workspace_label(workspace, self.lang());
        self.workspace_menu_open = false;
        self.confirm = Some(ConfirmState {
            title: self.t("workspace.deleteTitle").to_string(),
            description: i18n::fill(self.t("workspace.deleteDescription"), &[("name", &label)]),
            confirm_label: self.t("sidebar.deleteWorkspace").to_string(),
            action: ConfirmAction::RemoveWorkspace { workspace_id },
        });
        cx.notify();
    }

    pub fn complete_merge(&mut self, cx: &mut Context<Self>) {
        let Some(project) = self.active_project() else {
            return;
        };
        let path = project.path.clone();
        let id = project.id.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::complete_merge(&path) })
                .await;
            this.update(cx, |app, cx| match result {
                Ok(_) => {
                    app.toast(app.t("conflict.mergeCompleted"), ToastKind::Success, cx);
                    app.load_project(id, true, cx);
                }
                Err(error) => app.toast_error(&error, cx),
            })
            .ok();
        })
        .detach();
    }

    fn abort_merge(&mut self, project_id: &str, cx: &mut Context<Self>) {
        let Some(project) = self.project(project_id) else {
            return;
        };
        let path = project.path.clone();
        let project_id = project_id.to_string();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::abort_merge(&path) })
                .await;
            this.update(cx, |app, cx| {
                if let Err(error) = result {
                    app.toast_error(&error, cx);
                } else {
                    app.toast(app.t("conflict.mergeAborted"), ToastKind::Info, cx);
                    app.load_project(project_id, true, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn request_abort_merge(&mut self, cx: &mut Context<Self>) {
        if self.active_project_id.is_empty() {
            return;
        }
        self.confirm = Some(ConfirmState {
            title: self.t("conflict.abortTitle").to_string(),
            description: self.t("conflict.abortDescription").to_string(),
            confirm_label: self.t("conflict.abortConfirm").to_string(),
            action: ConfirmAction::AbortMerge {
                project_id: self.active_project_id.clone(),
            },
        });
        cx.notify();
    }

    pub fn handle_key(&mut self, action: &str, cx: &mut Context<Self>) {
        match action {
            "refresh" => self.refresh_active(cx),
            "open-repository" => self.open_add_repo(cx),
            "new-workspace" => self.open_workspace_dialog(cx),
            "toggle-sidebar" => {
                self.sidebar_collapsed = !self.sidebar_collapsed;
                cx.notify();
            }
            "open-palette" => self.open_search(cx),
            "split" => {
                self.mode = DiffMode::Split;
                cx.notify();
            }
            "unified" => {
                self.mode = DiffMode::Unified;
                cx.notify();
            }
            "settings" => self.open_settings(cx),
            "next-file" => {
                if !self.text_input_open() {
                    self.go_to_file(1, cx);
                }
            }
            "previous-file" => {
                if !self.text_input_open() {
                    self.go_to_file(-1, cx);
                }
            }
            "commit" => self.open_commit(cx),
            "fetch" => self.fetch_active(cx),
            "history" => self.open_history(cx),
            "overview" => {
                self.view = MainView::Overview;
                cx.notify();
            }
            "toggle-reviewed" => {
                let Some(project) = self.active_project() else {
                    return;
                };
                let project_id = project.id.clone();
                let Some(file) = self.active_file() else {
                    return;
                };
                let file_id = file.id.clone();
                self.toggle_reviewed(&project_id, &file_id, cx);
            }
            "stage-all" => self.stage_all(cx),
            "copy-pr" => self.copy_pr_description(cx),
            "branches" => {
                if let Some(project) = self.active_project() {
                    let id = project.id.clone();
                    self.open_branch_manager(id, BranchTab::Branches, cx);
                }
            }
            "toggle-diff" => {
                if self.text_input_open() {
                    return;
                }
                self.mode = if self.mode == DiffMode::Split {
                    DiffMode::Unified
                } else {
                    DiffMode::Split
                };
                cx.notify();
            }
            "close-overlay" => self.close_overlay(cx),
            _ => {}
        }
    }

    pub fn text_input_open(&self) -> bool {
        self.commit_dialog.is_some()
            || self.branch_dialog.is_some()
            || self.stash_dialog.is_some()
            || self.add_repo.is_some()
            || self.workspace_dialog.is_some()
            || self.settings_open
            || self.search.is_some()
            || self
                .history
                .as_ref()
                .is_some_and(|history| history.show_compare)
    }

    pub fn close_overlay(&mut self, cx: &mut Context<Self>) {
        if self.commit_dialog.is_some()
            || self.branch_dialog.is_some()
            || self.stash_dialog.is_some()
            || self.add_repo.is_some()
            || self.confirm.is_some()
            || self.push.is_some()
            || self.remote_dialog.is_some()
            || self.branch_manager.is_some()
            || self.workspace_dialog.is_some()
        {
            self.commit_dialog = None;
            self.branch_dialog = None;
            self.stash_dialog = None;
            self.add_repo = None;
            self.confirm = None;
            self.push = None;
            self.remote_dialog = None;
            self.branch_manager = None;
            self.workspace_dialog = None;
            self.pending_main_focus = true;
            cx.notify();
            return;
        }
        if self.settings_open {
            self.close_settings(cx);
            return;
        }
        if self.search.is_some() {
            self.search = None;
            self.pending_main_focus = true;
            cx.notify();
        }
    }

    pub fn open_workspace_dialog(&mut self, cx: &mut Context<Self>) {
        let entity = cx.entity();
        let submit_entity = entity.clone();
        let escape_entity = entity;
        let placeholder = self.t("workspace.namePlaceholder").to_string();
        let input = cx.new(|cx| {
            TextInput::new(cx, placeholder, "")
                .with_submit(move |_window, cx| {
                    submit_entity.update(cx, |app, cx| app.submit_workspace_dialog(cx));
                })
                .with_escape(move |_window, cx| {
                    escape_entity.update(cx, |app, cx| {
                        app.workspace_dialog = None;
                        app.pending_main_focus = true;
                        cx.notify();
                    });
                })
        });
        self.pending_focus = Some(input.clone());
        self.workspace_dialog = Some(WorkspaceDialogState { input, error: None });
        self.workspace_menu_open = false;
        cx.notify();
    }

    pub fn submit_workspace_dialog(&mut self, cx: &mut Context<Self>) {
        let Some(dialog) = self.workspace_dialog.as_ref() else {
            return;
        };
        let name = dialog.input.read(cx).content().trim().to_string();
        if name.is_empty() {
            let message = self.t("workspace.nameRequired").to_string();
            if let Some(dialog) = self.workspace_dialog.as_mut() {
                dialog.error = Some(message);
            }
            cx.notify();
            return;
        }
        if self
            .state
            .workspaces
            .iter()
            .any(|workspace| workspace.name.to_lowercase() == name.to_lowercase())
        {
            let message = self.t("workspace.nameExists").to_string();
            if let Some(dialog) = self.workspace_dialog.as_mut() {
                dialog.error = Some(message);
            }
            cx.notify();
            return;
        }
        self.workspace_dialog = None;
        self.pending_main_focus = true;
        self.create_workspace(&name, cx);
    }

    pub fn open_search(&mut self, cx: &mut Context<Self>) {
        if self.search.is_some() {
            return;
        }
        let entity = cx.entity();
        let submit_entity = entity.clone();
        let escape_entity = entity;
        let placeholder = self.t("search.placeholder").to_string();
        let input = cx.new(|cx| {
            TextInput::new(cx, placeholder, "")
                .with_submit(move |_window, cx| {
                    submit_entity.update(cx, |app, cx| app.run_search(cx));
                })
                .with_escape(move |_window, cx| {
                    escape_entity.update(cx, |app, cx| {
                        app.search = None;
                        app.pending_main_focus = true;
                        cx.notify();
                    });
                })
        });
        self.pending_focus = Some(input.clone());
        cx.observe(&input, |app, input, cx| {
            let query = input.read(cx).content().trim().to_string();
            let Some(search) = app.search.as_mut() else {
                return;
            };
            if search.query == query {
                return;
            }
            search.query = query.clone();
            if query.len() < 2 {
                search.results.commits.clear();
                search.results.code.clear();
                search.busy = false;
                search.generation += 1;
            }
            cx.notify();
            if query.len() >= 2 {
                app.schedule_search(cx);
            }
        })
        .detach();
        self.search = Some(SearchState {
            input,
            query: String::new(),
            generation: 0,
            busy: false,
            results: SearchResults {
                commits: Vec::new(),
                code: Vec::new(),
            },
        });
        self.settings_open = false;
        self.branch_menu_project = None;
        self.repo_menu_project = None;
        cx.notify();
    }

    pub fn schedule_search(&mut self, cx: &mut Context<Self>) {
        let Some(search) = self.search.as_mut() else {
            return;
        };
        search.generation += 1;
        let generation = search.generation;
        search.busy = true;
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(220))
                .await;
            this.update(cx, |app, cx| {
                let current = app
                    .search
                    .as_ref()
                    .map(|search| search.generation)
                    .unwrap_or_default();
                if current == generation {
                    app.run_search(cx);
                }
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    pub fn run_search(&mut self, cx: &mut Context<Self>) {
        let Some(search) = self.search.as_ref() else {
            return;
        };
        let query = search.query.clone();
        if query.len() < 2 {
            if let Some(search) = self.search.as_mut() {
                search.query = query;
                search.results.commits.clear();
                search.results.code.clear();
            }
            cx.notify();
            return;
        }
        let generation = search.generation + 1;
        let targets: Vec<(String, String)> = self
            .state
            .active()
            .map(|workspace| {
                workspace
                    .projects
                    .iter()
                    .filter(|project| project.is_local() && !project.is_worktree())
                    .map(|project| (project.id.clone(), project.path.clone()))
                    .collect()
            })
            .unwrap_or_default();
        if let Some(search) = self.search.as_mut() {
            search.generation = generation;
            search.query = query.clone();
            search.busy = true;
        }
        cx.notify();
        cx.spawn(async move |this, cx| {
            let mut commits = Vec::new();
            let mut code = Vec::new();
            for (project_id, path) in targets {
                let commit_query = query.clone();
                let commit_path = path.clone();
                let commit_result = cx
                    .background_spawn(async move {
                        flashbee_git::search_commits(&commit_path, &commit_query, 6)
                    })
                    .await;
                if let Ok(found) = commit_result {
                    for commit in found {
                        commits.push((project_id.clone(), path.clone(), commit));
                    }
                }
                let code_query = query.clone();
                let code_path = path.clone();
                let code_result = cx
                    .background_spawn(async move {
                        flashbee_git::search_files(&code_path, &code_query, 8)
                    })
                    .await;
                if let Ok(found) = code_result {
                    for hit in found {
                        code.push((project_id.clone(), hit.path, hit.line, hit.preview));
                    }
                }
            }
            this.update(cx, |app, cx| {
                if let Some(search) = app.search.as_mut() {
                    if search.generation == generation {
                        search.busy = false;
                        search.results.commits = commits;
                        search.results.code = code;
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn open_history(&mut self, cx: &mut Context<Self>) {
        let Some(project) = self.active_project() else {
            return;
        };
        if !project.is_local() {
            return;
        }
        let file_filter = self.active_file().map(|file| file.path.clone());
        self.view = MainView::History;
        let base_placeholder = self.t("history.base").to_string();
        let target_placeholder = self.t("history.target").to_string();
        let base_input = cx.new(|cx| TextInput::new(cx, base_placeholder, "HEAD~1"));
        let target_input = cx.new(|cx| TextInput::new(cx, target_placeholder, "HEAD"));
        self.history = Some(HistoryState {
            log: Vec::new(),
            selected: None,
            detail: None,
            detail_file: None,
            busy: true,
            loading_detail: false,
            error: None,
            file_filter,
            skip: 0,
            has_more: true,
            compare_base: base_input,
            compare_target: target_input,
            compare: None,
            compare_file: None,
            compare_busy: false,
            show_compare: false,
        });
        self.load_history(cx);
    }

    pub fn load_history(&mut self, cx: &mut Context<Self>) {
        let Some((path, project_id)) = self
            .active_project()
            .map(|project| (project.path.clone(), project.id.clone()))
        else {
            return;
        };
        let Some(history) = self.history.as_mut() else {
            return;
        };
        history.busy = true;
        history.error = None;
        let file_filter = history.file_filter.clone();
        let skip = history.skip;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    flashbee_git::commit_log(&path, None, file_filter.as_deref(), 80, skip)
                })
                .await;
            this.update(cx, |app, cx| {
                let mut pending_detail = None;
                if let Some(history) = app.history.as_mut() {
                    history.busy = false;
                    match result {
                        Ok(commits) => {
                            let count = commits.len();
                            if skip == 0 {
                                history.log = commits;
                                if history.selected.is_none() {
                                    history.selected =
                                        history.log.first().map(|commit| commit.hash.clone());
                                    pending_detail = history.selected.clone();
                                }
                            } else {
                                history.log.extend(commits);
                            }
                            history.has_more = count >= 80;
                        }
                        Err(error) => history.error = Some(error.message),
                    }
                }
                if let Some(hash) = pending_detail {
                    app.load_commit_detail(project_id, hash, cx);
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn load_more_history(&mut self, cx: &mut Context<Self>) {
        if let Some(history) = self.history.as_mut() {
            history.skip += 80;
        }
        self.load_history(cx);
    }

    pub fn select_history_file_filter(&mut self, file: Option<String>, cx: &mut Context<Self>) {
        if let Some(history) = self.history.as_mut() {
            history.file_filter = file;
            history.skip = 0;
            history.selected = None;
            history.detail = None;
        }
        self.load_history(cx);
    }

    pub fn load_commit_detail(&mut self, project_id: String, hash: String, cx: &mut Context<Self>) {
        let Some(project) = self.project(&project_id) else {
            return;
        };
        let path = project.path.clone();
        if let Some(history) = self.history.as_mut() {
            history.selected = Some(hash.clone());
            history.loading_detail = true;
        }
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::commit_detail(&path, &hash) })
                .await;
            this.update(cx, |app, cx| {
                if let Some(history) = app.history.as_mut() {
                    history.loading_detail = false;
                    match result {
                        Ok(detail) => {
                            history.detail_file = detail.files.first().map(|file| file.id.clone());
                            history.detail = Some(detail);
                        }
                        Err(error) => history.error = Some(error.message),
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn run_compare(&mut self, cx: &mut Context<Self>) {
        let Some(project) = self.active_project() else {
            return;
        };
        let Some(history) = self.history.as_ref() else {
            return;
        };
        let base = history.compare_base.read(cx).content().trim().to_string();
        let target = history.compare_target.read(cx).content().trim().to_string();
        if base.is_empty() || target.is_empty() {
            return;
        }
        let path = project.path.clone();
        if let Some(history) = self.history.as_mut() {
            history.compare_busy = true;
            history.show_compare = true;
            history.error = None;
        }
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::compare_refs(&path, &base, &target) })
                .await;
            this.update(cx, |app, cx| {
                if let Some(history) = app.history.as_mut() {
                    history.compare_busy = false;
                    match result {
                        Ok(comparison) => {
                            history.compare_file =
                                comparison.files.first().map(|file| file.id.clone());
                            history.compare = Some(comparison);
                        }
                        Err(error) => history.error = Some(error.message),
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn close_history(&mut self, cx: &mut Context<Self>) {
        self.view = MainView::Changes;
        self.history = None;
        cx.notify();
    }

    pub fn set_blame_hover(
        &mut self,
        file_id: String,
        side: BlameSide,
        line: u32,
        position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        self.blame_hover = Some(BlameHover {
            file_id: file_id.clone(),
            side,
            line,
            x: position.x.into(),
            y: position.y.into(),
        });
        self.blame_generation += 1;
        let generation = self.blame_generation;
        cx.notify();
        let clear_generation = generation;
        let clear_file = file_id.clone();
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(2600))
                .await;
            this.update(cx, |app, cx| {
                if app.blame_generation == clear_generation
                    && app
                        .blame_hover
                        .as_ref()
                        .is_some_and(|hover| hover.file_id == clear_file)
                {
                    app.blame_hover = None;
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
        let Some(project) = self.active_project().cloned() else {
            return;
        };
        let Some(file) = project
            .files
            .iter()
            .find(|file| file.id == file_id)
            .cloned()
        else {
            return;
        };
        let key = format!(
            "{}:{}:{:?}:{}",
            file.id,
            diff::review_fingerprint(&file),
            side,
            line
        );
        if self.blame_cache.contains_key(&key) || self.blame_loading.contains(&key) {
            return;
        }
        self.blame_loading.insert(key.clone());
        let repository_path = project.path.clone();
        let file_path = file.path.clone();
        let revision = match side {
            BlameSide::Old => Some(project.branch.clone()),
            BlameSide::New => None,
        };
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(140))
                .await;
            let still_hovered = this
                .update(cx, |app, _cx| {
                    app.blame_generation == generation
                        && app.blame_hover.as_ref().is_some_and(|hover| {
                            hover.file_id == file_id && hover.side == side && hover.line == line
                        })
                })
                .unwrap_or(false);
            if !still_hovered {
                this.update(cx, |app, _cx| {
                    app.blame_loading.remove(&key);
                })
                .ok();
                return;
            }
            let result = cx
                .background_spawn(async move {
                    flashbee_git::blame_file(
                        &repository_path,
                        &file_path,
                        revision.as_deref(),
                        Some(vec![line]),
                    )
                })
                .await;
            this.update(cx, |app, cx| {
                app.blame_loading.remove(&key);
                if let Ok(entries) = result {
                    if let Some(entry) = entries.into_iter().next() {
                        app.blame_cache.insert(key, entry);
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn clear_blame_hover(
        &mut self,
        file_id: &str,
        side: BlameSide,
        line: u32,
        cx: &mut Context<Self>,
    ) {
        if self.blame_hover.as_ref().is_some_and(|hover| {
            hover.file_id == file_id && hover.side == side && hover.line == line
        }) {
            self.blame_hover = None;
            cx.notify();
        }
    }

    pub fn ensure_image_diff(
        &mut self,
        project: &Project,
        file: &RepositoryFile,
        cx: &mut Context<Self>,
    ) {
        if file.lfs || !diff::is_image(&file.path) {
            return;
        }
        let key = file.id.clone();
        let fingerprint = diff::review_fingerprint(file);
        if self
            .image_diffs
            .get(&key)
            .is_some_and(|cached| cached.fingerprint == fingerprint)
        {
            return;
        }
        if !self.image_loading.insert(key.clone()) {
            return;
        }
        let repository_path = project.path.clone();
        let file_path = file.path.clone();
        let staged = file.staged;
        let untracked = file.untracked;
        let cache_dir = model::config_dir().join("image-cache");
        let fingerprint_for_task = fingerprint.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    use flashbee_git::FileSide;
                    let (old_side, new_side) = if untracked {
                        (None, FileSide::Worktree)
                    } else if staged {
                        (Some(FileSide::Head), FileSide::Index)
                    } else {
                        (Some(FileSide::Index), FileSide::Worktree)
                    };
                    let old = match old_side {
                        Some(side) => {
                            flashbee_git::read_file_side(&repository_path, &file_path, side)?
                        }
                        None => None,
                    };
                    let new = flashbee_git::read_file_side(&repository_path, &file_path, new_side)?;
                    let old_size = old.as_ref().map(|bytes| bytes.len() as u64).unwrap_or(0);
                    let new_size = new.as_ref().map(|bytes| bytes.len() as u64).unwrap_or(0);
                    let extension = file_path
                        .rsplit_once('.')
                        .map(|(_, extension)| extension.to_string())
                        .unwrap_or_else(|| "img".to_string());
                    let sanitized: String = file_path
                        .chars()
                        .map(|character| {
                            if character.is_ascii_alphanumeric() {
                                character
                            } else {
                                '_'
                            }
                        })
                        .collect();
                    let sanitized: String = sanitized.chars().take(80).collect();
                    let old_path = old.map(|bytes| {
                        let path = cache_dir.join(format!("{sanitized}-old.{extension}"));
                        if let Some(parent) = path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        let _ = std::fs::write(&path, bytes);
                        path
                    });
                    let new_path = new.map(|bytes| {
                        let path = cache_dir.join(format!("{sanitized}-new.{extension}"));
                        if let Some(parent) = path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        let _ = std::fs::write(&path, bytes);
                        path
                    });
                    Ok::<ImageDiff, CommandError>(ImageDiff {
                        fingerprint: fingerprint_for_task,
                        old_path,
                        new_path,
                        old_size,
                        new_size,
                    })
                })
                .await;
            this.update(cx, |app, cx| {
                app.image_loading.remove(&key);
                match result {
                    Ok(diff) => {
                        app.image_diffs.insert(key, diff);
                    }
                    Err(error) => app.toast_error(&error, cx),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn open_conflict_resolver(&mut self, cx: &mut Context<Self>) {
        let Some((has_work, first_conflict)) = self.active_project().map(|project| {
            (
                !project.conflicts.is_empty() || project.merge_in_progress,
                project
                    .conflicts
                    .first()
                    .map(|conflict| conflict.path.clone()),
            )
        }) else {
            return;
        };
        if has_work {
            self.conflict_open = true;
            self.view = MainView::Changes;
            if let Some(path) = first_conflict {
                self.load_conflict(path, cx);
            }
            cx.notify();
        }
    }

    pub fn select_conflict_file(&mut self, file_path: String, cx: &mut Context<Self>) {
        self.load_conflict(file_path, cx);
    }

    fn load_conflict(&mut self, file_path: String, cx: &mut Context<Self>) {
        let Some(project) = self.active_project() else {
            return;
        };
        let path = project.path.clone();
        self.conflict_file = Some(file_path.clone());
        self.conflict_detail = None;
        self.conflict_choices.clear();
        self.conflict_error = None;
        self.conflict_loading = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { flashbee_git::read_conflict(&path, &file_path) })
                .await;
            this.update(cx, |app, cx| {
                app.conflict_loading = false;
                match result {
                    Ok(detail) => {
                        app.conflict_choices = detail
                            .segments
                            .iter()
                            .filter_map(|segment| match segment {
                                ConflictSegment::Conflict { suggestion, .. } => Some(*suggestion),
                                ConflictSegment::Common { .. } => None,
                            })
                            .collect();
                        app.conflict_detail = Some(detail);
                    }
                    Err(error) => app.conflict_error = Some(error.message),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn choose_conflict(&mut self, index: usize, side: ConflictSide, cx: &mut Context<Self>) {
        if let Some(choice) = self.conflict_choices.get_mut(index) {
            *choice = Some(side);
        }
        cx.notify();
    }

    pub fn choose_all_conflicts(&mut self, side: ConflictSide, cx: &mut Context<Self>) {
        self.conflict_choices.fill(Some(side));
        cx.notify();
    }

    pub fn save_conflict(&mut self, cx: &mut Context<Self>) {
        if self.conflict_busy {
            return;
        }
        let Some(project) = self.active_project() else {
            return;
        };
        let Some(detail) = self.conflict_detail.as_ref() else {
            return;
        };
        if self.conflict_choices.iter().any(Option::is_none) {
            self.conflict_error = Some(self.t("conflict.allBlocksResolved").to_string());
            cx.notify();
            return;
        }
        let mut lines: Vec<String> = Vec::new();
        let mut choice_index = 0usize;
        for segment in &detail.segments {
            match segment {
                ConflictSegment::Common { lines: common } => {
                    lines.extend(common.iter().cloned());
                }
                ConflictSegment::Conflict { ours, theirs, .. } => {
                    let side = self.conflict_choices.get(choice_index).copied().flatten();
                    let selected = match side {
                        Some(ConflictSide::Ours) => ours,
                        Some(ConflictSide::Theirs) => theirs,
                        None => return,
                    };
                    lines.extend(selected.iter().cloned());
                    choice_index += 1;
                }
            }
        }
        let mut content = lines.join("\n");
        if detail.trailing_newline {
            content.push('\n');
        }
        let path = project.path.clone();
        let file_path = detail.path.clone();
        let project_id = project.id.clone();
        self.conflict_busy = true;
        self.conflict_error = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let save_path = path.clone();
            let save_file = file_path.clone();
            let result = cx
                .background_spawn(async move {
                    flashbee_git::save_conflict_resolution(&save_path, &save_file, &content)
                })
                .await;
            this.update(cx, |app, cx| {
                app.conflict_busy = false;
                match result {
                    Ok(()) => {
                        let message = i18n::fill(app.t("conflict.saved"), &[("name", &file_path)]);
                        app.toast(message, ToastKind::Success, cx);
                        if let Some(project) = app.project_mut(&project_id) {
                            project
                                .conflicts
                                .retain(|conflict| conflict.path != file_path);
                        }
                        let next = app
                            .project(&project_id)
                            .and_then(|project| project.conflicts.first())
                            .map(|conflict| conflict.path.clone());
                        if let Some(next) = next {
                            app.load_conflict(next, cx);
                        } else {
                            app.conflict_detail = None;
                            app.conflict_file = None;
                            app.conflict_choices.clear();
                        }
                        app.load_project(project_id.clone(), true, cx);
                    }
                    Err(error) => app.conflict_error = Some(error.message),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn resolve_conflict_file_side(&mut self, side: ConflictSide, cx: &mut Context<Self>) {
        let Some(project) = self.active_project() else {
            return;
        };
        let Some(file_path) = self.conflict_file.clone() else {
            return;
        };
        let path = project.path.clone();
        let project_id = project.id.clone();
        self.conflict_busy = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let file_path_for_call = file_path.clone();
            let result = cx
                .background_spawn(async move {
                    flashbee_git::resolve_conflict_side(
                        &path,
                        &file_path_for_call,
                        match side {
                            ConflictSide::Ours => "ours",
                            ConflictSide::Theirs => "theirs",
                        },
                    )
                })
                .await;
            this.update(cx, |app, cx| {
                app.conflict_busy = false;
                match result {
                    Ok(()) => {
                        app.toast(
                            i18n::fill(app.t("conflict.saved"), &[("name", &file_path)]),
                            ToastKind::Success,
                            cx,
                        );
                        if let Some(project) = app.project_mut(&project_id) {
                            project
                                .conflicts
                                .retain(|conflict| conflict.path != file_path);
                        }
                        let next = app
                            .project(&project_id)
                            .and_then(|project| project.conflicts.first())
                            .map(|conflict| conflict.path.clone());
                        if let Some(next) = next {
                            app.load_conflict(next, cx);
                        } else {
                            app.conflict_detail = None;
                            app.conflict_file = None;
                        }
                        app.load_project(project_id.clone(), true, cx);
                    }
                    Err(error) => app.conflict_error = Some(error.message),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn select_push_commit(&mut self, hash: &str, cx: &mut Context<Self>) {
        let Some(push) = self.push.as_mut() else {
            return;
        };
        let Some(info) = push.info.as_ref() else {
            return;
        };
        let Some(index) = info.commits.iter().position(|commit| commit.hash == hash) else {
            return;
        };
        // Pushing a commit always publishes every older commit on the branch,
        // so selecting one means selecting that commit and everything below it.
        push.selected = info
            .commits
            .iter()
            .skip(index)
            .map(|commit| commit.hash.clone())
            .collect();
        cx.notify();
    }

    pub fn open_history_with_file(&mut self, file_path: String, cx: &mut Context<Self>) {
        self.open_history(cx);
        if let Some(history) = self.history.as_mut() {
            history.file_filter = Some(file_path);
            history.skip = 0;
            history.selected = None;
        }
        self.load_history(cx);
    }

    pub fn close_conflict_resolver(&mut self, cx: &mut Context<Self>) {
        self.conflict_open = false;
        cx.notify();
    }

    pub fn commit_with(&mut self, push_after: bool, cx: &mut Context<Self>) {
        if let Some(dialog) = self.commit_dialog.as_mut() {
            dialog.push_after = push_after;
        }
        self.submit_commit(cx);
    }

    pub fn set_amend(&mut self, amend: bool, cx: &mut Context<Self>) {
        if let Some(dialog) = self.commit_dialog.as_mut() {
            dialog.amend = amend;
        }
        cx.notify();
    }

    pub fn set_stash_include_untracked(&mut self, include: bool, cx: &mut Context<Self>) {
        if let Some(dialog) = self.stash_dialog.as_mut() {
            dialog.include_untracked = include;
        }
        cx.notify();
    }

    pub fn toggle_discovered_repo(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(dialog) = self.add_repo.as_mut() {
            if let Some(repo) = dialog.results.get_mut(index) {
                repo.selected = !repo.selected;
            }
        }
        cx.notify();
    }

    pub fn open_overview(&mut self, cx: &mut Context<Self>) {
        self.view = MainView::Overview;
        self.refresh_all(cx);
        cx.notify();
    }

    pub fn close_overview(&mut self, cx: &mut Context<Self>) {
        self.view = MainView::Changes;
        cx.notify();
    }

    pub fn handle_conflict_resolved(&mut self, cx: &mut Context<Self>) {
        if let Some(project) = self.active_project() {
            let id = project.id.clone();
            self.load_project(id, true, cx);
        }
    }

    fn queue_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(input) = self.pending_focus.take() {
            let handle = input.read(cx).focus_handle(cx);
            window.focus(&handle);
            return;
        }
        if self.pending_main_focus {
            self.pending_main_focus = false;
            let handle = self.main_focus.clone();
            window.focus(&handle);
        }
    }

    fn render_titlebar_strip(&self, theme: Theme) -> AnyElement {
        let sidebar_width = if self.sidebar_collapsed {
            0.0
        } else {
            self.layout.sidebar_width
        };
        let title = self
            .active_project()
            .map(|project| format!("{} — FlashBee", project.name))
            .unwrap_or_else(|| "FlashBee".to_string());
        div()
            .flex()
            .flex_none()
            .items_center()
            .h(px(28.))
            .w_full()
            .on_mouse_down(gpui::MouseButton::Left, |event, window, _cx| {
                if event.click_count == 2 {
                    window.titlebar_double_click();
                } else {
                    window.start_window_move();
                }
            })
            .child(
                div()
                    .h_full()
                    .w(px(sidebar_width))
                    .flex_none()
                    .bg(theme.bg_deep)
                    .border_r_1()
                    .border_color(if self.sidebar_collapsed {
                        theme.border.opacity(0.0)
                    } else {
                        theme.border
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h_full()
                    .flex_1()
                    .bg(theme.bg)
                    .text_size(px(10.5))
                    .text_color(theme.dim)
                    .child(title),
            )
            .into_any_element()
    }

    fn render_blame_popover(&self, theme: Theme, window: &Window) -> AnyElement {
        let Some(hover) = self.blame_hover.as_ref() else {
            return div().into_any_element();
        };
        let Some(project) = self.active_project() else {
            return div().into_any_element();
        };
        let Some(file) = project.files.iter().find(|file| file.id == hover.file_id) else {
            return div().into_any_element();
        };
        let key = format!(
            "{}:{}:{:?}:{}",
            file.id,
            diff::review_fingerprint(file),
            hover.side,
            hover.line
        );
        let entry = self.blame_cache.get(&key);
        let loading = self.blame_loading.contains(&key);
        let width = 300.0;
        let viewport = window.viewport_size();
        let left = (hover.x + 14.0).clamp(12.0, f32::from(viewport.width) - width - 12.0);
        let above = hover.y + 96.0 > f32::from(viewport.height);
        let top = if above {
            hover.y - 86.0
        } else {
            hover.y + 18.0
        };

        let content = if let Some(entry) = entry.filter(|entry| !entry.uncommitted) {
            let initial = entry
                .author
                .trim()
                .chars()
                .next()
                .unwrap_or('?')
                .to_uppercase()
                .to_string();
            div()
                .flex()
                .items_center()
                .gap(px(9.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(28.))
                        .rounded_full()
                        .bg(theme.amber_soft)
                        .text_size(px(11.0))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme.amber)
                        .child(initial),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.))
                        .flex_1()
                        .min_w(px(0.))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(px(11.5))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(theme.text)
                                        .truncate()
                                        .child(if entry.author.is_empty() {
                                            self.t("blame.unknownAuthor").to_string()
                                        } else {
                                            entry.author.clone()
                                        }),
                                )
                                .child(
                                    div()
                                        .text_size(px(9.5))
                                        .text_color(theme.dim)
                                        .child(relative_blame_time(entry.time, self.lang())),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.))
                                .text_size(px(9.5))
                                .text_color(theme.muted)
                                .child(
                                    div()
                                        .font_family("SF Mono")
                                        .text_color(theme.amber)
                                        .child(entry.commit.chars().take(7).collect::<String>()),
                                )
                                .child(div().flex_1().min_w(px(0.)).truncate().child(
                                    if entry.summary.is_empty() {
                                        self.t("blame.noSummary").to_string()
                                    } else {
                                        entry.summary.clone()
                                    },
                                )),
                        ),
                )
                .into_any_element()
        } else if entry.is_some_and(|entry| entry.uncommitted) {
            div()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(ui::label(self.t("blame.uncommitted"), 11.0, theme.text))
                .child(ui::label(self.t("blame.uncommittedHint"), 9.5, theme.dim))
                .into_any_element()
        } else if loading {
            div()
                .flex()
                .items_center()
                .gap(px(7.))
                .text_size(px(10.5))
                .text_color(theme.muted)
                .child(ui::icon("loader-circle", 12.0, theme.amber))
                .child(self.t("blame.loading"))
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(ui::label(self.t("blame.unavailable"), 11.0, theme.text))
                .child(ui::label(self.t("blame.unavailableHint"), 9.5, theme.dim))
                .into_any_element()
        };

        div()
            .absolute()
            .left(px(left))
            .top(px(top))
            .w(px(width))
            .p(px(10.))
            .rounded(px(9.))
            .bg(theme.surface_raised)
            .border_1()
            .border_color(theme.border_strong)
            .shadow_lg()
            .child(content)
            .into_any_element()
    }

    fn render_toasts(&self, theme: Theme) -> AnyElement {
        div()
            .absolute()
            .bottom(px(20.))
            .right(px(20.))
            .flex()
            .flex_col()
            .items_end()
            .gap(px(8.))
            .children(self.toasts.iter().map(|toast| {
                let (icon_name, color) = match toast.kind {
                    ToastKind::Success => ("check-circle-2", theme.add),
                    ToastKind::Error => ("triangle-alert", theme.del),
                    ToastKind::Info => ("circle-dot", theme.amber),
                };
                div()
                    .id(SharedString::from(format!("toast-{}", toast.id)))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .max_w(px(460.))
                    .px(px(12.))
                    .py(px(9.))
                    .rounded(px(9.))
                    .bg(theme.surface_raised)
                    .border_1()
                    .border_color(theme.border_strong)
                    .shadow_lg()
                    .child(ui::icon(icon_name, 14.0, color))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(theme.text_soft)
                            .child(toast.text.clone()),
                    )
                    .into_any_element()
            }))
            .into_any_element()
    }
}

impl Render for FlashBeeApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.queue_focus(window, cx);
        let theme = self.theme();
        let title = self
            .active_project()
            .map(|project| format!("{} — FlashBee", project.name))
            .unwrap_or_else(|| "FlashBee".to_string());
        window.set_window_title(&title);

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.bg)
            .text_color(theme.text)
            .font_family(".SystemUIFont")
            .text_size(px(self.settings.ui_size))
            .relative()
            .key_context("FlashBee")
            .track_focus(&self.main_focus)
            .on_drag_move::<SidebarDrag>(cx.listener(
                |app, event: &gpui::DragMoveEvent<SidebarDrag>, _window, cx| {
                    let width = f32::from(event.event.position.x);
                    app.layout.sidebar_width = crate::model::clamp_sidebar_width(width);
                    app.resizing_sidebar = true;
                    cx.notify();
                },
            ))
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(|app, _event, _window, cx| {
                    if app.resizing_sidebar {
                        app.resizing_sidebar = false;
                        save_layout(&app.layout);
                        cx.notify();
                    }
                }),
            )
            .child(self.render_titlebar_strip(theme))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h(px(0.))
                    .child(self.render_sidebar(cx))
                    .when(!self.sidebar_collapsed, |this| {
                        this.child(
                            div()
                                .id("sidebar-resizer")
                                .flex_none()
                                .w(px(4.))
                                .h_full()
                                .cursor(gpui::CursorStyle::ResizeLeftRight)
                                .hover(move |style| style.bg(theme.border_strong))
                                .on_drag(SidebarDrag, |_drag, _position, _window, cx| {
                                    cx.new(|_| DragPreview)
                                }),
                        )
                    })
                    .child(self.render_main(cx)),
            )
            .child(self.render_overlays(cx))
            .child(self.render_blame_popover(theme, window))
            .child(self.render_toasts(theme))
            .into_any_element()
    }
}

fn relative_blame_time(time: i64, lang: Lang) -> String {
    if time == 0 {
        return String::new();
    }
    let elapsed = (chrono::Utc::now().timestamp() - time).max(0);
    let minutes = elapsed / 60;
    if minutes < 1 {
        return i18n::t(lang, "blame.justNow").to_string();
    }
    if minutes < 60 {
        return i18n::fill(
            i18n::t(lang, "blame.minutesAgo"),
            &[("count", &minutes.to_string())],
        );
    }
    let hours = minutes / 60;
    if hours < 24 {
        return i18n::fill(
            i18n::t(lang, "blame.hoursAgo"),
            &[("count", &hours.to_string())],
        );
    }
    let days = hours / 24;
    if days < 30 {
        return i18n::fill(
            i18n::t(lang, "blame.daysAgo"),
            &[("count", &days.to_string())],
        );
    }
    let months = days / 30;
    if months < 12 {
        return i18n::fill(
            i18n::t(lang, "blame.monthsAgo"),
            &[("count", &months.to_string())],
        );
    }
    i18n::fill(
        i18n::t(lang, "blame.yearsAgo"),
        &[("count", &(months / 12).to_string())],
    )
}
