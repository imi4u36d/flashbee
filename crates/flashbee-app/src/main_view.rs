use std::path::{Path, PathBuf};
use std::process::Command;

use flashbee_git::{DiffHunk, LineType, RepositoryFile};
use gpui::{
    AnyElement, ClipboardItem, Context, ObjectFit, SharedString, Window, div, img, prelude::*, px,
};

use crate::app::{BlameSide, DiffMode, FlashBeeApp, MainView};
use crate::diff::{self, DiffRow, SplitRow, path_parts};
use crate::highlight;
use crate::i18n;
use crate::theme::Theme;
use crate::ui::{self, ButtonTone};

impl FlashBeeApp {
    pub fn render_main(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let content = match self.view {
            MainView::Changes => self.render_changes(cx),
            MainView::Overview => self.render_overview(cx),
            MainView::History => self.render_history(cx),
        };
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.))
            .h_full()
            .child(self.render_repo_bar(cx))
            .child(content)
            .into_any_element()
    }

    fn render_repo_bar(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme();
        let project = self.active_project().cloned();
        let workspace_name = self
            .state
            .active()
            .map(|workspace| crate::model::workspace_label(workspace, self.lang()))
            .unwrap_or_default();
        let conflict_count = project
            .as_ref()
            .map(|project| project.conflicts.len())
            .unwrap_or(0);
        let merge_blocked = conflict_count > 0
            || project
                .as_ref()
                .is_some_and(|project| project.merge_in_progress);
        let staged = project
            .as_ref()
            .map(|project| project.files.iter().filter(|file| file.staged).count())
            .unwrap_or(0);
        let dirty = project
            .as_ref()
            .map(|project| project.files.len())
            .unwrap_or(0);
        let branch_open = project.as_ref().is_some_and(|project| {
            self.branch_menu_project.as_deref() == Some(project.id.as_str())
        });
        let actions_open = project
            .as_ref()
            .is_some_and(|project| self.repo_menu_project.as_deref() == Some(project.id.as_str()));

        let identity = match project.as_ref() {
            Some(project) => {
                let accent = ui::color_from_hex(&project.accent);
                div()
                    .flex()
                    .items_center()
                    .gap(px(9.))
                    .min_w(px(0.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(27.))
                            .rounded(px(8.))
                            .bg(accent.opacity(0.14))
                            .border_1()
                            .border_color(accent.opacity(0.22))
                            .text_size(px(11.5))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(accent)
                            .child(
                                project
                                    .name
                                    .chars()
                                    .next()
                                    .unwrap_or('R')
                                    .to_uppercase()
                                    .to_string(),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .min_w(px(0.))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .child(
                                        div()
                                            .text_size(px(13.5))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.text)
                                            .truncate()
                                            .child(project.name.clone()),
                                    )
                                    .child(ui::pill(
                                        self.t("common.local"),
                                        theme.muted,
                                        theme.surface_hover,
                                        9.0,
                                    ))
                                    .when(project.is_worktree(), |this| {
                                        this.child(ui::pill(
                                            self.t("repo.worktree"),
                                            theme.blue,
                                            theme.blue.opacity(0.1),
                                            9.0,
                                        ))
                                    }),
                            )
                            .child(
                                div()
                                    .text_size(px(10.5))
                                    .text_color(theme.dim)
                                    .truncate()
                                    .child(project.path.clone()),
                            ),
                    )
                    .into_any_element()
            }
            None => div()
                .flex()
                .items_center()
                .gap(px(9.))
                .child(ui::icon("layers", 18.0, theme.amber))
                .child(ui::label_semibold(workspace_name.clone(), 13.5, theme.text))
                .into_any_element(),
        };

        let mut bar = div()
            .relative()
            .flex()
            .items_center()
            .gap(px(10.))
            .h(px(56.))
            .flex_none()
            .px(px(16.))
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.surface.opacity(0.72));

        if self.sidebar_collapsed {
            bar = bar.child(ui::icon_button(
                "open-sidebar-button",
                "panel-left",
                theme,
                theme.text_soft,
                true,
                cx.listener(|app, _event, _window, cx| {
                    app.sidebar_collapsed = false;
                    cx.notify();
                }),
            ));
        }
        bar = bar.child(identity);

        if let Some(project) = project.as_ref() {
            bar = bar.child(
                div()
                    .relative()
                    .child(
                        div()
                            .id("branch-selector")
                            .flex()
                            .items_center()
                            .gap(px(5.))
                            .h(px(26.))
                            .px(px(8.))
                            .rounded(px(6.))
                            .border_1()
                            .border_color(theme.border)
                            .bg(theme.surface_raised)
                            .cursor_pointer()
                            .hover(move |style| style.bg(theme.surface_hover))
                            .on_click(cx.listener({
                                let id = project.id.clone();
                                move |app, _event, _window, cx| {
                                    if app.branch_menu_project.as_deref() == Some(id.as_str()) {
                                        app.branch_menu_project = None;
                                    } else {
                                        app.branch_menu_project = Some(id.clone());
                                        app.load_branch_info_cache(id.clone(), cx);
                                    }
                                    cx.notify();
                                }
                            }))
                            .child(ui::icon("git-branch", 12.0, theme.muted))
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme.text_soft)
                                    .child(project.current_branch.clone()),
                            )
                            .when(project.ahead > 0, |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(1.))
                                        .text_size(px(10.0))
                                        .text_color(theme.add)
                                        .child(ui::icon("arrow-up", 9.0, theme.add))
                                        .child(project.ahead.to_string()),
                                )
                            })
                            .when(project.behind > 0, |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(1.))
                                        .text_size(px(10.0))
                                        .text_color(theme.blue)
                                        .child(ui::icon("arrow-down", 9.0, theme.blue))
                                        .child(project.behind.to_string()),
                                )
                            })
                            .child(ui::icon("chevron-down", 11.0, theme.dim)),
                    )
                    .when_some(
                        branch_open.then(|| self.render_branch_menu(project, theme, cx)),
                        |this, menu| this.child(gpui::deferred(menu).with_priority(10)),
                    ),
            );

            bar = bar.child(div().flex_1());

            bar = bar.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(2.))
                    .child(ui::icon_button(
                        "repo-fetch",
                        "cloud-download",
                        theme,
                        theme.muted,
                        project.is_local() && !project.remotes.is_empty(),
                        cx.listener(|app, _event, _window, cx| app.fetch_active(cx)),
                    ))
                    .child(ui::icon_button(
                        "repo-pull",
                        "arrow-down-to-line",
                        theme,
                        theme.muted,
                        project.is_local() && !project.remotes.is_empty() && !merge_blocked,
                        cx.listener(|app, _event, _window, cx| app.pull_active(cx)),
                    ))
                    .child(ui::icon_button(
                        "repo-push",
                        "arrow-up-from-line",
                        theme,
                        theme.muted,
                        project.is_local() && !project.remotes.is_empty() && !merge_blocked,
                        cx.listener(|app, _event, _window, cx| app.open_push(cx)),
                    )),
            );

            if dirty > 0 {
                bar = bar.child(ui::button(
                    "repo-commit",
                    if staged > 0 {
                        format!("{}  {}", self.t("commit.commit"), staged)
                    } else {
                        self.t("commit.commit").to_string()
                    },
                    theme,
                    ButtonTone::Primary,
                    true,
                    cx.listener(|app, _event, _window, cx| app.open_commit(cx)),
                ));
            }

            bar = bar.child(
                div()
                    .id("repo-quick-search")
                    .flex()
                    .items_center()
                    .gap(px(7.))
                    .h(px(27.))
                    .px(px(9.))
                    .rounded(px(7.))
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.surface_raised)
                    .cursor_pointer()
                    .hover(move |style| style.bg(theme.surface_hover))
                    .on_click(cx.listener(|app, _event, _window, cx| app.open_search(cx)))
                    .child(ui::icon("search", 12.0, theme.muted))
                    .child(ui::label(self.t("repo.jumpToFile"), 11.0, theme.muted))
                    .child(ui::kbd("⌘K", theme)),
            );

            bar = bar.child(
                div()
                    .relative()
                    .child(ui::icon_button(
                        "repo-actions",
                        "ellipsis",
                        theme,
                        theme.muted,
                        true,
                        cx.listener({
                            let id = project.id.clone();
                            move |app, _event, _window, cx| {
                                if app.repo_menu_project.as_deref() == Some(id.as_str()) {
                                    app.repo_menu_project = None;
                                } else {
                                    app.repo_menu_project = Some(id.clone());
                                }
                                cx.notify();
                            }
                        }),
                    ))
                    .when_some(
                        actions_open.then(|| self.render_repo_actions_menu(project, theme, cx)),
                        |this, menu| this.child(gpui::deferred(menu).with_priority(10)),
                    ),
            );
        } else {
            bar = bar.child(div().flex_1());
            bar = bar.child(ui::button(
                "empty-add-repo",
                self.t("sidebar.addRepository"),
                theme,
                ButtonTone::Secondary,
                true,
                cx.listener(|app, _event, _window, cx| app.open_add_repo(cx)),
            ));
        }

        bar = bar.child(ui::icon_button(
            "repo-settings",
            "settings-2",
            theme,
            theme.muted,
            true,
            cx.listener(|app, _event, _window, cx| app.open_settings(cx)),
        ));

        bar.into_any_element()
    }

    fn render_branch_menu(
        &self,
        project: &crate::model::Project,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let cache = self
            .branch_info_cache
            .as_ref()
            .filter(|cache| cache.project_path == project.path);
        let current = project.current_branch.clone();
        let local: Vec<(String, bool, Option<String>, u32, u32, Option<String>)> = match cache {
            Some(cache) => cache
                .branches
                .iter()
                .filter(|branch| !branch.is_remote)
                .map(|branch| {
                    (
                        branch.name.clone(),
                        branch.is_current,
                        branch.worktree_path.clone(),
                        branch.ahead,
                        branch.behind,
                        branch.upstream.clone(),
                    )
                })
                .collect(),
            None => project
                .branches
                .iter()
                .map(|branch| (branch.clone(), branch == &current, None, 0, 0, None))
                .collect(),
        };
        let remote: Vec<(String, String)> = match cache {
            Some(cache) => cache
                .branches
                .iter()
                .filter(|branch| branch.is_remote)
                .map(|branch| {
                    (
                        branch.name.clone(),
                        branch.remote.clone().unwrap_or_default(),
                    )
                })
                .collect(),
            None => project
                .remote_branches
                .iter()
                .filter(|branch| !project.branches.iter().any(|local| local == &branch.name))
                .map(|branch| (branch.name.clone(), branch.remote.clone()))
                .collect(),
        };
        let project_id = project.id.clone();
        let entity = cx.entity();
        let branch_count = local.len();

        div()
            .id("branch-menu")
            .absolute()
            .top(px(32.))
            .left(px(0.))
            .w(px(292.))
            .max_h(px(520.))
            .overflow_scroll()
            .p(px(5.))
            .rounded(px(9.))
            .bg(theme.surface_raised)
            .border_1()
            .border_color(theme.border_strong)
            .shadow_lg()
            .child(menu_label(self.t("branch.local"), theme))
            .children(local.into_iter().map(
                |(branch, selected, worktree, ahead, behind, upstream)| {
                    let entity = entity.clone();
                    let project_id = project_id.clone();
                    let branch_for_click = branch.clone();
                    let worktree_for_click = worktree.clone();
                    let branch_id = branch.clone();
                    div()
                        .id(SharedString::from(format!("branch-{branch}")))
                        .flex()
                        .items_center()
                        .gap(px(7.))
                        .h(px(28.))
                        .px(px(7.))
                        .rounded(px(5.))
                        .cursor_pointer()
                        .when(selected, |this| this.bg(theme.surface_hover))
                        .hover(move |style| style.bg(theme.surface_hover))
                        .on_click(move |_event, _window, cx| {
                            if selected {
                                return;
                            }
                            entity.update(cx, |app, cx| {
                                if let Some(worktree) = worktree_for_click.clone() {
                                    app.open_worktree_path(&worktree, cx);
                                } else {
                                    app.switch_branch(&project_id, &branch_for_click, cx);
                                }
                            });
                        })
                        .child(ui::icon(
                            if selected { "check" } else { "git-branch" },
                            11.0,
                            if selected { theme.amber } else { theme.muted },
                        ))
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.))
                                .truncate()
                                .text_size(px(11.5))
                                .text_color(if selected {
                                    theme.text
                                } else {
                                    theme.text_soft
                                })
                                .child(branch.clone()),
                        )
                        .when(worktree.is_some() && !selected, |this| {
                            this.child(ui::pill(
                                self.t("repo.worktree"),
                                theme.blue,
                                theme.blue.opacity(0.1),
                                8.0,
                            ))
                        })
                        .when(ahead > 0, |this| {
                            this.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(1.))
                                    .text_size(px(9.0))
                                    .text_color(theme.add)
                                    .child(ui::icon("arrow-up", 8.0, theme.add))
                                    .child(ahead.to_string()),
                            )
                        })
                        .when(behind > 0, |this| {
                            this.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(1.))
                                    .text_size(px(9.0))
                                    .text_color(theme.blue)
                                    .child(ui::icon("arrow-down", 8.0, theme.blue))
                                    .child(behind.to_string()),
                            )
                        })
                        .when_some(upstream, |this, upstream| {
                            this.child(
                                div()
                                    .max_w(px(62.))
                                    .truncate()
                                    .text_size(px(8.5))
                                    .text_color(theme.dim)
                                    .child(upstream),
                            )
                        })
                        .when(selected, |this| {
                            this.child(ui::label(self.t("common.current"), 9.0, theme.amber))
                        })
                        .when(worktree.is_some() && !selected, |this| {
                            this.child(ui::icon("git-fork", 9.0, theme.blue))
                        })
                        .when(selected, |this| this)
                        .into_any_element()
                },
            ))
            .when(!remote.is_empty(), |this| {
                this.child(menu_label(self.t("branch.remote"), theme))
                    .children(remote.into_iter().map(|(name, remote_name)| {
                        let entity = entity.clone();
                        let name_for_click = name.clone();
                        let project_id = project_id.clone();
                        div()
                            .id(SharedString::from(format!("remote-{name}")))
                            .flex()
                            .items_center()
                            .gap(px(7.))
                            .h(px(28.))
                            .px(px(7.))
                            .rounded(px(5.))
                            .cursor_pointer()
                            .hover(move |style| style.bg(theme.surface_hover))
                            .on_click(move |_event, _window, cx| {
                                entity.update(cx, |app, cx| {
                                    app.switch_branch(&project_id, &name_for_click, cx);
                                });
                            })
                            .child(ui::icon("cloud-download", 11.0, theme.muted))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(0.))
                                    .truncate()
                                    .text_size(px(11.5))
                                    .text_color(theme.text_soft)
                                    .child(name.clone()),
                            )
                            .when(!remote_name.is_empty(), |this| {
                                this.child(
                                    div()
                                        .text_size(px(8.5))
                                        .text_color(theme.dim)
                                        .child(remote_name),
                                )
                            })
                            .into_any_element()
                    }))
            })
            .child(ui::separator(theme))
            .child(
                div()
                    .id("manage-branches")
                    .flex()
                    .items_center()
                    .gap(px(7.))
                    .h(px(28.))
                    .px(px(7.))
                    .rounded(px(5.))
                    .cursor_pointer()
                    .hover(move |style| style.bg(theme.surface_hover))
                    .on_click(cx.listener({
                        let project_id = project_id.clone();
                        move |app, _event, _window, cx| {
                            app.open_branch_manager(
                                project_id.clone(),
                                crate::app::BranchTab::Branches,
                                cx,
                            );
                        }
                    }))
                    .child(ui::icon("settings-2", 12.0, theme.text_soft))
                    .child(ui::label(self.t("branch.manage"), 11.5, theme.text_soft))
                    .child(div().flex_1())
                    .child(ui::label(branch_count.to_string(), 9.5, theme.dim)),
            )
            .child(
                div()
                    .id("manage-worktrees")
                    .flex()
                    .items_center()
                    .gap(px(7.))
                    .h(px(28.))
                    .px(px(7.))
                    .rounded(px(5.))
                    .cursor_pointer()
                    .hover(move |style| style.bg(theme.surface_hover))
                    .on_click(cx.listener({
                        let project_id = project_id.clone();
                        move |app, _event, _window, cx| {
                            app.open_branch_manager(
                                project_id.clone(),
                                crate::app::BranchTab::Worktrees,
                                cx,
                            );
                        }
                    }))
                    .child(ui::icon("git-fork", 12.0, theme.text_soft))
                    .child(ui::label(self.t("worktree.manage"), 11.5, theme.text_soft)),
            )
            .child(
                div()
                    .id("new-branch")
                    .flex()
                    .items_center()
                    .gap(px(7.))
                    .h(px(28.))
                    .px(px(7.))
                    .rounded(px(5.))
                    .cursor_pointer()
                    .hover(move |style| style.bg(theme.surface_hover))
                    .on_click(cx.listener(|app, _event, _window, cx| {
                        app.open_branch_dialog(cx);
                    }))
                    .child(ui::icon("plus", 12.0, theme.text_soft))
                    .child(ui::label(self.t("branch.new"), 11.5, theme.text_soft)),
            )
            .into_any_element()
    }

    fn render_repo_actions_menu(
        &self,
        project: &crate::model::Project,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let project_id = project.id.clone();
        let path = project.path.clone();
        let path_for_editor = path.clone();
        let path_for_reveal = path.clone();
        let path_for_copy = path.clone();
        div()
            .id("repo-actions-menu")
            .absolute()
            .top(px(30.))
            .right(px(0.))
            .w(px(210.))
            .p(px(5.))
            .rounded(px(9.))
            .bg(theme.surface_raised)
            .border_1()
            .border_color(theme.border_strong)
            .shadow_lg()
            .child(menu_label(&project.name, theme))
            .child(menu_action(
                "action-refresh",
                "refresh-cw",
                self.t("repo.refresh"),
                theme,
                false,
                cx.listener({
                    let project_id = project_id.clone();
                    move |app, _event, _window, cx| {
                        app.repo_menu_project = None;
                        app.refresh_project(&project_id, cx);
                    }
                }),
            ))
            .child(menu_action(
                "action-fetch",
                "cloud-download",
                self.t("repo.fetch"),
                theme,
                false,
                cx.listener({
                    let project_id = project_id.clone();
                    move |app, _event, _window, cx| {
                        app.repo_menu_project = None;
                        app.load_project(project_id.clone(), false, cx);
                        app.fetch_active(cx);
                    }
                }),
            ))
            .child(menu_action(
                "action-stash",
                "archive",
                self.t("stash.save"),
                theme,
                false,
                cx.listener(|app, _event, _window, cx| app.open_stash(cx)),
            ))
            .child(menu_action(
                "action-remote",
                "cloud-cog",
                self.t("repo.remoteSettings"),
                theme,
                false,
                cx.listener(|app, _event, _window, cx| app.open_remote_dialog(cx)),
            ))
            .child(menu_action(
                "action-branches",
                "git-branch",
                self.t("branch.manage"),
                theme,
                false,
                cx.listener(|app, _event, _window, cx| {
                    if let Some(project) = app.active_project() {
                        let id = project.id.clone();
                        app.open_branch_manager(id, crate::app::BranchTab::Branches, cx);
                    }
                }),
            ))
            .child(menu_action(
                "action-worktrees",
                "git-fork",
                self.t("worktree.manage"),
                theme,
                false,
                cx.listener(|app, _event, _window, cx| {
                    if let Some(project) = app.active_project() {
                        let id = project.id.clone();
                        app.open_branch_manager(id, crate::app::BranchTab::Worktrees, cx);
                    }
                }),
            ))
            .child(ui::separator(theme))
            .child(menu_action(
                "action-editor",
                "code-2",
                self.t("repo.openInEditor"),
                theme,
                false,
                cx.listener(move |app, _event, _window, cx| {
                    app.repo_menu_project = None;
                    open_in_editor(&path_for_editor);
                    cx.notify();
                }),
            ))
            .child(menu_action(
                "action-finder",
                "folder-open",
                self.t("repo.revealInFinder"),
                theme,
                false,
                cx.listener(move |app, _event, _window, cx| {
                    app.repo_menu_project = None;
                    reveal_in_finder(&path_for_reveal);
                    cx.notify();
                }),
            ))
            .child(menu_action(
                "action-copy-path",
                "copy",
                self.t("repo.copyPath"),
                theme,
                false,
                cx.listener(move |app, _event, _window, cx| {
                    app.repo_menu_project = None;
                    cx.write_to_clipboard(ClipboardItem::new_string(path_for_copy.clone()));
                    cx.notify();
                }),
            ))
            .child(menu_action(
                "action-copy-pr",
                "git-pull-request",
                self.t("diff.copyAsMarkdown"),
                theme,
                false,
                cx.listener(|app, _event, _window, cx| {
                    app.repo_menu_project = None;
                    app.copy_pr_description(cx);
                }),
            ))
            .when(!project.is_worktree(), |this| {
                this.child(ui::separator(theme)).child(menu_action(
                    "action-remove",
                    "trash-2",
                    self.t("repo.remove"),
                    theme,
                    true,
                    cx.listener({
                        let project_id = project_id.clone();
                        move |app, _event, _window, cx| {
                            app.request_remove_project(&project_id, cx);
                        }
                    }),
                ))
            })
            .into_any_element()
    }

    fn render_changes(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme();
        let Some(project) = self.active_project().cloned() else {
            return self.render_empty_workspace(theme, cx);
        };
        let conflict_count = project.conflicts.len();
        let merge_ready = project.merge_in_progress && conflict_count == 0;
        let file = self.active_file().cloned();
        let loading = project.loading && project.files.is_empty();
        if self.conflict_open && (conflict_count > 0 || project.merge_in_progress) {
            return self.render_conflict_view(&project, theme, cx);
        }

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h(px(0.))
            .when(conflict_count > 0 || project.merge_in_progress, |this| {
                this.child(self.render_conflict_banner(
                    &project,
                    conflict_count,
                    merge_ready,
                    theme,
                    cx,
                ))
            })
            .child(if let Some(file) = file {
                self.render_diff_view(&project, &file, theme, cx)
            } else if loading {
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(9.))
                            .text_size(px(12.0))
                            .text_color(theme.muted)
                            .child(ui::icon("loader-circle", 15.0, theme.amber))
                            .child(self.t("repo.loadingChanges")),
                    )
                    .into_any_element()
            } else {
                self.render_clean_state(&project, theme, cx)
            })
            .into_any_element()
    }

    fn render_conflict_banner(
        &self,
        project: &crate::model::Project,
        conflict_count: usize,
        merge_ready: bool,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .flex()
            .items_center()
            .gap(px(10.))
            .mx(px(16.))
            .mt(px(12.))
            .px(px(12.))
            .py(px(9.))
            .rounded(px(9.))
            .bg(if merge_ready {
                theme.add.opacity(0.08)
            } else {
                theme.del.opacity(0.08)
            })
            .border_1()
            .border_color(if merge_ready {
                theme.add.opacity(0.24)
            } else {
                theme.del.opacity(0.24)
            })
            .child(ui::icon(
                if merge_ready {
                    "check-circle-2"
                } else {
                    "triangle-alert"
                },
                15.0,
                if merge_ready { theme.add } else { theme.del },
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .gap(px(1.))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(if merge_ready { theme.add } else { theme.del })
                            .child(if merge_ready {
                                self.t("conflict.bannerComplete").to_string()
                            } else {
                                i18n::fill(
                                    self.t("conflict.banner"),
                                    &[("count", &conflict_count.to_string())],
                                )
                            }),
                    )
                    .child(div().text_size(px(10.5)).text_color(theme.muted).child(
                        if merge_ready {
                            self.t("conflict.bannerReadyHint")
                        } else {
                            self.t("conflict.bannerHint")
                        },
                    )),
            )
            .child(ui::button(
                "conflict-action",
                if merge_ready {
                    self.t("conflict.completeShort")
                } else {
                    self.t("conflict.open")
                },
                theme,
                if merge_ready {
                    ButtonTone::Primary
                } else {
                    ButtonTone::Secondary
                },
                true,
                cx.listener(move |app, _event, _window, cx| {
                    if merge_ready {
                        app.complete_merge(cx);
                    } else {
                        app.open_conflict_resolver(cx);
                    }
                }),
            ))
            .when(project.merge_in_progress, |this| {
                this.child(ui::button(
                    "abort-merge",
                    self.t("conflict.abortMerge"),
                    theme,
                    ButtonTone::Ghost,
                    true,
                    cx.listener(|app, _event, _window, cx| app.request_abort_merge(cx)),
                ))
            })
            .into_any_element()
    }

    fn render_clean_state(
        &self,
        project: &crate::model::Project,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let accent = ui::color_from_hex(&project.accent);
        div()
            .flex()
            .flex_1()
            .items_center()
            .justify_center()
            .p(px(32.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(10.))
                    .max_w(px(420.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(46.))
                            .rounded_full()
                            .bg(theme.add.opacity(0.1))
                            .border_1()
                            .border_color(theme.add.opacity(0.2))
                            .child(ui::icon("check", 20.0, theme.add)),
                    )
                    .child(
                        div()
                            .text_size(px(18.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(self.t("clean.title")),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(theme.muted)
                            .text_center()
                            .child(i18n::fill(
                                self.t("clean.description"),
                                &[("project", &project.name)],
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(5.))
                                    .px(px(8.))
                                    .py(px(4.))
                                    .rounded(px(6.))
                                    .bg(theme.surface_raised)
                                    .border_1()
                                    .border_color(theme.border)
                                    .child(ui::icon("git-branch", 11.0, accent))
                                    .child(ui::label(
                                        project.current_branch.clone(),
                                        11.0,
                                        theme.text_soft,
                                    )),
                            )
                            .child(ui::button(
                                "clean-refresh",
                                self.t("clean.check"),
                                theme,
                                ButtonTone::Secondary,
                                true,
                                cx.listener(|app, _event, _window, cx| app.refresh_active(cx)),
                            )),
                    ),
            )
            .into_any_element()
    }

    fn render_empty_workspace(&self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        let workspace_name = self
            .state
            .active()
            .map(|workspace| crate::model::workspace_label(workspace, self.lang()))
            .unwrap_or_default();
        div()
            .flex()
            .flex_1()
            .items_center()
            .justify_center()
            .p(px(32.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(10.))
                    .max_w(px(420.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(48.))
                            .rounded(px(14.))
                            .bg(theme.amber_soft)
                            .border_1()
                            .border_color(theme.amber.opacity(0.2))
                            .child(ui::icon("folder-git-2", 22.0, theme.amber)),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.amber)
                            .child(workspace_name),
                    )
                    .child(
                        div()
                            .text_size(px(19.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(self.t("workspace.emptyTitle")),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(theme.muted)
                            .text_center()
                            .child(self.t("workspace.emptyDescription")),
                    )
                    .child(ui::button(
                        "empty-workspace-add",
                        self.t("workspace.emptyCta"),
                        theme,
                        ButtonTone::Primary,
                        true,
                        cx.listener(|app, _event, _window, cx| app.open_add_repo(cx)),
                    ))
                    .child(ui::kbd("⌘ K", theme)),
            )
            .into_any_element()
    }

    fn render_diff_view(
        &mut self,
        project: &crate::model::Project,
        file: &RepositoryFile,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let stats = diff::file_stats(file);
        let (name, dir) = path_parts(&file.path);
        let language = diff::language_label(&file.language);
        let status_color = theme.status(file.status);
        let reviewed = self.reviewed(project, file);
        let is_binary = file.binary;
        let is_image = diff::is_image(&file.path) && !file.lfs;
        let has_hunks = !file.hunks.is_empty();
        if is_image {
            self.ensure_image_diff(project, file, cx);
        }
        let max_chars = file
            .hunks
            .iter()
            .flat_map(|hunk| hunk.lines.iter())
            .map(|line| line.content.chars().count())
            .max()
            .unwrap_or(80)
            .min(240);
        let use_split = self.mode == DiffMode::Split
            && !matches!(
                file.status,
                flashbee_git::ChangeStatus::Added | flashbee_git::ChangeStatus::Deleted
            );
        let surface_width = if use_split {
            (max_chars as f32 * 8.0 * 2.0).max(920.0)
        } else {
            (max_chars as f32 * 7.6).max(720.0)
        };
        let project_id = project.id.clone();
        let file_id = file.id.clone();

        let mut header = div()
            .flex()
            .items_center()
            .gap(px(10.))
            .h(px(50.))
            .flex_none()
            .px(px(16.))
            .border_b_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(22.))
                    .rounded(px(6.))
                    .bg(status_color.opacity(0.12))
                    .text_size(px(10.5))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(status_color)
                    .child(diff::status_letter(file.status)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .min_w(px(0.))
                    .flex_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(7.))
                            .child(
                                div()
                                    .text_size(px(14.0))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.text)
                                    .truncate()
                                    .child(name.to_string()),
                            )
                            .child(ui::pill(
                                language.clone(),
                                theme.muted,
                                theme.surface_hover,
                                9.0,
                            ))
                            .when(file.binary, |this| {
                                this.child(ui::pill(
                                    self.t("common.binary"),
                                    theme.violet,
                                    theme.violet.opacity(0.1),
                                    9.0,
                                ))
                            })
                            .when(file.lfs, |this| {
                                this.child(ui::pill(
                                    "LFS",
                                    theme.blue,
                                    theme.blue.opacity(0.1),
                                    9.0,
                                ))
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .text_size(px(10.0))
                            .text_color(theme.dim)
                            .child(if dir.is_empty() {
                                ".".to_string()
                            } else {
                                dir.to_string()
                            })
                            .child("·")
                            .child(if file.staged {
                                self.t("common.staged")
                            } else if file.untracked {
                                self.t("common.untracked")
                            } else {
                                self.t("common.unstaged")
                            }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(theme.add)
                            .child(format!("+{}", stats.additions)),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(theme.del)
                            .child(format!("−{}", stats.deletions)),
                    ),
            )
            .child(ui::vertical_separator(theme))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(1.))
                    .child(ui::icon_button(
                        "diff-split",
                        "columns-2",
                        theme,
                        if self.mode == DiffMode::Split {
                            theme.amber
                        } else {
                            theme.muted
                        },
                        true,
                        cx.listener(|app, _event, _window, cx| {
                            app.mode = DiffMode::Split;
                            cx.notify();
                        }),
                    ))
                    .child(ui::icon_button(
                        "diff-unified",
                        "rows-3",
                        theme,
                        if self.mode == DiffMode::Unified {
                            theme.amber
                        } else {
                            theme.muted
                        },
                        true,
                        cx.listener(|app, _event, _window, cx| {
                            app.mode = DiffMode::Unified;
                            cx.notify();
                        }),
                    )),
            )
            .child(ui::icon_button(
                "copy-path",
                "copy",
                theme,
                theme.muted,
                true,
                cx.listener({
                    let path = file.path.clone();
                    move |app, _event, _window, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(path.clone()));
                        app.toast(app.t("diff.pathCopied"), crate::app::ToastKind::Info, cx);
                    }
                }),
            ))
            .child(ui::icon_button(
                "open-at-line",
                "code-2",
                theme,
                theme.muted,
                true,
                cx.listener({
                    let project_path = project.path.clone();
                    let file_path = file.path.clone();
                    let line = first_hunk_line(file);
                    move |app, _event, _window, cx| {
                        let absolute = absolute_path_for_project(&project_path, &file_path);
                        open_in_editor_at_line(&absolute, line);
                        app.toast(app.t("diff.openAtLine"), crate::app::ToastKind::Info, cx);
                    }
                }),
            ))
            .child(ui::icon_button(
                "mark-reviewed",
                if reviewed { "eye-off" } else { "eye" },
                theme,
                if reviewed { theme.add } else { theme.muted },
                true,
                cx.listener({
                    let project_id = project_id.clone();
                    let file_id = file_id.clone();
                    move |app, _event, _window, cx| {
                        app.toggle_reviewed(&project_id, &file_id, cx);
                    }
                }),
            ));

        if file.staged {
            header = header.child(ui::button(
                "file-unstage",
                self.t("diff.unstageFile"),
                theme,
                ButtonTone::Secondary,
                true,
                cx.listener({
                    let project_id = project_id.clone();
                    let file_id = file_id.clone();
                    move |app, _event, _window, cx| {
                        app.unstage_file(&project_id, &file_id, cx);
                    }
                }),
            ));
        } else {
            header = header.child(ui::button(
                "file-stage",
                self.t("diff.stageFile"),
                theme,
                ButtonTone::Primary,
                true,
                cx.listener({
                    let project_id = project_id.clone();
                    let file_id = file_id.clone();
                    move |app, _event, _window, cx| {
                        app.stage_file(&project_id, &file_id, cx);
                    }
                }),
            ));
        }
        if !file.staged {
            header = header.child(ui::icon_button(
                "file-discard",
                "rotate-ccw",
                theme,
                theme.del,
                true,
                cx.listener({
                    let project_id = project_id.clone();
                    let file_id = file_id.clone();
                    move |app, _event, _window, cx| {
                        app.request_discard_file(&project_id, &file_id, cx);
                    }
                }),
            ));
        }

        let body: AnyElement = if is_image {
            self.render_image_diff(file, theme)
        } else if is_binary {
            self.render_binary_notice(file, theme, cx)
        } else if !has_hunks {
            div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .child(ui::label(self.t("diff.noChanges"), 12.0, theme.muted))
                .into_any_element()
        } else {
            div()
                .id(SharedString::from(format!("diff-scroll-{}", file.id)))
                .flex()
                .flex_col()
                .flex_1()
                .min_h(px(0.))
                .overflow_scroll()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .min_w(px(surface_width))
                        .when(file.lfs, |this| {
                            this.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .mx(px(14.))
                                    .mt(px(10.))
                                    .px(px(10.))
                                    .py(px(7.))
                                    .rounded(px(7.))
                                    .bg(theme.blue.opacity(0.08))
                                    .border_1()
                                    .border_color(theme.blue.opacity(0.2))
                                    .child(ui::icon("circle-dot", 12.0, theme.blue))
                                    .child(ui::label(
                                        self.t("diff.lfsDescription"),
                                        10.5,
                                        theme.blue,
                                    )),
                            )
                        })
                        .child(if use_split {
                            self.render_split_diff(project, file, theme, cx)
                        } else {
                            self.render_unified_diff(project, file, theme, cx)
                        }),
                )
                .into_any_element()
        };

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h(px(0.))
            .child(header)
            .child(body)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(25.))
                    .flex_none()
                    .px(px(16.))
                    .border_t_1()
                    .border_color(theme.border)
                    .text_size(px(10.0))
                    .text_color(theme.dim)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(ui::icon("circle-dot", 8.0, theme.add))
                            .child(self.t("common.workingTree"))
                            .child("·")
                            .child(language)
                            .child("UTF-8"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(format!(
                                "{} → {}",
                                project.branch,
                                self.t("common.workingTree")
                            ))
                            .child("·")
                            .child(format!(
                                "{} / {}",
                                self.file_position(project, file),
                                project.files.len()
                            )),
                    ),
            )
            .into_any_element()
    }

    fn file_position(&self, project: &crate::model::Project, file: &RepositoryFile) -> usize {
        project
            .files
            .iter()
            .position(|entry| entry.id == file.id)
            .map(|index| index + 1)
            .unwrap_or(1)
    }

    fn render_image_diff(&self, file: &RepositoryFile, theme: Theme) -> AnyElement {
        let cached = self.image_diffs.get(&file.id);
        let Some(cached) = cached else {
            return div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .gap(px(8.))
                .text_size(px(11.5))
                .text_color(theme.muted)
                .child(ui::icon("loader-circle", 14.0, theme.amber))
                .child(self.t("common.loading"))
                .into_any_element();
        };
        let before = image_panel(
            self.t("diff.imageBefore"),
            cached.old_path.clone(),
            cached.old_size,
            theme,
        );
        let after = image_panel(
            self.t("diff.imageAfter"),
            cached.new_path.clone(),
            cached.new_size,
            theme,
        );
        div()
            .flex()
            .flex_1()
            .gap(px(12.))
            .p(px(16.))
            .child(before)
            .child(after)
            .into_any_element()
    }

    fn render_binary_notice(
        &self,
        file: &RepositoryFile,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let path = file.path.clone();
        div()
            .flex()
            .flex_1()
            .items_center()
            .justify_center()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(9.))
                    .max_w(px(380.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(42.))
                            .rounded(px(12.))
                            .bg(theme.violet.opacity(0.1))
                            .child(ui::icon("file-code-2", 19.0, theme.violet)),
                    )
                    .child(
                        div()
                            .text_size(px(15.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(if diff::is_image(&file.path) {
                                self.t("diff.imageTitle")
                            } else {
                                self.t("diff.binaryTitle")
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(theme.muted)
                            .text_center()
                            .child(self.t("diff.binaryDescription")),
                    )
                    .child(ui::button(
                        "binary-open",
                        self.t("repo.openInDefaultApp"),
                        theme,
                        ButtonTone::Secondary,
                        true,
                        cx.listener(move |app, _event, _window, cx| {
                            let absolute = Path::new(
                                &app.active_project()
                                    .map(|p| p.path.clone())
                                    .unwrap_or_default(),
                            )
                            .join(&path);
                            open::that(absolute).ok();
                            cx.notify();
                        }),
                    )),
            )
            .into_any_element()
    }

    fn render_split_diff(
        &self,
        project: &crate::model::Project,
        file: &RepositoryFile,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let rows = diff::build_split_rows(&file.hunks);
        let language = file.language.clone();
        let project_id = project.id.clone();
        let file_id = file.id.clone();
        let staged = file.staged;
        let untracked = file.untracked;
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .h(px(26.))
                    .w_full()
                    .bg(theme.surface_raised)
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .flex_1()
                            .w(px(0.))
                            .px(px(10.))
                            .text_size(px(9.5))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.dim)
                            .child(self.t("diff.original").to_uppercase())
                            .child(
                                div()
                                    .text_size(px(9.5))
                                    .font_weight(gpui::FontWeight::NORMAL)
                                    .text_color(theme.muted)
                                    .child(project.branch.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .flex_1()
                            .w(px(0.))
                            .px(px(10.))
                            .border_l_1()
                            .border_color(theme.border.opacity(0.6))
                            .text_size(px(9.5))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.dim)
                            .child(self.t("diff.modified").to_uppercase())
                            .child(
                                div()
                                    .text_size(px(9.5))
                                    .font_weight(gpui::FontWeight::NORMAL)
                                    .text_color(theme.muted)
                                    .child(self.t("common.workingTree")),
                            ),
                    ),
            )
            .children(rows.into_iter().map(|row| match row {
                DiffRow::Hunk { index, hunk } => self.render_hunk_row(
                    &project_id,
                    &file_id,
                    index,
                    &hunk,
                    staged,
                    untracked,
                    theme,
                    cx,
                ),
                DiffRow::Split(row) => self.render_split_row(&language, &row, &file_id, theme, cx),
            }))
            .into_any_element()
    }

    fn render_unified_diff(
        &self,
        project: &crate::model::Project,
        file: &RepositoryFile,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let language = file.language.clone();
        let project_id = project.id.clone();
        let file_id = file.id.clone();
        let staged = file.staged;
        let untracked = file.untracked;
        let mut rows: Vec<AnyElement> = Vec::new();
        for (hunk_index, hunk) in file.hunks.iter().enumerate() {
            rows.push(self.render_hunk_row(
                &project_id,
                &file_id,
                hunk_index,
                hunk,
                staged,
                untracked,
                theme,
                cx,
            ));
            for line in &hunk.lines {
                let background = match line.kind {
                    LineType::Add => theme.add_soft,
                    LineType::Del => theme.del_soft,
                    LineType::Context => theme.bg,
                };
                let side = if line.kind == LineType::Del {
                    BlameSide::Old
                } else {
                    BlameSide::New
                };
                let hover_line = if side == BlameSide::Old {
                    line.old_no
                } else {
                    line.new_no.or(line.old_no)
                };
                rows.push(
                    div()
                        .flex()
                        .flex_row()
                        .h(px(20.))
                        .w_full()
                        .bg(background)
                        .when_some(hover_line, |this, line_no| {
                            let file_id = file_id.clone();
                            this.on_mouse_move(cx.listener(
                                move |app, event: &gpui::MouseMoveEvent, _window, cx| {
                                    app.set_blame_hover(
                                        file_id.clone(),
                                        side,
                                        line_no,
                                        event.position,
                                        cx,
                                    );
                                },
                            ))
                        })
                        .child(number_cell(line.old_no, theme))
                        .child(number_cell(line.new_no, theme))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(18.))
                                .flex_none()
                                .text_size(px(10.5))
                                .text_color(match line.kind {
                                    LineType::Add => theme.add,
                                    LineType::Del => theme.del,
                                    LineType::Context => theme.dim,
                                })
                                .child(match line.kind {
                                    LineType::Add => "+",
                                    LineType::Del => "−",
                                    LineType::Context => " ",
                                }),
                        )
                        .child(code_cell(
                            &line.content,
                            &language,
                            theme,
                            line.kind == LineType::Context,
                        ))
                        .into_any_element(),
                );
            }
        }
        div().flex().flex_col().children(rows).into_any_element()
    }

    fn render_hunk_row(
        &self,
        project_id: &str,
        file_id: &str,
        _hunk_index: usize,
        hunk: &DiffHunk,
        staged: bool,
        untracked: bool,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let project_id = project_id.to_string();
        let file_id_owned = file_id.to_string();
        let header = hunk.header.clone();
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .h(px(25.))
            .w_full()
            .px(px(10.))
            .bg(theme.surface_raised)
            .border_y_1()
            .border_color(theme.border.opacity(0.6))
            .child(
                div()
                    .text_size(px(10.0))
                    .font_family("SF Mono")
                    .text_color(theme.muted)
                    .child(hunk.header.clone()),
            )
            .when(!untracked, |this| {
                this.child(ui::icon_button(
                    "hunk-stage",
                    if staged { "minus" } else { "plus" },
                    theme,
                    if staged { theme.del } else { theme.add },
                    true,
                    cx.listener({
                        let project_id = project_id.clone();
                        let file_id = file_id_owned.clone();
                        let header = header.clone();
                        move |app, _event, _window, cx| {
                            if staged {
                                app.unstage_hunk(&project_id, &file_id, &header, cx);
                            } else {
                                app.stage_hunk(&project_id, &file_id, &header, cx);
                            }
                        }
                    }),
                ))
            })
            .child(div().flex_1())
            .when(!staged, |this| {
                this.child(ui::icon_button(
                    "hunk-discard",
                    "rotate-ccw",
                    theme,
                    theme.del,
                    true,
                    cx.listener({
                        let project_id = project_id.clone();
                        let file_id = file_id_owned.clone();
                        let header = header.clone();
                        move |app, _event, _window, cx| {
                            app.request_discard_hunk(&project_id, &file_id, &header, cx);
                        }
                    }),
                ))
            })
            .into_any_element()
    }

    fn render_split_row(
        &self,
        language: &str,
        row: &SplitRow,
        file_id: &str,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let left = row.left.as_ref();
        let right = row.right.as_ref();
        let paired = left.is_some_and(|line| line.kind == LineType::Del)
            && right.is_some_and(|line| line.kind == LineType::Add);
        let (left_words, right_words) = if paired {
            let old = left.map(|line| line.content.as_str()).unwrap_or("");
            let new = right.map(|line| line.content.as_str()).unwrap_or("");
            diff::word_diff(old, new)
        } else {
            (Vec::new(), Vec::new())
        };
        let left_bg = match left.map(|line| line.kind) {
            Some(LineType::Del) => theme.del_soft,
            _ => theme.bg,
        };
        let right_bg = match right.map(|line| line.kind) {
            Some(LineType::Add) => theme.add_soft,
            _ => theme.bg,
        };
        div()
            .flex()
            .flex_row()
            .h(px(20.))
            .w_full()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .w(px(0.))
                    .flex_1()
                    .bg(left_bg)
                    .border_r_1()
                    .border_color(theme.border.opacity(0.5))
                    .when_some(left.and_then(|line| line.old_no), |this, line_no| {
                        let file_id = file_id.to_string();
                        this.on_mouse_move(cx.listener(
                            move |app, event: &gpui::MouseMoveEvent, _window, cx| {
                                app.set_blame_hover(
                                    file_id.clone(),
                                    BlameSide::Old,
                                    line_no,
                                    event.position,
                                    cx,
                                );
                            },
                        ))
                    })
                    .child(number_cell(left.and_then(|line| line.old_no), theme))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(16.))
                            .flex_none()
                            .text_size(px(10.5))
                            .text_color(theme.del)
                            .child(if left.map(|line| line.kind) == Some(LineType::Del) {
                                "−"
                            } else {
                                ""
                            }),
                    )
                    .child(if paired && !left_words.is_empty() {
                        word_cell(&left_words, theme, false)
                    } else {
                        left.map(|line| {
                            code_cell(
                                &line.content,
                                language,
                                theme,
                                line.kind == LineType::Context,
                            )
                        })
                        .unwrap_or_else(empty_code_cell)
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .w(px(0.))
                    .flex_1()
                    .bg(right_bg)
                    .when_some(right.and_then(|line| line.new_no), |this, line_no| {
                        let file_id = file_id.to_string();
                        this.on_mouse_move(cx.listener(
                            move |app, event: &gpui::MouseMoveEvent, _window, cx| {
                                app.set_blame_hover(
                                    file_id.clone(),
                                    BlameSide::New,
                                    line_no,
                                    event.position,
                                    cx,
                                );
                            },
                        ))
                    })
                    .child(number_cell(right.and_then(|line| line.new_no), theme))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(16.))
                            .flex_none()
                            .text_size(px(10.5))
                            .text_color(theme.add)
                            .child(if right.map(|line| line.kind) == Some(LineType::Add) {
                                "+"
                            } else {
                                ""
                            }),
                    )
                    .child(if paired && !right_words.is_empty() {
                        word_cell(&right_words, theme, true)
                    } else {
                        right
                            .map(|line| {
                                code_cell(
                                    &line.content,
                                    language,
                                    theme,
                                    line.kind == LineType::Context,
                                )
                            })
                            .unwrap_or_else(empty_code_cell)
                    }),
            )
            .into_any_element()
    }
}

fn number_cell(number: Option<u32>, theme: Theme) -> AnyElement {
    div()
        .flex()
        .items_center()
        .justify_end()
        .w(px(42.))
        .flex_none()
        .pr(px(6.))
        .text_size(px(10.0))
        .font_family("SF Mono")
        .text_color(theme.dim)
        .child(number.map(|value| value.to_string()).unwrap_or_default())
        .into_any_element()
}

fn code_cell(content: &str, language: &str, theme: Theme, context: bool) -> AnyElement {
    let spans = highlight::highlight_line(content, language, theme);
    let mut cell = div()
        .flex()
        .flex_row()
        .items_center()
        .flex_1()
        .min_w(px(0.))
        .overflow_hidden()
        .pl(px(4.))
        .text_size(px(12.5))
        .font_family("SF Mono")
        .whitespace_nowrap();
    if content.is_empty() {
        cell = cell.child(" ");
    } else {
        for span in spans {
            let mut text = div()
                .flex_none()
                .text_color(if context { theme.text_soft } else { span.color })
                .child(span.text);
            if span.italic {
                text = text.italic();
            }
            cell = cell.child(text);
        }
    }
    cell.into_any_element()
}

fn empty_code_cell() -> AnyElement {
    div().flex_1().h(px(20.)).min_w(px(0.)).into_any_element()
}

fn word_cell(words: &[diff::WordSegment], theme: Theme, added: bool) -> AnyElement {
    let base = if added { theme.add } else { theme.del };
    let highlight = if added {
        theme.add.opacity(0.24)
    } else {
        theme.del.opacity(0.22)
    };
    let mut cell = div()
        .flex()
        .flex_row()
        .items_center()
        .flex_1()
        .min_w(px(0.))
        .overflow_hidden()
        .pl(px(4.))
        .text_size(px(12.5))
        .font_family("SF Mono")
        .whitespace_nowrap();
    if words.is_empty() {
        cell = cell.child(" ");
    }
    for word in words {
        let mut part = div()
            .flex_none()
            .text_color(if word.changed { base } else { theme.text_soft })
            .child(word.text.clone());
        if word.changed {
            part = part.bg(highlight).rounded(px(2.));
        }
        cell = cell.child(part);
    }
    cell.into_any_element()
}

fn menu_label(text: &str, theme: Theme) -> AnyElement {
    div()
        .px(px(8.))
        .pt(px(4.))
        .pb(px(2.))
        .text_size(px(9.5))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(theme.dim)
        .child(text.to_uppercase())
        .into_any_element()
}

fn menu_action(
    id: &'static str,
    icon_name: &'static str,
    text: &'static str,
    theme: Theme,
    danger: bool,
    handler: impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .flex()
        .items_center()
        .gap(px(8.))
        .h(px(28.))
        .px(px(8.))
        .rounded(px(5.))
        .cursor_pointer()
        .hover(move |style| {
            style.bg(if danger {
                theme.del.opacity(0.1)
            } else {
                theme.surface_hover
            })
        })
        .on_click(handler)
        .child(ui::icon(
            icon_name,
            12.0,
            if danger { theme.del } else { theme.muted },
        ))
        .child(ui::label(
            text,
            11.5,
            if danger { theme.del } else { theme.text_soft },
        ))
        .into_any_element()
}

fn reveal_in_finder(path: &str) {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").args(["-R", path]).spawn().ok();
    }
    #[cfg(not(target_os = "macos"))]
    {
        open::that(path).ok();
    }
}

fn open_in_editor(path: &str) {
    #[cfg(target_os = "macos")]
    {
        for application in [
            "Visual Studio Code",
            "Cursor",
            "Zed",
            "Sublime Text",
            "Nova",
        ] {
            if Command::new("open")
                .args(["-a", application, path])
                .status()
                .map(|status| status.success())
                .unwrap_or(false)
            {
                return;
            }
        }
    }
    open::that(path).ok();
}

pub fn open_path(path: &Path) {
    open::that(path).ok();
}

pub fn absolute_path_for_project(project_path: &str, file_path: &str) -> PathBuf {
    Path::new(project_path).join(file_path)
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

fn image_panel(label: &'static str, path: Option<PathBuf>, size: u64, theme: Theme) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w(px(0.))
        .rounded(px(9.))
        .border_1()
        .border_color(theme.border)
        .bg(theme.bg_deep)
        .overflow_hidden()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(10.))
                .py(px(6.))
                .border_b_1()
                .border_color(theme.border)
                .child(ui::label(label, 9.5, theme.dim))
                .when(size > 0, |this| {
                    this.child(ui::label(format_bytes(size), 9.5, theme.dim))
                }),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .h(px(320.))
                .p(px(10.))
                .when_some(path, |this, path| {
                    this.child(img(path).object_fit(ObjectFit::Contain).w_full().h_full())
                })
                .when(size == 0, |this| {
                    this.child(ui::label("No previous version", 10.5, theme.dim))
                }),
        )
        .into_any_element()
}

fn first_hunk_line(file: &RepositoryFile) -> u32 {
    file.hunks
        .first()
        .and_then(|hunk| {
            let start = hunk.header.split(" +").nth(1)?;
            start
                .split([',', ' '])
                .next()
                .and_then(|value| value.parse::<u32>().ok())
        })
        .unwrap_or(1)
}

fn open_in_editor_at_line(path: &Path, line: u32) {
    #[cfg(target_os = "macos")]
    {
        let target = format!("{}:{line}", path.to_string_lossy());
        for application in ["Visual Studio Code", "Cursor", "Zed"] {
            if Command::new("open")
                .args(["-a", application, "--args", "-g", &target])
                .status()
                .map(|status| status.success())
                .unwrap_or(false)
            {
                return;
            }
        }
    }
    open::that(path).ok();
}
