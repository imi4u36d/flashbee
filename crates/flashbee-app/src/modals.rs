use gpui::{AnyElement, Context, SharedString, div, prelude::*, px};

use crate::app::{ConfirmState, FlashBeeApp, ToastKind};
use crate::i18n;
use crate::i18n::LocalePreference;
use crate::input::input_frame;
use crate::model::{CODE_SIZE_MAX, CODE_SIZE_MIN, UI_SIZE_MAX, UI_SIZE_MIN};
use crate::theme::{Theme, ThemeMode};
use crate::ui::{self, ButtonTone};

impl FlashBeeApp {
    pub fn render_overlays(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme();
        if let Some(confirm) = self.confirm.clone() {
            return self.render_confirm_dialog(&confirm, theme, cx);
        }
        if self.branch_manager.is_some() {
            return self.render_branch_manager(theme, cx);
        }
        if self.commit_dialog.is_some() {
            return self.render_commit_dialog(theme, cx);
        }
        if self.branch_dialog.is_some() {
            return self.render_branch_dialog(theme, cx);
        }
        if self.stash_dialog.is_some() {
            return self.render_stash_dialog(theme, cx);
        }
        if self.add_repo.is_some() {
            return self.render_add_repo_dialog(theme, cx);
        }
        if self.push.is_some() {
            return self.render_push_dialog(theme, cx);
        }
        if self.remote_dialog.is_some() {
            return self.render_remote_dialog(theme, cx);
        }
        if self.workspace_dialog.is_some() {
            return self.render_workspace_dialog(theme, cx);
        }
        if self.settings_open {
            return self.render_settings_dialog(theme, cx);
        }
        if self.search.is_some() {
            return self.render_search_overlay(theme, cx);
        }
        div().into_any_element()
    }

    fn overlay_card(
        &self,
        theme: Theme,
        width: f32,
        child: AnyElement,
        _cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .size_full()
            .absolute()
            .top_0()
            .left_0()
            .bg(theme.overlay)
            .flex()
            .flex_col()
            .items_center()
            .pt(px(84.))
            .child(
                div()
                    .w(px(width))
                    .max_h(px(680.))
                    .flex()
                    .flex_col()
                    .rounded(px(12.))
                    .bg(theme.surface_raised)
                    .border_1()
                    .border_color(theme.border_strong)
                    .shadow_xl()
                    .child(child),
            )
            .into_any_element()
    }

    fn modal_header(
        &self,
        title: &str,
        subtitle: Option<String>,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .flex()
            .items_start()
            .gap(px(10.))
            .px(px(18.))
            .pt(px(16.))
            .pb(px(10.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .gap(px(3.))
                    .child(
                        div()
                            .text_size(px(16.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(title.to_string()),
                    )
                    .when_some(subtitle, |this, subtitle| {
                        this.child(
                            div()
                                .text_size(px(11.5))
                                .text_color(theme.muted)
                                .child(subtitle),
                        )
                    }),
            )
            .child(ui::icon_button(
                "modal-close",
                "x",
                theme,
                theme.muted,
                true,
                cx.listener(|app, _event, _window, cx| app.close_overlay(cx)),
            ))
            .into_any_element()
    }

    fn render_branch_manager(&mut self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        let manager = self.branch_manager.as_ref().unwrap();
        let project_name = self
            .project(&manager.project_id)
            .map(|project| project.name.clone())
            .unwrap_or_default();
        let current_branch = self
            .project(&manager.project_id)
            .map(|project| project.current_branch.clone())
            .unwrap_or_default();
        let tab = manager.tab;
        let branches = manager.branches.clone();
        let worktrees = manager.worktrees.clone();
        let loading = manager.loading;
        let busy = manager.busy;
        let error = manager.error.clone();
        let filter_input = manager.filter_input.clone();
        let filter = manager
            .filter_input
            .read(cx)
            .content()
            .trim()
            .to_lowercase();
        let create_name = manager.create_name.clone();
        let create_start = manager.create_start.clone();
        let create_switch = manager.create_switch;
        let rename_target = manager.rename_target.clone();
        let rename_input = manager.rename_input.clone();
        let worktree_branch = manager.worktree_branch.clone();
        let worktree_path = manager.worktree_path.clone();
        let worktree_start = manager.worktree_start.clone();
        let worktree_new_branch = manager.worktree_new_branch;
        let project_id = manager.project_id.clone();

        let local: Vec<_> = branches
            .iter()
            .filter(|branch| !branch.is_remote)
            .filter(|branch| filter.is_empty() || branch.name.to_lowercase().contains(&filter))
            .cloned()
            .collect();
        let remote: Vec<_> = branches
            .iter()
            .filter(|branch| branch.is_remote)
            .filter(|branch| filter.is_empty() || branch.name.to_lowercase().contains(&filter))
            .cloned()
            .collect();

        let tabs = div()
            .flex()
            .items_center()
            .gap(px(4.))
            .px(px(18.))
            .pb(px(8.))
            .child(choice_button(
                "tab-branches",
                self.t("branch.tabBranches"),
                tab == crate::app::BranchTab::Branches,
                theme,
                cx.listener(|app, _event, _window, cx| {
                    app.set_branch_manager_tab(crate::app::BranchTab::Branches, cx);
                }),
            ))
            .child(choice_button(
                "tab-worktrees",
                self.t("worktree.title"),
                tab == crate::app::BranchTab::Worktrees,
                theme,
                cx.listener(|app, _event, _window, cx| {
                    app.set_branch_manager_tab(crate::app::BranchTab::Worktrees, cx);
                }),
            ))
            .child(div().flex_1())
            .child(
                div()
                    .max_w(px(320.))
                    .truncate()
                    .text_size(px(9.5))
                    .text_color(theme.dim)
                    .child(i18n::fill(
                        self.t("worktree.description"),
                        &[("name", &project_name)],
                    )),
            );

        let content: AnyElement = if tab == crate::app::BranchTab::Worktrees {
            let mut list = div().flex().flex_col().gap(px(3.));
            if worktrees.is_empty() && !loading {
                list = list.child(
                    div()
                        .py(px(24.))
                        .text_size(px(11.5))
                        .text_color(theme.muted)
                        .text_center()
                        .child(self.t("worktree.empty")),
                );
            }
            for worktree in worktrees.iter() {
                let path = worktree.info.path.clone();
                let name = worktree.info.name.clone();
                let branch = worktree.info.branch.clone();
                let head = worktree.info.head.clone();
                let is_main = worktree.info.is_main;
                let is_current = worktree.info.is_current;
                let detached = worktree.info.detached;
                let changed = worktree.changed;
                let conflicts = worktree.conflicts;
                let path_for_open = path.clone();
                let path_for_remove = path.clone();
                list = list.child(
                    div()
                        .id(SharedString::from(format!("manager-worktree-{path}")))
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .px(px(10.))
                        .py(px(8.))
                        .rounded(px(8.))
                        .border_1()
                        .border_color(if is_current {
                            theme.border_strong
                        } else {
                            theme.border
                        })
                        .bg(if is_current {
                            theme.surface_hover
                        } else {
                            theme.bg
                        })
                        .child(ui::icon(
                            if is_main { "folder-git-2" } else { "git-fork" },
                            16.0,
                            if is_main { theme.amber } else { theme.blue },
                        ))
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
                                        .gap(px(6.))
                                        .child(
                                            div()
                                                .text_size(px(12.0))
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(theme.text)
                                                .child(name),
                                        )
                                        .when(is_main, |this| {
                                            this.child(ui::pill(
                                                self.t("worktree.main"),
                                                theme.amber,
                                                theme.amber_soft,
                                                8.0,
                                            ))
                                        })
                                        .when(is_current, |this| {
                                            this.child(ui::pill(
                                                self.t("worktree.current"),
                                                theme.add,
                                                theme.add.opacity(0.1),
                                                8.0,
                                            ))
                                        })
                                        .when(detached, |this| {
                                            this.child(ui::pill(
                                                self.t("worktree.detached"),
                                                theme.violet,
                                                theme.violet.opacity(0.1),
                                                8.0,
                                            ))
                                        })
                                        .when(changed > 0, |this| {
                                            this.child(ui::pill(
                                                i18n::fill(
                                                    self.t("worktree.changed"),
                                                    &[("count", &changed.to_string())],
                                                ),
                                                theme.amber,
                                                theme.amber_soft,
                                                8.0,
                                            ))
                                        })
                                        .when(conflicts > 0, |this| {
                                            this.child(ui::pill(
                                                i18n::fill(
                                                    self.t("worktree.conflicts"),
                                                    &[("count", &conflicts.to_string())],
                                                ),
                                                theme.del,
                                                theme.del.opacity(0.1),
                                                8.0,
                                            ))
                                        }),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.))
                                        .text_size(px(9.5))
                                        .text_color(theme.dim)
                                        .child(ui::icon("git-branch", 9.0, theme.dim))
                                        .child(if detached {
                                            head.clone()
                                        } else {
                                            branch.clone()
                                        })
                                        .child("·")
                                        .child(
                                            div()
                                                .flex_1()
                                                .min_w(px(0.))
                                                .truncate()
                                                .child(path.clone()),
                                        ),
                                ),
                        )
                        .child(ui::button(
                            "worktree-open",
                            self.t("worktree.open"),
                            theme,
                            ButtonTone::Secondary,
                            !busy,
                            cx.listener(move |app, _event, _window, cx| {
                                app.open_worktree_path(&path_for_open, cx);
                            }),
                        ))
                        .when(!is_main && !is_current, |this| {
                            this.child(ui::icon_button(
                                "worktree-remove",
                                "trash-2",
                                theme,
                                theme.del,
                                !busy,
                                cx.listener(move |app, _event, _window, cx| {
                                    app.request_remove_worktree(
                                        path_for_remove.clone(),
                                        changed > 0 || conflicts > 0,
                                        cx,
                                    );
                                }),
                            ))
                        })
                        .into_any_element(),
                );
            }
            div()
                .flex()
                .flex_col()
                .gap(px(10.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .px(px(18.))
                        .child(ui::checkbox(
                            "worktree-new-branch",
                            worktree_new_branch,
                            self.t("worktree.newBranch"),
                            theme,
                            cx.listener(|app, _event, _window, cx| {
                                if let Some(manager) = app.branch_manager.as_mut() {
                                    manager.worktree_new_branch = !manager.worktree_new_branch;
                                }
                                cx.notify();
                            }),
                        ))
                        .child(div().flex_1())
                        .child(ui::button(
                            "worktree-prune",
                            self.t("worktree.prune"),
                            theme,
                            ButtonTone::Ghost,
                            !busy,
                            cx.listener(|app, _event, _window, cx| app.prune_worktrees(cx)),
                        )),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .px(px(18.))
                        .child(
                            input_frame(theme)
                                .w(px(200.))
                                .child(worktree_branch.clone()),
                        )
                        .when(worktree_new_branch, |this| {
                            this.child(input_frame(theme).w(px(180.)).child(worktree_start.clone()))
                        })
                        .child(input_frame(theme).flex_1().child(worktree_path.clone()))
                        .child(ui::button(
                            "worktree-create",
                            self.t("worktree.create"),
                            theme,
                            ButtonTone::Primary,
                            !busy,
                            cx.listener(|app, _event, _window, cx| {
                                app.add_worktree_from_manager(cx);
                            }),
                        )),
                )
                .child(
                    div()
                        .id("manager-worktree-list")
                        .flex()
                        .flex_col()
                        .gap(px(3.))
                        .px(px(18.))
                        .flex_1()
                        .min_h(px(0.))
                        .overflow_scroll()
                        .child(list),
                )
                .into_any_element()
        } else {
            let mut list = div().flex().flex_col().gap(px(2.));
            if local.is_empty() && !loading {
                list = list.child(
                    div()
                        .py(px(20.))
                        .text_size(px(11.0))
                        .text_color(theme.muted)
                        .text_center()
                        .child(self.t("branch.noMatches")),
                );
            }
            for branch in local.iter() {
                let project_id = project_id.clone();
                let name = branch.name.clone();
                let name_for_switch = name.clone();
                let is_current = branch.is_current;
                let merged = branch.merged;
                let worktree_path = branch.worktree_path.clone();
                let worktree_for_click = worktree_path.clone();
                let ahead = branch.ahead;
                let behind = branch.behind;
                let upstream = branch.upstream.clone();
                let short_commit = branch.short_commit.clone();
                let subject = branch.subject.clone();
                let time = branch.time;
                let name_for_merge = name.clone();
                let name_for_rename = name.clone();
                let name_for_delete = name.clone();
                let name_for_delete_force = name.clone();
                list = list.child(
                    div()
                        .id(SharedString::from(format!("manager-branch-{name}")))
                        .flex()
                        .items_center()
                        .gap(px(9.))
                        .px(px(10.))
                        .py(px(7.))
                        .rounded(px(8.))
                        .border_1()
                        .border_color(if is_current {
                            theme.border_strong
                        } else {
                            theme.border.opacity(0.7)
                        })
                        .bg(if is_current {
                            theme.surface_hover
                        } else {
                            theme.bg
                        })
                        .child(ui::icon(
                            if is_current { "check" } else { "git-branch" },
                            13.0,
                            if is_current { theme.amber } else { theme.muted },
                        ))
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
                                        .gap(px(6.))
                                        .child(
                                            div()
                                                .text_size(px(12.5))
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(theme.text)
                                                .truncate()
                                                .child(name.clone()),
                                        )
                                        .when(is_current, |this| {
                                            this.child(ui::pill(
                                                self.t("common.current"),
                                                theme.amber,
                                                theme.amber_soft,
                                                8.0,
                                            ))
                                        })
                                        .when(!merged && !is_current, |this| {
                                            this.child(ui::pill(
                                                self.t("branch.notMerged"),
                                                theme.violet,
                                                theme.violet.opacity(0.1),
                                                8.0,
                                            ))
                                        })
                                        .when(worktree_path.is_some() && !is_current, |this| {
                                            this.child(ui::pill(
                                                self.t("repo.worktree"),
                                                theme.blue,
                                                theme.blue.opacity(0.1),
                                                8.0,
                                            ))
                                        })
                                        .when(ahead > 0, |this| {
                                            this.child(ui::label(
                                                format!("↑{ahead}"),
                                                9.0,
                                                theme.add,
                                            ))
                                        })
                                        .when(behind > 0, |this| {
                                            this.child(ui::label(
                                                format!("↓{behind}"),
                                                9.0,
                                                theme.blue,
                                            ))
                                        }),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.))
                                        .text_size(px(9.0))
                                        .text_color(theme.dim)
                                        .child(short_commit)
                                        .child(relative_branch_time(time, self.lang()))
                                        .when_some(upstream, |this, upstream| {
                                            this.child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap(px(2.))
                                                    .child(ui::icon(
                                                        "cloud-download",
                                                        8.0,
                                                        theme.dim,
                                                    ))
                                                    .child(upstream),
                                            )
                                        })
                                        .child(
                                            div().flex_1().min_w(px(0.)).truncate().child(subject),
                                        ),
                                ),
                        )
                        .when(!is_current, |this| {
                            this.child(ui::button(
                                "branch-merge",
                                self.t("branch.merge"),
                                theme,
                                ButtonTone::Ghost,
                                !busy,
                                cx.listener(move |app, _event, _window, cx| {
                                    app.request_merge_branch(name_for_merge.clone(), cx);
                                }),
                            ))
                        })
                        .child(if is_current {
                            div().w(px(58.)).into_any_element()
                        } else if let Some(worktree) = worktree_for_click {
                            ui::button(
                                "branch-open-worktree",
                                self.t("branch.openWorktree"),
                                theme,
                                ButtonTone::Secondary,
                                !busy,
                                cx.listener(move |app, _event, _window, cx| {
                                    app.open_worktree_path(&worktree, cx);
                                }),
                            )
                        } else {
                            ui::button(
                                "branch-switch",
                                self.t("branch.switch"),
                                theme,
                                ButtonTone::Primary,
                                !busy,
                                {
                                    let project_id = project_id.clone();
                                    cx.listener(move |app, _event, _window, cx| {
                                        app.branch_manager = None;
                                        app.pending_main_focus = true;
                                        app.switch_branch(&project_id, &name_for_switch, cx);
                                    })
                                },
                            )
                        })
                        .when(!is_current && worktree_path.is_none(), |this| {
                            this.child(ui::icon_button(
                                "branch-rename",
                                "pencil",
                                theme,
                                theme.muted,
                                !busy,
                                cx.listener(move |app, _event, _window, cx| {
                                    app.begin_rename_branch(name_for_rename.clone(), cx);
                                }),
                            ))
                            .child(ui::icon_button(
                                "branch-delete",
                                "trash-2",
                                theme,
                                theme.del,
                                !busy,
                                cx.listener(move |app, _event, _window, cx| {
                                    app.request_delete_branch(
                                        name_for_delete_force.clone(),
                                        !merged,
                                        cx,
                                    );
                                }),
                            ))
                        })
                        .into_any_element(),
                );
            }
            let mut remote_list = div().flex().flex_col().gap(px(1.));
            if !remote.is_empty() {
                remote_list = remote_list.child(
                    div()
                        .px(px(10.))
                        .pt(px(10.))
                        .pb(px(3.))
                        .text_size(px(9.0))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme.dim)
                        .child(self.t("branch.remote").to_uppercase()),
                );
            }
            for branch in remote.iter() {
                let project_id = project_id.clone();
                let name = branch.name.clone();
                let name_for_switch = name.clone();
                let remote_name = branch.remote.clone().unwrap_or_default();
                remote_list = remote_list.child(
                    div()
                        .id(SharedString::from(format!("manager-remote-{name}")))
                        .flex()
                        .items_center()
                        .gap(px(9.))
                        .px(px(10.))
                        .py(px(6.))
                        .rounded(px(7.))
                        .hover(move |style| style.bg(theme.surface_hover))
                        .child(ui::icon("cloud-download", 12.0, theme.muted))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.))
                                .flex_1()
                                .min_w(px(0.))
                                .text_size(px(11.0))
                                .text_color(theme.text_soft)
                                .child(name.clone())
                                .child(ui::label(remote_name, 9.0, theme.dim)),
                        )
                        .child(ui::button(
                            "remote-checkout",
                            self.t("branch.switch"),
                            theme,
                            ButtonTone::Ghost,
                            !busy,
                            cx.listener(move |app, _event, _window, cx| {
                                app.branch_manager = None;
                                app.pending_main_focus = true;
                                app.switch_branch(&project_id, &name_for_switch, cx);
                            }),
                        ))
                        .into_any_element(),
                );
            }

            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_h(px(0.))
                .gap(px(8.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .px(px(18.))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .flex_1()
                                .child(input_frame(theme).child(filter_input.clone())),
                        )
                        .child(
                            div()
                                .text_size(px(9.5))
                                .text_color(theme.dim)
                                .child(i18n::fill(
                                    self.t("branch.count"),
                                    &[("count", &local.len().to_string())],
                                )),
                        ),
                )
                .when_some(rename_target.clone(), |this, target| {
                    this.child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .px(px(18.))
                            .child(ui::label(
                                i18n::fill(self.t("branch.rename"), &[]),
                                10.0,
                                theme.muted,
                            ))
                            .child(ui::label(target.clone(), 10.5, theme.text_soft))
                            .child(
                                input_frame(theme).w(px(220.)).child(
                                    rename_input.clone().unwrap_or_else(|| create_name.clone()),
                                ),
                            )
                            .child(ui::button(
                                "rename-save",
                                self.t("common.save"),
                                theme,
                                ButtonTone::Primary,
                                !busy,
                                cx.listener(|app, _event, _window, cx| {
                                    app.submit_rename_branch(cx);
                                }),
                            ))
                            .child(ui::button(
                                "rename-cancel",
                                self.t("common.cancel"),
                                theme,
                                ButtonTone::Ghost,
                                !busy,
                                cx.listener(|app, _event, _window, cx| {
                                    app.cancel_rename_branch(cx);
                                }),
                            )),
                    )
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .px(px(18.))
                        .child(input_frame(theme).w(px(220.)).child(create_name.clone()))
                        .child(input_frame(theme).w(px(160.)).child(create_start.clone()))
                        .child(ui::checkbox(
                            "branch-create-switch",
                            create_switch,
                            self.t("branch.createAndSwitch"),
                            theme,
                            cx.listener(|app, _event, _window, cx| {
                                let value = app
                                    .branch_manager
                                    .as_ref()
                                    .map(|manager| manager.create_switch)
                                    .unwrap_or(false);
                                app.set_create_switch(!value, cx);
                            }),
                        ))
                        .child(div().flex_1())
                        .child(ui::button(
                            "branch-create",
                            self.t("common.create"),
                            theme,
                            ButtonTone::Primary,
                            !busy,
                            cx.listener(|app, _event, _window, cx| {
                                app.create_branch_from_manager(cx);
                            }),
                        )),
                )
                .child(
                    div()
                        .id("manager-branch-list")
                        .flex()
                        .flex_col()
                        .gap(px(2.))
                        .px(px(18.))
                        .flex_1()
                        .min_h(px(0.))
                        .overflow_scroll()
                        .child(list)
                        .child(remote_list),
                )
                .into_any_element()
        };

        let body = div()
            .flex()
            .flex_col()
            .h(px(620.))
            .gap(px(8.))
            .child(self.modal_header(
                self.t("branch.manageTitle"),
                Some(i18n::fill(
                    self.t("branch.managerHint"),
                    &[("name", &project_name)],
                )),
                theme,
                cx,
            ))
            .child(tabs)
            .when_some(error, |this, error| {
                this.child(div().px(px(18.)).child(error_line(&error, theme)))
            })
            .when(loading, |this| {
                this.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .px(px(18.))
                        .py(px(6.))
                        .text_size(px(10.5))
                        .text_color(theme.muted)
                        .child(ui::icon("loader-circle", 12.0, theme.amber))
                        .child(self.t("common.loading")),
                )
            })
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.))
                    .child(content),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(18.))
                    .pb(px(12.))
                    .pt(px(4.))
                    .child(ui::label(
                        format!(
                            "{} · {}",
                            current_branch,
                            if tab == crate::app::BranchTab::Worktrees {
                                self.t("worktree.title")
                            } else {
                                self.t("branch.tabBranches")
                            }
                        ),
                        9.5,
                        theme.dim,
                    ))
                    .child(ui::button(
                        "branch-manager-close",
                        self.t("common.close"),
                        theme,
                        ButtonTone::Ghost,
                        true,
                        cx.listener(|app, _event, _window, cx| {
                            app.branch_manager = None;
                            app.pending_main_focus = true;
                            cx.notify();
                        }),
                    )),
            )
            .into_any_element();

        self.overlay_card(theme, 820.0, body, cx)
    }

    fn render_commit_dialog(&mut self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        let (input, amend, busy, error, push_after) = {
            let dialog = self.commit_dialog.as_ref().unwrap();
            (
                dialog.input.clone(),
                dialog.amend,
                dialog.busy,
                dialog.error.clone(),
                dialog.push_after,
            )
        };
        let project = self.active_project().cloned();
        let staged: Vec<_> = project
            .as_ref()
            .map(|project| {
                project
                    .files
                    .iter()
                    .filter(|file| file.staged)
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let unstaged = project
            .as_ref()
            .map(|project| project.files.iter().filter(|file| !file.staged).count())
            .unwrap_or(0);
        let branch = project
            .as_ref()
            .map(|project| project.current_branch.clone())
            .unwrap_or_default();
        let staged_count = staged.len();
        let subtitle = i18n::fill(
            self.t("commit.description"),
            &[("branch", &branch), ("count", &staged_count.to_string())],
        );

        let body = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .px(px(18.))
            .pb(px(16.))
            .child(self.modal_header(self.t("commit.title"), Some(subtitle), theme, cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(field_label(self.t("commit.messageLabel"), theme))
                    .child(input_frame(theme).child(input)),
            )
            .when_some(error, |this, error| this.child(error_line(&error, theme)))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(field_label(
                                &format!("{} · {}", self.t("commit.showStaged"), staged_count),
                                theme,
                            ))
                            .when(unstaged > 0, |this| {
                                this.child(ui::button(
                                    "stage-all-from-commit",
                                    self.t("diff.stageAll"),
                                    theme,
                                    ButtonTone::Ghost,
                                    true,
                                    cx.listener(|app, _event, _window, cx| {
                                        app.stage_all(cx);
                                    }),
                                ))
                            }),
                    )
                    .child(
                        div()
                            .id("commit-staged-list")
                            .flex()
                            .flex_col()
                            .max_h(px(140.))
                            .overflow_scroll()
                            .rounded(px(8.))
                            .border_1()
                            .border_color(theme.border)
                            .bg(theme.bg)
                            .when(staged.is_empty(), |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(2.))
                                        .px(px(10.))
                                        .py(px(8.))
                                        .child(ui::label(
                                            self.t("commit.noStagedTitle"),
                                            11.5,
                                            theme.text_soft,
                                        ))
                                        .child(ui::label(
                                            self.t("commit.noStagedDescription"),
                                            10.5,
                                            theme.muted,
                                        )),
                                )
                            })
                            .children(staged.iter().map(|file| {
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(7.))
                                    .h(px(24.))
                                    .px(px(8.))
                                    .border_b_1()
                                    .border_color(theme.border.opacity(0.6))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .size(px(14.))
                                            .rounded(px(4.))
                                            .bg(theme.status(file.status).opacity(0.12))
                                            .text_size(px(9.0))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.status(file.status))
                                            .child(crate::diff::status_letter(file.status)),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w(px(0.))
                                            .text_size(px(11.0))
                                            .text_color(theme.text_soft)
                                            .truncate()
                                            .child(file.path.clone()),
                                    )
                                    .into_any_element()
                            })),
                    ),
            )
            .child(ui::checkbox(
                "commit-amend",
                amend,
                self.t("commit.amend"),
                theme,
                cx.listener(|app, _event, _window, cx| {
                    let amend = app
                        .commit_dialog
                        .as_ref()
                        .map(|dialog| dialog.amend)
                        .unwrap_or(false);
                    app.set_amend(!amend, cx);
                }),
            ))
            .when(amend, |this| {
                this.child(
                    div()
                        .text_size(px(10.5))
                        .text_color(theme.dim)
                        .child(self.t("commit.amendHint")),
                )
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap(px(8.))
                    .pt(px(4.))
                    .child(ui::button(
                        "commit-cancel",
                        self.t("common.cancel"),
                        theme,
                        ButtonTone::Ghost,
                        !busy,
                        cx.listener(|app, _event, _window, cx| app.close_overlay(cx)),
                    ))
                    .child(ui::button(
                        "commit-push",
                        self.t("commit.commitAndPush"),
                        theme,
                        ButtonTone::Secondary,
                        !busy,
                        cx.listener(|app, _event, _window, cx| {
                            app.commit_with(true, cx);
                        }),
                    ))
                    .child(ui::button(
                        "commit-submit",
                        if busy {
                            self.t("commit.committing")
                        } else if push_after {
                            self.t("commit.commitAndPush")
                        } else {
                            self.t("commit.commit")
                        },
                        theme,
                        ButtonTone::Primary,
                        !busy,
                        cx.listener(|app, _event, _window, cx| {
                            let push = app
                                .commit_dialog
                                .as_ref()
                                .map(|dialog| dialog.push_after)
                                .unwrap_or(false);
                            app.commit_with(push, cx);
                        }),
                    )),
            );

        self.overlay_card(theme, 540.0, body.into_any_element(), cx)
    }

    fn render_branch_dialog(&mut self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        let (input, busy, error) = {
            let dialog = self.branch_dialog.as_ref().unwrap();
            (dialog.input.clone(), dialog.busy, dialog.error.clone())
        };
        let body = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .px(px(18.))
            .pb(px(16.))
            .child(self.modal_header(
                self.t("branch.newTitle"),
                Some(self.t("branch.newDescription").to_string()),
                theme,
                cx,
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(field_label(self.t("branch.nameLabel"), theme))
                    .child(input_frame(theme).child(input)),
            )
            .when_some(error, |this, error| this.child(error_line(&error, theme)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap(px(8.))
                    .child(ui::button(
                        "branch-cancel",
                        self.t("common.cancel"),
                        theme,
                        ButtonTone::Ghost,
                        !busy,
                        cx.listener(|app, _event, _window, cx| app.close_overlay(cx)),
                    ))
                    .child(ui::button(
                        "branch-create",
                        self.t("common.create"),
                        theme,
                        ButtonTone::Primary,
                        !busy,
                        cx.listener(|app, _event, _window, cx| app.submit_new_branch(cx)),
                    )),
            );
        self.overlay_card(theme, 430.0, body.into_any_element(), cx)
    }

    fn render_stash_dialog(&mut self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        let (input, include_untracked, busy, error, entries, loading) = {
            let dialog = self.stash_dialog.as_ref().unwrap();
            (
                dialog.input.clone(),
                dialog.include_untracked,
                dialog.busy,
                dialog.error.clone(),
                dialog.entries.clone(),
                dialog.loading,
            )
        };
        let project_name = self
            .active_project()
            .map(|project| project.name.clone())
            .unwrap_or_default();
        let body = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .px(px(18.))
            .pb(px(16.))
            .child(self.modal_header(
                self.t("stash.title"),
                Some(i18n::fill(
                    self.t("stash.description"),
                    &[("name", &project_name)],
                )),
                theme,
                cx,
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(field_label(self.t("stash.messageLabel"), theme))
                    .child(input_frame(theme).child(input)),
            )
            .child(ui::checkbox(
                "stash-untracked",
                include_untracked,
                self.t("stash.includeUntracked"),
                theme,
                cx.listener(|app, _event, _window, cx| {
                    let include = app
                        .stash_dialog
                        .as_ref()
                        .map(|dialog| dialog.include_untracked)
                        .unwrap_or(false);
                    app.set_stash_include_untracked(!include, cx);
                }),
            ))
            .when_some(error, |this, error| this.child(error_line(&error, theme)))
            .child(ui::button(
                "stash-save",
                if busy {
                    self.t("commit.committing")
                } else {
                    self.t("stash.saveAction")
                },
                theme,
                ButtonTone::Primary,
                !busy,
                cx.listener(|app, _event, _window, cx| app.save_stash(cx)),
            ))
            .child(ui::separator(theme))
            .child(field_label(self.t("stash.title"), theme))
            .child(
                div()
                    .id("stash-list")
                    .flex()
                    .flex_col()
                    .max_h(px(260.))
                    .overflow_scroll()
                    .rounded(px(8.))
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.bg)
                    .when(loading, |this| {
                        this.child(
                            div()
                                .px(px(10.))
                                .py(px(10.))
                                .text_size(px(11.0))
                                .text_color(theme.muted)
                                .child(self.t("common.loading")),
                        )
                    })
                    .when(!loading && entries.is_empty(), |this| {
                        this.child(
                            div()
                                .px(px(10.))
                                .py(px(12.))
                                .text_size(px(11.0))
                                .text_color(theme.muted)
                                .child(self.t("stash.empty")),
                        )
                    })
                    .children(entries.iter().map(|entry| {
                        let message = if entry.message.is_empty() {
                            entry.reference.clone()
                        } else {
                            entry.message.clone()
                        };
                        let branch = if entry.branch.is_empty() {
                            self.t("stash.unknownBranch").to_string()
                        } else {
                            i18n::fill(self.t("stash.onBranch"), &[("branch", &entry.branch)])
                        };
                        let index = entry.index;
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .px(px(9.))
                            .py(px(6.))
                            .border_b_1()
                            .border_color(theme.border.opacity(0.6))
                            .child(ui::icon("archive", 12.0, theme.muted))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .min_w(px(0.))
                                    .child(
                                        div()
                                            .text_size(px(11.5))
                                            .text_color(theme.text)
                                            .truncate()
                                            .child(message),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(9.5))
                                            .text_color(theme.dim)
                                            .child(branch),
                                    ),
                            )
                            .child(ui::button(
                                "stash-apply",
                                self.t("stash.apply"),
                                theme,
                                ButtonTone::Secondary,
                                true,
                                cx.listener(move |app, _event, _window, cx| {
                                    app.apply_stash(index, cx);
                                }),
                            ))
                            .child(ui::icon_button(
                                "stash-drop",
                                "trash-2",
                                theme,
                                theme.del,
                                true,
                                cx.listener(move |app, _event, _window, cx| {
                                    app.request_drop_stash(index, cx);
                                }),
                            ))
                            .into_any_element()
                    })),
            );
        self.overlay_card(theme, 560.0, body.into_any_element(), cx)
    }

    fn render_add_repo_dialog(&mut self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        let state = self.add_repo.as_ref().unwrap();
        let (input, scanning, error, results, note) = (
            state.input.clone(),
            state.scanning,
            state.error.clone(),
            state.results.clone(),
            state.manual_note.clone(),
        );
        let selected = results.iter().filter(|repo| repo.selected).count();
        let body = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .px(px(18.))
            .pb(px(16.))
            .child(self.modal_header(
                self.t("repo.addTitle"),
                Some(self.t("repo.addDescription").to_string()),
                theme,
                cx,
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(field_label(self.t("repo.pathLabel"), theme))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(input_frame(theme).child(input))
                            .child(ui::button(
                                "add-repo-browse",
                                self.t("repo.browse"),
                                theme,
                                ButtonTone::Secondary,
                                !scanning,
                                cx.listener(|app, _event, _window, cx| {
                                    app.browse_repo_folder(cx);
                                }),
                            ))
                            .child(ui::button(
                                "add-repo-inspect",
                                self.t("repo.inspectPath"),
                                theme,
                                ButtonTone::Primary,
                                !scanning,
                                cx.listener(|app, _event, _window, cx| {
                                    app.inspect_add_repo_path(cx);
                                }),
                            )),
                    ),
            )
            .when(scanning, |this| {
                this.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .text_size(px(11.0))
                        .text_color(theme.muted)
                        .child(ui::icon("loader-circle", 13.0, theme.amber))
                        .child(self.t("repo.scanning")),
                )
            })
            .when_some(note, |this, note| {
                this.child(div().text_size(px(10.5)).text_color(theme.dim).child(note))
            })
            .when_some(error, |this, error| this.child(error_line(&error, theme)))
            .when(!results.is_empty(), |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(field_label(
                                    &i18n::fill(
                                        self.t("repo.discoveryHint"),
                                        &[("count", &results.len().to_string())],
                                    ),
                                    theme,
                                ))
                                .child(ui::button(
                                    "add-repo-selected",
                                    i18n::fill(
                                        self.t("repo.addSelected"),
                                        &[("count", &selected.to_string())],
                                    ),
                                    theme,
                                    ButtonTone::Primary,
                                    selected > 0,
                                    cx.listener(|app, _event, _window, cx| {
                                        app.add_selected_repositories(cx);
                                    }),
                                )),
                        )
                        .child(
                            div()
                                .id("discovered-repos")
                                .flex()
                                .flex_col()
                                .max_h(px(260.))
                                .overflow_scroll()
                                .rounded(px(8.))
                                .border_1()
                                .border_color(theme.border)
                                .bg(theme.bg)
                                .children(results.iter().enumerate().map(|(index, repo)| {
                                    let path = repo.path.clone();
                                    div()
                                        .id(SharedString::from(format!("discovered-{index}")))
                                        .flex()
                                        .items_center()
                                        .gap(px(8.))
                                        .px(px(9.))
                                        .py(px(6.))
                                        .cursor_pointer()
                                        .border_b_1()
                                        .border_color(theme.border.opacity(0.6))
                                        .hover(move |style| style.bg(theme.surface_hover))
                                        .on_click(cx.listener(move |app, _event, _window, cx| {
                                            app.toggle_discovered_repo(index, cx);
                                        }))
                                        .child(ui::checkbox(
                                            SharedString::from(format!("discovered-check-{index}")),
                                            repo.selected,
                                            "",
                                            theme,
                                            cx.listener(move |app, _event, _window, cx| {
                                                app.toggle_discovered_repo(index, cx);
                                            }),
                                        ))
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .flex_1()
                                                .min_w(px(0.))
                                                .child(
                                                    div()
                                                        .text_size(px(11.5))
                                                        .text_color(theme.text)
                                                        .child(repo.name.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(9.5))
                                                        .text_color(theme.dim)
                                                        .truncate()
                                                        .child(path),
                                                ),
                                        )
                                        .into_any_element()
                                })),
                        ),
                )
            });
        self.overlay_card(theme, 590.0, body.into_any_element(), cx)
    }

    fn render_remote_dialog(&mut self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        let dialog = self.remote_dialog.as_ref().unwrap();
        let (name_input, url_input, remotes, editing, busy, error, loading) = (
            dialog.name_input.clone(),
            dialog.url_input.clone(),
            dialog.remotes.clone(),
            dialog.editing.clone(),
            dialog.busy,
            dialog.error.clone(),
            dialog.loading,
        );
        let project_name = self
            .active_project()
            .map(|project| project.name.clone())
            .unwrap_or_default();
        let editing_name = editing.clone();
        let body = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .px(px(18.))
            .pb(px(16.))
            .child(self.modal_header(
                self.t("remote.title"),
                Some(i18n::fill(
                    self.t("remote.description"),
                    &[("name", &project_name)],
                )),
                theme,
                cx,
            ))
            .child(field_label(self.t("remote.current"), theme))
            .child(
                div()
                    .id("remote-list")
                    .flex()
                    .flex_col()
                    .max_h(px(150.))
                    .overflow_scroll()
                    .rounded(px(8.))
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.bg)
                    .when(loading, |this| {
                        this.child(
                            div()
                                .px(px(10.))
                                .py(px(8.))
                                .text_size(px(10.5))
                                .text_color(theme.muted)
                                .child(self.t("remote.loading")),
                        )
                    })
                    .when(!loading && remotes.is_empty(), |this| {
                        this.child(
                            div()
                                .px(px(10.))
                                .py(px(10.))
                                .text_size(px(10.5))
                                .text_color(theme.muted)
                                .child(self.t("remote.empty")),
                        )
                    })
                    .children(remotes.iter().map(|remote| {
                        let name = remote.name.clone();
                        let selected = editing_name.as_deref() == Some(name.as_str());
                        div()
                            .id(SharedString::from(format!("remote-{}", remote.name)))
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .px(px(9.))
                            .py(px(6.))
                            .cursor_pointer()
                            .border_b_1()
                            .border_color(theme.border.opacity(0.6))
                            .when(selected, |this| this.bg(theme.surface_hover))
                            .hover(move |style| style.bg(theme.surface_hover))
                            .on_click(cx.listener(move |app, _event, _window, cx| {
                                app.select_remote(name.clone(), cx);
                            }))
                            .child(ui::icon("cloud-cog", 12.0, theme.blue))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .min_w(px(0.))
                                    .child(
                                        div()
                                            .text_size(px(11.5))
                                            .text_color(theme.text)
                                            .child(remote.name.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(9.5))
                                            .text_color(theme.dim)
                                            .truncate()
                                            .child(remote.url.clone()),
                                    ),
                            )
                            .when(selected, |this| {
                                this.child(ui::label(self.t("remote.editing"), 9.0, theme.amber))
                            })
                            .into_any_element()
                    })),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(field_label(self.t("remote.nameLabel"), theme))
                    .child(input_frame(theme).child(name_input)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(field_label(self.t("remote.urlLabel"), theme))
                    .child(input_frame(theme).child(url_input)),
            )
            .child(
                div()
                    .text_size(px(10.0))
                    .text_color(theme.dim)
                    .child(self.t("remote.hint")),
            )
            .when_some(error, |this, error| this.child(error_line(&error, theme)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap(px(8.))
                    .child(ui::button(
                        "remote-close",
                        self.t("common.close"),
                        theme,
                        ButtonTone::Ghost,
                        !busy,
                        cx.listener(|app, _event, _window, cx| {
                            app.remote_dialog = None;
                            app.pending_main_focus = true;
                            cx.notify();
                        }),
                    ))
                    .child(ui::button(
                        "remote-save",
                        if editing.is_some() && !editing.as_deref().unwrap_or("").is_empty() {
                            self.t("remote.update")
                        } else {
                            self.t("remote.save")
                        },
                        theme,
                        ButtonTone::Primary,
                        !busy,
                        cx.listener(|app, _event, _window, cx| app.save_remote(cx)),
                    )),
            );
        self.overlay_card(theme, 560.0, body.into_any_element(), cx)
    }

    fn render_push_dialog(&mut self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        let push = self.push.as_ref().unwrap();
        let (info, selected, force, tags, busy, loading, error) = (
            push.info.clone(),
            push.selected.clone(),
            push.force,
            push.tags,
            push.busy,
            push.loading,
            push.error.clone(),
        );
        let project_name = self
            .active_project()
            .map(|project| project.name.clone())
            .unwrap_or_default();
        let mut body = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .px(px(18.))
            .pb(px(16.))
            .child(self.modal_header(
                &i18n::fill(self.t("push.title"), &[("name", &project_name)]),
                info.as_ref().map(|info| {
                    i18n::fill(
                        self.t("push.description"),
                        &[
                            ("branch", &info.branch),
                            ("target", &format!("{}/{}", info.remote, info.remote_branch)),
                        ],
                    )
                }),
                theme,
                cx,
            ));

        if loading {
            body = body.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .text_size(px(11.5))
                    .text_color(theme.muted)
                    .child(ui::icon("loader-circle", 14.0, theme.amber))
                    .child(self.t("push.loading")),
            );
        } else if let Some(info) = info.as_ref() {
            if let Some(error) = error.clone() {
                body = body.child(error_line(&error, theme));
            }
            if info.commits.is_empty() {
                body = body
                    .child(ui::label(self.t("push.noCommits"), 12.0, theme.text_soft))
                    .child(ui::label(self.t("push.noCommitsHint"), 10.5, theme.muted));
            } else {
                body = body
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(field_label(
                                &i18n::fill(
                                    self.t("push.commitsCount"),
                                    &[("count", &info.commits.len().to_string())],
                                ),
                                theme,
                            ))
                            .child(ui::label(
                                format!(
                                    "{} · {}",
                                    i18n::fill(
                                        self.t("overview.ahead"),
                                        &[("count", &info.ahead.to_string())]
                                    ),
                                    i18n::fill(
                                        self.t("overview.behind"),
                                        &[("count", &info.behind.to_string())]
                                    )
                                ),
                                10.0,
                                theme.muted,
                            )),
                    )
                    .child(
                        div()
                            .id("push-commit-list")
                            .flex()
                            .flex_col()
                            .max_h(px(280.))
                            .overflow_scroll()
                            .rounded(px(8.))
                            .border_1()
                            .border_color(theme.border)
                            .bg(theme.bg)
                            .children(info.commits.iter().enumerate().map(|(index, commit)| {
                                let checked = selected.contains(&commit.hash);
                                let hash = commit.hash.clone();
                                div()
                                    .id(SharedString::from(format!("push-commit-{index}")))
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .px(px(9.))
                                    .py(px(6.))
                                    .cursor_pointer()
                                    .border_b_1()
                                    .border_color(theme.border.opacity(0.6))
                                    .when(checked, |this| this.bg(theme.amber_soft.opacity(0.4)))
                                    .hover(move |style| style.bg(theme.surface_hover))
                                    .on_click(cx.listener(move |app, _event, _window, cx| {
                                        app.select_push_commit(&hash, cx);
                                    }))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .size(px(15.))
                                            .rounded(px(4.))
                                            .border_1()
                                            .border_color(if checked {
                                                theme.amber
                                            } else {
                                                theme.border_strong
                                            })
                                            .bg(if checked {
                                                theme.amber
                                            } else {
                                                theme.surface_raised
                                            })
                                            .when(checked, |this| {
                                                this.child(ui::icon("check", 10.0, theme.bg_deep))
                                            }),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .flex_1()
                                            .min_w(px(0.))
                                            .child(
                                                div()
                                                    .text_size(px(11.5))
                                                    .text_color(theme.text)
                                                    .truncate()
                                                    .child(if commit.subject.is_empty() {
                                                        self.t("push.noSubject").to_string()
                                                    } else {
                                                        commit.subject.clone()
                                                    }),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap(px(6.))
                                                    .text_size(px(9.5))
                                                    .text_color(theme.dim)
                                                    .child(commit.short_hash.clone())
                                                    .child(commit.author.clone()),
                                            ),
                                    )
                                    .into_any_element()
                            })),
                    )
                    .when(info.behind > 0, |this| {
                        this.child(div().text_size(px(10.5)).text_color(theme.del).child(
                            i18n::fill(
                                self.t("push.behind"),
                                &[("count", &info.behind.to_string())],
                            ),
                        ))
                    });
            }
            body = body
                .child(ui::checkbox(
                    "push-tags",
                    tags,
                    self.t("push.tags"),
                    theme,
                    cx.listener(|app, _event, _window, cx| {
                        if let Some(push) = app.push.as_mut() {
                            push.tags = !push.tags;
                        }
                        cx.notify();
                    }),
                ))
                .child(ui::checkbox(
                    "push-force",
                    force,
                    self.t("push.forcePush"),
                    theme,
                    cx.listener(|app, _event, _window, cx| {
                        if let Some(push) = app.push.as_mut() {
                            push.force = !push.force;
                        }
                        cx.notify();
                    }),
                ))
                .when(force, |this| {
                    this.child(
                        div()
                            .text_size(px(10.5))
                            .text_color(theme.del)
                            .child(self.t("push.forceWarning")),
                    )
                });
        } else if let Some(error) = error {
            body = body.child(error_line(&error, theme));
        }

        body = body.child(
            div()
                .flex()
                .items_center()
                .justify_end()
                .gap(px(8.))
                .child(ui::button(
                    "push-cancel",
                    self.t("common.cancel"),
                    theme,
                    ButtonTone::Ghost,
                    !busy,
                    cx.listener(|app, _event, _window, cx| app.close_overlay(cx)),
                ))
                .child(ui::button(
                    "push-submit",
                    if busy {
                        self.t("push.pushing")
                    } else {
                        self.t("push.push")
                    },
                    theme,
                    ButtonTone::Primary,
                    !busy && !loading,
                    cx.listener(|app, _event, _window, cx| app.execute_push(cx)),
                )),
        );
        self.overlay_card(theme, 600.0, body.into_any_element(), cx)
    }

    fn render_workspace_dialog(&mut self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        let dialog = self.workspace_dialog.as_ref().unwrap();
        let input = dialog.input.clone();
        let error = dialog.error.clone();
        let body = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .px(px(18.))
            .pb(px(16.))
            .child(self.modal_header(
                self.t("workspace.createTitle"),
                Some(self.t("workspace.createDescription").to_string()),
                theme,
                cx,
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(field_label(self.t("workspace.nameLabel"), theme))
                    .child(input_frame(theme).child(input)),
            )
            .when_some(error, |this, error| this.child(error_line(&error, theme)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap(px(8.))
                    .child(ui::button(
                        "workspace-cancel",
                        self.t("common.cancel"),
                        theme,
                        ButtonTone::Ghost,
                        true,
                        cx.listener(|app, _event, _window, cx| app.close_overlay(cx)),
                    ))
                    .child(ui::button(
                        "workspace-create",
                        self.t("common.create"),
                        theme,
                        ButtonTone::Primary,
                        true,
                        cx.listener(|app, _event, _window, cx| app.submit_workspace_dialog(cx)),
                    )),
            );
        self.overlay_card(theme, 420.0, body.into_any_element(), cx)
    }

    fn render_confirm_dialog(
        &mut self,
        confirm: &ConfirmState,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let body = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .px(px(18.))
            .pb(px(16.))
            .child(self.modal_header(&confirm.title, None, theme, cx))
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(theme.muted)
                    .child(confirm.description.clone()),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap(px(8.))
                    .child(ui::button(
                        "confirm-cancel",
                        self.t("common.cancel"),
                        theme,
                        ButtonTone::Ghost,
                        true,
                        cx.listener(|app, _event, _window, cx| app.close_overlay(cx)),
                    ))
                    .child(ui::button(
                        "confirm-action",
                        confirm.confirm_label.clone(),
                        theme,
                        ButtonTone::Danger,
                        true,
                        cx.listener(|app, _event, _window, cx| app.confirm_action(cx)),
                    )),
            );
        self.overlay_card(theme, 430.0, body.into_any_element(), cx)
    }

    fn render_settings_dialog(&mut self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        let settings = self.settings.clone();
        let system_dark = self.system_dark;
        let body = div()
            .flex()
            .flex_col()
            .gap(px(14.))
            .px(px(18.))
            .pb(px(16.))
            .child(self.modal_header(self.t("settings.title"), None, theme, cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(field_label(self.t("settings.theme"), theme))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .child(choice_button(
                                "theme-system",
                                self.t("settings.system"),
                                settings.theme == ThemeMode::System,
                                theme,
                                cx.listener(|app, _event, _window, cx| {
                                    app.update_settings(cx, |settings| {
                                        settings.theme = ThemeMode::System;
                                    });
                                }),
                            ))
                            .child(choice_button(
                                "theme-light",
                                self.t("settings.light"),
                                settings.theme == ThemeMode::Light,
                                theme,
                                cx.listener(|app, _event, _window, cx| {
                                    app.update_settings(cx, |settings| {
                                        settings.theme = ThemeMode::Light;
                                    });
                                }),
                            ))
                            .child(choice_button(
                                "theme-dark",
                                self.t("settings.dark"),
                                settings.theme == ThemeMode::Dark,
                                theme,
                                cx.listener(|app, _event, _window, cx| {
                                    app.update_settings(cx, |settings| {
                                        settings.theme = ThemeMode::Dark;
                                    });
                                }),
                            )),
                    )
                    .child(
                        div()
                            .text_size(px(10.5))
                            .text_color(theme.dim)
                            .child(i18n::fill(
                                self.t("settings.systemDescription"),
                                &[(
                                    "theme",
                                    if system_dark {
                                        self.t("settings.dark")
                                    } else {
                                        self.t("settings.light")
                                    },
                                )],
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(field_label(self.t("settings.language"), theme))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .child(choice_button(
                                "lang-system",
                                self.t("settings.languageAuto"),
                                settings.locale == LocalePreference::System,
                                theme,
                                cx.listener(|app, _event, _window, cx| {
                                    app.update_settings(cx, |settings| {
                                        settings.locale = LocalePreference::System;
                                    });
                                }),
                            ))
                            .child(choice_button(
                                "lang-en",
                                self.t("settings.languageEnglish"),
                                settings.locale == LocalePreference::En,
                                theme,
                                cx.listener(|app, _event, _window, cx| {
                                    app.update_settings(cx, |settings| {
                                        settings.locale = LocalePreference::En;
                                    });
                                }),
                            ))
                            .child(choice_button(
                                "lang-zh",
                                self.t("settings.languageChinese"),
                                settings.locale == LocalePreference::Zh,
                                theme,
                                cx.listener(|app, _event, _window, cx| {
                                    app.update_settings(cx, |settings| {
                                        settings.locale = LocalePreference::Zh;
                                    });
                                }),
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(field_label(self.t("settings.typography"), theme))
                    .child(size_row(
                        self.t("settings.interfaceSize"),
                        settings.ui_size,
                        UI_SIZE_MIN,
                        UI_SIZE_MAX,
                        theme,
                        cx.listener(|app, _event, _window, cx| {
                            app.update_settings(cx, |settings| {
                                settings.ui_size += 1.0;
                            });
                        }),
                        cx.listener(|app, _event, _window, cx| {
                            app.update_settings(cx, |settings| {
                                settings.ui_size -= 1.0;
                            });
                        }),
                    ))
                    .child(size_row(
                        self.t("settings.codeSize"),
                        settings.code_size,
                        CODE_SIZE_MIN,
                        CODE_SIZE_MAX,
                        theme,
                        cx.listener(|app, _event, _window, cx| {
                            app.update_settings(cx, |settings| {
                                settings.code_size += 1.0;
                            });
                        }),
                        cx.listener(|app, _event, _window, cx| {
                            app.update_settings(cx, |settings| {
                                settings.code_size -= 1.0;
                            });
                        }),
                    )),
            )
            .child(ui::checkbox(
                "settings-auto-fetch",
                settings.auto_fetch,
                self.t("settings.autoFetch"),
                theme,
                cx.listener(|app, _event, _window, cx| {
                    app.update_settings(cx, |settings| {
                        settings.auto_fetch = !settings.auto_fetch;
                    });
                }),
            ))
            .child(
                div()
                    .text_size(px(10.5))
                    .text_color(theme.dim)
                    .child(self.t("settings.autoFetchDescription")),
            )
            .child(ui::separator(theme))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(ui::button(
                        "settings-reset",
                        self.t("settings.restore"),
                        theme,
                        ButtonTone::Ghost,
                        true,
                        cx.listener(|app, _event, _window, cx| {
                            let defaults = crate::model::Settings::default();
                            app.update_settings(cx, |settings| *settings = defaults);
                            app.toast(app.t("settings.resetToast"), ToastKind::Info, cx);
                        }),
                    ))
                    .child(ui::button(
                        "settings-done",
                        self.t("settings.done"),
                        theme,
                        ButtonTone::Primary,
                        true,
                        cx.listener(|app, _event, _window, cx| app.close_settings(cx)),
                    )),
            );
        let card = div()
            .size_full()
            .absolute()
            .top_0()
            .left_0()
            .flex()
            .items_start()
            .justify_center()
            .pt(px(72.))
            .bg(theme.overlay)
            .child(
                div()
                    .id("settings-card")
                    .w(px(520.))
                    .max_h(px(720.))
                    .overflow_scroll()
                    .flex()
                    .flex_col()
                    .rounded(px(12.))
                    .bg(theme.surface_raised)
                    .border_1()
                    .border_color(theme.border_strong)
                    .shadow_xl()
                    .child(body),
            );
        card.into_any_element()
    }
}

fn field_label(text: &str, theme: Theme) -> AnyElement {
    div()
        .text_size(px(10.5))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(theme.muted)
        .child(text.to_uppercase())
        .into_any_element()
}

fn error_line(text: &str, theme: Theme) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .px(px(8.))
        .py(px(6.))
        .rounded(px(6.))
        .bg(theme.del.opacity(0.08))
        .text_size(px(10.5))
        .text_color(theme.del)
        .child(ui::icon("triangle-alert", 12.0, theme.del))
        .child(text.to_string())
        .into_any_element()
}

fn choice_button(
    id: &'static str,
    text: &'static str,
    selected: bool,
    theme: Theme,
    handler: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .h(px(28.))
        .px(px(10.))
        .rounded(px(6.))
        .border_1()
        .border_color(if selected { theme.amber } else { theme.border })
        .bg(if selected {
            theme.amber_soft
        } else {
            theme.surface_raised
        })
        .cursor_pointer()
        .text_size(px(11.5))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(if selected {
            theme.amber
        } else {
            theme.text_soft
        })
        .hover(move |style| style.bg(theme.surface_hover))
        .on_click(handler)
        .child(text.to_string())
        .into_any_element()
}

fn size_row(
    label: &'static str,
    value: f32,
    min: f32,
    max: f32,
    theme: Theme,
    plus: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    minus: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> AnyElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(8.))
        .child(ui::label(label, 11.5, theme.text_soft))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .child(ui::icon_button(
                    "size-minus",
                    "minus",
                    theme,
                    theme.muted,
                    value > min,
                    minus,
                ))
                .child(
                    div()
                        .min_w(px(28.))
                        .text_size(px(11.5))
                        .text_color(theme.text)
                        .text_center()
                        .child(format!("{value:.0}")),
                )
                .child(ui::icon_button(
                    "size-plus",
                    "plus",
                    theme,
                    theme.muted,
                    value < max,
                    plus,
                )),
        )
        .into_any_element()
}

fn relative_branch_time(time: i64, lang: crate::i18n::Lang) -> String {
    if time == 0 {
        return String::new();
    }
    let elapsed = (chrono::Utc::now().timestamp() - time).max(0);
    let days = elapsed / 86_400;
    if days < 1 {
        let hours = (elapsed / 3_600).max(1);
        return i18n::fill(
            i18n::t(lang, "blame.hoursAgo"),
            &[("count", &hours.to_string())],
        );
    }
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
