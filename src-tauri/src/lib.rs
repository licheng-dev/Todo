mod shortcut;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fs, sync::Mutex};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

#[cfg(target_os = "macos")]
use tauri::WebviewWindow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    id: u64,
    text: String,
    done: bool,
    #[serde(default)]
    created_at: u64,
    #[serde(default)]
    completed_at: Option<u64>,
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn load_json<T: Default + DeserializeOwned>(app: &AppHandle, name: &str) -> T {
    app.path()
        .app_config_dir()
        .ok()
        .map(|d| d.join(name))
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save_json<T: Serialize>(app: &AppHandle, name: &str, data: &T) {
    if let Some(dir) = app.path().app_config_dir().ok() {
        let _ = fs::create_dir_all(&dir);
        if let Ok(raw) = serde_json::to_string_pretty(data) {
            let _ = fs::write(dir.join(name), raw);
        }
    }
}

fn io_err(msg: &str) -> tauri::Error {
    tauri::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, msg))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppSettings {
    #[serde(default)]
    always_on_top: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self { always_on_top: false }
    }
}

pub struct AppState {
    settings: Mutex<AppSettings>,
}

#[cfg(target_os = "macos")]
fn center_window(window: &WebviewWindow) -> tauri::Result<()> {
    let monitor = window
        .current_monitor()?
        .or(window.primary_monitor()?)
        .ok_or_else(|| io_err("no monitor available"))?;

    let work = monitor.work_area();
    let size = window.outer_size()?;
    let x = work.position.x + (work.size.width as i32 - size.width as i32) / 2;
    let y = work.position.y + (work.size.height as i32 - size.height as i32) / 2;

    use tauri::{PhysicalPosition, Position};
    window.set_position(Position::Physical(PhysicalPosition::new(x, y)))?;
    Ok(())
}

fn apply_always_on_top(app: &AppHandle, enabled: bool) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_always_on_top(enabled);
        #[cfg(target_os = "macos")]
        {
            let _ = window.set_visible_on_all_workspaces(enabled);
        }
    }
}

#[cfg(target_os = "macos")]
fn ensure_popup(app: &AppHandle, label: &str, title: &str, w: f64, h: f64) -> WebviewWindow {
    match app.get_webview_window(label) {
        Some(old) => old,
        None => WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
            .title(title)
            .inner_size(w, h)
            .center()
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .shadow(true)
            .visible(false)
            .build()
            .expect("failed to create popup window"),
    }
}

#[cfg(not(target_os = "macos"))]
fn ensure_popup(app: &AppHandle, label: &str, title: &str, w: f64, h: f64) -> tauri::WebviewWindow {
    match app.get_webview_window(label) {
        Some(old) => old,
        None => WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
            .title(title)
            .inner_size(w, h)
            .center()
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .shadow(true)
            .visible(false)
            .build()
            .expect("failed to create popup window"),
    }
}

fn emit_todos(app: &AppHandle) {
    let todos: Vec<Todo> = load_json(app, "todos.json");
    let _ = app.emit("todos-changed", todos);
}

#[cfg(target_os = "macos")]
fn unhide_app(app: &AppHandle) {
    let _ = app.show();
}

#[cfg(not(target_os = "macos"))]
fn unhide_app(_app: &AppHandle) {}

#[tauri::command]
fn open_settings(app: AppHandle) {
    unhide_app(&app);
    let w = ensure_popup(&app, "settings", "设置", 360.0, 300.0);
    let _ = w.show();
    let _ = w.set_focus();
}

#[tauri::command]
fn get_always_on_top(state: tauri::State<AppState>) -> bool {
    state.settings.lock().unwrap().always_on_top
}

#[tauri::command]
fn set_always_on_top(
    app: AppHandle,
    state: tauri::State<AppState>,
    enabled: bool,
) -> Result<(), String> {
    {
        let mut settings = state.settings.lock().unwrap();
        settings.always_on_top = enabled;
    }
    apply_always_on_top(&app, enabled);
    save_json(&app, "settings.json", &*state.settings.lock().unwrap());
    Ok(())
}

#[tauri::command]
fn hide_window(app: AppHandle) {
    #[cfg(target_os = "macos")]
    {
        let _ = app.hide();
    }
    #[cfg(not(target_os = "macos"))]
    {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.hide();
        }
    }
}

#[tauri::command]
fn minimize_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.minimize();
    }
}

#[tauri::command]
fn get_todos(app: AppHandle) -> Vec<Todo> {
    load_json(&app, "todos.json")
}

#[tauri::command]
fn add_todo(app: AppHandle, text: String) -> Result<(), String> {
    if text.trim().is_empty() {
        return Ok(());
    }
    let mut todos: Vec<Todo> = load_json(&app, "todos.json");
    todos.push(Todo {
        id: now_ms(),
        text: text.trim().to_string(),
        done: false,
        created_at: now_ms(),
        completed_at: None,
    });
    save_json(&app, "todos.json", &todos);
    emit_todos(&app);
    Ok(())
}

#[tauri::command]
fn delete_todo(app: AppHandle, id: u64) -> Result<(), String> {
    let mut todos: Vec<Todo> = load_json(&app, "todos.json");
    todos.retain(|item| item.id != id);
    save_json(&app, "todos.json", &todos);
    emit_todos(&app);
    Ok(())
}

#[tauri::command]
fn toggle_todo(app: AppHandle, id: u64, done: bool) -> Result<(), String> {
    let mut todos: Vec<Todo> = load_json(&app, "todos.json");
    if let Some(item) = todos.iter_mut().find(|item| item.id == id) {
        item.done = done;
        item.completed_at = if done { Some(now_ms()) } else { None };
    }
    save_json(&app, "todos.json", &todos);
    emit_todos(&app);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    builder
        .setup(|app| {
            let settings: AppSettings = load_json(app.handle(), "settings.json");

            app.manage(AppState {
                settings: Mutex::new(settings.clone()),
            });

            apply_always_on_top(app.handle(), settings.always_on_top);

            #[cfg(target_os = "macos")]
            {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = center_window(&window);
                }
            }

            #[cfg(target_os = "macos")]
            {
                let handle = app.handle().clone();
                let (tx, rx) = std::sync::mpsc::channel();
                if let Err(err) = shortcut::install(tx) {
                    eprintln!("global shortcuts unavailable: {err}");
                } else {
                    std::thread::spawn(move || {
                        while let Ok(kind) = rx.recv() {
                            match kind {
                                shortcut::ShortcutKind::AddTodo => {
                                    unhide_app(&handle);
                                    let w = ensure_popup(&handle, "add-todo", "添加待办", 480.0, 260.0);
                                    let _ = w.show();
                                    let _ = w.set_focus();
                                }
                                shortcut::ShortcutKind::Settings => {
                                    unhide_app(&handle);
                                    let w = ensure_popup(&handle, "settings", "设置", 360.0, 300.0);
                                    let _ = w.show();
                                    let _ = w.set_focus();
                                }
                            }
                        }
                    });
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_settings,
            get_always_on_top,
            set_always_on_top,
            hide_window,
            minimize_window,
            get_todos,
            add_todo,
            delete_todo,
            toggle_todo,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Reopen {
                has_visible_windows,
                ..
            } = event
            {
                if !has_visible_windows {
                    #[cfg(target_os = "macos")]
                    let _ = app_handle.show();
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                    }
                }
            }
        });
}