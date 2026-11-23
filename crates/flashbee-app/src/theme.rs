use gpui::{Hsla, rgb};

fn color(hex: u32) -> Hsla {
    rgb(hex).into()
}
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolvedTheme {
    Light,
    Dark,
}

impl ResolvedTheme {
    pub fn is_dark(self) -> bool {
        matches!(self, ResolvedTheme::Dark)
    }
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub bg: Hsla,
    pub bg_deep: Hsla,
    pub surface: Hsla,
    pub surface_raised: Hsla,
    pub surface_hover: Hsla,
    pub border: Hsla,
    pub border_strong: Hsla,
    pub text: Hsla,
    pub text_soft: Hsla,
    pub muted: Hsla,
    pub dim: Hsla,
    pub amber: Hsla,
    pub amber_bright: Hsla,
    pub amber_soft: Hsla,
    pub add: Hsla,
    pub add_soft: Hsla,
    pub del: Hsla,
    pub del_soft: Hsla,
    pub blue: Hsla,
    pub violet: Hsla,
    pub shadow: Hsla,
    pub overlay: Hsla,
    pub dark: bool,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            bg: color(0x262628),
            bg_deep: color(0x222224),
            surface: color(0x262628),
            surface_raised: color(0x2a2a2c),
            surface_hover: color(0x37373a),
            border: color(0x3a3a3d),
            border_strong: color(0x4a4a4d),
            text: color(0xf5f5f5),
            text_soft: color(0xd4d4d6),
            muted: color(0x9a9a9d),
            dim: color(0x6e6e71),
            amber: color(0xf8bc65),
            amber_bright: color(0xffcf7d),
            amber_soft: color(0xf8bc65).opacity(0.12),
            add: color(0x8acc98),
            add_soft: color(0x80c28e).opacity(0.10),
            del: color(0xe08798),
            del_soft: color(0xd77f90).opacity(0.09),
            blue: color(0x90c8e9),
            violet: color(0xc0aaec),
            shadow: color(0x000000).opacity(0.35),
            overlay: color(0x111113).opacity(0.55),
            dark: true,
        }
    }

    pub fn light() -> Self {
        Self {
            bg: color(0xf3f5f8),
            bg_deep: color(0xedf0f4),
            surface: color(0xf8fafc),
            surface_raised: color(0xffffff),
            surface_hover: color(0xeef2f6),
            border: color(0xe1e6ed),
            border_strong: color(0xcfd7e1),
            text: color(0x1e2836),
            text_soft: color(0x3f4b5c),
            muted: color(0x6b7788),
            dim: color(0x8b96a5),
            amber: color(0xb97c17),
            amber_bright: color(0xdda23b),
            amber_soft: color(0xb97c17).opacity(0.10),
            add: color(0x1b8559),
            add_soft: color(0x1b8559).opacity(0.09),
            del: color(0xbd4258),
            del_soft: color(0xbd4258).opacity(0.08),
            blue: color(0x2d709e),
            violet: color(0x7d50c4),
            shadow: color(0x1e2836).opacity(0.14),
            overlay: color(0x1e2836).opacity(0.28),
            dark: false,
        }
    }

    pub fn for_theme(theme: ResolvedTheme) -> Self {
        match theme {
            ResolvedTheme::Dark => Self::dark(),
            ResolvedTheme::Light => Self::light(),
        }
    }

    /// A calm accent that changes with status without shouting.
    pub fn status(&self, status: flashbee_git::ChangeStatus) -> Hsla {
        use flashbee_git::ChangeStatus;
        match status {
            ChangeStatus::Added => self.add,
            ChangeStatus::Deleted => self.del,
            ChangeStatus::Renamed => self.blue,
            ChangeStatus::Modified => self.amber,
        }
    }
}

/// The resolved theme, exposed to small standalone widgets such as text inputs.
#[derive(Clone, Copy)]
pub struct GlobalTheme(pub Theme);

impl gpui::Global for GlobalTheme {}
