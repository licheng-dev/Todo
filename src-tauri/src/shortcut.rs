#[cfg(target_os = "macos")]
use std::ffi::c_void;
#[cfg(target_os = "macos")]
use std::sync::{mpsc, Mutex, OnceLock};

#[cfg(target_os = "macos")]
type OSStatus = i32;

#[cfg(target_os = "macos")]
#[repr(C)]
struct EventHotKeyID {
    signature: u32,
    id: u32,
}

#[cfg(target_os = "macos")]
#[repr(C)]
struct EventTypeSpec {
    event_class: u32,
    event_kind: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum ShortcutKind {
    AddTodo,
}

#[cfg(target_os = "macos")]
static TX: OnceLock<Mutex<mpsc::Sender<ShortcutKind>>> = OnceLock::new();

#[cfg(target_os = "macos")]
static ADD_REF: Mutex<Option<usize>> = Mutex::new(None);

#[cfg(target_os = "macos")]
static ADD_SPEC: Mutex<Option<(u32, u32)>> = Mutex::new(None);

#[cfg(target_os = "macos")]
extern "C" {
    fn GetApplicationEventTarget() -> *mut c_void;
    fn RegisterEventHotKey(
        inHotKeyCode: u32,
        inHotKeyModifiers: u32,
        inHotKeyID: EventHotKeyID,
        inTarget: *mut c_void,
        inOptions: u32,
        outRef: *mut *mut c_void,
    ) -> OSStatus;
    fn UnregisterEventHotKey(inHotKeyRef: *mut c_void) -> OSStatus;
    fn InstallEventHandler(
        inTarget: *mut c_void,
        inHandler: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> OSStatus,
        inNumTypes: u32,
        inList: *const EventTypeSpec,
        inUserData: *mut c_void,
        outRef: *mut *mut c_void,
    ) -> OSStatus;
    fn GetEventParameter(
        inEvent: *mut c_void,
        inName: u32,
        inDesiredType: u32,
        outActualType: *mut u32,
        inBufferSize: u32,
        outActualSize: *mut u32,
        outData: *mut c_void,
    ) -> OSStatus;
}

#[cfg(target_os = "macos")]
const KEVENT_CLASS_KEYBOARD: u32 = u32::from_be_bytes(*b"keyb");
#[cfg(target_os = "macos")]
const KEVENT_HOTKEY_PRESSED: u32 = 5;
#[cfg(target_os = "macos")]
const KEVENT_PARAM_DIRECT_OBJECT: u32 = u32::from_be_bytes(*b"----");
#[cfg(target_os = "macos")]
const TYPE_EVENT_HOTKEY_ID: u32 = u32::from_be_bytes(*b"hkid");

#[cfg(target_os = "macos")]
const SIGNATURE: u32 = 0x746F646F; // 'todo'
#[cfg(target_os = "macos")]
const HOTKEY_ADD_TODO: u32 = 1;

#[cfg(target_os = "macos")]
unsafe extern "C" fn hotkey_handler(
    _call_ref: *mut c_void,
    event: *mut c_void,
    _user_data: *mut c_void,
) -> OSStatus {
    let mut hotkey_id = EventHotKeyID {
        signature: 0,
        id: 0,
    };
    let mut _actual_size: u32 = 0;
    let status = GetEventParameter(
        event,
        KEVENT_PARAM_DIRECT_OBJECT,
        TYPE_EVENT_HOTKEY_ID,
        std::ptr::null_mut(),
        std::mem::size_of::<EventHotKeyID>() as u32,
        &mut _actual_size,
        &mut hotkey_id as *mut _ as *mut c_void,
    );

    if status == 0 && hotkey_id.id == HOTKEY_ADD_TODO {
        if let Some(tx) = TX.get().and_then(|m| m.lock().ok()).as_ref() {
            let _ = tx.send(ShortcutKind::AddTodo);
        }
    }
    0
}

/// Maps a W3C `KeyboardEvent.code` to a macOS virtual key code (kVK).
pub fn key_code_for(code: &str) -> Option<u32> {
    Some(match code {
        "KeyA" => 0x00,
        "KeyS" => 0x01,
        "KeyD" => 0x02,
        "KeyF" => 0x03,
        "KeyH" => 0x04,
        "KeyG" => 0x05,
        "KeyZ" => 0x06,
        "KeyX" => 0x07,
        "KeyC" => 0x08,
        "KeyV" => 0x09,
        "KeyB" => 0x0B,
        "KeyQ" => 0x0C,
        "KeyW" => 0x0D,
        "KeyE" => 0x0E,
        "KeyR" => 0x0F,
        "KeyY" => 0x10,
        "KeyT" => 0x11,
        "KeyO" => 0x1F,
        "KeyU" => 0x20,
        "KeyI" => 0x22,
        "KeyP" => 0x23,
        "KeyL" => 0x25,
        "KeyJ" => 0x26,
        "KeyK" => 0x28,
        "KeyN" => 0x2D,
        "KeyM" => 0x2E,
        "Digit1" => 0x12,
        "Digit2" => 0x13,
        "Digit3" => 0x14,
        "Digit4" => 0x15,
        "Digit6" => 0x16,
        "Digit5" => 0x17,
        "Digit9" => 0x19,
        "Digit7" => 0x1A,
        "Digit8" => 0x1C,
        "Digit0" => 0x1D,
        "Equal" => 0x18,
        "Minus" => 0x1B,
        "BracketRight" => 0x1E,
        "BracketLeft" => 0x21,
        "Quote" => 0x27,
        "Semicolon" => 0x29,
        "Backslash" => 0x2A,
        "Comma" => 0x2B,
        "Slash" => 0x2C,
        "Period" => 0x2F,
        "Backquote" => 0x32,
        "Space" => 0x31,
        "Enter" => 0x24,
        "Tab" => 0x30,
        "F1" => 0x7A,
        "F2" => 0x78,
        "F3" => 0x63,
        "F4" => 0x76,
        "F5" => 0x60,
        "F6" => 0x61,
        "F7" => 0x62,
        "F8" => 0x64,
        "F9" => 0x65,
        "F10" => 0x6D,
        "F11" => 0x67,
        "F12" => 0x6F,
        _ => return None,
    })
}

/// Builds the Carbon modifier mask from boolean modifier states.
pub fn modifier_mask(meta: bool, ctrl: bool, alt: bool, shift: bool) -> u32 {
    let mut mask = 0u32;
    if meta {
        mask |= 1 << 8;
    }
    if shift {
        mask |= 1 << 9;
    }
    if alt {
        mask |= 1 << 11;
    }
    if ctrl {
        mask |= 1 << 12;
    }
    mask
}

#[cfg(target_os = "macos")]
fn register_hotkey(key_code: u32, modifiers: u32) -> Result<usize, String> {
    unsafe {
        let target = GetApplicationEventTarget();
        let mut ref_out: *mut c_void = std::ptr::null_mut();
        let status = RegisterEventHotKey(
            key_code,
            modifiers,
            EventHotKeyID {
                signature: SIGNATURE,
                id: HOTKEY_ADD_TODO,
            },
            target,
            0,
            &mut ref_out,
        );
        if status != 0 {
            return Err(format!("该快捷键可能已被系统占用 (错误 {status})"));
        }
        Ok(ref_out as usize)
    }
}

#[cfg(target_os = "macos")]
fn resolve(code: &str, meta: bool, ctrl: bool, alt: bool, shift: bool) -> Result<(u32, u32), String> {
    let key_code = key_code_for(code).ok_or_else(|| format!("不支持的按键: {code}"))?;
    Ok((key_code, modifier_mask(meta, ctrl, alt, shift)))
}

#[cfg(target_os = "macos")]
pub fn install(
    tx: mpsc::Sender<ShortcutKind>,
    code: &str,
    meta: bool,
    ctrl: bool,
    alt: bool,
    shift: bool,
) -> Result<(), String> {
    let (key_code, modifiers) = resolve(code, meta, ctrl, alt, shift)?;

    unsafe {
        TX.set(Mutex::new(tx))
            .map_err(|_| "shortcuts already installed".to_string())?;

        let target = GetApplicationEventTarget();
        let spec = EventTypeSpec {
            event_class: KEVENT_CLASS_KEYBOARD,
            event_kind: KEVENT_HOTKEY_PRESSED,
        };

        let mut handler_ref: *mut c_void = std::ptr::null_mut();
        let status = InstallEventHandler(
            target,
            hotkey_handler,
            1,
            &spec,
            std::ptr::null_mut(),
            &mut handler_ref,
        );
        if status != 0 {
            return Err(format!("InstallEventHandler failed: {status}"));
        }
    }

    let new_ref = register_hotkey(key_code, modifiers)?;
    *ADD_REF.lock().unwrap() = Some(new_ref);
    *ADD_SPEC.lock().unwrap() = Some((key_code, modifiers));
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn update_add(
    code: &str,
    meta: bool,
    ctrl: bool,
    alt: bool,
    shift: bool,
) -> Result<(), String> {
    let (key_code, modifiers) = resolve(code, meta, ctrl, alt, shift)?;

    if *ADD_SPEC.lock().unwrap() == Some((key_code, modifiers)) {
        return Ok(());
    }

    // Register the new combo first so a failure leaves the old one intact.
    let new_ref = register_hotkey(key_code, modifiers)?;
    let old_ref = ADD_REF.lock().unwrap().replace(new_ref);
    *ADD_SPEC.lock().unwrap() = Some((key_code, modifiers));

    if let Some(ptr) = old_ref {
        unsafe {
            let _ = UnregisterEventHotKey(ptr as *mut c_void);
        }
    }
    Ok(())
}

// Non-macOS: stub so the module compiles on other platforms
#[cfg(not(target_os = "macos"))]
pub fn install(
    _tx: std::sync::mpsc::Sender<ShortcutKind>,
    _code: &str,
    _meta: bool,
    _ctrl: bool,
    _alt: bool,
    _shift: bool,
) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn update_add(
    _code: &str,
    _meta: bool,
    _ctrl: bool,
    _alt: bool,
    _shift: bool,
) -> Result<(), String> {
    Ok(())
}
