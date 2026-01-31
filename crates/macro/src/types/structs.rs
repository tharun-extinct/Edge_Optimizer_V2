//! Struct Type Definitions
//!
//! Data structures for keyboard and mouse events.

use super::enums::{KeyFlags, MouseButton, VirtualKey};
use std::time::Instant;

/// Keyboard event data
#[derive(Debug, Clone)]
pub struct KeyboardData {
    /// Virtual key code
    pub key: VirtualKey,
    /// Hardware scan code
    pub scan_code: u32,
    /// Key state (down/up)
    pub flags: KeyFlags,
    /// System time when event occurred
    pub time: u32,
    /// High-precision timestamp
    pub timestamp: Instant,
}

impl KeyboardData {
    /// Create a new keyboard event
    pub fn new(key: VirtualKey, scan_code: u32, flags: KeyFlags, time: u32) -> Self {
        Self {
            key,
            scan_code,
            flags,
            time,
            timestamp: Instant::now(),
        }
    }
}

/// Mouse event data
#[derive(Debug, Clone)]
pub struct MouseData {
    /// Mouse button involved (if any)
    pub button: MouseButton,
    /// Button state (down/up)
    pub flags: KeyFlags,
    /// Absolute screen position
    pub position_absolute: (i32, i32),
    /// Relative movement delta
    pub position_relative: (i32, i32),
    /// Wheel scroll delta
    pub wheel_delta: i16,
    /// High-precision timestamp
    pub timestamp: Instant,
}

impl MouseData {
    /// Create a new mouse click event
    pub fn new_click(button: MouseButton, flags: KeyFlags, position: (i32, i32)) -> Self {
        Self {
            button,
            flags,
            position_absolute: position,
            position_relative: (0, 0),
            wheel_delta: 0,
            timestamp: Instant::now(),
        }
    }

    /// Create a new mouse move event
    pub fn new_move(position_absolute: (i32, i32), position_relative: (i32, i32)) -> Self {
        Self {
            button: MouseButton::None,
            flags: KeyFlags::Down,
            position_absolute,
            position_relative,
            wheel_delta: 0,
            timestamp: Instant::now(),
        }
    }

    /// Create a new mouse wheel event
    pub fn new_wheel(delta: i16, position: (i32, i32)) -> Self {
        Self {
            button: MouseButton::None,
            flags: KeyFlags::Down,
            position_absolute: position,
            position_relative: (0, 0),
            wheel_delta: delta,
            timestamp: Instant::now(),
        }
    }
}

/// Tracks currently held modifier keys
#[derive(Debug, Clone, Copy, Default)]
pub struct ModifierKeys {
    pub control: bool,
    pub shift: bool,
    pub alt: bool,
    pub win: bool,
}

impl ModifierKeys {
    /// Update modifier state based on key event
    pub fn update(&mut self, key: VirtualKey, flags: KeyFlags) {
        let is_down = flags == KeyFlags::Down;
        match key {
            VirtualKey::Control | VirtualKey::LControl | VirtualKey::RControl => {
                self.control = is_down;
            }
            VirtualKey::Shift | VirtualKey::LShift | VirtualKey::RShift => {
                self.shift = is_down;
            }
            VirtualKey::Alt | VirtualKey::LAlt | VirtualKey::RAlt => {
                self.alt = is_down;
            }
            VirtualKey::LWin | VirtualKey::RWin => {
                self.win = is_down;
            }
            _ => {}
        }
    }

    /// Check if any modifier is held
    pub fn any(&self) -> bool {
        self.control || self.shift || self.alt || self.win
    }
}
