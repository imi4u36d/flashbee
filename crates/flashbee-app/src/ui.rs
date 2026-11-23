use gpui::{
    AnyElement, ClickEvent, ElementId, Hsla, SharedString, Window, div, prelude::*, px, rgb, svg,
};

use crate::theme::Theme;

pub fn color_from_hex(value: &str) -> Hsla {
    let hex = value.trim().trim_start_matches('#');
    if hex.len() >= 6 {
        if let Ok(number) = u32::from_str_radix(&hex[..6], 16) {
            return rgb(number).into();
        }
    }
    rgb(0xf6bd60).into()
}

pub fn icon(name: &str, size: f32, color: Hsla) -> AnyElement {
    svg()
        .path(format!("icons/{name}.svg"))
        .size(px(size))
        .text_color(color)
        .into_any_element()
}

pub fn label(text: impl Into<SharedString>, size: f32, color: Hsla) -> AnyElement {
    div()
        .text_size(px(size))
        .text_color(color)
        .child(text.into())
        .into_any_element()
}

pub fn label_medium(text: impl Into<SharedString>, size: f32, color: Hsla) -> AnyElement {
    div()
        .text_size(px(size))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(color)
        .child(text.into())
        .into_any_element()
}

pub fn label_semibold(text: impl Into<SharedString>, size: f32, color: Hsla) -> AnyElement {
    div()
        .text_size(px(size))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(color)
        .child(text.into())
        .into_any_element()
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonTone {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

pub fn button(
    id: impl Into<ElementId>,
    text: impl Into<SharedString>,
    theme: Theme,
    tone: ButtonTone,
    enabled: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static,
) -> AnyElement {
    let (background, foreground, border) = match tone {
        ButtonTone::Primary => (theme.amber, theme.bg_deep, theme.amber),
        ButtonTone::Secondary => (theme.surface_raised, theme.text, theme.border),
        ButtonTone::Ghost => (
            theme.surface.opacity(0.0),
            theme.text_soft,
            theme.border.opacity(0.0),
        ),
        ButtonTone::Danger => (theme.del.opacity(0.12), theme.del, theme.del.opacity(0.28)),
    };
    let hover = match tone {
        ButtonTone::Primary => theme.amber_bright,
        ButtonTone::Secondary => theme.surface_hover,
        ButtonTone::Ghost => theme.surface_hover,
        ButtonTone::Danger => theme.del.opacity(0.2),
    };

    div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .h(px(30.))
        .px(px(12.))
        .rounded(px(7.))
        .bg(background)
        .border_1()
        .border_color(border)
        .text_size(px(12.5))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(foreground)
        .opacity(if enabled { 1.0 } else { 0.42 })
        .when(enabled, |this| {
            this.cursor_pointer()
                .hover(move |style| style.bg(hover))
                .on_click(move |event, window, cx| on_click(event, window, cx))
        })
        .child(text.into())
        .into_any_element()
}

pub fn icon_button(
    id: impl Into<ElementId>,
    icon_name: &'static str,
    theme: Theme,
    color: Hsla,
    enabled: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(26.))
        .rounded(px(6.))
        .opacity(if enabled { 1.0 } else { 0.4 })
        .when(enabled, |this| {
            this.cursor_pointer()
                .hover(move |style| style.bg(theme.surface_hover))
                .on_click(move |event, window, cx| on_click(event, window, cx))
        })
        .child(icon(icon_name, 14.0, color))
        .into_any_element()
}

pub fn checkbox(
    id: impl Into<ElementId>,
    checked: bool,
    text: impl Into<SharedString>,
    theme: Theme,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .flex()
        .items_center()
        .gap(px(8.))
        .cursor_pointer()
        .on_click(move |event, window, cx| on_click(event, window, cx))
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
                    this.child(icon("check", 10.0, theme.bg_deep))
                }),
        )
        .child(label(text.into(), 12.5, theme.text_soft))
        .into_any_element()
}

pub fn separator(theme: Theme) -> AnyElement {
    div()
        .h(px(1.))
        .w_full()
        .my(px(6.))
        .bg(theme.border)
        .into_any_element()
}

pub fn vertical_separator(theme: Theme) -> AnyElement {
    div()
        .w(px(1.))
        .h(px(18.))
        .mx(px(6.))
        .bg(theme.border)
        .into_any_element()
}

pub fn pill(
    text: impl Into<SharedString>,
    foreground: Hsla,
    background: Hsla,
    size: f32,
) -> AnyElement {
    div()
        .flex()
        .items_center()
        .h(px(size + 4.0))
        .px(px(6.))
        .rounded(px(5.))
        .bg(background)
        .text_size(px(size))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(foreground)
        .child(text.into())
        .into_any_element()
}

pub fn kbd(text: impl Into<SharedString>, theme: Theme) -> AnyElement {
    div()
        .flex()
        .items_center()
        .h(px(17.))
        .px(px(4.))
        .rounded(px(4.))
        .bg(theme.surface_hover)
        .border_1()
        .border_color(theme.border)
        .text_size(px(10.5))
        .text_color(theme.muted)
        .child(text.into())
        .into_any_element()
}
