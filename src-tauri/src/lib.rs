mod shortcut;

use chrono::Local;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    fs,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Duration,
};
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
    #[serde(default)]
    defer_until: Option<u64>,
    #[serde(default)]
    pinned: bool,
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
#[serde(default)]
struct ShortcutSetting {
    code: String,
    meta: bool,
    ctrl: bool,
    alt: bool,
    shift: bool,
}

impl Default for ShortcutSetting {
    fn default() -> Self {
        Self {
            code: "KeyN".to_string(),
            meta: true,
            ctrl: false,
            alt: false,
            shift: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct AppSettings {
    always_on_top: bool,
    add_shortcut: ShortcutSetting,
    ai_provider: String,
    ai_api_key: String,
    reminder_enabled: bool,
    reminder_time: String,
    reminder_include_pinned: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            always_on_top: false,
            add_shortcut: ShortcutSetting::default(),
            ai_provider: "deepseek".to_string(),
            ai_api_key: String::new(),
            reminder_enabled: false,
            reminder_time: "17:00".to_string(),
            reminder_include_pinned: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReminderSetting {
    enabled: bool,
    time: String,
    include_pinned: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct ReminderState {
    snooze_until: Option<u64>,
    dismissed_on: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AiSettings {
    provider: String,
    api_key: String,
}

const REPO_LATEST_RELEASE_API: &str = "https://api.github.com/repos/licheng-dev/Todo/releases/latest";

#[derive(Debug, Clone, Serialize)]
struct UpdateInfo {
    current: String,
    latest: String,
    has_update: bool,
    url: String,
}

fn parse_version(version: &str) -> Vec<u32> {
    version
        .trim()
        .trim_start_matches(['v', 'V'])
        .split(['.', '-', '+'])
        .map_while(|part| part.parse::<u32>().ok())
        .collect()
}

fn is_newer(latest: &[u32], current: &[u32]) -> bool {
    for i in 0..latest.len().max(current.len()) {
        let l = latest.get(i).copied().unwrap_or(0);
        let c = current.get(i).copied().unwrap_or(0);
        if l != c {
            return l > c;
        }
    }
    false
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum AiEvent {
    Chunk { text: String },
    Done,
    Error { message: String },
}

pub struct AppState {
    settings: Mutex<AppSettings>,
    reminder: Mutex<ReminderState>,
    summarizing: AtomicBool,
    cancel: AtomicBool,
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

fn provider_config(provider: &str) -> Result<(&'static str, &'static str), String> {
    match provider {
        "deepseek" => Ok(("https://api.deepseek.com/chat/completions", "deepseek-chat")),
        "qwen" => Ok((
            "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions",
            "qwen-plus",
        )),
        "kimi" => Ok((
            "https://api.moonshot.cn/v1/chat/completions",
            "moonshot-v1-8k",
        )),
        other => Err(format!("未知的 AI 提供商: {other}")),
    }
}

fn extract_error(text: &str) -> String {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
        if let Some(msg) = json["error"]["message"].as_str() {
            return msg.to_string();
        }
        if let Some(msg) = json["message"].as_str() {
            return msg.to_string();
        }
    }
    if text.is_empty() {
        "未知错误".to_string()
    } else {
        text.chars().take(300).collect()
    }
}

fn emit_sse_line(app: &AppHandle, line: &str) {
    let line = line.trim_end_matches(['\r', '\n']);
    let Some(data) = line.strip_prefix("data:") else {
        return;
    };
    let data = data.trim();
    if data.is_empty() || data == "[DONE]" {
        return;
    }
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
        if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
            if !content.is_empty() {
                let _ = app.emit_to(
                    "main",
                    "ai-summary",
                    AiEvent::Chunk {
                        text: content.to_string(),
                    },
                );
            }
        }
    }
}

async fn stream_summary(
    app: &AppHandle,
    url: &str,
    model: &str,
    api_key: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(|e| format!("初始化请求失败: {e}"))?;

    let body = serde_json::json!({
        "model": model,
        "stream": true,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt },
        ],
    });

    let mut resp = client
        .post(url)
        .bearer_auth(api_key)
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!(
            "服务返回错误 ({status}): {}",
            extract_error(&text)
        ));
    }

    let mut buf = String::new();
    loop {
        if app.state::<AppState>().cancel.load(Ordering::SeqCst) {
            break;
        }
        match resp.chunk().await {
            Ok(Some(bytes)) => {
                buf.push_str(&String::from_utf8_lossy(&bytes));
                while let Some(pos) = buf.find('\n') {
                    let line: String = buf.drain(..=pos).collect();
                    emit_sse_line(app, &line);
                }
            }
            Ok(None) => break,
            Err(e) => return Err(format!("读取响应失败: {e}")),
        }
    }

    if !buf.trim().is_empty() {
        emit_sse_line(app, &buf);
    }

    let _ = app.emit_to("main", "ai-summary", AiEvent::Done);
    Ok(())
}

#[tauri::command]
fn get_ai_settings(state: tauri::State<AppState>) -> AiSettings {
    let settings = state.settings.lock().unwrap();
    AiSettings {
        provider: settings.ai_provider.clone(),
        api_key: settings.ai_api_key.clone(),
    }
}

#[tauri::command]
fn set_ai_settings(
    app: AppHandle,
    state: tauri::State<AppState>,
    provider: String,
    api_key: String,
) -> Result<(), String> {
    if provider_config(&provider).is_err() {
        return Err("不支持的 AI 提供商".to_string());
    }
    {
        let mut settings = state.settings.lock().unwrap();
        settings.ai_provider = provider;
        settings.ai_api_key = api_key.trim().to_string();
    }
    save_json(&app, "settings.json", &*state.settings.lock().unwrap());
    Ok(())
}

#[tauri::command]
fn start_summary(
    app: AppHandle,
    state: tauri::State<AppState>,
    system_prompt: String,
    user_prompt: String,
) -> Result<(), String> {
    let (provider, api_key) = {
        let settings = state.settings.lock().unwrap();
        (settings.ai_provider.clone(), settings.ai_api_key.clone())
    };
    if api_key.trim().is_empty() {
        return Err("尚未配置 API Key，请先在设置中填写".to_string());
    }
    let (url, model) = provider_config(&provider)?;

    if state.summarizing.swap(true, Ordering::SeqCst) {
        return Err("正在生成中，请稍候".to_string());
    }
    state.cancel.store(false, Ordering::SeqCst);

    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let result =
            stream_summary(&handle, url, model, &api_key, &system_prompt, &user_prompt).await;
        if let Err(message) = result {
            let _ = handle.emit_to("main", "ai-summary", AiEvent::Error { message });
        }
        handle
            .state::<AppState>()
            .summarizing
            .store(false, Ordering::SeqCst);
    });

    Ok(())
}

#[tauri::command]
fn cancel_summary(state: tauri::State<AppState>) {
    state.cancel.store(true, Ordering::SeqCst);
}

#[tauri::command]
async fn check_update(app: AppHandle) -> Result<UpdateInfo, String> {
    let current = app.package_info().version.to_string();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("初始化请求失败: {e}"))?;

    let resp = client
        .get(REPO_LATEST_RELEASE_API)
        .header("User-Agent", "zhaomu-updater")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("检查更新失败 (HTTP {status})"));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {e}"))?;

    let tag = json["tag_name"]
        .as_str()
        .ok_or_else(|| "未找到版本信息".to_string())?;
    let url = json["html_url"]
        .as_str()
        .unwrap_or("https://github.com/licheng-dev/Todo/releases")
        .to_string();

    Ok(UpdateInfo {
        current: current.clone(),
        latest: tag.to_string(),
        has_update: is_newer(&parse_version(tag), &parse_version(&current)),
        url,
    })
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(&url).spawn();
    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("cmd")
        .args(["/C", "start", "", &url])
        .spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let result = std::process::Command::new("xdg-open").arg(&url).spawn();

    result.map(|_| ()).map_err(|e| format!("打开链接失败: {e}"))
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
fn get_add_shortcut(state: tauri::State<AppState>) -> ShortcutSetting {
    state.settings.lock().unwrap().add_shortcut.clone()
}

#[tauri::command]
fn set_add_shortcut(
    app: AppHandle,
    state: tauri::State<AppState>,
    shortcut: ShortcutSetting,
) -> Result<(), String> {
    if !shortcut.meta && !shortcut.ctrl && !shortcut.alt {
        return Err("请至少包含 Command、Control 或 Option 中的一个修饰键".to_string());
    }
    shortcut::update_add(
        &shortcut.code,
        shortcut.meta,
        shortcut.ctrl,
        shortcut.alt,
        shortcut.shift,
    )?;
    {
        let mut settings = state.settings.lock().unwrap();
        settings.add_shortcut = shortcut;
    }
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

fn parse_hhmm(value: &str) -> Option<(u32, u32)> {
    let (h, m) = value.split_once(':')?;
    let h: u32 = h.parse().ok()?;
    let m: u32 = m.parse().ok()?;
    if h > 23 || m > 59 {
        return None;
    }
    Some((h, m))
}

fn local_today() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn start_of_tomorrow_ms() -> u64 {
    let today = Local::now().date_naive();
    let tomorrow = today.succ_opt().unwrap_or(today);
    tomorrow
        .and_hms_opt(0, 0, 0)
        .and_then(|dt| dt.and_local_timezone(Local).earliest())
        .map(|dt| dt.timestamp_millis() as u64)
        .unwrap_or_else(|| now_ms() + 24 * 60 * 60 * 1000)
}

fn is_active(todo: &Todo) -> bool {
    !todo.done && todo.defer_until.map_or(true, |d| d <= now_ms())
}

fn reminder_pending_count(app: &AppHandle) -> usize {
    let include_pinned = app
        .state::<AppState>()
        .settings
        .lock()
        .unwrap()
        .reminder_include_pinned;
    let todos: Vec<Todo> = load_json(app, "todos.json");
    todos
        .iter()
        .filter(|t| is_active(t) && (include_pinned || !t.pinned))
        .count()
}

fn hide_reminder_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("reminder") {
        let _ = window.hide();
    }
}

fn maybe_show_reminder(app: &AppHandle) {
    let state = app.state::<AppState>();
    let (enabled, time) = {
        let settings = state.settings.lock().unwrap();
        (settings.reminder_enabled, settings.reminder_time.clone())
    };
    if !enabled {
        return;
    }

    if let Some(window) = app.get_webview_window("reminder") {
        if window.is_visible().unwrap_or(false) {
            return;
        }
    }

    let Some((hour, minute)) = parse_hhmm(&time) else {
        return;
    };
    let now = Local::now();
    let Some(target) = now.date_naive().and_hms_opt(hour, minute, 0) else {
        return;
    };
    if now.naive_local() < target {
        return;
    }

    let today = local_today();
    {
        let rs = state.reminder.lock().unwrap();
        if rs.dismissed_on.as_deref() == Some(today.as_str()) {
            return;
        }
        if rs.snooze_until.is_some_and(|until| until > now_ms()) {
            return;
        }
    }

    if reminder_pending_count(app) == 0 {
        return;
    }

    unhide_app(app);
    let window = ensure_popup(app, "reminder", "待办提醒", 380.0, 280.0);
    let _ = window.show();
    let _ = window.set_focus();
    let _ = app.emit_to("reminder", "reminder-shown", ());
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
        defer_until: None,
        pinned: false,
    });
    save_json(&app, "todos.json", &todos);
    emit_todos(&app);
    Ok(())
}

#[tauri::command]
fn delete_todo(app: AppHandle, id: u64) -> Result<(), String> {
    let mut todos: Vec<Todo> = load_json(&app, "todos.json");
    if let Some(item) = todos.iter().find(|item| item.id == id) {
        if item.pinned {
            return Ok(());
        }
    }
    todos.retain(|item| item.id != id);
    save_json(&app, "todos.json", &todos);
    emit_todos(&app);
    Ok(())
}

#[tauri::command]
fn toggle_todo(app: AppHandle, id: u64, done: bool) -> Result<(), String> {
    let mut todos: Vec<Todo> = load_json(&app, "todos.json");
    if let Some(item) = todos.iter_mut().find(|item| item.id == id) {
        if item.pinned && done {
            return Ok(());
        }
        item.done = done;
        item.completed_at = if done { Some(now_ms()) } else { None };
        if done {
            item.defer_until = None;
        }
    }
    save_json(&app, "todos.json", &todos);
    emit_todos(&app);
    Ok(())
}

#[tauri::command]
fn set_todo_pinned(app: AppHandle, id: u64, pinned: bool) -> Result<(), String> {
    let mut todos: Vec<Todo> = load_json(&app, "todos.json");
    if let Some(item) = todos.iter_mut().find(|item| item.id == id) {
        item.pinned = pinned;
        if pinned {
            item.done = false;
            item.completed_at = None;
            item.defer_until = None;
        }
    }
    save_json(&app, "todos.json", &todos);
    emit_todos(&app);
    Ok(())
}

#[tauri::command]
fn get_reminder_settings(state: tauri::State<AppState>) -> ReminderSetting {
    let settings = state.settings.lock().unwrap();
    ReminderSetting {
        enabled: settings.reminder_enabled,
        time: settings.reminder_time.clone(),
        include_pinned: settings.reminder_include_pinned,
    }
}

#[tauri::command]
fn set_reminder_settings(
    app: AppHandle,
    state: tauri::State<AppState>,
    enabled: bool,
    time: String,
    include_pinned: bool,
) -> Result<(), String> {
    if parse_hhmm(&time).is_none() {
        return Err("时间格式无效，应为 HH:MM".to_string());
    }
    {
        let mut settings = state.settings.lock().unwrap();
        settings.reminder_enabled = enabled;
        settings.reminder_time = time;
        settings.reminder_include_pinned = include_pinned;
    }
    save_json(&app, "settings.json", &*state.settings.lock().unwrap());

    {
        let mut rs = state.reminder.lock().unwrap();
        rs.snooze_until = None;
        rs.dismissed_on = None;
    }
    save_json(&app, "reminder_state.json", &*state.reminder.lock().unwrap());

    if !enabled {
        hide_reminder_window(&app);
    }
    Ok(())
}

#[tauri::command]
fn get_reminder_pending_count(app: AppHandle) -> usize {
    reminder_pending_count(&app)
}

#[tauri::command]
fn snooze_reminder(app: AppHandle, state: tauri::State<AppState>) -> Result<(), String> {
    {
        let mut rs = state.reminder.lock().unwrap();
        rs.snooze_until = Some(now_ms() + 30 * 60 * 1000);
    }
    save_json(&app, "reminder_state.json", &*state.reminder.lock().unwrap());
    Ok(())
}

#[tauri::command]
fn dismiss_reminder_today(app: AppHandle, state: tauri::State<AppState>) -> Result<(), String> {
    {
        let mut rs = state.reminder.lock().unwrap();
        rs.dismissed_on = Some(local_today());
    }
    save_json(&app, "reminder_state.json", &*state.reminder.lock().unwrap());
    Ok(())
}

#[tauri::command]
fn complete_all_pending(app: AppHandle, state: tauri::State<AppState>) -> Result<(), String> {
    let mut todos: Vec<Todo> = load_json(&app, "todos.json");
    let now = now_ms();
    for item in todos.iter_mut() {
        if is_active(item) && !item.pinned {
            item.done = true;
            item.completed_at = Some(now);
            item.defer_until = None;
        }
    }
    save_json(&app, "todos.json", &todos);
    emit_todos(&app);

    // Pinned todos can't be completed, so suppress the reminder for the rest
    // of the day to avoid it re-firing immediately.
    {
        let mut rs = state.reminder.lock().unwrap();
        rs.dismissed_on = Some(local_today());
    }
    save_json(&app, "reminder_state.json", &*state.reminder.lock().unwrap());
    Ok(())
}

#[tauri::command]
fn defer_all_pending_to_tomorrow(app: AppHandle) -> Result<(), String> {
    let mut todos: Vec<Todo> = load_json(&app, "todos.json");
    let target = start_of_tomorrow_ms();
    for item in todos.iter_mut() {
        if is_active(item) && !item.pinned {
            item.defer_until = Some(target);
        }
    }
    save_json(&app, "todos.json", &todos);
    emit_todos(&app);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    builder
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    window.app_handle().exit(0);
                }
            }
        })
        .setup(|app| {
            let settings: AppSettings = load_json(app.handle(), "settings.json");
            let reminder: ReminderState = load_json(app.handle(), "reminder_state.json");

            app.manage(AppState {
                settings: Mutex::new(settings.clone()),
                reminder: Mutex::new(reminder),
                summarizing: AtomicBool::new(false),
                cancel: AtomicBool::new(false),
            });

            apply_always_on_top(app.handle(), settings.always_on_top);

            {
                let handle = app.handle().clone();
                std::thread::spawn(move || loop {
                    std::thread::sleep(Duration::from_secs(20));
                    maybe_show_reminder(&handle);
                });
            }

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
                let s = settings.add_shortcut.clone();
                if let Err(err) =
                    shortcut::install(tx, &s.code, s.meta, s.ctrl, s.alt, s.shift)
                {
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
                            }
                        }
                    });
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_always_on_top,
            set_always_on_top,
            get_add_shortcut,
            set_add_shortcut,
            hide_window,
            minimize_window,
            get_todos,
            add_todo,
            delete_todo,
            toggle_todo,
            set_todo_pinned,
            get_reminder_settings,
            set_reminder_settings,
            get_reminder_pending_count,
            snooze_reminder,
            dismiss_reminder_today,
            complete_all_pending,
            defer_all_pending_to_tomorrow,
            get_ai_settings,
            set_ai_settings,
            start_summary,
            cancel_summary,
            check_update,
            open_url,
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