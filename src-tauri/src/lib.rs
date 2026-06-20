mod models;
mod vault;

use models::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::menu::{MenuBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{ActivationPolicy, Emitter, Manager, State, WindowEvent};
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

struct TrayState {
    icon: Mutex<Option<TrayIcon>>,
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BackupResult {
    path: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct MenuActionPayload {
    action: String,
    detail: Option<MenuActionDetail>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct MenuActionDetail {
    slug: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
struct PersistedConfig {
    #[serde(default)]
    vault_path: Option<String>,
    #[serde(default)]
    settings: AppSettings,
}

fn updater_state() -> UpdaterState {
    UpdaterState {
        endpoint: option_env!("CADENCE_UPDATER_ENDPOINT").map(str::to_string),
        pubkey: option_env!("CADENCE_UPDATER_PUBKEY").map(str::to_string),
        pending: Mutex::new(None),
    }
}

fn emit_menu_action(app: &tauri::AppHandle, action: &str, detail: Option<MenuActionDetail>) {
    let _ = app.emit(
        "menu-action",
        MenuActionPayload {
            action: action.to_string(),
            detail,
        },
    );
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn build_tray_menu<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    vault_path: &PathBuf,
) -> tauri::Result<tauri::menu::Menu<R>> {
    let people = vault::list_people(vault_path).unwrap_or_default();

    let mut one_on_one_menu = SubmenuBuilder::new(app, "New 1:1 for…");
    if people.is_empty() {
      one_on_one_menu = one_on_one_menu.text("new_1on1_empty", "No people yet");
    } else {
      for person in people {
        one_on_one_menu = one_on_one_menu.text(
            format!("new_1on1::{}", person.slug),
            person.name,
        );
      }
    }

    MenuBuilder::new(app)
        .text("open_window", "Open SideEye")
        .separator()
        .text("show_people", "Show People")
        .text("show_tasks", "Show Tasks")
        .separator()
        .text("new_person", "New Person")
        .text("new_task", "New Task")
        .item(&one_on_one_menu.build()?)
        .separator()
        .item(&PredefinedMenuItem::quit(app, Some("Quit SideEye"))?)
        .build()
}

fn refresh_tray_menu<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    tray_state: &TrayState,
    vault_path: &PathBuf,
) {
    if let Ok(menu) = build_tray_menu(app, vault_path) {
        if let Some(tray) = tray_state.icon.lock().unwrap().as_ref() {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

fn default_vault() -> PathBuf {
    // ~/Documents/CadenceVault (falls back to current dir if Documents is unavailable).
    dirs_document()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("CadenceVault")
}

fn app_config_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Library")
            .join("Application Support")
            .join("SideEye")
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("."))
            .join("SideEye")
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sideeye")
    }
}

fn app_config_file() -> PathBuf {
    app_config_dir().join("config.json")
}

fn read_persisted_config() -> PersistedConfig {
    let path = app_config_file();
    if !path.exists() {
        return PersistedConfig::default();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<PersistedConfig>(&content).ok())
        .unwrap_or_default()
}

fn write_persisted_config(config: &PersistedConfig) -> Result<(), String> {
    fs::create_dir_all(app_config_dir()).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(app_config_file(), json).map_err(|e| e.to_string())
}

fn initial_vault() -> PathBuf {
    let config = read_persisted_config();
    config
        .vault_path
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(default_vault)
}

fn current_app_settings(vault: &PathBuf) -> AppSettings {
    let config = read_persisted_config();
    if config != PersistedConfig::default() || app_config_file().exists() {
        return config.settings;
    }
    vault::read_legacy_app_settings(vault).unwrap_or_default()
}

fn copy_dir_recursive(source: &PathBuf, target: &PathBuf) -> Result<(), String> {
    fs::create_dir_all(target).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
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
fn set_vault_path(
    path: String,
    state: State<AppState>,
    tray: State<TrayState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let p = PathBuf::from(&path);
    vault::ensure_vault(&p).map_err(|e| e.to_string())?;
    *state.vault.lock().unwrap() = p;
    let mut config = read_persisted_config();
    config.vault_path = Some(path);
    write_persisted_config(&config)?;
    refresh_tray_menu(&app, &tray, &state.vault.lock().unwrap().clone());
    Ok(())
}

#[tauri::command]
fn create_vault_backup(state: State<AppState>) -> Result<BackupResult, String> {
    let source = state.vault.lock().unwrap().clone();
    vault::ensure_vault(&source).map_err(|e| e.to_string())?;

    let vault_name = source
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("vault");

    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let backups_root = source
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("SideEye Backups");
    let target = backups_root.join(format!("{vault_name}-{stamp}"));

    copy_dir_recursive(&source, &target)?;

    Ok(BackupResult {
        path: target.to_string_lossy().to_string(),
    })
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
    let settings = current_app_settings(&v);
    vault::list_tasks(&v, &settings).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_tasks(tasks: Vec<Task>, state: State<AppState>) -> Result<(), String> {
    let v = state.vault.lock().unwrap().clone();
    let settings = current_app_settings(&v);
    vault::save_tasks(&v, &tasks, &settings).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_app_settings(state: State<AppState>) -> Result<AppSettings, String> {
    let v = state.vault.lock().unwrap().clone();
    Ok(current_app_settings(&v))
}

#[tauri::command]
fn save_app_settings(settings: AppSettings, state: State<AppState>) -> Result<AppSettings, String> {
    let v = state.vault.lock().unwrap().clone();
    let mut config = read_persisted_config();
    if config.vault_path.is_none() {
        config.vault_path = Some(v.to_string_lossy().to_string());
    }
    config.settings = settings.clone();
    write_persisted_config(&config)?;
    let current_tasks = vault::list_tasks(&v, &settings).map_err(|e| e.to_string())?;
    vault::save_tasks(&v, &current_tasks, &settings).map_err(|e| e.to_string())?;
    Ok(settings)
}

#[tauri::command]
fn create_person(
    name: String,
    role: String,
    color: String,
    group: String,
    state: State<AppState>,
    tray: State<TrayState>,
    app: tauri::AppHandle,
) -> Result<Person, String> {
    let v = state.vault.lock().unwrap().clone();
    let fm = Frontmatter {
        name,
        role,
        bio: String::new(),
        color,
        group,
    };
    let person = vault::create_person(&v, &fm).map_err(|e| e.to_string())?;
    refresh_tray_menu(&app, &tray, &v);
    Ok(person)
}

#[tauri::command]
fn delete_person(
    slug: String,
    state: State<AppState>,
    tray: State<TrayState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let v = state.vault.lock().unwrap().clone();
    vault::delete_person(&v, &slug).map_err(|e| e.to_string())?;
    refresh_tray_menu(&app, &tray, &v);
    Ok(())
}

#[tauri::command]
fn update_person(
    slug: String,
    name: String,
    role: String,
    bio: String,
    color: String,
    group: String,
    state: State<AppState>,
    tray: State<TrayState>,
    app: tauri::AppHandle,
) -> Result<Person, String> {
    let v = state.vault.lock().unwrap().clone();
    let fm = Frontmatter {
        name,
        role,
        bio,
        color,
        group,
    };
    let person = vault::update_person(&v, &slug, &fm).map_err(|e| e.to_string())?;
    refresh_tray_menu(&app, &tray, &v);
    Ok(person)
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
fn list_folders(state: State<AppState>) -> Result<Vec<Folder>, String> {
    let v = state.vault.lock().unwrap().clone();
    vault::list_folders(&v).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_folder(name: String, color: String, state: State<AppState>) -> Result<(), String> {
    let v = state.vault.lock().unwrap().clone();
    vault::create_folder(&v, &name, &color).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_folder(name: String, next_name: String, color: String, state: State<AppState>) -> Result<(), String> {
    let v = state.vault.lock().unwrap().clone();
    vault::update_folder(&v, &name, &next_name, &color).map_err(|e| e.to_string())
}

#[tauri::command]
fn reorder_folders(names: Vec<String>, state: State<AppState>) -> Result<(), String> {
    let v = state.vault.lock().unwrap().clone();
    vault::reorder_folders(&v, &names).map_err(|e| e.to_string())
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
    let vault = initial_vault();
    let _ = vault::ensure_vault(&vault);
    let context_menu_guard = r#"
      window.addEventListener('contextmenu', (event) => {
        event.preventDefault();
      }, true);
    "#;

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .on_page_load(move |webview, _| {
            let _ = webview.eval(context_menu_guard);
        })
        .manage(AppState {
            vault: Mutex::new(vault),
        })
        .manage(updater_state())
        .manage(TrayState {
            icon: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            get_vault_path,
            set_vault_path,
            create_vault_backup,
            pick_vault,
            list_people,
            get_person,
            add_conversation,
            list_tasks,
            save_tasks,
            get_app_settings,
            save_app_settings,
            create_person,
            delete_person,
            update_person,
            delete_conversation,
            update_conversation,
            list_folders,
            create_folder,
            update_folder,
            reorder_folders,
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

            let tray_menu = build_tray_menu(&app.handle(), &v)?;

            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(ActivationPolicy::Accessory);
                app.set_dock_visibility(false);
            }

            let tray_icon = TrayIconBuilder::with_id("sideeye-tray")
                .icon(app.default_window_icon().cloned().expect("default icon missing"))
                .icon_as_template(false)
                .tooltip("SideEye")
                .menu(&tray_menu)
                .show_menu_on_left_click(true)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(&tray.app_handle());
                    }
                })
                .build(app)?;

            let tray_state: State<TrayState> = app.state();
            *tray_state.icon.lock().unwrap() = Some(tray_icon);

            show_main_window(&app.handle());
            Ok(())
        })
        .on_window_event(|window, event| {
            #[cfg(target_os = "macos")]
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .on_menu_event(|app, event| {
            let action = match event.id().as_ref() {
                "open_window" => {
                    show_main_window(app);
                    None
                }
                "show_people" => Some(("show-people", None)),
                "show_tasks" => Some(("show-tasks", None)),
                "new_person" => Some(("new-person", None)),
                "new_task" => Some(("new-task", None)),
                "new_1on1_empty" => None,
                id if id.starts_with("new_1on1::") => Some((
                    "new-1on1",
                    Some(MenuActionDetail {
                        slug: id.trim_start_matches("new_1on1::").to_string(),
                    }),
                )),
                _ => None,
            };

            if let Some((action, detail)) = action {
                show_main_window(app);
                emit_menu_action(app, action, detail);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Cadence");
}
