use tauri::menu::{AboutMetadata, Menu, MenuBuilder, MenuItem, SubmenuBuilder};
use tauri::{AppHandle, Runtime};

#[derive(Clone, Copy)]
pub enum Locale {
    En,
    Zh,
}

impl Locale {
    pub fn from_tag(tag: &str) -> Self {
        if tag.to_lowercase().starts_with("zh") {
            Self::Zh
        } else {
            Self::En
        }
    }

    fn labels(self) -> Labels {
        match self {
            Self::En => Labels {
                file: "File",
                edit: "Edit",
                view: "View",
                window: "Window",
                help: "Help",
                settings: "Settings…",
                open_repository: "Open Repository…",
                new_workspace: "New Workspace…",
                command_palette: "Command Palette",
                toggle_sidebar: "Toggle Sidebar",
                split_diff: "Split Diff",
                unified_diff: "Unified Diff",
                previous_file: "Previous File",
                next_file: "Next File",
                refresh: "Refresh",
                github: "FlashBee on GitHub",
                help_item: "FlashBee Help",
            },
            Self::Zh => Labels {
                file: "文件",
                edit: "编辑",
                view: "显示",
                window: "窗口",
                help: "帮助",
                settings: "设置…",
                open_repository: "打开仓库…",
                new_workspace: "新建工作区…",
                command_palette: "命令面板",
                toggle_sidebar: "显示/隐藏边栏",
                split_diff: "分栏对比",
                unified_diff: "统一对比",
                previous_file: "上一个文件",
                next_file: "下一个文件",
                refresh: "刷新",
                github: "FlashBee 的 GitHub",
                help_item: "FlashBee 帮助",
            },
        }
    }
}

#[derive(Clone, Copy)]
struct Labels {
    file: &'static str,
    edit: &'static str,
    view: &'static str,
    window: &'static str,
    help: &'static str,
    settings: &'static str,
    open_repository: &'static str,
    new_workspace: &'static str,
    command_palette: &'static str,
    toggle_sidebar: &'static str,
    split_diff: &'static str,
    unified_diff: &'static str,
    previous_file: &'static str,
    next_file: &'static str,
    refresh: &'static str,
    github: &'static str,
    help_item: &'static str,
}

pub fn build<R: Runtime>(app: &AppHandle<R>, locale: Locale) -> tauri::Result<Menu<R>> {
    let labels = locale.labels();

    let about = AboutMetadata {
        name: Some("FlashBee".to_string()),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
        short_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        copyright: Some("MIT © 2026 wangzhuo".to_string()),
        credits: Some("A calm, native Git viewer for macOS.".to_string()),
        ..Default::default()
    };

    let settings = MenuItem::with_id(
        app,
        "settings",
        labels.settings,
        true,
        Some("CmdOrCtrl+,"),
    )?;

    let app_menu = SubmenuBuilder::new(app, "FlashBee")
        .about(Some(about))
        .separator()
        .item(&settings)
        .separator()
        .services()
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .quit()
        .build()?;

    let open_repository = MenuItem::with_id(
        app,
        "open-repository",
        labels.open_repository,
        true,
        Some("CmdOrCtrl+O"),
    )?;
    let new_workspace = MenuItem::with_id(
        app,
        "new-workspace",
        labels.new_workspace,
        true,
        Some("CmdOrCtrl+Shift+N"),
    )?;
    let file_menu = SubmenuBuilder::new(app, labels.file)
        .item(&open_repository)
        .item(&new_workspace)
        .separator()
        .close_window()
        .build()?;

    let edit_menu = SubmenuBuilder::new(app, labels.edit)
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    let command_palette = MenuItem::with_id(
        app,
        "command-palette",
        labels.command_palette,
        true,
        Some("CmdOrCtrl+K"),
    )?;
    let toggle_sidebar = MenuItem::with_id(
        app,
        "toggle-sidebar",
        labels.toggle_sidebar,
        true,
        Some("CmdOrCtrl+\\"),
    )?;
    let split_diff = MenuItem::with_id(
        app,
        "split-diff",
        labels.split_diff,
        true,
        Some("CmdOrCtrl+1"),
    )?;
    let unified_diff = MenuItem::with_id(
        app,
        "unified-diff",
        labels.unified_diff,
        true,
        Some("CmdOrCtrl+2"),
    )?;
    let previous_file = MenuItem::with_id(
        app,
        "previous-file",
        labels.previous_file,
        true,
        Some("CmdOrCtrl+["),
    )?;
    let next_file = MenuItem::with_id(
        app,
        "next-file",
        labels.next_file,
        true,
        Some("CmdOrCtrl+]"),
    )?;
    let refresh = MenuItem::with_id(
        app,
        "refresh",
        labels.refresh,
        true,
        Some("CmdOrCtrl+R"),
    )?;

    let view_menu = SubmenuBuilder::new(app, labels.view)
        .item(&command_palette)
        .item(&toggle_sidebar)
        .separator()
        .item(&split_diff)
        .item(&unified_diff)
        .separator()
        .item(&previous_file)
        .item(&next_file)
        .separator()
        .item(&refresh)
        .separator()
        .fullscreen()
        .build()?;

    let window_menu = SubmenuBuilder::new(app, labels.window)
        .minimize()
        .maximize()
        .separator()
        .bring_all_to_front()
        .build()?;

    let github = MenuItem::with_id(app, "open-github", labels.github, true, None::<&str>)?;
    let help_item = MenuItem::with_id(app, "open-help", labels.help_item, true, None::<&str>)?;
    let help_menu = SubmenuBuilder::new(app, labels.help)
        .item(&help_item)
        .item(&github)
        .build()?;

    MenuBuilder::new(app)
        .items(&[
            &app_menu,
            &file_menu,
            &edit_menu,
            &view_menu,
            &window_menu,
            &help_menu,
        ])
        .build()
}
