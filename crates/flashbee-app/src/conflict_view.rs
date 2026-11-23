use flashbee_git::{ConflictSegment, ConflictSide};
use gpui::{AnyElement, Context, SharedString, div, prelude::*, px};

use crate::app::FlashBeeApp;
use crate::highlight;
use crate::i18n;
use crate::model::Project;
use crate::theme::Theme;
use crate::ui::{self, ButtonTone};

impl FlashBeeApp {
    pub fn render_conflict_view(
        &mut self,
        project: &Project,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let conflicts = project.conflicts.clone();
        let conflict_count = conflicts.len();
        let merge_ready = project.merge_in_progress && conflict_count == 0;
        let selected = self
            .conflict_file
            .clone()
            .or_else(|| conflicts.first().map(|conflict| conflict.path.clone()));
        let detail = self.conflict_detail.clone();
        let choices = self.conflict_choices.clone();
        let busy = self.conflict_busy;
        let loading = self.conflict_loading;
        let error = self.conflict_error.clone();
        let unresolved = choices.iter().filter(|choice| choice.is_none()).count();

        let header =
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .h(px(50.))
                .flex_none()
                .px(px(16.))
                .border_b_1()
                .border_color(theme.border)
                .child(ui::icon_button(
                    "conflict-back",
                    "arrow-left",
                    theme,
                    theme.muted,
                    true,
                    cx.listener(|app, _event, _window, cx| app.close_conflict_resolver(cx)),
                ))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w(px(0.))
                        .child(
                            div()
                                .text_size(px(14.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme.text)
                                .child(self.t("conflict.title")),
                        )
                        .child(div().text_size(px(10.5)).text_color(theme.muted).child(
                            i18n::fill(
                                self.t("conflict.countChip"),
                                &[("count", &conflict_count.to_string())],
                            ),
                        )),
                )
                .when(merge_ready, |this| {
                    this.child(ui::button(
                        "conflict-merge",
                        self.t("conflict.completeMerge"),
                        theme,
                        ButtonTone::Primary,
                        !busy,
                        cx.listener(|app, _event, _window, cx| app.complete_merge(cx)),
                    ))
                })
                .when(project.merge_in_progress, |this| {
                    this.child(ui::button(
                        "conflict-abort",
                        self.t("conflict.abortMerge"),
                        theme,
                        ButtonTone::Ghost,
                        !busy,
                        cx.listener(|app, _event, _window, cx| app.request_abort_merge(cx)),
                    ))
                });

        let list = div()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(228.))
            .h_full()
            .border_r_1()
            .border_color(theme.border)
            .bg(theme.bg_deep)
            .child(
                div()
                    .px(px(12.))
                    .pt(px(10.))
                    .pb(px(6.))
                    .text_size(px(9.5))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.dim)
                    .child(self.t("conflict.files").to_uppercase()),
            )
            .child(
                div()
                    .id("conflict-file-list")
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .px(px(8.))
                    .overflow_scroll()
                    .children(conflicts.iter().map(|conflict| {
                        let path = conflict.path.clone();
                        let is_selected = selected.as_deref() == Some(path.as_str());
                        let kind = match conflict.kind.as_str() {
                            "bothModified" => self.t("conflict.kindBothModified").to_string(),
                            "bothAdded" => self.t("conflict.kindBothAdded").to_string(),
                            "deletedByUs" => self.t("conflict.kindDeletedByUs").to_string(),
                            "deletedByThem" => self.t("conflict.kindDeletedByThem").to_string(),
                            "addedByUs" => self.t("conflict.kindAddedByUs").to_string(),
                            "addedByThem" => self.t("conflict.kindAddedByThem").to_string(),
                            other => other.to_string(),
                        };
                        let entity = cx.entity();
                        div()
                            .id(SharedString::from(format!("conflict-{}", conflict.path)))
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .px(px(8.))
                            .py(px(6.))
                            .rounded(px(6.))
                            .cursor_pointer()
                            .when(is_selected, |this| {
                                this.bg(theme.surface_hover)
                                    .border_1()
                                    .border_color(theme.border)
                            })
                            .hover(move |style| style.bg(theme.surface_hover.opacity(0.8)))
                            .on_click(move |_event, _window, cx| {
                                entity.update(cx, |app, cx| {
                                    app.select_conflict_file(path.clone(), cx);
                                });
                            })
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .child(div().size(px(6.)).rounded_full().bg(theme.del))
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w(px(0.))
                                            .text_size(px(11.0))
                                            .text_color(if is_selected {
                                                theme.text
                                            } else {
                                                theme.text_soft
                                            })
                                            .truncate()
                                            .child(conflict.path.clone()),
                                    ),
                            )
                            .child(
                                div()
                                    .pl(px(12.))
                                    .text_size(px(9.5))
                                    .text_color(theme.dim)
                                    .child(kind),
                            )
                            .into_any_element()
                    })),
            );

        let detail_view = if let Some(detail) = detail {
            let file_level = !detail.binary
                && (detail.ours_exists != detail.theirs_exists
                    || (!detail.ours_exists && !detail.theirs_exists));
            if detail.binary || file_level {
                self.render_file_level_conflict(&detail, theme, cx)
            } else {
                let mut lines: Vec<AnyElement> = Vec::new();
                let mut conflict_index = 0usize;
                for segment in &detail.segments {
                    match segment {
                        ConflictSegment::Common { lines: common } => {
                            lines.push(render_common_block(common, &detail.language, theme));
                        }
                        ConflictSegment::Conflict {
                            ours,
                            theirs,
                            suggestion,
                            ..
                        } => {
                            let choice = choices.get(conflict_index).copied().flatten();
                            lines.push(self.render_conflict_block(
                                conflict_index,
                                ours,
                                theirs,
                                *suggestion,
                                choice,
                                &detail.language,
                                theme,
                                cx,
                            ));
                            conflict_index += 1;
                        }
                    }
                }
                div()
                    .id("conflict-detail-scroll")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.))
                    .overflow_scroll()
                    .p(px(14.))
                    .gap(px(10.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(ui::label_medium(detail.path.clone(), 12.0, theme.text))
                            .child(ui::pill(
                                detail.language.to_uppercase(),
                                theme.muted,
                                theme.surface_hover,
                                9.0,
                            ))
                            .child(div().flex_1())
                            .child(ui::button(
                                "conflict-all-ours",
                                self.t("conflict.left"),
                                theme,
                                ButtonTone::Ghost,
                                !busy,
                                cx.listener(|app, _event, _window, cx| {
                                    app.choose_all_conflicts(ConflictSide::Ours, cx);
                                }),
                            ))
                            .child(ui::button(
                                "conflict-all-theirs",
                                self.t("conflict.right"),
                                theme,
                                ButtonTone::Ghost,
                                !busy,
                                cx.listener(|app, _event, _window, cx| {
                                    app.choose_all_conflicts(ConflictSide::Theirs, cx);
                                }),
                            )),
                    )
                    .children(lines)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_size(px(10.5))
                                    .text_color(if unresolved == 0 {
                                        theme.add
                                    } else {
                                        theme.muted
                                    })
                                    .child(if unresolved == 0 {
                                        self.t("conflict.allBlocksResolved").to_string()
                                    } else if unresolved == 1 {
                                        self.t("conflict.unresolvedOne").to_string()
                                    } else {
                                        i18n::fill(
                                            self.t("conflict.unresolved"),
                                            &[("count", &unresolved.to_string())],
                                        )
                                    }),
                            )
                            .child(ui::button(
                                "conflict-save",
                                if busy {
                                    self.t("common.loading")
                                } else {
                                    self.t("conflict.save")
                                },
                                theme,
                                ButtonTone::Primary,
                                !busy && unresolved == 0,
                                cx.listener(|app, _event, _window, cx| app.save_conflict(cx)),
                            )),
                    )
                    .when_some(error, |this, error| {
                        this.child(
                            div()
                                .px(px(8.))
                                .py(px(6.))
                                .rounded(px(6.))
                                .bg(theme.del.opacity(0.08))
                                .text_size(px(10.5))
                                .text_color(theme.del)
                                .child(error),
                        )
                    })
                    .into_any_element()
            }
        } else if loading {
            div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .text_size(px(11.5))
                        .text_color(theme.muted)
                        .child(ui::icon("loader-circle", 14.0, theme.amber))
                        .child(self.t("conflict.loading")),
                )
                .into_any_element()
        } else if let Some(error) = error {
            div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .text_size(px(11.5))
                .text_color(theme.del)
                .child(error)
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .text_size(px(11.5))
                .text_color(theme.muted)
                .child(self.t("conflict.allResolved"))
                .into_any_element()
        };

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h(px(0.))
            .child(header)
            .child(
                div().flex().flex_1().min_h(px(0.)).child(list).child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w(px(0.))
                        .child(detail_view),
                ),
            )
            .into_any_element()
    }

    fn render_file_level_conflict(
        &self,
        detail: &flashbee_git::ConflictDetail,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let both_deleted = !detail.ours_exists && !detail.theirs_exists;
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
                    .gap(px(10.))
                    .max_w(px(460.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(42.))
                            .rounded(px(12.))
                            .bg(theme.del.opacity(0.1))
                            .child(ui::icon("triangle-alert", 19.0, theme.del)),
                    )
                    .child(
                        div()
                            .text_size(px(15.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(if detail.binary {
                                self.t("conflict.binaryTitle")
                            } else {
                                self.t("conflict.fileLevelTitle")
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(theme.muted)
                            .text_center()
                            .child(if detail.binary {
                                self.t("conflict.binaryDescription")
                            } else {
                                self.t("conflict.fileLevelDescription")
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(ui::button(
                                "conflict-keep-ours",
                                if detail.ours_exists {
                                    self.t("conflict.useOursFile")
                                } else {
                                    self.t("conflict.keepDeleted")
                                },
                                theme,
                                ButtonTone::Secondary,
                                !self.conflict_busy,
                                cx.listener(|app, _event, _window, cx| {
                                    app.resolve_conflict_file_side(ConflictSide::Ours, cx);
                                }),
                            ))
                            .child(ui::button(
                                "conflict-keep-theirs",
                                if detail.theirs_exists {
                                    self.t("conflict.useTheirsFile")
                                } else {
                                    self.t("conflict.keepDeleted")
                                },
                                theme,
                                ButtonTone::Secondary,
                                !self.conflict_busy,
                                cx.listener(|app, _event, _window, cx| {
                                    app.resolve_conflict_file_side(ConflictSide::Theirs, cx);
                                }),
                            )),
                    )
                    .when(both_deleted, |this| {
                        this.child(
                            div()
                                .text_size(px(10.5))
                                .text_color(theme.dim)
                                .child(self.t("conflict.bothDeletedDescription")),
                        )
                    }),
            )
            .into_any_element()
    }

    fn render_conflict_block(
        &self,
        index: usize,
        ours: &[String],
        theirs: &[String],
        suggestion: Option<ConflictSide>,
        choice: Option<ConflictSide>,
        language: &str,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let suggestion_label = suggestion.map(|side| {
            format!(
                "{} {}",
                self.t("conflict.autoHint"),
                match side {
                    ConflictSide::Ours => self.t("conflict.left"),
                    ConflictSide::Theirs => self.t("conflict.right"),
                }
            )
        });
        div()
            .flex()
            .flex_col()
            .rounded(px(9.))
            .border_1()
            .border_color(theme.del.opacity(0.28))
            .bg(theme.bg)
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .px(px(10.))
                    .py(px(6.))
                    .bg(theme.del.opacity(0.07))
                    .child(
                        div()
                            .text_size(px(10.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.del)
                            .child(i18n::fill(
                                self.t("conflict.block"),
                                &[("index", &(index + 1).to_string())],
                            )),
                    )
                    .when_some(suggestion_label, |this, label| {
                        this.child(ui::label(label, 9.5, theme.dim))
                    })
                    .child(div().flex_1())
                    .child(ui::button(
                        "conflict-ours",
                        self.t("conflict.left"),
                        theme,
                        if choice == Some(ConflictSide::Ours) {
                            ButtonTone::Primary
                        } else {
                            ButtonTone::Ghost
                        },
                        !self.conflict_busy,
                        cx.listener(move |app, _event, _window, cx| {
                            app.choose_conflict(index, ConflictSide::Ours, cx);
                        }),
                    ))
                    .child(ui::button(
                        "conflict-theirs",
                        self.t("conflict.right"),
                        theme,
                        if choice == Some(ConflictSide::Theirs) {
                            ButtonTone::Primary
                        } else {
                            ButtonTone::Ghost
                        },
                        !self.conflict_busy,
                        cx.listener(move |app, _event, _window, cx| {
                            app.choose_conflict(index, ConflictSide::Theirs, cx);
                        }),
                    )),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .child(side_column(
                        self.t("conflict.left"),
                        ours,
                        language,
                        theme,
                        choice == Some(ConflictSide::Ours),
                    ))
                    .child(side_column(
                        self.t("conflict.right"),
                        theirs,
                        language,
                        theme,
                        choice == Some(ConflictSide::Theirs),
                    )),
            )
            .into_any_element()
    }
}

fn render_common_block(lines: &[String], language: &str, theme: Theme) -> AnyElement {
    let mut block = div()
        .flex()
        .flex_col()
        .rounded(px(8.))
        .border_1()
        .border_color(theme.border.opacity(0.6))
        .bg(theme.bg)
        .px(px(10.))
        .py(px(5.));
    for line in lines.iter().take(120) {
        block = block.child(highlight_line_element(
            line,
            language,
            theme,
            theme.text_soft,
        ));
    }
    if lines.len() > 120 {
        block = block.child(div().text_size(px(9.5)).text_color(theme.dim).child("…"));
    }
    block.into_any_element()
}

fn side_column(
    label: &'static str,
    lines: &[String],
    language: &str,
    theme: Theme,
    selected: bool,
) -> AnyElement {
    let mut column = div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w(px(0.))
        .border_t_1()
        .border_color(theme.border.opacity(0.5))
        .bg(if selected {
            theme.amber_soft.opacity(0.45)
        } else {
            theme.bg
        })
        .px(px(9.))
        .py(px(5.))
        .child(
            div()
                .text_size(px(9.0))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(if selected { theme.amber } else { theme.dim })
                .child(label.to_uppercase()),
        );
    if lines.is_empty() {
        column = column.child(div().text_size(px(10.0)).text_color(theme.dim).child("—"));
    }
    for line in lines.iter().take(80) {
        column = column.child(highlight_line_element(
            line,
            language,
            theme,
            theme.text_soft,
        ));
    }
    if lines.len() > 80 {
        column = column.child(div().text_size(px(9.5)).text_color(theme.dim).child("…"));
    }
    column.into_any_element()
}

fn highlight_line_element(
    line: &str,
    language: &str,
    theme: Theme,
    plain_color: gpui::Hsla,
) -> AnyElement {
    let spans = highlight::highlight_line(line, language, theme);
    let mut row = div()
        .flex()
        .flex_row()
        .h(px(18.))
        .text_size(px(11.0))
        .font_family("SF Mono")
        .whitespace_nowrap();
    if line.is_empty() {
        row = row.child(" ");
    } else {
        for span in spans {
            row = row.child(
                div()
                    .flex_none()
                    .text_color(if span.color == theme.text {
                        plain_color
                    } else {
                        span.color
                    })
                    .child(span.text),
            );
        }
    }
    row.into_any_element()
}
