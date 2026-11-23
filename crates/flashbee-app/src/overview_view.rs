use gpui::{AnyElement, Context, SharedString, div, prelude::*, px};

use crate::app::{FlashBeeApp, MainView};
use crate::i18n;
use crate::model::Project;
use crate::ui::{self, ButtonTone};

impl FlashBeeApp {
    pub fn render_overview(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme();
        let Some(workspace) = self.state.active() else {
            return div().into_any_element();
        };
        let workspace_name = crate::model::workspace_label(workspace, self.lang());
        let mut projects: Vec<Project> = workspace.projects.clone();
        projects.sort_by(|left, right| {
            attention_score(right)
                .cmp(&attention_score(left))
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
        });
        let repo_count = projects
            .iter()
            .filter(|project| !project.is_worktree())
            .count();
        let attention = projects
            .iter()
            .filter(|project| attention_score(project) > 0)
            .count();
        let entity = cx.entity();

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h(px(0.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .h(px(56.))
                    .flex_none()
                    .px(px(20.))
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w(px(0.))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(7.))
                                    .child(
                                        div()
                                            .text_size(px(15.0))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.text)
                                            .child(self.t("overview.title")),
                                    )
                                    .child(ui::pill(
                                        workspace_name,
                                        theme.amber,
                                        theme.amber_soft,
                                        9.5,
                                    )),
                            )
                            .child(div().text_size(px(10.5)).text_color(theme.muted).child(
                                i18n::fill(
                                    self.t("overview.description"),
                                    &[("count", &repo_count.to_string())],
                                ),
                            )),
                    )
                    .child(ui::button(
                        "overview-refresh",
                        self.t("overview.refreshAll"),
                        theme,
                        ButtonTone::Secondary,
                        true,
                        cx.listener(|app, _event, _window, cx| app.refresh_all(cx)),
                    ))
                    .child(ui::button(
                        "overview-fetch",
                        self.t("overview.fetchAll"),
                        theme,
                        ButtonTone::Secondary,
                        true,
                        cx.listener(|app, _event, _window, cx| app.fetch_all(false, cx)),
                    ))
                    .child(ui::icon_button(
                        "overview-close",
                        "x",
                        theme,
                        theme.muted,
                        true,
                        cx.listener(|app, _event, _window, cx| app.close_overview(cx)),
                    )),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(7.))
                    .px(px(20.))
                    .py(px(10.))
                    .child(ui::icon(
                        if attention > 0 {
                            "triangle-alert"
                        } else {
                            "check-circle-2"
                        },
                        13.0,
                        if attention > 0 {
                            theme.amber
                        } else {
                            theme.add
                        },
                    ))
                    .child(ui::label(
                        if attention > 0 {
                            i18n::fill(self.t("overview.attention"), &[])
                        } else {
                            self.t("overview.allClear").to_string()
                        },
                        11.5,
                        if attention > 0 {
                            theme.amber
                        } else {
                            theme.add
                        },
                    ))
                    .child(div().text_size(px(10.0)).text_color(theme.dim).child(
                        if attention > 0 {
                            format!("· {attention}")
                        } else {
                            self.t("overview.allClearHint").to_string()
                        },
                    )),
            )
            .child(
                div()
                    .id("overview-scroll")
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .flex_1()
                    .min_h(px(0.))
                    .overflow_scroll()
                    .px(px(20.))
                    .pb(px(20.))
                    .children(projects.iter().map(|project| {
                        let mut badges: Vec<AnyElement> = Vec::new();
                        if !project.conflicts.is_empty() {
                            badges.push(ui::pill(
                                i18n::fill(
                                    self.t("overview.conflicts"),
                                    &[("count", &project.conflicts.len().to_string())],
                                ),
                                theme.del,
                                theme.del.opacity(0.1),
                                9.0,
                            ));
                        }
                        if project.merge_in_progress {
                            badges.push(ui::pill(
                                self.t("overview.merge"),
                                theme.amber,
                                theme.amber_soft,
                                9.0,
                            ));
                        }
                        let staged = project.files.iter().filter(|file| file.staged).count();
                        let dirty = project.files.len();
                        if dirty > 0 {
                            badges.push(ui::pill(
                                i18n::fill(
                                    self.t("overview.dirty"),
                                    &[("count", &dirty.to_string())],
                                ),
                                theme.text_soft,
                                theme.surface_hover,
                                9.0,
                            ));
                        }
                        if staged > 0 {
                            badges.push(ui::pill(
                                i18n::fill(
                                    self.t("overview.staged"),
                                    &[("count", &staged.to_string())],
                                ),
                                theme.add,
                                theme.add.opacity(0.1),
                                9.0,
                            ));
                        }
                        if project.ahead > 0 {
                            badges.push(ui::pill(
                                i18n::fill(
                                    self.t("overview.ahead"),
                                    &[("count", &project.ahead.to_string())],
                                ),
                                theme.add,
                                theme.add.opacity(0.08),
                                9.0,
                            ));
                        }
                        if project.behind > 0 {
                            badges.push(ui::pill(
                                i18n::fill(
                                    self.t("overview.behind"),
                                    &[("count", &project.behind.to_string())],
                                ),
                                theme.blue,
                                theme.blue.opacity(0.08),
                                9.0,
                            ));
                        }
                        if dirty == 0 && project.conflicts.is_empty() && !project.merge_in_progress
                        {
                            badges.push(ui::pill(
                                self.t("overview.clean"),
                                theme.add,
                                theme.add.opacity(0.08),
                                9.0,
                            ));
                        }
                        if project.current_branch == "HEAD" {
                            badges.push(ui::pill(
                                self.t("overview.detached"),
                                theme.violet,
                                theme.violet.opacity(0.1),
                                9.0,
                            ));
                        }
                        let (done, total) = self.review_counts(project);
                        let accent = ui::color_from_hex(&project.accent);
                        let project_id = project.id.clone();
                        let entity = entity.clone();
                        div()
                            .id(SharedString::from(format!("overview-{}", project.id)))
                            .flex()
                            .items_center()
                            .gap(px(12.))
                            .px(px(12.))
                            .py(px(10.))
                            .rounded(px(9.))
                            .border_1()
                            .border_color(theme.border.opacity(0.7))
                            .bg(theme.surface_raised.opacity(0.5))
                            .cursor_pointer()
                            .hover(move |style| {
                                style
                                    .bg(theme.surface_hover)
                                    .border_color(theme.border_strong)
                            })
                            .on_click(move |_event, _window, cx| {
                                entity.update(cx, |app, cx| {
                                    app.view = MainView::Changes;
                                    app.activate_project(&project_id, cx);
                                });
                            })
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .size(px(30.))
                                    .rounded(px(8.))
                                    .bg(accent.opacity(0.14))
                                    .border_1()
                                    .border_color(accent.opacity(0.22))
                                    .text_size(px(12.0))
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
                                    .gap(px(3.))
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
                                                    .child(project.name.clone()),
                                            )
                                            .when(project.is_worktree(), |this| {
                                                this.child(ui::pill(
                                                    self.t("repo.worktree"),
                                                    theme.blue,
                                                    theme.blue.opacity(0.1),
                                                    8.5,
                                                ))
                                            })
                                            .child(
                                                div()
                                                    .text_size(px(9.5))
                                                    .text_color(theme.dim)
                                                    .truncate()
                                                    .child(project.path.clone()),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(5.))
                                            .flex_wrap()
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap(px(3.))
                                                    .text_size(px(9.5))
                                                    .text_color(theme.muted)
                                                    .child(ui::icon("git-branch", 9.0, theme.dim))
                                                    .child(project.current_branch.clone()),
                                            )
                                            .children(badges),
                                    ),
                            )
                            .when(total > 0, |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(5.))
                                        .child(
                                            div()
                                                .w(px(44.))
                                                .h(px(3.))
                                                .rounded_full()
                                                .bg(theme.border)
                                                .child(
                                                    div()
                                                        .h_full()
                                                        .rounded_full()
                                                        .w(px(44.0 * done as f32 / total as f32))
                                                        .bg(theme.amber),
                                                ),
                                        )
                                        .child(ui::label(
                                            format!("{done}/{total}"),
                                            9.0,
                                            theme.dim,
                                        )),
                                )
                            })
                            .child(ui::icon("chevron-right", 13.0, theme.dim))
                            .into_any_element()
                    })),
            )
            .into_any_element()
    }
}

fn attention_score(project: &Project) -> u32 {
    let mut score = 0;
    if !project.conflicts.is_empty() {
        score += 1000 + project.conflicts.len() as u32;
    }
    if project.merge_in_progress {
        score += 800;
    }
    if !project.files.is_empty() {
        score += 500 + project.files.len() as u32;
    }
    if project.behind > 0 {
        score += 300;
    }
    if project.ahead > 0 {
        score += 200;
    }
    if project.current_branch == "HEAD" {
        score += 100;
    }
    score
}
