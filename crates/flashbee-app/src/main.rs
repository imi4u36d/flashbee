mod app;
mod assets;
mod conflict_view;
mod diff;
mod highlight;
mod history_view;
mod i18n;
mod input;
mod main_view;
mod modals;
mod model;
mod overview_view;
mod search_view;
mod sidebar_view;
mod theme;
mod ui;

use gpui::{
    App, Application, Bounds, KeyBinding, Menu, MenuItem, SystemMenuType,
    WindowBackgroundAppearance, WindowBounds, WindowOptions, actions, point, prelude::*, px, size,
};

use app::{
    CloseOverlayAction, CopyPrDescriptionAction, FetchRepository, FlashBeeApp, NextFileAction,
    OpenBranchesAction, OpenCommitAction, OpenHistoryView, OpenOverviewView, OpenPalette,
    OpenRepository, OpenSettings, PreviousFileAction, RefreshRepository, SplitDiffAction,
    StageAllAction, ToggleDiffLayoutAction, ToggleReviewedAction, ToggleSidebar, UnifiedDiffAction,
};
use assets::Assets;
use input::bind_text_input_keys;
use model::load_window_state;

actions!(flashbee_main, [Quit]);

fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        bind_text_input_keys(cx);
        bind_app_keys(cx);
        install_menus(cx);

        let saved = load_window_state();
        let width = saved.width.max(960.0);
        let height = saved.height.max(620.0);
        let bounds = if saved.width > 0.0 {
            Bounds::new(point(px(saved.x), px(saved.y)), size(px(width), px(height)))
        } else {
            Bounds::centered(None, size(px(1360.0), px(880.0)), cx)
        };
        let window_bounds = if saved.maximized {
            WindowBounds::Maximized(bounds)
        } else {
            WindowBounds::Windowed(bounds)
        };

        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(window_bounds),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some("FlashBee".into()),
                        appears_transparent: true,
                        traffic_light_position: Some(point(px(18.0), px(21.0))),
                    }),
                    window_min_size: Some(size(px(960.0), px(620.0))),
                    window_background: WindowBackgroundAppearance::Opaque,
                    ..Default::default()
                },
                |_window, cx| cx.new(FlashBeeApp::new),
            )
            .expect("Could not open the FlashBee window");

        let view = window
            .update(cx, |_, _, cx| cx.entity())
            .expect("Could not read the FlashBee view");

        window
            .update(cx, |app, window, cx| {
                app.install_window_observers(window, cx);
                let focus = app.main_focus.clone();
                window.focus(&focus);
                app.pending_main_focus = false;
            })
            .ok();

        install_action_handlers(cx, &view);
        cx.activate(true);

        cx.on_action(|_: &Quit, cx| {
            cx.quit();
        });

        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
    });
}

fn bind_app_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-r", RefreshRepository, None),
        KeyBinding::new("cmd-o", OpenRepository, None),
        KeyBinding::new("shift-cmd-n", app::NewWorkspace, None),
        KeyBinding::new("cmd-\\", ToggleSidebar, None),
        KeyBinding::new("cmd-k", OpenPalette, None),
        KeyBinding::new("cmd-1", SplitDiffAction, None),
        KeyBinding::new("cmd-2", UnifiedDiffAction, None),
        KeyBinding::new("cmd-,", OpenSettings, None),
        KeyBinding::new("cmd-enter", OpenCommitAction, None),
        KeyBinding::new("cmd-shift-f", FetchRepository, None),
        KeyBinding::new("cmd-shift-h", OpenHistoryView, None),
        KeyBinding::new("cmd-shift-o", OpenOverviewView, None),
        KeyBinding::new("cmd-shift-r", ToggleReviewedAction, None),
        KeyBinding::new("cmd-shift-a", StageAllAction, None),
        KeyBinding::new("cmd-shift-c", CopyPrDescriptionAction, None),
        KeyBinding::new("cmd-shift-b", OpenBranchesAction, None),
        KeyBinding::new("j", NextFileAction, None),
        KeyBinding::new("k", PreviousFileAction, None),
        KeyBinding::new("s", ToggleDiffLayoutAction, None),
        KeyBinding::new("escape", CloseOverlayAction, None),
        KeyBinding::new("cmd-q", Quit, None),
    ]);
}

fn install_menus(cx: &mut App) {
    cx.set_menus(vec![
        Menu {
            name: "FlashBee".into(),
            items: vec![
                MenuItem::action("Settings…", OpenSettings),
                MenuItem::separator(),
                MenuItem::os_submenu("Services", SystemMenuType::Services),
                MenuItem::separator(),
                MenuItem::action("Quit FlashBee", Quit),
            ],
        },
        Menu {
            name: "File".into(),
            items: vec![
                MenuItem::action("Add Repository…", OpenRepository),
                MenuItem::action("New Workspace…", app::NewWorkspace),
                MenuItem::separator(),
                MenuItem::action("Refresh", RefreshRepository),
            ],
        },
        Menu {
            name: "View".into(),
            items: vec![
                MenuItem::action("Overview", OpenOverviewView),
                MenuItem::action("History", OpenHistoryView),
                MenuItem::action("Branches…", OpenBranchesAction),
                MenuItem::separator(),
                MenuItem::action("Split Diff", SplitDiffAction),
                MenuItem::action("Unified Diff", UnifiedDiffAction),
                MenuItem::separator(),
                MenuItem::action("Toggle Sidebar", ToggleSidebar),
                MenuItem::action("Quick Open", OpenPalette),
            ],
        },
    ]);
}

fn install_action_handlers(cx: &mut App, view: &gpui::Entity<FlashBeeApp>) {
    macro_rules! on_action {
        ($action:ty, $command:expr) => {{
            let view = view.clone();
            cx.on_action(move |_: &$action, cx| {
                view.update(cx, |app, cx| app.handle_key($command, cx));
            });
        }};
    }

    on_action!(RefreshRepository, "refresh");
    on_action!(OpenRepository, "open-repository");
    on_action!(app::NewWorkspace, "new-workspace");
    on_action!(ToggleSidebar, "toggle-sidebar");
    on_action!(OpenPalette, "open-palette");
    on_action!(SplitDiffAction, "split");
    on_action!(UnifiedDiffAction, "unified");
    on_action!(OpenSettings, "settings");
    on_action!(NextFileAction, "next-file");
    on_action!(PreviousFileAction, "previous-file");
    on_action!(OpenCommitAction, "commit");
    on_action!(FetchRepository, "fetch");
    on_action!(OpenHistoryView, "history");
    on_action!(OpenOverviewView, "overview");
    on_action!(ToggleReviewedAction, "toggle-reviewed");
    on_action!(StageAllAction, "stage-all");
    on_action!(CopyPrDescriptionAction, "copy-pr");
    on_action!(OpenBranchesAction, "branches");
    on_action!(ToggleDiffLayoutAction, "toggle-diff");
    on_action!(CloseOverlayAction, "close-overlay");
}
