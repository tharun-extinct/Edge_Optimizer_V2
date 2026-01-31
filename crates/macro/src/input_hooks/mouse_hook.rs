//! Mouse Hook Implementation
//!
//! Windows low-level mouse hook for capturing mouse events.

use crate::types::{KeyFlags, MouseButton, MouseData};
use crossbeam_channel::Sender;
use parking_lot::Mutex;
use std::sync::OnceLock;
use tracing::{debug, error};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, MSLLHOOKSTRUCT,
    WH_MOUSE_LL, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_XBUTTONDOWN, WM_XBUTTONUP,
};

/// Global mouse hook handle
static MOUSE_HOOK: OnceLock<Mutex<Option<HHOOK>>> = OnceLock::new();

/// Channel sender for mouse events
static MOUSE_SENDER: OnceLock<Mutex<Option<Sender<MouseData>>>> = OnceLock::new();

/// Last known mouse position for calculating deltas
static LAST_MOUSE_POS: OnceLock<Mutex<(i32, i32)>> = OnceLock::new();

/// Initialize global statics
fn init_statics() {
    let _ = MOUSE_HOOK.get_or_init(|| Mutex::new(None));
    let _ = MOUSE_SENDER.get_or_init(|| Mutex::new(None));
    let _ = LAST_MOUSE_POS.get_or_init(|| Mutex::new((0, 0)));
}

/// Install the low-level mouse hook
///
/// # Arguments
/// * `sender` - Channel sender for mouse events
///
/// # Returns
/// * `Ok(())` if hook installed successfully
/// * `Err(String)` if hook installation failed
pub fn install_mouse_hook(sender: Sender<MouseData>) -> Result<(), String> {
    init_statics();

    // Store the sender
    if let Some(sender_lock) = MOUSE_SENDER.get() {
        *sender_lock.lock() = Some(sender);
    }

    // Install the hook
    let hook = unsafe {
        SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), None, 0)
            .map_err(|e| format!("Failed to install mouse hook: {}", e))?
    };

    debug!("Mouse hook installed successfully");

    // Store the hook handle
    if let Some(hook_lock) = MOUSE_HOOK.get() {
        *hook_lock.lock() = Some(hook);
    }

    Ok(())
}

/// Uninstall the mouse hook
pub fn uninstall_mouse_hook() {
    if let Some(hook_lock) = MOUSE_HOOK.get() {
        let mut hook = hook_lock.lock();
        if let Some(h) = hook.take() {
            unsafe {
                let _ = UnhookWindowsHookEx(h);
            }
            debug!("Mouse hook uninstalled");
        }
    }

    // Clear the sender
    if let Some(sender_lock) = MOUSE_SENDER.get() {
        *sender_lock.lock() = None;
    }
}

/// Mouse hook callback procedure
unsafe extern "system" fn mouse_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let ms_struct = &*(l_param.0 as *const MSLLHOOKSTRUCT);
        let position = (ms_struct.pt.x, ms_struct.pt.y);

        let data = match w_param.0 as u32 {
            WM_LBUTTONDOWN => Some(MouseData::new_click(MouseButton::Left, KeyFlags::Down, position)),
            WM_LBUTTONUP => Some(MouseData::new_click(MouseButton::Left, KeyFlags::Up, position)),
            WM_RBUTTONDOWN => Some(MouseData::new_click(MouseButton::Right, KeyFlags::Down, position)),
            WM_RBUTTONUP => Some(MouseData::new_click(MouseButton::Right, KeyFlags::Up, position)),
            WM_MBUTTONDOWN => Some(MouseData::new_click(MouseButton::Middle, KeyFlags::Down, position)),
            WM_MBUTTONUP => Some(MouseData::new_click(MouseButton::Middle, KeyFlags::Up, position)),
            WM_XBUTTONDOWN => {
                let button = get_x_button(ms_struct.mouseData);
                Some(MouseData::new_click(button, KeyFlags::Down, position))
            }
            WM_XBUTTONUP => {
                let button = get_x_button(ms_struct.mouseData);
                Some(MouseData::new_click(button, KeyFlags::Up, position))
            }
            WM_MOUSEMOVE => {
                // Calculate relative movement
                let last_pos = LAST_MOUSE_POS
                    .get()
                    .map(|p| *p.lock())
                    .unwrap_or((0, 0));
                let delta = (position.0 - last_pos.0, position.1 - last_pos.1);

                // Update last position
                if let Some(pos_lock) = LAST_MOUSE_POS.get() {
                    *pos_lock.lock() = position;
                }

                Some(MouseData::new_move(position, delta))
            }
            WM_MOUSEWHEEL => {
                // High word of mouseData contains wheel delta
                let delta = (ms_struct.mouseData >> 16) as i16;
                Some(MouseData::new_wheel(delta, position))
            }
            _ => None,
        };

        // Send through channel if we have data
        if let Some(mouse_data) = data {
            if let Some(sender_lock) = MOUSE_SENDER.get() {
                if let Some(sender) = sender_lock.lock().as_ref() {
                    if let Err(e) = sender.try_send(mouse_data) {
                        error!("Failed to send mouse event: {}", e);
                    }
                }
            }
        }
    }

    // Always pass to next hook
    CallNextHookEx(None, n_code, w_param, l_param)
}

/// Extract X button number from mouseData
fn get_x_button(mouse_data: u32) -> MouseButton {
    let hi_word = (mouse_data >> 16) as u16;
    match hi_word {
        1 => MouseButton::X1,
        2 => MouseButton::X2,
        _ => MouseButton::None,
    }
}

/// Check if mouse hook is installed
pub fn is_mouse_hook_installed() -> bool {
    MOUSE_HOOK
        .get()
        .map(|h| h.lock().is_some())
        .unwrap_or(false)
}
