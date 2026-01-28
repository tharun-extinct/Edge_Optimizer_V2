//! Macro Executor - Executes recorded macro sequences
//!
//! Handles keyboard and mouse input simulation with precise timing.
//! Uses the enigo crate for cross-platform input simulation.

use anyhow::Result;
use edge_optimizer_core::macro_config::{CycleMode, MacroAction, MacroDefinition};
use enigo::{
    Direction::{Press, Release},
    Enigo, Key, Keyboard, Mouse, Settings,
};
use std::thread;
use std::time::Duration;
use tracing::{debug, info};

/// Macro executor that simulates keyboard and mouse input
pub struct MacroExecutor {
    // Enigo instance created per execution for thread safety
}

impl MacroExecutor {
    /// Create a new macro executor
    pub fn new() -> Self {
        Self {}
    }

    /// Execute a macro definition
    pub fn execute(&self, macro_def: &MacroDefinition) -> Result<()> {
        info!("Starting macro execution: {}", macro_def.name);

        match &macro_def.cycle_mode {
            CycleMode::Once => {
                self.execute_actions(&macro_def.actions)?;
            }
            CycleMode::Count(times) => {
                for i in 0..*times {
                    debug!("Cycle {}/{}", i + 1, times);
                    self.execute_actions(&macro_def.actions)?;
                }
            }
            CycleMode::UntilKeyPressed(stop_key) => {
                // For now, execute once - proper implementation would need
                // a separate thread to monitor for the stop key
                info!("UntilKeyPressed mode - executing once (stop key: {})", stop_key);
                self.execute_actions(&macro_def.actions)?;
            }
        }

        info!("Macro execution completed: {}", macro_def.name);
        Ok(())
    }

    /// Execute a sequence of macro actions
    fn execute_actions(&self, actions: &[MacroAction]) -> Result<()> {
        // Need mutable reference for enigo operations
        let mut enigo = Enigo::new(&Settings::default()).expect("Failed to create Enigo");

        for action in actions {
            match action {
                MacroAction::KeyPress { key, delay_ms } => {
                    debug!("KeyPress: {} (delay: {}ms)", key, delay_ms);
                    if let Some(enigo_key) = self.string_to_enigo_key(key) {
                        enigo.key(enigo_key, Press)?;
                    }
                    if *delay_ms > 0 {
                        thread::sleep(Duration::from_millis(*delay_ms));
                    }
                }
                MacroAction::KeyRelease { key, delay_ms } => {
                    debug!("KeyRelease: {} (delay: {}ms)", key, delay_ms);
                    if let Some(enigo_key) = self.string_to_enigo_key(key) {
                        enigo.key(enigo_key, Release)?;
                    }
                    if *delay_ms > 0 {
                        thread::sleep(Duration::from_millis(*delay_ms));
                    }
                }
                MacroAction::MouseClick { button, press } => {
                    debug!("MouseClick: {:?} (press: {})", button, press);
                    let enigo_button = self.to_enigo_button(button);
                    let direction = if *press { Press } else { Release };
                    enigo.button(enigo_button, direction)?;
                }
                MacroAction::MouseMove { x, y } => {
                    debug!("MouseMove: ({}, {})", x, y);
                    enigo.move_mouse(*x, *y, enigo::Coordinate::Abs)?;
                }
                MacroAction::Delay { ms } => {
                    debug!("Delay: {}ms", ms);
                    thread::sleep(Duration::from_millis(*ms));
                }
            }
        }

        Ok(())
    }

    /// Convert string key name to enigo Key
    fn string_to_enigo_key(&self, key: &str) -> Option<Key> {
        match key.to_uppercase().as_str() {
            // Letters
            "A" => Some(Key::Unicode('a')),
            "B" => Some(Key::Unicode('b')),
            "C" => Some(Key::Unicode('c')),
            "D" => Some(Key::Unicode('d')),
            "E" => Some(Key::Unicode('e')),
            "F" => Some(Key::Unicode('f')),
            "G" => Some(Key::Unicode('g')),
            "H" => Some(Key::Unicode('h')),
            "I" => Some(Key::Unicode('i')),
            "J" => Some(Key::Unicode('j')),
            "K" => Some(Key::Unicode('k')),
            "L" => Some(Key::Unicode('l')),
            "M" => Some(Key::Unicode('m')),
            "N" => Some(Key::Unicode('n')),
            "O" => Some(Key::Unicode('o')),
            "P" => Some(Key::Unicode('p')),
            "Q" => Some(Key::Unicode('q')),
            "R" => Some(Key::Unicode('r')),
            "S" => Some(Key::Unicode('s')),
            "T" => Some(Key::Unicode('t')),
            "U" => Some(Key::Unicode('u')),
            "V" => Some(Key::Unicode('v')),
            "W" => Some(Key::Unicode('w')),
            "X" => Some(Key::Unicode('x')),
            "Y" => Some(Key::Unicode('y')),
            "Z" => Some(Key::Unicode('z')),
            // Numbers
            "0" => Some(Key::Unicode('0')),
            "1" => Some(Key::Unicode('1')),
            "2" => Some(Key::Unicode('2')),
            "3" => Some(Key::Unicode('3')),
            "4" => Some(Key::Unicode('4')),
            "5" => Some(Key::Unicode('5')),
            "6" => Some(Key::Unicode('6')),
            "7" => Some(Key::Unicode('7')),
            "8" => Some(Key::Unicode('8')),
            "9" => Some(Key::Unicode('9')),
            // Function keys
            "F1" => Some(Key::F1),
            "F2" => Some(Key::F2),
            "F3" => Some(Key::F3),
            "F4" => Some(Key::F4),
            "F5" => Some(Key::F5),
            "F6" => Some(Key::F6),
            "F7" => Some(Key::F7),
            "F8" => Some(Key::F8),
            "F9" => Some(Key::F9),
            "F10" => Some(Key::F10),
            "F11" => Some(Key::F11),
            "F12" => Some(Key::F12),
            // Special keys
            "SPACE" => Some(Key::Space),
            "ENTER" | "RETURN" => Some(Key::Return),
            "TAB" => Some(Key::Tab),
            "ESCAPE" | "ESC" => Some(Key::Escape),
            "BACKSPACE" => Some(Key::Backspace),
            "DELETE" => Some(Key::Delete),
            "UP" => Some(Key::UpArrow),
            "DOWN" => Some(Key::DownArrow),
            "LEFT" => Some(Key::LeftArrow),
            "RIGHT" => Some(Key::RightArrow),
            "SHIFT" => Some(Key::Shift),
            "CTRL" | "CONTROL" => Some(Key::Control),
            "ALT" => Some(Key::Alt),
            _ => None,
        }
    }

    /// Convert MouseButton enum to enigo Button
    fn to_enigo_button(&self, button: &edge_optimizer_core::macro_config::MouseButton) -> enigo::Button {
        use edge_optimizer_core::macro_config::MouseButton;
        match button {
            MouseButton::Left => enigo::Button::Left,
            MouseButton::Right => enigo::Button::Right,
            MouseButton::Middle => enigo::Button::Middle,
        }
    }
}

impl Default for MacroExecutor {
    fn default() -> Self {
        Self::new()
    }
}
