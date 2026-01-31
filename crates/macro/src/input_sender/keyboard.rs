//! Keyboard Input Sending
//!
//! Functions for simulating keyboard input.

use crate::types::VirtualKey;
use tracing::debug;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, VIRTUAL_KEY,
};

/// Send a key press (key down)
pub fn key_down(key: VirtualKey) -> Result<(), String> {
    send_key_event(key, false)
}

/// Send a key release (key up)
pub fn key_up(key: VirtualKey) -> Result<(), String> {
    send_key_event(key, true)
}

/// Send a complete key press (down + up)
pub fn key_press(key: VirtualKey) -> Result<(), String> {
    key_down(key)?;
    key_up(key)?;
    Ok(())
}

/// Send a key event
fn send_key_event(key: VirtualKey, key_up: bool) -> Result<(), String> {
    let vk: VIRTUAL_KEY = key.into();
    let flags = if key_up {
        KEYEVENTF_KEYUP
    } else {
        KEYBD_EVENT_FLAGS(0)
    };

    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let result = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };

    if result == 0 {
        return Err("Failed to send key input".to_string());
    }

    debug!("Key event sent: {:?} (up: {})", key, key_up);
    Ok(())
}

/// Send a key event using scan code
pub fn key_down_scan(scan_code: u16) -> Result<(), String> {
    send_scan_event(scan_code, false)
}

/// Send a key release using scan code
pub fn key_up_scan(scan_code: u16) -> Result<(), String> {
    send_scan_event(scan_code, true)
}

/// Send a scan code event
fn send_scan_event(scan_code: u16, key_up: bool) -> Result<(), String> {
    let mut flags = KEYEVENTF_SCANCODE;
    if key_up {
        flags |= KEYEVENTF_KEYUP;
    }

    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: scan_code,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let result = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };

    if result == 0 {
        return Err("Failed to send scan code input".to_string());
    }

    debug!("Scan code event sent: {} (up: {})", scan_code, key_up);
    Ok(())
}

/// Send multiple key events at once (for key combinations)
pub fn send_key_combination(keys: &[VirtualKey]) -> Result<(), String> {
    // Press all keys
    for key in keys {
        key_down(*key)?;
    }

    // Release all keys in reverse order
    for key in keys.iter().rev() {
        key_up(*key)?;
    }

    Ok(())
}
