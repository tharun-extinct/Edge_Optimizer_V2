//! Macro Configuration Types
//!
//! Defines the data structures for gaming macros:
//! - MacroDefinition: A complete macro with name, actions, and trigger
//! - MacroAction: Individual actions (key press, mouse click, delay)
//! - MacroShortcut: The hotkey combination to trigger a macro
//! - MacroConfig: Collection of macros for a profile

use serde::{Deserialize, Serialize};

/// Mouse button types for click actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl std::fmt::Display for MouseButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MouseButton::Left => write!(f, "Left"),
            MouseButton::Right => write!(f, "Right"),
            MouseButton::Middle => write!(f, "Middle"),
        }
    }
}

/// Individual action within a macro sequence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MacroAction {
    /// Key press (key down) with optional delay in milliseconds
    KeyPress {
        key: String,
        delay_ms: u64,
    },
    /// Key release (key up) with optional delay in milliseconds
    KeyRelease {
        key: String,
        delay_ms: u64,
    },
    /// Mouse button click (press or release)
    MouseClick {
        button: MouseButton,
        /// true = press (down), false = release (up)
        press: bool,
    },
    /// Move mouse to absolute position
    MouseMove {
        x: i32,
        y: i32,
    },
    /// Pure delay between actions
    Delay {
        ms: u64,
    },
}

impl MacroAction {
    /// Get a display-friendly description of this action
    pub fn display_text(&self) -> String {
        match self {
            MacroAction::KeyPress { key, delay_ms } => {
                if *delay_ms > 0 {
                    format!("Key: {} ⬇ ({}ms)", key, delay_ms)
                } else {
                    format!("Key: {} ⬇", key)
                }
            }
            MacroAction::KeyRelease { key, delay_ms } => {
                if *delay_ms > 0 {
                    format!("Key: {} ⬆ ({}ms)", key, delay_ms)
                } else {
                    format!("Key: {} ⬆", key)
                }
            }
            MacroAction::MouseClick { button, press } => {
                let direction = if *press { "⬇" } else { "⬆" };
                format!("{} Click {}", button, direction)
            }
            MacroAction::MouseMove { x, y } => {
                format!("Move: ({}, {})", x, y)
            }
            MacroAction::Delay { ms } => {
                format!("Delay: {}ms", ms)
            }
        }
    }

    /// Get the delay value if this action has one
    pub fn get_delay(&self) -> Option<u64> {
        match self {
            MacroAction::KeyPress { delay_ms, .. } => Some(*delay_ms),
            MacroAction::KeyRelease { delay_ms, .. } => Some(*delay_ms),
            MacroAction::Delay { ms } => Some(*ms),
            _ => None,
        }
    }
}

/// Hotkey combination to trigger a macro
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MacroShortcut {
    /// CTRL modifier
    pub ctrl: bool,
    /// ALT modifier
    pub alt: bool,
    /// SHIFT modifier
    pub shift: bool,
    /// WIN/Super modifier
    pub win: bool,
    /// Main key (A-Z, 0-9, F1-F12)
    pub key: String,
}

impl MacroShortcut {
    /// Create a new empty shortcut
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if shortcut is valid (has at least one modifier and a key)
    pub fn is_valid(&self) -> bool {
        let has_modifier = self.ctrl || self.alt || self.shift || self.win;
        let has_key = !self.key.is_empty();
        has_modifier && has_key
    }

    /// Get display string for the shortcut
    pub fn display_text(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.win {
            parts.push("Win");
        }
        if !self.key.is_empty() {
            parts.push(&self.key);
        }
        if parts.is_empty() {
            "Not set".to_string()
        } else {
            parts.join(" + ")
        }
    }
}

/// How the macro should cycle/repeat
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CycleMode {
    /// Execute once
    Once,
    /// Execute a specified number of times
    Count(u32),
    /// Keep executing until the specified key is pressed
    UntilKeyPressed(String),
}

impl Default for CycleMode {
    fn default() -> Self {
        CycleMode::Once
    }
}

impl CycleMode {
    /// Get display string for the cycle mode
    pub fn display_text(&self) -> String {
        match self {
            CycleMode::Once => "Once".to_string(),
            CycleMode::Count(n) => format!("{} times", n),
            CycleMode::UntilKeyPressed(key) => format!("Until {} pressed", key),
        }
    }
}

/// A complete macro definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroDefinition {
    /// Unique name for this macro
    pub name: String,
    /// Whether this macro is enabled
    pub enabled: bool,
    /// The sequence of actions to execute
    pub actions: Vec<MacroAction>,
    /// The hotkey combination to trigger this macro
    pub shortcut: Option<MacroShortcut>,
    /// How the macro should repeat
    pub cycle_mode: CycleMode,
}

impl MacroDefinition {
    /// Create a new empty macro with the given name
    pub fn new(name: String) -> Self {
        Self {
            name,
            enabled: true,
            actions: Vec::new(),
            shortcut: None,
            cycle_mode: CycleMode::default(),
        }
    }

    /// Validate the macro definition
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Macro name cannot be empty".to_string());
        }
        if self.name.len() > 50 {
            return Err("Macro name must be 50 characters or less".to_string());
        }
        if self.actions.is_empty() {
            return Err("Macro must have at least one action".to_string());
        }
        if let Some(ref shortcut) = self.shortcut {
            if !shortcut.is_valid() {
                return Err("Shortcut must have at least one modifier and a key".to_string());
            }
        }
        Ok(())
    }
}

/// Configuration for all macros in a profile
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MacroConfig {
    /// List of macro definitions
    pub macros: Vec<MacroDefinition>,
}

impl MacroConfig {
    /// Create a new empty macro config
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new macro
    pub fn add_macro(&mut self, macro_def: MacroDefinition) {
        self.macros.push(macro_def);
    }

    /// Remove a macro by index
    pub fn remove_macro(&mut self, index: usize) -> Option<MacroDefinition> {
        if index < self.macros.len() {
            Some(self.macros.remove(index))
        } else {
            None
        }
    }

    /// Check if a macro name is unique
    pub fn is_name_unique(&self, name: &str, exclude_index: Option<usize>) -> bool {
        let name_lower = name.to_lowercase();
        for (i, m) in self.macros.iter().enumerate() {
            if let Some(exclude) = exclude_index {
                if i == exclude {
                    continue;
                }
            }
            if m.name.to_lowercase() == name_lower {
                return false;
            }
        }
        true
    }

    /// Get a macro by name
    pub fn get_by_name(&self, name: &str) -> Option<&MacroDefinition> {
        self.macros.iter().find(|m| m.name == name)
    }

    /// Get enabled macros only
    pub fn enabled_macros(&self) -> Vec<&MacroDefinition> {
        self.macros.iter().filter(|m| m.enabled).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_shortcut_display() {
        let shortcut = MacroShortcut {
            ctrl: true,
            alt: false,
            shift: true,
            win: false,
            key: "A".to_string(),
        };
        assert_eq!(shortcut.display_text(), "Ctrl + Shift + A");
    }

    #[test]
    fn test_macro_action_display() {
        let action = MacroAction::KeyPress {
            key: "A".to_string(),
            delay_ms: 10,
        };
        assert_eq!(action.display_text(), "Key: A ⬇ (10ms)");
    }

    #[test]
    fn test_macro_validation() {
        let mut macro_def = MacroDefinition::new("Test".to_string());
        assert!(macro_def.validate().is_err()); // No actions

        macro_def.actions.push(MacroAction::KeyPress {
            key: "A".to_string(),
            delay_ms: 0,
        });
        assert!(macro_def.validate().is_ok());
    }
}
