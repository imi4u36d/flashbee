use gpui::{AnyElement, Context, SharedString, div, prelude::*, px};

use crate::app::FlashBeeApp;
use crate::diff;
use crate::input::input_frame;
use crate::theme::Theme;
use crate::ui;

impl FlashBeeApp {
    pub fn render_search_overlay(&mut self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        let Some(search) = self.search.as_ref() else {
            return div().into_any_element();
        };
        let input = search.input.clone();
        let query = search.query.clone();
        let normalized = query.to_lowercase();
        let busy = search.busy;
        let commits = search.results.commits.clone();
        let code = search.results.code.clone();

        let workspace = self.state.active();
        let projects: Vec<_> = workspace
            .map(|workspace| workspace.projects.clone())
            .unwrap_or_default();
        let repo_matches: Vec<_> = projects
            .iter()
            .filter(|project| {
                normalized.is_empty()
                    || project.name.to_lowercase().contains(&normalized)
                    || project.path.to_lowercase().contains(&normalized)
                    || project.current_branch.to_lowercase().contains(&normalized)
            })
            .take(6)
            .cloned()
            .collect();
        let mut file_matches: Vec<(String, String, String, flashbee_git::ChangeStatus)> =
            Vec::new();
        if !normalized.is_empty() {
            'outer: for project in &projects {
                for file in &project.files {
                    if file.path.to_lowercase().contains(&normalized) {
                        file_matches.push((
                            project.id.clone(),
                            project.name.clone(),
                            file.path.clone(),
                            file.status,
                        ));
                        if file_matches.len() >= 14 {
                            break 'outer;
                        }
                    }
                }
            }
        }

        let mut body = div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .px(px(10.))
            .pb(px(12.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .px(px(8.))
                    .child(ui::icon("search", 15.0, theme.muted))
                    .child(input_frame(theme).child(input))
                    .child(ui::kbd("esc", theme)),
            );

        if query.trim().is_empty() {
            body = body.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .px(px(10.))
                    .py(px(14.))
                    .text_size(px(11.0))
                    .text_color(theme.muted)
                    .child(ui::icon("sparkles", 13.0, theme.dim))
                    .child(self.t("search.empty")),
            );
        } else {
            if !repo_matches.is_empty() {
                body = body.child(section_label(self.t("search.repositories"), theme));
                body = body.child(div().flex().flex_col().children(repo_matches.iter().map(
                    |project| {
                        let project_id = project.id.clone();
                        let entity = cx.entity();
                        let accent = ui::color_from_hex(&project.accent);
                        div()
                            .id(SharedString::from(format!("search-repo-{}", project.id)))
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .px(px(10.))
                            .py(px(6.))
                            .rounded(px(6.))
                            .cursor_pointer()
                            .hover(move |style| style.bg(theme.surface_hover))
                            .on_click(move |_event, _window, cx| {
                                entity.update(cx, |app, cx| {
                                    app.search = None;
                                    app.pending_main_focus = true;
                                    app.activate_project(&project_id, cx);
                                });
                            })
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .size(px(20.))
                                    .rounded(px(5.))
                                    .bg(accent.opacity(0.14))
                                    .text_size(px(9.5))
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
                                    .flex_1()
                                    .min_w(px(0.))
                                    .child(
                                        div()
                                            .text_size(px(11.5))
                                            .text_color(theme.text)
                                            .child(project.name.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(9.5))
                                            .text_color(theme.dim)
                                            .truncate()
                                            .child(format!(
                                                "{} · {}",
                                                project.current_branch, project.path
                                            )),
                                    ),
                            )
                            .child(ui::icon("corner-down-left", 11.0, theme.dim))
                            .into_any_element()
                    },
                )));
            }

            if !file_matches.is_empty() {
                body = body.child(section_label(self.t("search.changedFiles"), theme));
                body = body.child(div().flex().flex_col().children(file_matches.iter().map(
                    |(project_id, project_name, path, status)| {
                        let project_id = project_id.clone();
                        let path = path.clone();
                        let path_for_click = path.clone();
                        let entity = cx.entity();
                        div()
                            .id(SharedString::from(format!(
                                "search-file-{}-{}",
                                project_id, path
                            )))
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .px(px(10.))
                            .py(px(5.))
                            .rounded(px(6.))
                            .cursor_pointer()
                            .hover(move |style| style.bg(theme.surface_hover))
                            .on_click(move |_event, _window, cx| {
                                entity.update(cx, |app, cx| {
                                    if let Some(file) = app
                                        .file(&project_id, &format!("unstaged:{path_for_click}"))
                                        .or_else(|| {
                                            app.file(
                                                &project_id,
                                                &format!("staged:{path_for_click}"),
                                            )
                                        })
                                        .or_else(|| {
                                            app.file(
                                                &project_id,
                                                &format!("untracked:{path_for_click}"),
                                            )
                                        })
                                        .map(|file| file.id.clone())
                                    {
                                        app.search = None;
                                        app.pending_main_focus = true;
                                        app.select_file(&project_id, &file, cx);
                                    }
                                });
                            })
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .size(px(13.))
                                    .rounded(px(3.))
                                    .bg(theme.status(*status).opacity(0.12))
                                    .text_size(px(8.5))
                                    .text_color(theme.status(*status))
                                    .child(diff::status_letter(*status)),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(0.))
                                    .truncate()
                                    .text_size(px(11.0))
                                    .text_color(theme.text_soft)
                                    .child(path),
                            )
                            .child(
                                div()
                                    .text_size(px(9.0))
                                    .text_color(theme.dim)
                                    .child(project_name.clone()),
                            )
                            .into_any_element()
                    },
                )));
            }

            if !commits.is_empty() {
                body = body.child(section_label(self.t("search.commits"), theme));
                body = body.child(div().flex().flex_col().children(commits.iter().map(
                    |(project_id, project_name, commit)| {
                        let project_id = project_id.clone();
                        let hash = commit.hash.clone();
                        let entity = cx.entity();
                        div()
                            .id(SharedString::from(format!("search-commit-{}", commit.hash)))
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .px(px(10.))
                            .py(px(5.))
                            .rounded(px(6.))
                            .cursor_pointer()
                            .hover(move |style| style.bg(theme.surface_hover))
                            .on_click(move |_event, _window, cx| {
                                entity.update(cx, |app, cx| {
                                    app.search = None;
                                    app.pending_main_focus = true;
                                    app.activate_project(&project_id, cx);
                                    app.open_history(cx);
                                    app.load_commit_detail(project_id.clone(), hash.clone(), cx);
                                });
                            })
                            .child(ui::icon("git-commit-horizontal", 12.0, theme.muted))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .min_w(px(0.))
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(theme.text)
                                            .truncate()
                                            .child(commit.subject.clone()),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(6.))
                                            .text_size(px(9.0))
                                            .text_color(theme.dim)
                                            .child(commit.short_hash.clone())
                                            .child(project_name.clone()),
                                    ),
                            )
                            .into_any_element()
                    },
                )));
            }

            if !code.is_empty() {
                body = body.child(section_label(self.t("search.code"), theme));
                body = body.child(div().flex().flex_col().children(code.iter().map(
                    |(project_id, path, line, preview)| {
                        let project_id = project_id.clone();
                        let path = path.clone();
                        let path_for_click = path.clone();
                        let line = *line;
                        let preview = preview.clone();
                        let entity = cx.entity();
                        div()
                            .id(SharedString::from(format!("search-code-{}:{}", path, line)))
                            .flex()
                            .items_start()
                            .gap(px(8.))
                            .px(px(10.))
                            .py(px(5.))
                            .rounded(px(6.))
                            .cursor_pointer()
                            .hover(move |style| style.bg(theme.surface_hover))
                            .on_click(move |_event, _window, cx| {
                                entity.update(cx, |app, cx| {
                                    app.search = None;
                                    app.pending_main_focus = true;
                                    app.activate_project(&project_id, cx);
                                    app.open_history_with_file(path_for_click.clone(), cx);
                                });
                            })
                            .child(ui::icon("file-code-2", 12.0, theme.blue))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .min_w(px(0.))
                                    .child(
                                        div()
                                            .text_size(px(10.5))
                                            .text_color(theme.text_soft)
                                            .truncate()
                                            .child(path),
                                    )
                                    .when(!preview.is_empty(), |this| {
                                        this.child(
                                            div()
                                                .text_size(px(9.5))
                                                .text_color(theme.dim)
                                                .truncate()
                                                .child(format!("{line}: {preview}")),
                                        )
                                    }),
                            )
                            .into_any_element()
                    },
                )));
            }

            if busy && commits.is_empty() && code.is_empty() && file_matches.is_empty() {
                body = body.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .px(px(10.))
                        .py(px(12.))
                        .text_size(px(10.5))
                        .text_color(theme.muted)
                        .child(ui::icon("loader-circle", 12.0, theme.amber))
                        .child(self.t("search.searching")),
                );
            }

            if !busy
                && repo_matches.is_empty()
                && file_matches.is_empty()
                && commits.is_empty()
                && code.is_empty()
            {
                body = body.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.))
                        .px(px(10.))
                        .py(px(14.))
                        .child(ui::label(self.t("search.noMatches"), 11.5, theme.text_soft))
                        .child(ui::label(self.t("search.noMatchesHint"), 10.0, theme.dim)),
                );
            }
        }

        div()
            .size_full()
            .absolute()
            .top_0()
            .left_0()
            .flex()
            .items_start()
            .justify_center()
            .pt(px(78.))
            .bg(theme.overlay)
            .child(
                div()
                    .w(px(640.))
                    .max_h(px(600.))
                    .flex()
                    .flex_col()
                    .rounded(px(12.))
                    .bg(theme.surface_raised)
                    .border_1()
                    .border_color(theme.border_strong)
                    .shadow_xl()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px(px(16.))
                            .pt(px(12.))
                            .pb(px(6.))
                            .child(
                                div()
                                    .text_size(px(10.0))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.dim)
                                    .child(self.t("search.title").to_uppercase()),
                            )
                            .child(
                                div()
                                    .text_size(px(9.5))
                                    .text_color(theme.dim)
                                    .child(self.t("search.allRepositories")),
                            ),
                    )
                    .child(
                        div()
                            .id("search-results")
                            .flex()
                            .flex_col()
                            .overflow_scroll()
                            .max_h(px(520.))
                            .child(body),
                    ),
            )
            .into_any_element()
    }
}

fn section_label(text: &str, theme: Theme) -> AnyElement {
    div()
        .px(px(10.))
        .pt(px(8.))
        .pb(px(2.))
        .text_size(px(9.0))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(theme.dim)
        .child(text.to_uppercase())
        .into_any_element()
}
