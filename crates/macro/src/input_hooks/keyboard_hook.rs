//! Keyboard Hook Implementation
//!
//! Windows low-level keyboard hook for capturing key events.

use crate::types::{KeyFlags, KeyboardData, VirtualKey};
use crossbeam_channel::Sender;
use parking_lot::Mutex;
use std::sync::OnceLock;
use tracing::{debug, error};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT,
    WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

/// Global keyboard hook handle
static KEYBOARD_HOOK: OnceLock<Mutex<Option<HHOOK>>> = OnceLock::new();

/// Channel sender for keyboard events
static KEYBOARD_SENDER: OnceLock<Mutex<Option<Sender<KeyboardData>>>> = OnceLock::new();

/// Initialize global statics
fn init_statics() {
    let _ = KEYBOARD_HOOK.get_or_init(|| Mutex::new(None));
    let _ = KEYBOARD_SENDER.get_or_init(|| Mutex::new(None));
}

/// Install the low-level keyboard hook
///
/// # Arguments
/// * `sender` - Channel sender for keyboard events
///
/// # Returns
/// * `Ok(())` if hook installed successfully
/// * `Err(String)` if hook installation failed
pub fn install_keyboard_hook(sender: Sender<KeyboardData>) -> Result<(), String> {
    init_statics();

    // Store the sender
    if let Some(sender_lock) = KEYBOARD_SENDER.get() {
        *sender_lock.lock() = Some(sender);
    }

    // Install the hook
    let hook = unsafe {
        SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0)
            .map_err(|e| format!("Failed to install keyboard hook: {}", e))?
    };

    debug!("Keyboard hook installed successfully");

    // Store the hook handle
    if let Some(hook_lock) = KEYBOARD_HOOK.get() {
        *hook_lock.lock() = Some(hook);
    }

    Ok(())
}

/// Uninstall the keyboard hook
pub fn uninstall_keyboard_hook() {
    if let Some(hook_lock) = KEYBOARD_HOOK.get() {
        let mut hook = hook_lock.lock();
        if let Some(h) = hook.take() {
            unsafe {
                let _ = UnhookWindowsHookEx(h);
            }
            debug!("Keyboard hook uninstalled");
        }
    }

    // Clear the sender
    if let Some(sender_lock) = KEYBOARD_SENDER.get() {
        *sender_lock.lock() = None;
    }
}

/// Keyboard hook callback procedure
unsafe extern "system" fn keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    // Process the event if code is >= 0
    if n_code >= 0 {
        let kb_struct = &*(l_param.0 as *const KBDLLHOOKSTRUCT);

        // Determine key state (down or up)
        let flags = match w_param.0 as u32 {
            WM_KEYDOWN | WM_SYSKEYDOWN => KeyFlags::Down,
            WM_KEYUP | WM_SYSKEYUP => KeyFlags::Up,
            _ => KeyFlags::Down,
        };

        let vk = VirtualKey::from(kb_struct.vkCode);
        let data = KeyboardData::new(vk, kb_struct.scanCode, flags, kb_struct.time);

        // Send through channel if available
        if let Some(sender_lock) = KEYBOARD_SENDER.get() {
            if let Some(sender) = sender_lock.lock().as_ref() {
                if let Err(e) = sender.try_send(data) {
                    error!("Failed to send keyboard event: {}", e);
                }
            }
        }
    }

    // Always pass to next hook - don't block input
    CallNextHookEx(None, n_code, w_param, l_param)
}

/// Check if keyboard hook is installed
pub fn is_keyboard_hook_installed() -> bool {
    KEYBOARD_HOOK
        .get()
        .map(|h| h.lock().is_some())
        .unwrap_or(false)
}
