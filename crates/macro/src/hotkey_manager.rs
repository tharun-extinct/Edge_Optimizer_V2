//! Hotkey Manager - Global hotkey registration and listening
//!
//! Uses the global-hotkey crate for cross-platform hotkey handling.
//! On Windows, this requires a Win32 message loop.

use crate::executor::MacroExecutor;
use crate::MacroAppState;
use anyhow::Result;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, warn};

/// Convert our macro modifier flags to global-hotkey Modifiers
fn to_hotkey_modifiers(ctrl: bool, alt: bool, shift: bool, win: bool) -> Modifiers {
    let mut mods = Modifiers::empty();
    if ctrl {
        mods |= Modifiers::CONTROL;
    }
    if alt {
        mods |= Modifiers::ALT;
    }
    if shift {
        mods |= Modifiers::SHIFT;
    }
    if win {
        mods |= Modifiers::META;
    }
    mods
}

/// Convert key string to global-hotkey Code
fn string_to_code(key: &str) -> Option<Code> {
    match key.to_uppercase().as_str() {
        // Letters
        "A" => Some(Code::KeyA),
        "B" => Some(Code::KeyB),
        "C" => Some(Code::KeyC),
        "D" => Some(Code::KeyD),
        "E" => Some(Code::KeyE),
        "F" => Some(Code::KeyF),
        "G" => Some(Code::KeyG),
        "H" => Some(Code::KeyH),
        "I" => Some(Code::KeyI),
        "J" => Some(Code::KeyJ),
        "K" => Some(Code::KeyK),
        "L" => Some(Code::KeyL),
        "M" => Some(Code::KeyM),
        "N" => Some(Code::KeyN),
        "O" => Some(Code::KeyO),
        "P" => Some(Code::KeyP),
        "Q" => Some(Code::KeyQ),
        "R" => Some(Code::KeyR),
        "S" => Some(Code::KeyS),
        "T" => Some(Code::KeyT),
        "U" => Some(Code::KeyU),
        "V" => Some(Code::KeyV),
        "W" => Some(Code::KeyW),
        "X" => Some(Code::KeyX),
        "Y" => Some(Code::KeyY),
        "Z" => Some(Code::KeyZ),
        // Numbers
        "0" => Some(Code::Digit0),
        "1" => Some(Code::Digit1),
        "2" => Some(Code::Digit2),
        "3" => Some(Code::Digit3),
        "4" => Some(Code::Digit4),
        "5" => Some(Code::Digit5),
        "6" => Some(Code::Digit6),
        "7" => Some(Code::Digit7),
        "8" => Some(Code::Digit8),
        "9" => Some(Code::Digit9),
        // Function keys
        "F1" => Some(Code::F1),
        "F2" => Some(Code::F2),
        "F3" => Some(Code::F3),
        "F4" => Some(Code::F4),
        "F5" => Some(Code::F5),
        "F6" => Some(Code::F6),
        "F7" => Some(Code::F7),
        "F8" => Some(Code::F8),
        "F9" => Some(Code::F9),
        "F10" => Some(Code::F10),
        "F11" => Some(Code::F11),
        "F12" => Some(Code::F12),
        _ => None,
    }
}

/// Run the main hotkey listening loop
/// This function blocks and processes global hotkey events
pub fn run_hotkey_loop(state: Arc<Mutex<MacroAppState>>) -> Result<()> {
    info!("Starting hotkey manager...");

    // Create the global hotkey manager
    let manager = GlobalHotKeyManager::new().map_err(|e| anyhow::anyhow!("Failed to create hotkey manager: {:?}", e))?;

    // Map of hotkey ID -> macro name for quick lookup
    let mut hotkey_map: HashMap<u32, String> = HashMap::new();

    // Track registered hotkeys for cleanup
    let mut registered_hotkeys: Vec<HotKey> = Vec::new();

    info!("Hotkey manager initialized, entering event loop...");

    // Main event loop
    loop {
        // Check for hotkey events
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            debug!("Hotkey event received: {:?}", event);

            if let Some(macro_name) = hotkey_map.get(&event.id) {
                let should_execute = {
                    let state_guard = state.lock().unwrap();
                    state_guard.enabled && !state_guard.executing
                };

                if should_execute {
                    // Find and execute the macro
                    let macro_to_execute = {
                        let state_guard = state.lock().unwrap();
                        state_guard
                            .config
                            .macros
                            .iter()
                            .find(|m| m.name == *macro_name && m.enabled)
                            .cloned()
                    };

                    if let Some(macro_def) = macro_to_execute {
                        info!("Executing macro: {}", macro_def.name);

                        // Mark as executing
                        {
                            let mut state_guard = state.lock().unwrap();
                            state_guard.executing = true;
                        }

                        // Execute the macro
                        let executor = MacroExecutor::new();
                        if let Err(e) = executor.execute(&macro_def) {
                            error!("Macro execution error: {}", e);
                        }

                        // Mark as done
                        {
                            let mut state_guard = state.lock().unwrap();
                            state_guard.executing = false;
                        }
                    }
                }
            }
        }

        // Check if we need to update hotkey registrations
        // (This would be signaled by IPC handler updating the state)
        let needs_update = {
            let _state_guard = state.lock().unwrap();
            // Check if config changed - simple version: re-register periodically
            // In production, use a flag or version number
            false // Placeholder - implement proper change detection
        };

        if needs_update {
            // Unregister old hotkeys
            for hotkey in &registered_hotkeys {
                if let Err(e) = manager.unregister(*hotkey) {
                    warn!("Failed to unregister hotkey: {:?}", e);
                }
            }
            registered_hotkeys.clear();
            hotkey_map.clear();

            // Register new hotkeys from config
            let state_guard = state.lock().unwrap();
            for macro_def in &state_guard.config.macros {
                if !macro_def.enabled {
                    continue;
                }

                if let Some(ref shortcut) = macro_def.shortcut {
                    if let Some(code) = string_to_code(&shortcut.key) {
                        let modifiers = to_hotkey_modifiers(
                            shortcut.ctrl,
                            shortcut.alt,
                            shortcut.shift,
                            shortcut.win,
                        );

                        let hotkey = HotKey::new(Some(modifiers), code);

                        match manager.register(hotkey) {
                            Ok(_) => {
                                info!(
                                    "Registered hotkey for macro '{}': {:?}",
                                    macro_def.name, hotkey
                                );
                                hotkey_map.insert(hotkey.id(), macro_def.name.clone());
                                registered_hotkeys.push(hotkey);
                            }
                            Err(e) => {
                                error!(
                                    "Failed to register hotkey for macro '{}': {:?}",
                                    macro_def.name, e
                                );
                            }
                        }
                    }
                }
            }
        }

        // Small sleep to prevent busy-waiting
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
