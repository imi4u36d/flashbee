mod git;
mod menu;
mod storage;
mod window_state;

use serde::Serialize;
use std::process::Command;
use tauri::{AppHandle, Emitter, Manager, Window};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    name: &'static str,
    version: &'static str,
    identifier: &'static str,
}

#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        name: "FlashBee",
        version: env!("CARGO_PKG_VERSION"),
        identifier: "com.flashbee.app",
    }
}

#[tauri::command]
fn app_ready(window: Window) -> Result<(), String> {
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
fn set_window_title(window: Window, title: String) -> Result<(), String> {
    window.set_title(&title).map_err(|error| error.to_string())
}

#[tauri::command]
async fn inspect_repository(path: String) -> Result<git::RepositoryInspection, git::CommandError> {
    tauri::async_runtime::spawn_blocking(move || git::inspect_repository(&path))
        .await
        .map_err(|error| git::CommandError::new("internal_error", error.to_string()))?
}

#[tauri::command]
async fn discover_repositories(
    path: String,
) -> Result<git::RepositoryDiscovery, git::CommandError> {
    tauri::async_runtime::spawn_blocking(move || git::discover_repositories(&path))
        .await
        .map_err(|error| git::CommandError::new("internal_error", error.to_string()))?
}

#[tauri::command]
async fn initialize_repository(
    path: String,
) -> Result<git::RepositoryInspection, git::CommandError> {
    tauri::async_runtime::spawn_blocking(move || git::initialize_repository(&path))
        .await
        .map_err(|error| git::CommandError::new("internal_error", error.to_string()))?
}

#[tauri::command]
async fn repository_snapshot(
    path: String,
    base: Option<String>,
) -> Result<git::RepositorySnapshot, git::CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        git::repository_snapshot(&path, base.as_deref())
    })
    .await
    .map_err(|error| git::CommandError::new("internal_error", error.to_string()))?
}

#[tauri::command]
async fn blame_file(
    path: String,
    file_path: String,
    revision: Option<String>,
    lines: Option<Vec<u32>>,
) -> Result<Vec<git::BlameLine>, git::CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        git::blame_file(&path, &file_path, revision.as_deref(), lines)
    })
    .await
    .map_err(|error| git::CommandError::new("internal_error", error.to_string()))?
}

#[tauri::command]
async fn list_remotes(path: String) -> Result<Vec<git::RemoteInfo>, git::CommandError> {
    tauri::async_runtime::spawn_blocking(move || git::list_remotes(&path))
        .await
        .map_err(|error| git::CommandError::new("internal_error", error.to_string()))?
}

#[tauri::command]
async fn set_remote_url(
    path: String,
    name: String,
    url: String,
) -> Result<Vec<git::RemoteInfo>, git::CommandError> {
    tauri::async_runtime::spawn_blocking(move || git::set_remote_url(&path, &name, &url))
        .await
        .map_err(|error| git::CommandError::new("internal_error", error.to_string()))?
}

#[tauri::command]
async fn pull_repository(path: String) -> Result<git::SyncOutcome, git::CommandError> {
    tauri::async_runtime::spawn_blocking(move || git::pull_repository(&path))
        .await
        .map_err(|error| git::CommandError::new("internal_error", error.to_string()))?
}

#[tauri::command]
async fn push_repository(path: String) -> Result<git::SyncOutcome, git::CommandError> {
    tauri::async_runtime::spawn_blocking(move || git::push_repository(&path))
        .await
        .map_err(|error| git::CommandError::new("internal_error", error.to_string()))?
}

#[tauri::command]
async fn read_conflict(
    path: String,
    file_path: String,
) -> Result<git::ConflictDetail, git::CommandError> {
    tauri::async_runtime::spawn_blocking(move || git::read_conflict(&path, &file_path))
        .await
        .map_err(|error| git::CommandError::new("internal_error", error.to_string()))?
}

#[tauri::command]
async fn save_conflict_resolution(
    path: String,
    file_path: String,
    content: String,
) -> Result<(), git::CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        git::save_conflict_resolution(&path, &file_path, &content)
    })
    .await
    .map_err(|error| git::CommandError::new("internal_error", error.to_string()))?
}

#[tauri::command]
async fn resolve_conflict_side(
    path: String,
    file_path: String,
    side: String,
) -> Result<(), git::CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        git::resolve_conflict_side(&path, &file_path, &side)
    })
    .await
    .map_err(|error| git::CommandError::new("internal_error", error.to_string()))?
}

#[tauri::command]
async fn complete_merge(path: String) -> Result<git::SyncOutcome, git::CommandError> {
    tauri::async_runtime::spawn_blocking(move || git::complete_merge(&path))
        .await
        .map_err(|error| git::CommandError::new("internal_error", error.to_string()))?
}

#[tauri::command]
async fn abort_merge(path: String) -> Result<(), git::CommandError> {
    tauri::async_runtime::spawn_blocking(move || git::abort_merge(&path))
        .await
        .map_err(|error| git::CommandError::new("internal_error", error.to_string()))?
}

#[tauri::command]
fn set_menu_locale(app: AppHandle, locale: String) -> Result<(), String> {
    let locale = menu::Locale::from_tag(&locale);
    let menu = menu::build(&app, locale).map_err(|error| error.to_string())?;
    app.set_menu(menu).map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
fn reveal_in_finder(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        return open_with_args(&["-R", &path]);
    }
    #[cfg(not(target_os = "macos"))]
    {
        open_path(path)
    }
}

#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let status = Command::new("open").arg(&path).status();
    #[cfg(target_os = "windows")]
    let status = Command::new("explorer").arg(&path).status();
    #[cfg(all(unix, not(target_os = "macos")))]
    let status = Command::new("xdg-open").arg(&path).status();

    report_status(status, "The folder could not be opened.")
}

#[tauri::command]
fn open_in_editor(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        for application in [
            "Visual Studio Code",
            "Cursor",
            "Zed",
            "Sublime Text",
            "Nova",
        ] {
            let status = Command::new("open")
                .arg("-a")
                .arg(application)
                .arg(&path)
                .status();
            if matches!(status, Ok(status) if status.success()) {
                return Ok(());
            }
        }
    }

    open_path(path)
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err("Only http and https links can be opened.".to_string());
    }

    #[cfg(target_os = "macos")]
    let status = Command::new("open").arg(&url).status();
    #[cfg(target_os = "windows")]
    let status = Command::new("cmd").args(["/C", "start", "", &url]).status();
    #[cfg(all(unix, not(target_os = "macos")))]
    let status = Command::new("xdg-open").arg(&url).status();

    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => Err("The link could not be opened.".to_string()),
        Err(error) => Err(error.to_string()),
    }
}

fn report_status(
    status: std::io::Result<std::process::ExitStatus>,
    failure: &str,
) -> Result<(), String> {
    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => Err(failure.to_string()),
        Err(error) => Err(error.to_string()),
    }
}

#[cfg(target_os = "macos")]
fn open_with_args(args: &[&str]) -> Result<(), String> {
    let status = Command::new("open").args(args).status();
    report_status(status, "The folder could not be opened.")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            app_info,
            app_ready,
            set_window_title,
            inspect_repository,
            discover_repositories,
            initialize_repository,
            repository_snapshot,
            blame_file,
            list_remotes,
            set_remote_url,
            pull_repository,
            push_repository,
            read_conflict,
            save_conflict_resolution,
            resolve_conflict_side,
            complete_merge,
            abort_merge,
            set_menu_locale,
            reveal_in_finder,
            open_path,
            open_in_editor,
            open_url,
            storage::load_store,
            storage::save_store
        ])
        .setup(|app| {
            let locale = sys_locale::get_locale()
                .map(|tag| menu::Locale::from_tag(&tag))
                .unwrap_or(menu::Locale::En);
            let menu = menu::build(app.handle(), locale)?;
            app.set_menu(menu)?;
            window_state::restore(app.handle());

            // The window starts hidden to avoid a white flash. If the frontend
            // fails to boot, still reveal it instead of leaving a ghost process.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(1800));
                if let Some(window) = handle.get_webview_window("main") {
                    if !window.is_visible().unwrap_or(true) {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            });
            Ok(())
        })
        .on_menu_event(|app, event| {
            let id = event.id().as_ref().to_string();
            match id.as_str() {
                "open-github" => {
                    if let Err(error) =
                        open_url("https://github.com/imi4u36d/flashbee".to_string())
                    {
                        eprintln!("Failed to open GitHub: {error}");
                    }
                    return;
                }
                "open-help" => {
                    if let Err(error) =
                        open_url("https://github.com/imi4u36d/flashbee#readme".to_string())
                    {
                        eprintln!("Failed to open help: {error}");
                    }
                    return;
                }
                _ => {}
            }
            let _ = app.emit("menu-command", id);
        })
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                window_state::save(window.app_handle());
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building FlashBee")
        .run(|app, event| {
            if matches!(
                event,
                tauri::RunEvent::Exit | tauri::RunEvent::ExitRequested { .. }
            ) {
                window_state::save(app);
            }
        });
}
