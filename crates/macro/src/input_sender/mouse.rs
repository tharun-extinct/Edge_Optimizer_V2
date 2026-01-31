//! Mouse Input Sending
//!
//! Functions for simulating mouse input.

use crate::types::MouseButton;
use tracing::debug;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_VIRTUALDESK, MOUSEEVENTF_WHEEL,
    MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MOUSEINPUT, MOUSE_EVENT_FLAGS,
};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

/// Move mouse to absolute screen position
pub fn move_to(x: i32, y: i32) -> Result<(), String> {
    let (norm_x, norm_y) = normalize_coords(x, y);

    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: norm_x,
                dy: norm_y,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let result = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };

    if result == 0 {
        return Err("Failed to send mouse move".to_string());
    }

    debug!("Mouse moved to ({}, {})", x, y);
    Ok(())
}

/// Move mouse by relative offset
pub fn move_by(dx: i32, dy: i32) -> Result<(), String> {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_MOVE,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let result = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };

    if result == 0 {
        return Err("Failed to send mouse relative move".to_string());
    }

    debug!("Mouse moved by ({}, {})", dx, dy);
    Ok(())
}

/// Press a mouse button (button down)
pub fn button_down(button: MouseButton) -> Result<(), String> {
    let (flags, mouse_data) = button_to_down_flags(button);

    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: mouse_data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let result = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };

    if result == 0 {
        return Err("Failed to send mouse button down".to_string());
    }

    debug!("Mouse button down: {:?}", button);
    Ok(())
}

/// Release a mouse button (button up)
pub fn button_up(button: MouseButton) -> Result<(), String> {
    let (flags, mouse_data) = button_to_up_flags(button);

    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: mouse_data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let result = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };

    if result == 0 {
        return Err("Failed to send mouse button up".to_string());
    }

    debug!("Mouse button up: {:?}", button);
    Ok(())
}

/// Click a mouse button (down + up)
pub fn click(button: MouseButton) -> Result<(), String> {
    button_down(button)?;
    button_up(button)?;
    Ok(())
}

/// Double click a mouse button
pub fn double_click(button: MouseButton) -> Result<(), String> {
    click(button)?;
    click(button)?;
    Ok(())
}

/// Scroll the mouse wheel
pub fn scroll(delta: i32) -> Result<(), String> {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: delta as u32,
                dwFlags: MOUSEEVENTF_WHEEL,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let result = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };

    if result == 0 {
        return Err("Failed to send mouse scroll".to_string());
    }

    debug!("Mouse scrolled: {}", delta);
    Ok(())
}

/// Normalize screen coordinates to 0-65535 range for absolute positioning
fn normalize_coords(x: i32, y: i32) -> (i32, i32) {
    let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };

    let norm_x = (x * 65535) / screen_width;
    let norm_y = (y * 65535) / screen_height;

    (norm_x, norm_y)
}

/// Convert button to down event flags
fn button_to_down_flags(button: MouseButton) -> (MOUSE_EVENT_FLAGS, u32) {
    match button {
        MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, 0),
        MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, 0),
        MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, 0),
        MouseButton::X1 => (MOUSEEVENTF_XDOWN, 1),
        MouseButton::X2 => (MOUSEEVENTF_XDOWN, 2),
        MouseButton::None => (MOUSE_EVENT_FLAGS(0), 0),
    }
}

/// Convert button to up event flags
fn button_to_up_flags(button: MouseButton) -> (MOUSE_EVENT_FLAGS, u32) {
    match button {
        MouseButton::Left => (MOUSEEVENTF_LEFTUP, 0),
        MouseButton::Right => (MOUSEEVENTF_RIGHTUP, 0),
        MouseButton::Middle => (MOUSEEVENTF_MIDDLEUP, 0),
        MouseButton::X1 => (MOUSEEVENTF_XUP, 1),
        MouseButton::X2 => (MOUSEEVENTF_XUP, 2),
        MouseButton::None => (MOUSE_EVENT_FLAGS(0), 0),
    }
}
