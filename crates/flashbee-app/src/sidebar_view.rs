use gpui::{AnyElement, Context, SharedString, div, prelude::*, px};

use crate::app::{FlashBeeApp, MainView};
use crate::diff::{self, path_parts};
use crate::i18n;
use crate::model::{Project, ProjectGroup};
use crate::theme::Theme;
use crate::ui::{self, ButtonTone};

impl FlashBeeApp {
    pub fn render_sidebar(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme();
        let collapsed = self.sidebar_collapsed;
        let width = if collapsed {
            0.0
        } else {
            self.layout.sidebar_width
        };
        div()
            .flex()
            .flex_col()
            .flex_none()
            .h_full()
            .w(px(width))
            .overflow_hidden()
            .bg(theme.bg_deep)
            .border_r_1()
            .border_color(if collapsed {
                theme.border.opacity(0.0)
            } else {
                theme.border
            })
            .children(if collapsed {
                Vec::new()
            } else {
                vec![
                    self.render_sidebar_top(theme, cx),
                    self.render_sidebar_nav(theme, cx),
                    self.render_sidebar_repos(theme, cx),
                    self.render_sidebar_footer(theme, cx),
                ]
            })
            .into_any_element()
    }

    fn render_sidebar_top(&self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        let workspace = self.state.active();
        let label = workspace
            .map(|workspace| crate::model::workspace_label(workspace, self.lang()))
            .unwrap_or_default();
        let repo_count = workspace
            .map(|workspace| {
                workspace
                    .projects
                    .iter()
                    .filter(|project| !project.is_worktree())
                    .count()
            })
            .unwrap_or(0);
        let initial = label
            .chars()
            .next()
            .unwrap_or('W')
            .to_uppercase()
            .to_string();
        let menu_open = self.workspace_menu_open;
        let workspaces: Vec<(String, String, bool, usize)> = self
            .state
            .workspaces
            .iter()
            .map(|workspace| {
                (
                    workspace.id.clone(),
                    crate::model::workspace_label(workspace, self.lang()),
                    workspace.id == self.state.active_workspace_id,
                    workspace
                        .projects
                        .iter()
                        .filter(|project| !project.is_worktree())
                        .count(),
                )
            })
            .collect();
        let can_delete = self.state.workspaces.len() > 1;
        let labels = (
            self.t("sidebar.workspaces").to_string(),
            self.t("sidebar.newWorkspace").to_string(),
            self.t("sidebar.deleteWorkspace").to_string(),
        );
        let entity = cx.entity();

        div()
            .relative()
            .flex()
            .items_center()
            .gap(px(4.))
            .h(px(56.))
            .flex_none()
            .pl(px(10.))
            .pr(px(8.))
            .child(
                div()
                    .id("workspace-switcher")
                    .flex()
                    .flex_1()
                    .min_w(px(0.))
                    .items_center()
                    .gap(px(9.))
                    .px(px(7.))
                    .py(px(5.))
                    .rounded(px(8.))
                    .cursor_pointer()
                    .hover(move |style| style.bg(theme.surface_hover.opacity(0.7)))
                    .on_click(cx.listener(|app, _event, _window, cx| {
                        app.workspace_menu_open = !app.workspace_menu_open;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(24.))
                            .rounded(px(7.))
                            .bg(theme.amber_soft)
                            .text_size(px(11.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.amber)
                            .child(initial),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .min_w(px(0.))
                            .flex_1()
                            .child(ui::label_medium(label.clone(), 12.5, theme.text))
                            .child(ui::label(
                                if repo_count == 1 {
                                    self.t("sidebar.repoCountOne").to_string()
                                } else {
                                    i18n::fill(
                                        self.t("sidebar.repoCountMany"),
                                        &[("count", &repo_count.to_string())],
                                    )
                                },
                                10.5,
                                theme.muted,
                            )),
                    )
                    .child(ui::icon(
                        "chevron-down",
                        13.0,
                        if menu_open { theme.text } else { theme.muted },
                    )),
            )
            .child(ui::icon_button(
                "collapse-sidebar",
                "panel-left-close",
                theme,
                theme.muted,
                true,
                cx.listener(|app, _event, _window, cx| {
                    app.sidebar_collapsed = true;
                    cx.notify();
                }),
            ))
            .when(menu_open, |this| {
                this.child(
                    gpui::deferred(
                        div()
                            .absolute()
                            .top(px(50.))
                            .left(px(12.))
                            .w(px(238.))
                            .p(px(6.))
                            .rounded(px(10.))
                            .bg(theme.surface_raised)
                            .border_1()
                            .border_color(theme.border_strong)
                            .shadow_lg()
                            .child(
                                div()
                                    .px(px(8.))
                                    .py(px(4.))
                                    .text_size(px(10.5))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme.muted)
                                    .child(labels.0),
                            )
                            .children(workspaces.into_iter().map(|(id, name, active, count)| {
                                let entity = entity.clone();
                                div()
                                    .id(SharedString::from(format!("workspace-{id}")))
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .px(px(8.))
                                    .py(px(6.))
                                    .rounded(px(6.))
                                    .cursor_pointer()
                                    .when(active, |this| this.bg(theme.surface_hover))
                                    .hover(move |style| style.bg(theme.surface_hover))
                                    .on_click(move |_event, _window, cx| {
                                        entity.update(cx, |app, cx| {
                                            app.switch_workspace(&id, cx);
                                        });
                                    })
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .size(px(20.))
                                            .rounded(px(6.))
                                            .bg(theme.surface_hover)
                                            .text_size(px(10.))
                                            .text_color(theme.text_soft)
                                            .child(
                                                name.chars()
                                                    .next()
                                                    .unwrap_or('W')
                                                    .to_uppercase()
                                                    .to_string(),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .flex_1()
                                            .min_w(px(0.))
                                            .child(
                                                div()
                                                    .text_size(px(12.))
                                                    .text_color(theme.text)
                                                    .truncate()
                                                    .child(name),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(10.))
                                                    .text_color(theme.muted)
                                                    .child(if count == 1 {
                                                        self.t("sidebar.repoCountOne").to_string()
                                                    } else {
                                                        i18n::fill(
                                                            self.t("sidebar.repoCountMany"),
                                                            &[("count", &count.to_string())],
                                                        )
                                                    }),
                                            ),
                                    )
                                    .when(active, |this| {
                                        this.child(ui::icon("check", 12.0, theme.amber))
                                    })
                                    .into_any_element()
                            }))
                            .child(ui::separator(theme))
                            .child(
                                div()
                                    .id("new-workspace")
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .px(px(8.))
                                    .py(px(6.))
                                    .rounded(px(6.))
                                    .cursor_pointer()
                                    .hover(move |style| style.bg(theme.surface_hover))
                                    .on_click(cx.listener(|app, _event, _window, cx| {
                                        app.open_workspace_dialog(cx);
                                    }))
                                    .child(ui::icon("plus", 13.0, theme.muted))
                                    .child(ui::label(labels.1, 12.0, theme.text_soft)),
                            )
                            .child(
                                div()
                                    .id("delete-workspace")
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .px(px(8.))
                                    .py(px(6.))
                                    .rounded(px(6.))
                                    .opacity(if can_delete { 1.0 } else { 0.4 })
                                    .when(can_delete, |this| {
                                        this.cursor_pointer()
                                            .hover(move |style| style.bg(theme.del.opacity(0.1)))
                                            .on_click(cx.listener(|app, _event, _window, cx| {
                                                app.request_remove_workspace(cx);
                                            }))
                                    })
                                    .child(ui::icon(
                                        "trash-2",
                                        13.0,
                                        if can_delete { theme.del } else { theme.dim },
                                    ))
                                    .child(ui::label(
                                        labels.2,
                                        12.0,
                                        if can_delete { theme.del } else { theme.dim },
                                    )),
                            ),
                    )
                    .with_priority(10),
                )
            })
            .into_any_element()
    }

    fn render_sidebar_nav(&self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        let has_project = self.active_project().is_some();
        let view = self.view;
        div()
            .flex()
            .flex_col()
            .px(px(10.))
            .pb(px(6.))
            .gap(px(1.))
            .child(self.nav_row(
                "nav-overview",
                "layers",
                self.t("sidebar.overview"),
                view == MainView::Overview,
                theme,
                cx,
                |app, cx| {
                    app.open_overview(cx);
                },
            ))
            .when(has_project, |this| {
                this.child(self.nav_row(
                    "nav-history",
                    "history",
                    self.t("sidebar.history"),
                    view == MainView::History,
                    theme,
                    cx,
                    |app, cx| {
                        app.open_history(cx);
                    },
                ))
            })
            .into_any_element()
    }

    fn nav_row(
        &self,
        id: &'static str,
        icon_name: &'static str,
        text: &'static str,
        active: bool,
        theme: Theme,
        cx: &mut Context<Self>,
        action: impl Fn(&mut FlashBeeApp, &mut Context<FlashBeeApp>) + 'static,
    ) -> AnyElement {
        div()
            .id(id)
            .flex()
            .items_center()
            .gap(px(9.))
            .h(px(29.))
            .px(px(9.))
            .rounded(px(6.))
            .cursor_pointer()
            .when(active, |this| this.bg(theme.surface_hover))
            .hover(move |style| style.bg(theme.surface_hover))
            .on_click(cx.listener(move |app, _event, _window, cx| {
                action(app, cx);
            }))
            .child(ui::icon(
                icon_name,
                14.0,
                if active { theme.amber } else { theme.muted },
            ))
            .child(ui::label_medium(
                text,
                12.0,
                if active { theme.text } else { theme.text_soft },
            ))
            .into_any_element()
    }

    fn render_sidebar_repos(&mut self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        let workspace = match self.state.active() {
            Some(workspace) => workspace,
            None => return div().into_any_element(),
        };
        let projects = workspace.projects.clone();
        let groups = workspace.groups.clone();
        let root_count = projects
            .iter()
            .filter(|project| !project.is_worktree())
            .count();
        let mut sections: Vec<AnyElement> = Vec::new();
        let mut rendered_groups: Vec<String> = Vec::new();

        for project in projects.iter().filter(|project| !project.is_worktree()) {
            if let Some(group_id) = project.group_id.as_ref() {
                if rendered_groups.contains(group_id) {
                    continue;
                }
                if let Some(group) = groups.iter().find(|group| &group.id == group_id) {
                    rendered_groups.push(group_id.clone());
                    sections.push(self.render_group(group, &projects, theme, cx));
                }
                continue;
            }
            sections.push(self.render_project(project, &projects, false, theme, cx));
        }
        for group in groups.iter() {
            if rendered_groups.contains(&group.id) {
                continue;
            }
            sections.push(self.render_group(group, &projects, theme, cx));
        }
        for project in projects
            .iter()
            .filter(|project| project.is_worktree())
            .filter(|project| {
                !projects
                    .iter()
                    .any(|parent| parent.path == project.worktree_of.clone().unwrap_or_default())
            })
        {
            sections.push(self.render_project(project, &projects, true, theme, cx));
        }

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h(px(0.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .px(px(14.))
                    .pt(px(2.))
                    .pb(px(6.))
                    .child(
                        div()
                            .text_size(px(10.5))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.muted)
                            .child(self.t("sidebar.repositories")),
                    )
                    .child(
                        div()
                            .text_size(px(10.))
                            .text_color(theme.dim)
                            .child(root_count.to_string()),
                    )
                    .child(div().flex_1().h(px(1.)).bg(theme.border))
                    .child(ui::icon_button(
                        "add-repository",
                        "plus",
                        theme,
                        theme.muted,
                        true,
                        cx.listener(|app, _event, _window, cx| app.open_add_repo(cx)),
                    )),
            )
            .child(
                div()
                    .id("sidebar-repo-scroll")
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .flex_1()
                    .min_h(px(0.))
                    .overflow_scroll()
                    .px(px(8.))
                    .pb(px(8.))
                    .when(sections.is_empty(), |this| {
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(px(10.))
                                .pt(px(42.))
                                .child(ui::icon("layers", 20.0, theme.dim))
                                .child(ui::label_medium(
                                    self.t("sidebar.noRepositories"),
                                    12.0,
                                    theme.text_soft,
                                ))
                                .child(ui::button(
                                    "sidebar-add-empty",
                                    self.t("repo.add"),
                                    theme,
                                    ButtonTone::Secondary,
                                    true,
                                    cx.listener(|app, _event, _window, cx| {
                                        app.open_add_repo(cx);
                                    }),
                                )),
                        )
                    })
                    .children(sections),
            )
            .into_any_element()
    }

    fn render_group(
        &self,
        group: &ProjectGroup,
        projects: &[Project],
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let collapsed = self.folding_groups.contains(&group.id);
        let group_projects: Vec<&Project> = projects
            .iter()
            .filter(|project| project.group_id.as_deref() == Some(group.id.as_str()))
            .collect();
        let roots = group_projects
            .iter()
            .filter(|project| !project.is_worktree())
            .count();
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .id(gpui::SharedString::from(format!("group-{}", group.id)))
                    .flex()
                    .items_center()
                    .gap(px(7.))
                    .h(px(30.))
                    .px(px(6.))
                    .rounded(px(6.))
                    .cursor_pointer()
                    .hover(move |style| style.bg(theme.surface_hover.opacity(0.7)))
                    .on_click(cx.listener({
                        let group_id = group.id.clone();
                        move |app, _event, _window, cx| {
                            if !app.folding_groups.remove(&group_id) {
                                app.folding_groups.insert(group_id.clone());
                            }
                            cx.notify();
                        }
                    }))
                    .child(ui::icon("folder-git-2", 13.0, theme.muted))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .text_size(px(11.5))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme.text_soft)
                            .truncate()
                            .child(group.name.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(10.))
                            .text_color(theme.dim)
                            .child(roots.to_string()),
                    )
                    .child(ui::icon(
                        if collapsed {
                            "chevron-right"
                        } else {
                            "chevron-down"
                        },
                        12.0,
                        theme.dim,
                    )),
            )
            .when(!collapsed, |this| {
                this.children(
                    projects
                        .iter()
                        .filter(|project| {
                            project.group_id.as_deref() == Some(group.id.as_str())
                                && !project.is_worktree()
                        })
                        .map(|project| self.render_project(project, projects, false, theme, cx)),
                )
            })
            .into_any_element()
    }

    fn render_project(
        &self,
        project: &Project,
        all_projects: &[Project],
        nested: bool,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let expanded = self.expanded_project_id.as_deref() == Some(project.id.as_str());
        let active = self.active_project_id == project.id;
        let accent = ui::color_from_hex(&project.accent);
        let children: Vec<&Project> = all_projects
            .iter()
            .filter(|entry| entry.worktree_of.as_deref() == Some(project.path.as_str()))
            .collect();
        let (done, total) = self.review_counts(project);
        let visible = self.visible_files(project);
        let branch_label = project
            .worktree_head
            .as_ref()
            .map(|head| format!("@{head}"))
            .unwrap_or_else(|| project.current_branch.clone());
        let id = project.id.clone();

        let mut name_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for file in &project.files {
            let (name, _) = path_parts(&file.path);
            *name_counts.entry(name.to_string()).or_default() += 1;
        }

        let header = div()
            .id(gpui::SharedString::from(format!("project-{}", project.id)))
            .flex()
            .items_center()
            .gap(px(8.))
            .px(px(6.))
            .py(px(6.))
            .rounded(px(7.))
            .cursor_pointer()
            .when(active, |this| {
                this.bg(theme.surface_hover)
                    .border_1()
                    .border_color(theme.border)
            })
            .when(!active, |this| {
                this.border_1().border_color(theme.border.opacity(0.0))
            })
            .hover(move |style| style.bg(theme.surface_hover.opacity(0.8)))
            .on_click(cx.listener({
                let id = id.clone();
                move |app, _event, _window, cx| app.activate_project(&id, cx)
            }))
            .child(
                div()
                    .relative()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(24.))
                    .rounded(px(7.))
                    .bg(accent.opacity(0.14))
                    .border_1()
                    .border_color(accent.opacity(0.22))
                    .text_size(px(10.5))
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
                    )
                    .child(
                        div()
                            .absolute()
                            .right(px(2.))
                            .bottom(px(2.))
                            .size(px(5.))
                            .rounded_full()
                            .bg(accent),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w(px(0.))
                    .gap(px(1.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(0.))
                                    .text_size(px(12.5))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(if active { theme.text } else { theme.text_soft })
                                    .truncate()
                                    .child(project.name.clone()),
                            )
                            .when(nested, |this| {
                                this.child(ui::pill(
                                    self.t("repo.worktree"),
                                    theme.blue,
                                    theme.blue.opacity(0.1),
                                    9.0,
                                ))
                            })
                            .when(
                                project.files.is_empty()
                                    && project.conflicts.is_empty()
                                    && !project.merge_in_progress,
                                |this| {
                                    this.child(
                                        div()
                                            .text_size(px(9.5))
                                            .text_color(theme.dim)
                                            .child(self.t("common.clean")),
                                    )
                                },
                            )
                            .when(!project.conflicts.is_empty(), |this| {
                                this.child(ui::pill(
                                    format!(
                                        "{} {}",
                                        project.conflicts.len(),
                                        self.t("conflict.badge")
                                    ),
                                    theme.del,
                                    theme.del.opacity(0.12),
                                    8.5,
                                ))
                            })
                            .when(project.merge_in_progress, |this| {
                                this.child(ui::pill(
                                    self.t("conflict.mergeBadge"),
                                    theme.amber,
                                    theme.amber_soft,
                                    8.5,
                                ))
                            })
                            .when(!project.files.is_empty(), |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .min_w(px(16.))
                                        .h(px(15.))
                                        .px(px(4.))
                                        .rounded(px(4.))
                                        .bg(theme.amber_soft)
                                        .text_size(px(9.5))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(theme.amber)
                                        .child(project.files.len().to_string()),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(5.))
                            .text_size(px(10.))
                            .text_color(theme.muted)
                            .child(ui::icon("git-branch", 10.0, theme.dim))
                            .child(div().min_w(px(0.)).truncate().child(branch_label))
                            .when(project.ahead > 0, |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(1.))
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
                                        .text_color(theme.blue)
                                        .child(ui::icon("arrow-down", 9.0, theme.blue))
                                        .child(project.behind.to_string()),
                                )
                            })
                            .when(children.len() > 0, |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(2.))
                                        .child(ui::icon("git-fork", 9.0, theme.dim))
                                        .child(children.len().to_string()),
                                )
                            })
                            .when(
                                self.branch_menu_project.as_deref() == Some(project.id.as_str()),
                                |this| this,
                            ),
                    ),
            )
            .child(ui::icon(
                if expanded {
                    "chevron-down"
                } else {
                    "chevron-right"
                },
                12.0,
                theme.dim,
            ));

        let staged_files: Vec<_> = visible.iter().filter(|file| file.staged).copied().collect();
        let unstaged_files: Vec<_> = visible
            .iter()
            .filter(|file| !file.staged)
            .copied()
            .collect();

        let card = div()
            .flex()
            .flex_col()
            .when(nested, |this| this.ml(px(18.)))
            .child(header)
            .when(expanded, |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .pl(px(10.))
                        .pt(px(2.))
                        .when(!project.files.is_empty(), |this| {
                            this.child(self.render_filter_row(project, theme, cx))
                        })
                        .when(project.files.is_empty(), |this| {
                            this.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .px(px(10.))
                                    .py(px(8.))
                                    .child(
                                        div().text_size(px(11.)).text_color(theme.add).child("✓"),
                                    )
                                    .child(ui::label(
                                        self.t("sidebar.workingTreeClean"),
                                        11.0,
                                        theme.muted,
                                    )),
                            )
                        })
                        .when(!project.files.is_empty(), |this| {
                            this.when(!staged_files.is_empty(), |this| {
                                this.child(self.render_file_group(
                                    "staged",
                                    self.t("common.staged"),
                                    &staged_files,
                                    project,
                                    &name_counts,
                                    theme,
                                    cx,
                                ))
                            })
                            .when(!unstaged_files.is_empty(), |this| {
                                this.child(self.render_file_group(
                                    "unstaged",
                                    self.t("common.unstaged"),
                                    &unstaged_files,
                                    project,
                                    &name_counts,
                                    theme,
                                    cx,
                                ))
                            })
                            .when(visible.is_empty(), |this| {
                                this.child(
                                    div()
                                        .px(px(10.))
                                        .py(px(8.))
                                        .text_size(px(10.5))
                                        .text_color(theme.dim)
                                        .child(self.t("sidebar.noMatchingFiles")),
                                )
                            })
                            .when(total > 0, |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.))
                                        .px(px(10.))
                                        .pt(px(6.))
                                        .pb(px(2.))
                                        .child(
                                            div()
                                                .flex_1()
                                                .h(px(2.))
                                                .rounded_full()
                                                .bg(theme.border)
                                                .child(
                                                    div()
                                                        .h_full()
                                                        .rounded_full()
                                                        .w(px(48.0 * (done as f32 / total as f32)))
                                                        .bg(theme.amber),
                                                ),
                                        )
                                        .child(
                                            div().text_size(px(9.5)).text_color(theme.dim).child(
                                                i18n::fill(
                                                    self.t("review.progress"),
                                                    &[
                                                        ("done", &done.to_string()),
                                                        ("total", &total.to_string()),
                                                    ],
                                                ),
                                            ),
                                        ),
                                )
                            })
                        }),
                )
            })
            .children(
                children
                    .iter()
                    .map(|child| self.render_project(child, all_projects, true, theme, cx))
                    .collect::<Vec<_>>(),
            );

        card.into_any_element()
    }

    fn render_filter_row(
        &self,
        _project: &Project,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let chip = |id: &'static str,
                    icon_name: &'static str,
                    active: bool,
                    tone: gpui::Hsla,
                    handler: fn(&mut FlashBeeApp, &mut Context<FlashBeeApp>)| {
            div()
                .id(id)
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .size(px(22.))
                .rounded(px(5.))
                .cursor_pointer()
                .when(active, |this| this.bg(tone.opacity(0.14)))
                .hover(move |style| style.bg(theme.surface_hover))
                .on_click(cx.listener(move |app, _event, _window, cx| handler(app, cx)))
                .child(ui::icon(
                    icon_name,
                    11.0,
                    if active { tone } else { theme.dim },
                ))
                .into_any_element()
        };

        div()
            .flex()
            .items_center()
            .gap(px(4.))
            .pr(px(6.))
            .pb(px(3.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .flex_1()
                    .min_w(px(0.))
                    .h(px(22.))
                    .px(px(6.))
                    .rounded(px(5.))
                    .bg(theme.bg)
                    .border_1()
                    .border_color(theme.border)
                    .child(ui::icon("search", 10.0, theme.dim))
                    .child(self.filter_input.clone()),
            )
            .when(!self.file_filter.is_empty(), |this| {
                this.child(
                    div()
                        .id("clear-file-filter")
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(20.))
                        .rounded(px(5.))
                        .cursor_pointer()
                        .hover(move |style| style.bg(theme.surface_hover))
                        .on_click(cx.listener(|app, _event, window, cx| {
                            app.filter_input.update(cx, |input, cx| input.clear(cx));
                            app.file_filter.clear();
                            app.file_filter_focus =
                                window.focused(cx).unwrap_or_else(|| app.main_focus.clone());
                            cx.notify();
                        }))
                        .child(ui::icon("x", 10.0, theme.muted)),
                )
            })
            .child(chip(
                "filter-unreviewed",
                "eye-off",
                self.filter_unreviewed,
                theme.amber,
                |app, cx| {
                    app.filter_unreviewed = !app.filter_unreviewed;
                    cx.notify();
                },
            ))
            .child(chip(
                "filter-generated",
                "file-code-2",
                self.hide_generated,
                theme.blue,
                |app, cx| {
                    app.hide_generated = !app.hide_generated;
                    cx.notify();
                },
            ))
            .child(chip(
                "filter-whitespace",
                "text-search",
                self.hide_whitespace,
                theme.violet,
                |app, cx| {
                    app.hide_whitespace = !app.hide_whitespace;
                    cx.notify();
                },
            ))
            .into_any_element()
    }

    fn render_file_group(
        &self,
        _id: &'static str,
        label: &'static str,
        files: &[&flashbee_git::RepositoryFile],
        project: &Project,
        name_counts: &std::collections::HashMap<String, usize>,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let project_id = project.id.clone();
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .px(px(10.))
                    .pt(px(6.))
                    .pb(px(2.))
                    .child(
                        div()
                            .text_size(px(9.5))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.dim)
                            .child(label.to_uppercase()),
                    )
                    .child(
                        div()
                            .text_size(px(9.))
                            .text_color(theme.dim)
                            .child(files.len().to_string()),
                    ),
            )
            .children(files.iter().map(|file| {
                let project_id = project_id.clone();
                let (name, _) = path_parts(&file.path);
                let display_name = if name_counts.get(name).copied().unwrap_or(0) > 1 {
                    file.path.clone()
                } else {
                    name.to_string()
                };
                let stats = diff::file_stats(file);
                let selected = self
                    .selected_by_project
                    .get(&project_id)
                    .is_some_and(|id| id == &file.id)
                    && self.active_project_id == project_id;
                let reviewed = self.reviewed(project, file);
                let file_id = file.id.clone();
                let status_color = theme.status(file.status);
                let entity = cx.entity();
                div()
                    .id(SharedString::from(format!(
                        "file-{}-{}",
                        project_id, file.id
                    )))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .h(px(24.))
                    .px(px(8.))
                    .rounded(px(5.))
                    .cursor_pointer()
                    .when(selected, |this| {
                        this.bg(theme.surface_raised)
                            .border_1()
                            .border_color(theme.border)
                    })
                    .hover(move |style| style.bg(theme.surface_hover.opacity(0.75)))
                    .on_click(move |_event, _window, cx| {
                        entity.update(cx, |app, cx| {
                            app.select_file(&project_id, &file_id, cx);
                        });
                    })
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(14.))
                            .rounded(px(4.))
                            .bg(status_color.opacity(0.12))
                            .text_size(px(9.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(status_color)
                            .child(diff::status_letter(file.status)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .text_size(px(11.5))
                            .text_color(if selected {
                                theme.text
                            } else {
                                theme.text_soft
                            })
                            .truncate()
                            .child(display_name),
                    )
                    .when(reviewed, |this| {
                        this.child(ui::icon("check", 10.0, theme.add))
                    })
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(3.))
                            .text_size(px(9.5))
                            .child(
                                div()
                                    .text_color(theme.add)
                                    .child(format!("+{}", stats.additions)),
                            )
                            .child(
                                div()
                                    .text_color(theme.del)
                                    .child(format!("−{}", stats.deletions)),
                            ),
                    )
                    .into_any_element()
            }))
            .into_any_element()
    }

    fn render_sidebar_footer(&self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        let project = self.active_project();
        let (done, total) = project
            .map(|project| self.review_counts(project))
            .unwrap_or((0, 0));
        let has_remotes = self.state.active().is_some_and(|workspace| {
            workspace
                .projects
                .iter()
                .any(|project| !project.remotes.is_empty())
        });
        div()
            .flex()
            .flex_col()
            .flex_none()
            .gap(px(1.))
            .px(px(10.))
            .py(px(8.))
            .border_t_1()
            .border_color(theme.border)
            .child(
                div()
                    .id("quick-open-footer")
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(28.))
                    .px(px(8.))
                    .rounded(px(6.))
                    .cursor_pointer()
                    .hover(move |style| style.bg(theme.surface_hover))
                    .on_click(cx.listener(|app, _event, _window, cx| {
                        app.open_search(cx);
                    }))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(ui::icon("command", 12.0, theme.muted))
                            .child(ui::label(
                                self.t("sidebar.quickOpen"),
                                11.5,
                                theme.text_soft,
                            )),
                    )
                    .child(ui::kbd("⌘ K", theme)),
            )
            .child(
                div()
                    .id("fetch-all-footer")
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(28.))
                    .px(px(8.))
                    .rounded(px(6.))
                    .cursor_pointer()
                    .hover(move |style| style.bg(theme.surface_hover))
                    .on_click(cx.listener(|app, _event, _window, cx| {
                        app.fetch_all(false, cx);
                    }))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(ui::icon(
                                "cloud-download",
                                12.0,
                                if has_remotes { theme.muted } else { theme.dim },
                            ))
                            .child(ui::label(
                                self.t("sidebar.fetchAll"),
                                11.5,
                                if has_remotes {
                                    theme.text_soft
                                } else {
                                    theme.dim
                                },
                            )),
                    )
                    .when(total > 0, |this| {
                        this.child(ui::label(format!("{done}/{total}"), 10.0, theme.dim))
                    })
                    .when(total == 0, |this| {
                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(5.))
                                .child(div().size(px(5.)).rounded_full().bg(theme.add))
                                .child(ui::label(self.t("sidebar.localGit"), 9.5, theme.dim)),
                        )
                    }),
            )
            .into_any_element()
    }
}
