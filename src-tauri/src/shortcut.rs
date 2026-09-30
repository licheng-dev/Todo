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
    Settings,
}

#[cfg(target_os = "macos")]
static TX: OnceLock<Mutex<mpsc::Sender<ShortcutKind>>> = OnceLock::new();

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
const CMDSHIFT_MASK: u32 = (1 << 8) | (1 << 9);
#[cfg(target_os = "macos")]
const CMD_MASK: u32 = 1 << 8;
#[cfg(target_os = "macos")]
const KEVENT_PARAM_DIRECT_OBJECT: u32 = u32::from_be_bytes(*b"----");
#[cfg(target_os = "macos")]
const TYPE_EVENT_HOTKEY_ID: u32 = u32::from_be_bytes(*b"hkid");

#[cfg(target_os = "macos")]
const HOTKEY_ADD_TODO: u32 = 1;
#[cfg(target_os = "macos")]
const HOTKEY_SETTINGS: u32 = 2;

#[cfg(target_os = "macos")]
unsafe extern "C" fn hotkey_handler(
    _call_ref: *mut c_void,
    event: *mut c_void,
    _user_data: *mut c_void,
) -> OSStatus {
    let mut hotkey_id = EventHotKeyID { signature: 0, id: 0 };
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

    if status == 0 {
        if let Some(tx) = TX.get().and_then(|m| m.lock().ok()).as_ref() {
            let kind = match hotkey_id.id {
                HOTKEY_ADD_TODO => ShortcutKind::AddTodo,
                HOTKEY_SETTINGS => ShortcutKind::Settings,
                _ => return 0,
            };
            let _ = tx.send(kind);
        }
    }
    0
}

#[cfg(target_os = "macos")]
pub fn install(tx: mpsc::Sender<ShortcutKind>) -> Result<(), String> {
    unsafe {
        TX.set(Mutex::new(tx)).map_err(|_| "shortcuts already installed".to_string())?;

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

        // 0x2D = kVK_ANSI_N
        let mut ref_a: *mut c_void = std::ptr::null_mut();
        let s = RegisterEventHotKey(
            0x2D,
            CMDSHIFT_MASK,
            EventHotKeyID { signature: 0x746F646F, id: HOTKEY_ADD_TODO }, // 'todo'
            target,
            0,
            &mut ref_a,
        );
        if s != 0 {
            return Err(format!("RegisterEventHotKey(add-todo) failed: {s}"));
        }

        // 0x2B = kVK_ANSI_Comma
        let mut ref_s: *mut c_void = std::ptr::null_mut();
        let s = RegisterEventHotKey(
            0x2B,
            CMD_MASK,
            EventHotKeyID { signature: 0x746F646F, id: HOTKEY_SETTINGS },
            target,
            0,
            &mut ref_s,
        );
        if s != 0 {
            return Err(format!("RegisterEventHotKey(settings) failed: {s}"));
        }

        Ok(())
    }
}

// Non-macOS: stub so the module compiles on other platforms
#[cfg(not(target_os = "macos"))]
pub fn install(_tx: std::sync::mpsc::Sender<ShortcutKind>) -> Result<(), String> {
    Ok(())
}