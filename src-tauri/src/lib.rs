mod shortcut;

use chrono::{Local, Timelike};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    fs,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Duration,
};
use tauri::utils::config::BackgroundThrottlingPolicy;
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
    reminder_mode: String,
    reminder_interval_start: String,
    reminder_interval_end: String,
    reminder_interval_minutes: u32,
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
            reminder_mode: "fixed".to_string(),
            reminder_interval_start: "09:00".to_string(),
            reminder_interval_end: "18:00".to_string(),
            reminder_interval_minutes: 30,
            reminder_include_pinned: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReminderSetting {
    enabled: bool,
    mode: String,
    time: String,
    interval_start: String,
    interval_end: String,
    interval_minutes: u32,
    include_pinned: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct ReminderState {
    dismissed_on: Option<String>,
    prompted_on: Option<String>,
    last_prompted_at: Option<u64>,
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
            .background_throttling(BackgroundThrottlingPolicy::Disabled)
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
            .background_throttling(BackgroundThrottlingPolicy::Disabled)
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
    // Hide the window instead of the whole application: hiding the app makes
    // the next popup activate it and drag the main window back into view.
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
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

/// 循环提醒是否应当触发。
///
/// - `now_minutes` / `start_minutes` / `end_minutes`：当天 0 点起的分钟数
/// - `last_prompted_at`：上次提醒的毫秒时间戳，`None` 表示今天还没提醒过
fn interval_due(
    now_minutes: u32,
    start_minutes: u32,
    end_minutes: u32,
    last_prompted_at: Option<u64>,
    now_ms: u64,
    interval_minutes: u32,
) -> bool {
    if now_minutes < start_minutes || now_minutes > end_minutes {
        return false;
    }
    match last_prompted_at {
        Some(last) => now_ms.saturating_sub(last) >= interval_minutes as u64 * 60_000,
        None => true,
    }
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

fn trigger_reminder(app: &AppHandle) {
    unhide_app(app);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
    let _ = app.emit_to("main", "reminder-triggered", ());
}

fn maybe_show_reminder(app: &AppHandle) {
    let state = app.state::<AppState>();
    let (enabled, mode, time, interval_start, interval_end, interval_minutes) = {
        let settings = state.settings.lock().unwrap();
        (
            settings.reminder_enabled,
            settings.reminder_mode.clone(),
            settings.reminder_time.clone(),
            settings.reminder_interval_start.clone(),
            settings.reminder_interval_end.clone(),
            settings.reminder_interval_minutes,
        )
    };
    if !enabled {
        return;
    }

    let today = local_today();
    if state.reminder.lock().unwrap().dismissed_on.as_deref() == Some(today.as_str()) {
        return;
    }

    let now = Local::now();

    if mode == "interval" {
        let (Some((sh, sm)), Some((eh, em))) =
            (parse_hhmm(&interval_start), parse_hhmm(&interval_end))
        else {
            return;
        };
        let current = now.hour() * 60 + now.minute();
        let last = state.reminder.lock().unwrap().last_prompted_at;
        let current_ms = now_ms();
        if !interval_due(
            current,
            sh * 60 + sm,
            eh * 60 + em,
            last,
            current_ms,
            interval_minutes,
        ) {
            return;
        }

        if reminder_pending_count(app) == 0 {
            return;
        }

        {
            let mut rs = state.reminder.lock().unwrap();
            rs.last_prompted_at = Some(current_ms);
        }
        save_json(app, "reminder_state.json", &*state.reminder.lock().unwrap());
        trigger_reminder(app);
        return;
    }

    let Some((hour, minute)) = parse_hhmm(&time) else {
        return;
    };
    let Some(target) = now.date_naive().and_hms_opt(hour, minute, 0) else {
        return;
    };
    if now.naive_local() < target {
        return;
    }

    if state.reminder.lock().unwrap().prompted_on.as_deref() == Some(today.as_str()) {
        return;
    }

    if reminder_pending_count(app) == 0 {
        return;
    }

    {
        let mut rs = state.reminder.lock().unwrap();
        rs.prompted_on = Some(today);
    }
    save_json(app, "reminder_state.json", &*state.reminder.lock().unwrap());
    trigger_reminder(app);
}

#[tauri::command]
fn get_todos(app: AppHandle) -> Vec<Todo> {
    load_json(&app, "todos.json")
}

/// 把输入按行拆成多条待办：每行一条，去掉首尾空白，忽略空行。
fn split_todo_lines(raw: &str) -> Vec<&str> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

#[tauri::command]
fn add_todo(app: AppHandle, text: String) -> Result<(), String> {
    let lines = split_todo_lines(&text);
    if lines.is_empty() {
        return Ok(());
    }
    let mut todos: Vec<Todo> = load_json(&app, "todos.json");
    // 同一批待办可能在同一毫秒内创建，用递增 id 避免重复
    let mut next_id = now_ms().max(todos.iter().map(|t| t.id).max().unwrap_or(0) + 1);
    let created_at = now_ms();
    for line in lines {
        todos.push(Todo {
            id: next_id,
            text: line.to_string(),
            done: false,
            created_at,
            completed_at: None,
            defer_until: None,
            pinned: false,
        });
        next_id += 1;
    }
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
        mode: settings.reminder_mode.clone(),
        time: settings.reminder_time.clone(),
        interval_start: settings.reminder_interval_start.clone(),
        interval_end: settings.reminder_interval_end.clone(),
        interval_minutes: settings.reminder_interval_minutes,
        include_pinned: settings.reminder_include_pinned,
    }
}

#[tauri::command]
fn set_reminder_settings(
    app: AppHandle,
    state: tauri::State<AppState>,
    enabled: bool,
    mode: String,
    time: String,
    interval_start: String,
    interval_end: String,
    interval_minutes: u32,
    include_pinned: bool,
) -> Result<(), String> {
    if mode != "fixed" && mode != "interval" {
        return Err("提醒方式无效".to_string());
    }
    if parse_hhmm(&time).is_none() {
        return Err("时间格式无效，应为 HH:MM".to_string());
    }
    if mode == "interval" {
        let Some((start_h, start_m)) = parse_hhmm(&interval_start) else {
            return Err("开始时间格式无效，应为 HH:MM".to_string());
        };
        let Some((end_h, end_m)) = parse_hhmm(&interval_end) else {
            return Err("结束时间格式无效，应为 HH:MM".to_string());
        };
        if start_h * 60 + start_m >= end_h * 60 + end_m {
            return Err("结束时间必须晚于开始时间".to_string());
        }
        if interval_minutes == 0 || interval_minutes > 1440 {
            return Err("提醒间隔需在 1 到 1440 分钟之间".to_string());
        }
    }
    {
        let mut settings = state.settings.lock().unwrap();
        settings.reminder_enabled = enabled;
        settings.reminder_mode = mode.clone();
        settings.reminder_time = time;
        if mode == "interval" {
            settings.reminder_interval_start = interval_start;
            settings.reminder_interval_end = interval_end;
            settings.reminder_interval_minutes = interval_minutes;
        }
        settings.reminder_include_pinned = include_pinned;
    }
    save_json(&app, "settings.json", &*state.settings.lock().unwrap());

    // 注意：这里不清空 reminder_state。
    // 之前每次保存设置都会把 last_prompted_at 重置为 None，导致改完时间后
    // 下一轮 20 秒轮询立刻再弹一次（看起来像“不到一分钟就提示一遍”）。
    // 调度本身每轮都会读取最新设置，不需要重置提醒状态。

    Ok(())
}

#[tauri::command]
fn get_reminder_pending_count(app: AppHandle) -> usize {
    reminder_pending_count(&app)
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
            dismiss_reminder_today,
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

#[cfg(test)]
mod tests {
    use super::*;

    const MINUTE_MS: u64 = 60_000;

    #[test]
    fn parse_hhmm_parses_valid_and_rejects_invalid() {
        assert_eq!(parse_hhmm("09:30"), Some((9, 30)));
        assert_eq!(parse_hhmm("23:59"), Some((23, 59)));
        assert_eq!(parse_hhmm("9:00"), Some((9, 0)));
        assert_eq!(parse_hhmm("24:00"), None);
        assert_eq!(parse_hhmm("12:60"), None);
        assert_eq!(parse_hhmm("abc"), None);
    }

    #[test]
    fn split_todo_lines_treats_each_line_as_a_todo() {
        assert_eq!(split_todo_lines("买牛奶\n写周报"), vec!["买牛奶", "写周报"]);
        assert_eq!(
            split_todo_lines("  买牛奶  \n\n 写周报 \r\n"),
            vec!["买牛奶", "写周报"]
        );
        assert_eq!(split_todo_lines("单条待办"), vec!["单条待办"]);
        assert!(split_todo_lines("  \n\n  ").is_empty());
        assert!(split_todo_lines("").is_empty());
    }

    #[test]
    fn interval_only_fires_inside_time_range() {
        let (start, end) = (9 * 60, 18 * 60);
        assert!(!interval_due(8 * 60 + 59, start, end, None, 0, 30));
        assert!(interval_due(9 * 60, start, end, None, 0, 30));
        assert!(interval_due(18 * 60, start, end, None, 0, 30));
        assert!(!interval_due(18 * 60 + 1, start, end, None, 0, 30));
    }

    #[test]
    fn interval_fires_when_never_prompted_today() {
        assert!(interval_due(10 * 60, 9 * 60, 18 * 60, None, 1_000_000, 30));
    }

    /// 固定 09:00–18:00 区间，简化下面的断言行
    fn due(now: u64, last: Option<u64>, interval: u32) -> bool {
        interval_due(10 * 60, 9 * 60, 18 * 60, last, now, interval)
    }

    #[test]
    fn interval_does_not_fire_again_within_interval() {
        let now = 1_000_000_000_000;
        // 20 秒前刚提醒过：即使轮询每 20 秒跑一次，1 分钟的间隔还没到
        assert!(!due(now, Some(now - 20_000), 1));
        // 差 1 毫秒到 1 分钟，也不应触发
        assert!(!due(now, Some(now - 59_999), 1));
        // 30 分钟间隔：29 分钟前提醒过，仍不应触发
        assert!(!due(now, Some(now - 29 * MINUTE_MS), 30));
    }

    #[test]
    fn interval_fires_after_interval_elapsed() {
        let now = 1_000_000_000_000;
        assert!(due(now, Some(now - MINUTE_MS), 1));
        assert!(due(now, Some(now - 30 * MINUTE_MS), 30));
        assert!(due(now, Some(now - 31 * MINUTE_MS), 30));
    }

    #[test]
    fn interval_ignores_clock_going_backwards() {
        // last_prompted_at 在未来（例如系统时间被调整）时不 panic，也不触发
        let now = 1_000_000_000_000;
        assert!(!due(now, Some(now + MINUTE_MS), 1));
    }
}
