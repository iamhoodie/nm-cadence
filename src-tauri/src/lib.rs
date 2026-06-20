mod models;
mod vault;

use models::*;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_updater::{Update, UpdaterExt};
use url::Url;

/// App-wide state: the path to the active markdown vault.
struct AppState {
    vault: Mutex<PathBuf>,
}

struct UpdaterState {
    endpoint: Option<String>,
    pubkey: Option<String>,
    pending: Mutex<Option<Update>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdaterStatus {
    current_version: String,
    configured: bool,
    endpoint: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateMetadata {
    version: String,
    current_version: String,
    notes: Option<String>,
    pub_date: Option<String>,
}

fn updater_state() -> UpdaterState {
    UpdaterState {
        endpoint: option_env!("CADENCE_UPDATER_ENDPOINT").map(str::to_string),
        pubkey: option_env!("CADENCE_UPDATER_PUBKEY").map(str::to_string),
        pending: Mutex::new(None),
    }
}

fn default_vault() -> PathBuf {
    // ~/Documents/CadenceVault (falls back to current dir if Documents is unavailable).
    dirs_document()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("CadenceVault")
}

/// Minimal Documents-dir lookup without pulling in the `dirs` crate.
fn dirs_document() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join("Documents"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var_os("HOME").map(|p| PathBuf::from(p).join("Documents"))
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_vault_path(state: State<AppState>) -> String {
    state.vault.lock().unwrap().to_string_lossy().to_string()
}

#[tauri::command]
fn set_vault_path(path: String, state: State<AppState>) -> Result<(), String> {
    let p = PathBuf::from(&path);
    vault::ensure_vault(&p).map_err(|e| e.to_string())?;
    *state.vault.lock().unwrap() = p;
    Ok(())
}

#[tauri::command]
async fn pick_vault(app: tauri::AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .blocking_pick_folder()
        .map(|p| p.to_string())
}

#[tauri::command]
fn list_people(state: State<AppState>) -> Result<Vec<Person>, String> {
    let v = state.vault.lock().unwrap().clone();
    vault::list_people(&v).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_person(slug: String, state: State<AppState>) -> Result<Person, String> {
    let v = state.vault.lock().unwrap().clone();
    vault::get_person(&v, &slug).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_conversation(
    slug: String,
    conv: Conversation,
    state: State<AppState>,
) -> Result<Person, String> {
    let v = state.vault.lock().unwrap().clone();
    vault::append_conversation(&v, &slug, &conv).map_err(|e| e.to_string())?;
    vault::get_person(&v, &slug).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_tasks(state: State<AppState>) -> Result<Vec<Task>, String> {
    let v = state.vault.lock().unwrap().clone();
    vault::list_tasks(&v).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_tasks(tasks: Vec<Task>, state: State<AppState>) -> Result<(), String> {
    let v = state.vault.lock().unwrap().clone();
    vault::save_tasks(&v, &tasks).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_person(
    name: String,
    role: String,
    cadence_weeks: u32,
    color: String,
    state: State<AppState>,
) -> Result<Person, String> {
    let v = state.vault.lock().unwrap().clone();
    let fm = Frontmatter {
        name,
        role,
        bio: String::new(),
        cadence_weeks,
        color,
        joined: String::new(),
        next_1on1: String::new(),
        group: String::new(),
    };
    vault::create_person(&v, &fm).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_person(slug: String, state: State<AppState>) -> Result<(), String> {
    let v = state.vault.lock().unwrap().clone();
    vault::delete_person(&v, &slug).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_person(
    slug: String,
    name: String,
    role: String,
    bio: String,
    cadence_weeks: u32,
    color: String,
    next_1on1: String,
    joined: String,
    group: String,
    state: State<AppState>,
) -> Result<Person, String> {
    let v = state.vault.lock().unwrap().clone();
    let fm = Frontmatter {
        name,
        role,
        bio,
        cadence_weeks,
        color,
        next_1on1,
        joined,
        group,
    };
    vault::update_person(&v, &slug, &fm).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_conversation(
    slug: String,
    date: String,
    title: String,
    state: State<AppState>,
) -> Result<(), String> {
    let v = state.vault.lock().unwrap().clone();
    vault::delete_conversation(&v, &slug, &date, &title).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_conversation(
    slug: String,
    orig_date: String,
    orig_title: String,
    conv: Conversation,
    state: State<AppState>,
) -> Result<Person, String> {
    let v = state.vault.lock().unwrap().clone();
    vault::update_conversation(&v, &slug, &orig_date, &orig_title, &conv)
        .map_err(|e| e.to_string())?;
    vault::get_person(&v, &slug).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_folders(state: State<AppState>) -> Result<Vec<String>, String> {
    let v = state.vault.lock().unwrap().clone();
    vault::list_folders(&v).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_folder(name: String, state: State<AppState>) -> Result<(), String> {
    let v = state.vault.lock().unwrap().clone();
    vault::create_folder(&v, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_folder(name: String, state: State<AppState>) -> Result<(), String> {
    let v = state.vault.lock().unwrap().clone();
    vault::delete_folder(&v, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_updater_status(app: tauri::AppHandle, updater: State<UpdaterState>) -> UpdaterStatus {
    UpdaterStatus {
        current_version: app.package_info().version.to_string(),
        configured: updater.endpoint.is_some() && updater.pubkey.is_some(),
        endpoint: updater.endpoint.clone(),
    }
}

#[tauri::command]
async fn check_for_update(
    app: tauri::AppHandle,
    updater: State<'_, UpdaterState>,
) -> Result<Option<UpdateMetadata>, String> {
    let endpoint = updater
        .endpoint
        .clone()
        .ok_or_else(|| "Updater endpoint is not configured in this build.".to_string())?;
    let pubkey = updater
        .pubkey
        .clone()
        .ok_or_else(|| "Updater public key is not configured in this build.".to_string())?;

    let update = app
        .updater_builder()
        .pubkey(pubkey)
        .endpoints(vec![Url::parse(&endpoint).map_err(|e| e.to_string())?])
        .map_err(|e| e.to_string())?
        .build()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())?;

    let metadata = update.as_ref().map(|update| UpdateMetadata {
        version: update.version.clone(),
        current_version: update.current_version.clone(),
        notes: update.body.clone(),
        pub_date: update.date.map(|date| date.to_string()),
    });

    *updater.pending.lock().unwrap() = update;
    Ok(metadata)
}

#[tauri::command]
async fn install_update(
    app: tauri::AppHandle,
    updater: State<'_, UpdaterState>,
) -> Result<(), String> {
    let update = updater
        .pending
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| "No pending update. Check for updates first.".to_string())?;

    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| e.to_string())?;

    app.restart();
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let vault = default_vault();
    let _ = vault::ensure_vault(&vault);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState {
            vault: Mutex::new(vault),
        })
        .manage(updater_state())
        .invoke_handler(tauri::generate_handler![
            get_vault_path,
            set_vault_path,
            pick_vault,
            list_people,
            get_person,
            add_conversation,
            list_tasks,
            save_tasks,
            create_person,
            delete_person,
            update_person,
            delete_conversation,
            update_conversation,
            list_folders,
            create_folder,
            delete_folder,
            get_updater_status,
            check_for_update,
            install_update,
        ])
        .setup(|app| {
            // Make sure the managed default vault exists on first launch.
            let state: State<AppState> = app.state();
            let v = state.vault.lock().unwrap().clone();
            let _ = vault::ensure_vault(&v);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Cadence");
}
