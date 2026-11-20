use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Runtime, WebviewWindow};

const MIN_WIDTH: f64 = 960.0;
const MIN_HEIGHT: f64 = 620.0;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SavedWindowState {
    width: f64,
    height: f64,
    x: f64,
    y: f64,
    #[serde(default)]
    maximized: bool,
    #[serde(default)]
    fullscreen: bool,
}

impl Default for SavedWindowState {
    fn default() -> Self {
        Self {
            width: 1360.0,
            height: 880.0,
            x: 0.0,
            y: 0.0,
            maximized: false,
            fullscreen: false,
        }
    }
}

fn state_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_data_dir()
        .ok()
        .map(|directory| directory.join("window-state.json"))
}

fn load(app: &AppHandle) -> Option<SavedWindowState> {
    let path = state_path(app)?;
    let contents = fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}

fn write(app: &AppHandle, state: &SavedWindowState) {
    let Some(path) = state_path(app) else {
        return;
    };
    if let Some(directory) = path.parent() {
        let _ = fs::create_dir_all(directory);
    }
    if let Ok(contents) = serde_json::to_string_pretty(state) {
        let _ = fs::write(path, contents);
    }
}

fn monitor_contains<R: Runtime>(
    window: &WebviewWindow<R>,
    state: &SavedWindowState,
) -> bool {
    let Ok(monitors) = window.available_monitors() else {
        return true;
    };

    monitors.iter().any(|monitor| {
        let scale = monitor.scale_factor();
        let position = monitor.position().to_logical::<f64>(scale);
        let size = monitor.size().to_logical::<f64>(scale);
        state.x < position.x + size.width
            && state.x + state.width > position.x
            && state.y < position.y + size.height
            && state.y + state.height > position.y
    })
}

pub fn restore(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let Some(state) = load(app) else {
        return;
    };

    if state.fullscreen {
        let _ = window.set_fullscreen(true);
        return;
    }

    let width = if state.width >= MIN_WIDTH {
        state.width
    } else {
        MIN_WIDTH
    };
    let height = if state.height >= MIN_HEIGHT {
        state.height
    } else {
        MIN_HEIGHT
    };
    let _ = window.set_size(LogicalSize::new(width, height));

    let bounded = SavedWindowState {
        width,
        height,
        ..state.clone()
    };
    if monitor_contains(&window, &bounded) {
        let _ = window.set_position(LogicalPosition::new(state.x, state.y));
    } else {
        let _ = window.center();
    }

    if state.maximized {
        let _ = window.maximize();
    }
}

pub fn save(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if window.is_minimized().unwrap_or(false) {
        return;
    }

    let maximized = window.is_maximized().unwrap_or(false);
    let fullscreen = window.is_fullscreen().unwrap_or(false);
    let mut state = load(app).unwrap_or_default();
    state.maximized = maximized;
    state.fullscreen = fullscreen;

    if !maximized && !fullscreen {
        let scale = window.scale_factor().unwrap_or(1.0);
        if let Ok(size) = window.inner_size() {
            let logical = size.to_logical::<f64>(scale);
            if logical.width >= MIN_WIDTH && logical.height >= MIN_HEIGHT {
                state.width = logical.width;
                state.height = logical.height;
            }
        }
        if let Ok(position) = window.outer_position() {
            let logical = position.to_logical::<f64>(scale);
            state.x = logical.x;
            state.y = logical.y;
        }
    }

    write(app, &state);
}
