use flashbee_git::{LineType, RepositoryFile};
use gpui::{AnyElement, Context, SharedString, div, prelude::*, px};

use crate::app::{DiffMode, FlashBeeApp};
use crate::diff::{self, path_parts};
use crate::highlight;
use crate::i18n;
use crate::input::input_frame;
use crate::theme::Theme;
use crate::ui::{self, ButtonTone};

impl FlashBeeApp {
    pub fn render_history(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme();
        let Some(project) = self.active_project().cloned() else {
            return div().into_any_element();
        };
        let Some(history) = self.history.as_ref() else {
            return div().into_any_element();
        };
        let log = history.log.clone();
        let selected = history.selected.clone();
        let detail = history.detail.clone();
        let detail_file = history.detail_file.clone();
        let busy = history.busy;
        let loading_detail = history.loading_detail;
        let error = history.error.clone();
        let file_filter = history.file_filter.clone();
        let has_more = history.has_more;
        let show_compare = history.show_compare;
        let compare = history.compare.clone();
        let compare_file = history.compare_file.clone();
        let compare_busy = history.compare_busy;
        let base_input = history.compare_base.clone();
        let target_input = history.compare_target.clone();

        let header = div()
            .flex()
            .items_center()
            .gap(px(10.))
            .h(px(52.))
            .flex_none()
            .px(px(16.))
            .border_b_1()
            .border_color(theme.border)
            .child(ui::icon_button(
                "history-back",
                "arrow-left",
                theme,
                theme.muted,
                true,
                cx.listener(|app, _event, _window, cx| app.close_history(cx)),
            ))
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
                                    .text_size(px(14.5))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.text)
                                    .child(self.t("history.title")),
                            )
                            .child(ui::pill(
                                project.name.clone(),
                                theme.amber,
                                theme.amber_soft,
                                9.0,
                            )),
                    )
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(theme.dim)
                            .child(i18n::fill(
                                self.t("history.description"),
                                &[("name", &project.name)],
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .child(
                        div()
                            .id("history-all-files")
                            .flex()
                            .items_center()
                            .h(px(26.))
                            .px(px(8.))
                            .rounded(px(6.))
                            .cursor_pointer()
                            .when(file_filter.is_none(), |this| this.bg(theme.amber_soft))
                            .hover(move |style| style.bg(theme.surface_hover))
                            .on_click(cx.listener(|app, _event, _window, cx| {
                                app.select_history_file_filter(None, cx);
                            }))
                            .child(ui::label(
                                self.t("history.all"),
                                11.0,
                                if file_filter.is_none() {
                                    theme.amber
                                } else {
                                    theme.text_soft
                                },
                            )),
                    )
                    .when_some(file_filter.clone(), |this, filter| {
                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(4.))
                                .h(px(26.))
                                .px(px(8.))
                                .rounded(px(6.))
                                .bg(theme.surface_hover)
                                .child(ui::icon("file-code-2", 11.0, theme.blue))
                                .child(
                                    div()
                                        .max_w(px(180.))
                                        .truncate()
                                        .text_size(px(10.5))
                                        .text_color(theme.text_soft)
                                        .child(filter),
                                )
                                .child(ui::icon_button(
                                    "clear-file-filter",
                                    "x",
                                    theme,
                                    theme.muted,
                                    true,
                                    cx.listener(|app, _event, _window, cx| {
                                        app.select_history_file_filter(None, cx);
                                    }),
                                )),
                        )
                    })
                    .child(ui::button(
                        "history-compare",
                        self.t("history.compareAction"),
                        theme,
                        if show_compare {
                            ButtonTone::Primary
                        } else {
                            ButtonTone::Secondary
                        },
                        true,
                        cx.listener(|app, _event, _window, cx| {
                            if let Some(history) = app.history.as_mut() {
                                history.show_compare = !history.show_compare;
                            }
                            cx.notify();
                        }),
                    )),
            );

        let body: AnyElement = if show_compare {
            let files = compare
                .as_ref()
                .map(|comparison| comparison.files.clone())
                .unwrap_or_default();
            let selected_file = compare_file
                .as_ref()
                .and_then(|id| files.iter().find(|file| &file.id == id))
                .or_else(|| files.first());
            let file_list = div()
                .flex()
                .flex_col()
                .w(px(250.))
                .h_full()
                .border_r_1()
                .border_color(theme.border)
                .bg(theme.bg_deep)
                .child(
                    div()
                        .px(px(12.))
                        .py(px(8.))
                        .text_size(px(9.5))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme.dim)
                        .child(format!("{} {}", files.len(), self.t("common.files"))),
                )
                .child(
                    div()
                        .id("compare-file-list")
                        .flex()
                        .flex_col()
                        .px(px(8.))
                        .gap(px(2.))
                        .overflow_scroll()
                        .when(files.is_empty(), |this| {
                            this.child(
                                div()
                                    .px(px(6.))
                                    .py(px(8.))
                                    .text_size(px(10.5))
                                    .text_color(theme.muted)
                                    .child(self.t("history.empty")),
                            )
                        })
                        .children(files.iter().map(|file| {
                            let id = file.id.clone();
                            let selected = selected_file.is_some_and(|entry| entry.id == file.id);
                            div()
                                .id(SharedString::from(format!("compare-file-{}", file.id)))
                                .flex()
                                .items_center()
                                .gap(px(6.))
                                .h(px(24.))
                                .px(px(7.))
                                .rounded(px(5.))
                                .cursor_pointer()
                                .when(selected, |this| this.bg(theme.surface_hover))
                                .hover(move |style| style.bg(theme.surface_hover))
                                .on_click(cx.listener(move |app, _event, _window, cx| {
                                    if let Some(history) = app.history.as_mut() {
                                        history.compare_file = Some(id.clone());
                                    }
                                    cx.notify();
                                }))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .size(px(13.))
                                        .rounded(px(3.))
                                        .bg(theme.status(file.status).opacity(0.12))
                                        .text_size(px(8.5))
                                        .text_color(theme.status(file.status))
                                        .child(diff::status_letter(file.status)),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w(px(0.))
                                        .truncate()
                                        .text_size(px(10.5))
                                        .text_color(theme.text_soft)
                                        .child(file.path.clone()),
                                )
                                .into_any_element()
                        })),
                );

            let diff_view = selected_file
                .map(|file| render_readonly_diff(file, self.mode, theme))
                .unwrap_or_else(|| {
                    div()
                        .flex()
                        .flex_1()
                        .items_center()
                        .justify_center()
                        .text_size(px(11.5))
                        .text_color(theme.muted)
                        .child(self.t("history.selectCommit"))
                        .into_any_element()
                });

            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_h(px(0.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .px(px(16.))
                        .py(px(10.))
                        .border_b_1()
                        .border_color(theme.border)
                        .child(ui::label(self.t("history.base"), 10.5, theme.muted))
                        .child(input_frame(theme).w(px(160.)).child(base_input))
                        .child(ui::label(self.t("history.target"), 10.5, theme.muted))
                        .child(input_frame(theme).w(px(160.)).child(target_input))
                        .child(ui::button(
                            "history-compare-run",
                            if compare_busy {
                                self.t("common.loading")
                            } else {
                                self.t("history.compare")
                            },
                            theme,
                            ButtonTone::Primary,
                            !compare_busy,
                            cx.listener(|app, _event, _window, cx| app.run_compare(cx)),
                        ))
                        .when_some(error.clone(), |this, error| {
                            this.child(div().text_size(px(10.5)).text_color(theme.del).child(error))
                        }),
                )
                .child(
                    div()
                        .flex()
                        .flex_1()
                        .min_h(px(0.))
                        .child(file_list)
                        .child(diff_view),
                )
                .into_any_element()
        } else {
            let list = div()
                .flex()
                .flex_col()
                .w(px(392.))
                .h_full()
                .border_r_1()
                .border_color(theme.border)
                .bg(theme.bg_deep)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(12.))
                        .py(px(8.))
                        .child(
                            div()
                                .text_size(px(9.5))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme.dim)
                                .child(if file_filter.is_some() {
                                    self.t("history.file")
                                } else {
                                    self.t("history.all")
                                }),
                        )
                        .child(
                            div()
                                .text_size(px(9.5))
                                .text_color(theme.dim)
                                .child(log.len().to_string()),
                        ),
                )
                .child(
                    div()
                        .id("history-log")
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_h(px(0.))
                        .overflow_scroll()
                        .when(busy && log.is_empty(), |this| {
                            this.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .px(px(12.))
                                    .py(px(12.))
                                    .text_size(px(11.0))
                                    .text_color(theme.muted)
                                    .child(ui::icon("loader-circle", 13.0, theme.amber))
                                    .child(self.t("history.loading")),
                            )
                        })
                        .when(!busy && log.is_empty(), |this| {
                            this.child(
                                div()
                                    .px(px(12.))
                                    .py(px(12.))
                                    .text_size(px(11.0))
                                    .text_color(theme.muted)
                                    .child(if file_filter.is_some() {
                                        self.t("history.emptyFile")
                                    } else {
                                        self.t("history.empty")
                                    }),
                            )
                        })
                        .children(log.iter().map(|commit| {
                            let hash = commit.hash.clone();
                            let is_selected = selected.as_deref() == Some(hash.as_str());
                            let entity = cx.entity();
                            let project_id = project.id.clone();
                            div()
                                .id(SharedString::from(format!("log-{}", commit.hash)))
                                .flex()
                                .flex_col()
                                .gap(px(2.))
                                .px(px(12.))
                                .py(px(7.))
                                .border_b_1()
                                .border_color(theme.border.opacity(0.5))
                                .cursor_pointer()
                                .when(is_selected, |this| this.bg(theme.surface_hover))
                                .hover(move |style| style.bg(theme.surface_hover.opacity(0.7)))
                                .on_click(move |_event, _window, cx| {
                                    entity.update(cx, |app, cx| {
                                        app.load_commit_detail(
                                            project_id.clone(),
                                            hash.clone(),
                                            cx,
                                        );
                                    });
                                })
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.))
                                        .child(commit_lane(commit, is_selected, theme))
                                        .child(
                                            div()
                                                .flex_1()
                                                .min_w(px(0.))
                                                .truncate()
                                                .text_size(px(11.5))
                                                .text_color(if is_selected {
                                                    theme.text
                                                } else {
                                                    theme.text_soft
                                                })
                                                .child(if commit.subject.is_empty() {
                                                    self.t("push.noSubject").to_string()
                                                } else {
                                                    commit.subject.clone()
                                                }),
                                        )
                                        .when(commit.parents.len() > 1, |this| {
                                            this.child(ui::pill(
                                                self.t("history.merge"),
                                                theme.violet,
                                                theme.violet.opacity(0.1),
                                                8.0,
                                            ))
                                        }),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.))
                                        .pl(px(12.))
                                        .text_size(px(9.0))
                                        .text_color(theme.dim)
                                        .child(commit.short_hash.clone())
                                        .child(commit.author.clone())
                                        .child(relative_time(commit.time, self.lang()))
                                        .when(!commit.refs.is_empty(), |this| {
                                            this.child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap(px(3.))
                                                    .child(ui::icon("git-branch", 8.0, theme.amber))
                                                    .child(commit.refs.join(", ")),
                                            )
                                        }),
                                )
                                .into_any_element()
                        }))
                        .when(has_more && !log.is_empty(), |this| {
                            this.child(
                                div()
                                    .id("history-load-more")
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .py(px(8.))
                                    .cursor_pointer()
                                    .hover(move |style| style.bg(theme.surface_hover))
                                    .on_click(cx.listener(|app, _event, _window, cx| {
                                        app.load_more_history(cx);
                                    }))
                                    .child(ui::label(
                                        self.t("history.loadMore"),
                                        10.5,
                                        theme.muted,
                                    )),
                            )
                        }),
                );

            let detail_panel = if let Some(detail) = detail {
                let files = detail.files.clone();
                let selected_file = detail_file
                    .as_ref()
                    .and_then(|id| files.iter().find(|file| &file.id == id))
                    .or_else(|| files.first());
                let mut panel = div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w(px(0.))
                    .min_h(px(0.))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .px(px(16.))
                            .py(px(10.))
                            .border_b_1()
                            .border_color(theme.border)
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .child(
                                        div()
                                            .text_size(px(14.0))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.text)
                                            .child(if detail.subject.is_empty() {
                                                self.t("push.noSubject").to_string()
                                            } else {
                                                detail.subject.clone()
                                            }),
                                    )
                                    .child(ui::pill(
                                        detail.short_hash.clone(),
                                        theme.amber,
                                        theme.amber_soft,
                                        9.0,
                                    )),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .text_size(px(9.5))
                                    .text_color(theme.dim)
                                    .child(i18n::fill(
                                        self.t("history.author"),
                                        &[("author", &detail.author)],
                                    ))
                                    .child(relative_time(detail.time, self.lang()))
                                    .child(format!(
                                        "{} {}",
                                        detail.file_count,
                                        self.t("common.files")
                                    )),
                            )
                            .when(!detail.body.is_empty(), |this| {
                                this.child(
                                    div()
                                        .text_size(px(10.5))
                                        .text_color(theme.muted)
                                        .child(detail.body.clone()),
                                )
                            }),
                    );
                if !files.is_empty() {
                    panel = panel.child(
                        div()
                            .id("commit-file-list")
                            .flex()
                            .items_center()
                            .gap(px(4.))
                            .px(px(14.))
                            .py(px(7.))
                            .overflow_scroll()
                            .border_b_1()
                            .border_color(theme.border)
                            .children(files.iter().map(|file| {
                                let id = file.id.clone();
                                let selected =
                                    selected_file.is_some_and(|entry| entry.id == file.id);
                                div()
                                    .id(SharedString::from(format!("commit-file-{}", file.id)))
                                    .flex()
                                    .items_center()
                                    .gap(px(4.))
                                    .h(px(23.))
                                    .px(px(7.))
                                    .rounded(px(5.))
                                    .flex_none()
                                    .cursor_pointer()
                                    .border_1()
                                    .border_color(if selected {
                                        theme.border_strong
                                    } else {
                                        theme.border.opacity(0.0)
                                    })
                                    .when(selected, |this| this.bg(theme.surface_hover))
                                    .hover(move |style| style.bg(theme.surface_hover))
                                    .on_click(cx.listener(move |app, _event, _window, cx| {
                                        if let Some(history) = app.history.as_mut() {
                                            history.detail_file = Some(id.clone());
                                        }
                                        cx.notify();
                                    }))
                                    .child(
                                        div()
                                            .text_size(px(8.5))
                                            .text_color(theme.status(file.status))
                                            .child(diff::status_letter(file.status)),
                                    )
                                    .child(
                                        div()
                                            .max_w(px(150.))
                                            .truncate()
                                            .text_size(px(10.0))
                                            .text_color(theme.text_soft)
                                            .child(path_parts(&file.path).0.to_string()),
                                    )
                                    .into_any_element()
                            })),
                    );
                }
                let diff_view = selected_file
                    .map(|file| render_readonly_diff(file, self.mode, theme))
                    .unwrap_or_else(|| {
                        div()
                            .flex()
                            .flex_1()
                            .items_center()
                            .justify_center()
                            .text_size(px(11.5))
                            .text_color(theme.muted)
                            .child(self.t("push.noFiles"))
                            .into_any_element()
                    });
                panel = panel.child(diff_view);
                panel.into_any_element()
            } else if loading_detail {
                div()
                    .flex()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .text_size(px(11.5))
                    .text_color(theme.muted)
                    .child(self.t("common.loading"))
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .text_size(px(11.5))
                    .text_color(theme.muted)
                    .child(self.t("history.selectCommit"))
                    .into_any_element()
            };

            div()
                .flex()
                .flex_1()
                .min_h(px(0.))
                .child(list)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w(px(0.))
                        .min_h(px(0.))
                        .when_some(error.clone(), |this, error| {
                            this.child(
                                div()
                                    .px(px(16.))
                                    .py(px(6.))
                                    .text_size(px(10.5))
                                    .text_color(theme.del)
                                    .child(error),
                            )
                        })
                        .child(detail_panel),
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
            .into_any_element()
    }
}

fn relative_time(time: i64, lang: crate::i18n::Lang) -> String {
    let now = chrono::Utc::now().timestamp();
    let elapsed = (now - time).max(0);
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

fn render_readonly_diff(file: &RepositoryFile, mode: DiffMode, theme: Theme) -> AnyElement {
    if file.binary {
        return div()
            .flex()
            .flex_1()
            .items_center()
            .justify_center()
            .text_size(px(11.5))
            .text_color(theme.muted)
            .child("Binary")
            .into_any_element();
    }
    if file.hunks.is_empty() {
        return div()
            .flex()
            .flex_1()
            .items_center()
            .justify_center()
            .text_size(px(11.5))
            .text_color(theme.muted)
            .child("No changes")
            .into_any_element();
    }
    let max_chars = file
        .hunks
        .iter()
        .flat_map(|hunk| hunk.lines.iter())
        .map(|line| line.content.chars().count())
        .max()
        .unwrap_or(80)
        .min(240);
    let width = if mode == DiffMode::Split {
        (max_chars as f32 * 7.2 * 2.0).max(880.0)
    } else {
        (max_chars as f32 * 7.2).max(700.0)
    };
    let mut rows: Vec<AnyElement> = Vec::new();
    if mode == DiffMode::Split {
        for row in diff::build_split_rows(&file.hunks) {
            match row {
                diff::DiffRow::Hunk { hunk, .. } => rows.push(hunk_header_row(&hunk.header, theme)),
                diff::DiffRow::Split(row) => {
                    rows.push(render_split_readonly_row(&row, &file.language, theme))
                }
            }
        }
    } else {
        for hunk in &file.hunks {
            rows.push(hunk_header_row(&hunk.header, theme));
            for line in &hunk.lines {
                rows.push(render_unified_readonly_line(line, &file.language, theme));
            }
        }
    }
    div()
        .id(SharedString::from(format!("readonly-diff-{}", file.id)))
        .flex()
        .flex_col()
        .flex_1()
        .min_h(px(0.))
        .overflow_scroll()
        .child(div().flex().flex_col().min_w(px(width)).children(rows))
        .into_any_element()
}

fn hunk_header_row(header: &str, theme: Theme) -> AnyElement {
    div()
        .h(px(23.))
        .w_full()
        .px(px(10.))
        .bg(theme.surface_raised)
        .border_y_1()
        .border_color(theme.border.opacity(0.6))
        .text_size(px(10.0))
        .font_family("SF Mono")
        .text_color(theme.muted)
        .child(header.to_string())
        .into_any_element()
}

fn render_unified_readonly_line(
    line: &flashbee_git::DiffLine,
    language: &str,
    theme: Theme,
) -> AnyElement {
    let bg = match line.kind {
        LineType::Add => theme.add_soft,
        LineType::Del => theme.del_soft,
        LineType::Context => theme.bg,
    };
    div()
        .flex()
        .flex_row()
        .h(px(20.))
        .w_full()
        .bg(bg)
        .child(number(line.old_no, theme))
        .child(number(line.new_no, theme))
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
        .child(code_line(&line.content, language, theme))
        .into_any_element()
}

fn render_split_readonly_row(row: &diff::SplitRow, language: &str, theme: Theme) -> AnyElement {
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
                .border_r_1()
                .border_color(theme.border.opacity(0.5))
                .when(
                    row.left
                        .as_ref()
                        .is_some_and(|line| line.kind == LineType::Del),
                    |this| this.bg(theme.del_soft),
                )
                .child(number(
                    row.left.as_ref().and_then(|line| line.old_no),
                    theme,
                ))
                .child(
                    div()
                        .w(px(16.))
                        .flex_none()
                        .text_size(px(10.5))
                        .text_color(theme.del)
                        .child(
                            if row
                                .left
                                .as_ref()
                                .is_some_and(|line| line.kind == LineType::Del)
                            {
                                "−"
                            } else {
                                ""
                            },
                        ),
                )
                .child(
                    row.left
                        .as_ref()
                        .map(|line| code_line(&line.content, language, theme))
                        .unwrap_or_else(|| div().flex_1().into_any_element()),
                ),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .w(px(0.))
                .flex_1()
                .when(
                    row.right
                        .as_ref()
                        .is_some_and(|line| line.kind == LineType::Add),
                    |this| this.bg(theme.add_soft),
                )
                .child(number(
                    row.right.as_ref().and_then(|line| line.new_no),
                    theme,
                ))
                .child(
                    div()
                        .w(px(16.))
                        .flex_none()
                        .text_size(px(10.5))
                        .text_color(theme.add)
                        .child(
                            if row
                                .right
                                .as_ref()
                                .is_some_and(|line| line.kind == LineType::Add)
                            {
                                "+"
                            } else {
                                ""
                            },
                        ),
                )
                .child(
                    row.right
                        .as_ref()
                        .map(|line| code_line(&line.content, language, theme))
                        .unwrap_or_else(|| div().flex_1().into_any_element()),
                ),
        )
        .into_any_element()
}

fn number(value: Option<u32>, theme: Theme) -> AnyElement {
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
        .child(value.map(|value| value.to_string()).unwrap_or_default())
        .into_any_element()
}

fn code_line(content: &str, language: &str, theme: Theme) -> AnyElement {
    let spans = highlight::highlight_line(content, language, theme);
    let mut cell = div()
        .flex()
        .flex_row()
        .items_center()
        .flex_1()
        .min_w(px(0.))
        .overflow_hidden()
        .pl(px(4.))
        .text_size(px(12.0))
        .font_family("SF Mono")
        .whitespace_nowrap();
    if content.is_empty() {
        cell = cell.child(" ");
    } else {
        for span in spans {
            cell = cell.child(div().flex_none().text_color(span.color).child(span.text));
        }
    }
    cell.into_any_element()
}

fn commit_lane(commit: &flashbee_git::LogCommit, selected: bool, theme: Theme) -> AnyElement {
    let is_merge = commit.parents.len() > 1;
    let tone = if selected {
        theme.amber
    } else if is_merge {
        theme.violet
    } else {
        theme.border_strong
    };
    div()
        .relative()
        .flex_none()
        .w(px(12.))
        .h(px(28.))
        .child(
            div()
                .absolute()
                .left(px(5.))
                .top_0()
                .bottom_0()
                .w(px(1.))
                .bg(theme.border.opacity(0.7)),
        )
        .child(
            div()
                .absolute()
                .left(px(2.))
                .top(px(10.))
                .size(px(7.))
                .rounded_full()
                .bg(tone),
        )
        .when(is_merge, |this| {
            this.child(
                div()
                    .absolute()
                    .left(px(8.))
                    .top(px(11.))
                    .size(px(5.))
                    .rounded_full()
                    .bg(theme.violet),
            )
        })
        .into_any_element()
}
