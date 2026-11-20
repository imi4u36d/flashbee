use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn safe_store_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .filter(|value| value.is_ascii_alphanumeric() || *value == '-' || *value == '_')
        .collect();
    if sanitized.is_empty() {
        "store".to_string()
    } else {
        sanitized
    }
}

fn store_path(app: &AppHandle, name: &str) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory.join(format!("{}.json", safe_store_name(name))))
}

#[tauri::command]
pub fn load_store(app: AppHandle, name: String) -> Result<Option<String>, String> {
    let path = store_path(&app, &name)?;
    match fs::read_to_string(&path) {
        Ok(contents) => Ok(Some(contents)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

#[tauri::command]
pub fn save_store(app: AppHandle, name: String, contents: String) -> Result<(), String> {
    let path = store_path(&app, &name)?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, contents).map_err(|error| error.to_string())?;
    fs::rename(&temporary, &path).map_err(|error| error.to_string())?;
    Ok(())
}
