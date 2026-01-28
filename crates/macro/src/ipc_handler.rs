//! IPC Handler - Communication with Settings process
//!
//! Listens for configuration updates from the Settings UI via Named Pipes.

use crate::MacroAppState;
use anyhow::Result;
use edge_optimizer_core::ipc::MACRO_PIPE_NAME;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, warn};

#[cfg(windows)]
use windows::Win32::{Foundation::*, Storage::FileSystem::*, System::Pipes::*};

/// Messages from Settings to Macro process
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SettingsToMacro {
    /// Update macro configuration (when profile changes or macros edited)
    ConfigUpdated(edge_optimizer_core::macro_config::MacroConfig),
    /// Enable/disable macro execution globally
    SetEnabled(bool),
    /// Shutdown the macro process
    Shutdown,
}

/// Messages from Macro to Settings process
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MacroToSettings {
    /// Macro was triggered (for UI feedback)
    MacroTriggered(String),
    /// Error occurred during execution
    ExecutionError(String, String), // (macro_name, error_message)
    /// Macro process ready
    Ready,
}

/// Run the IPC listener that receives config updates from Settings
#[cfg(windows)]
pub fn run_ipc_listener(state: Arc<Mutex<MacroAppState>>) -> Result<()> {
    info!("Starting Macro IPC listener...");

    loop {
        // Create named pipe server
        let pipe_name: Vec<u16> = MACRO_PIPE_NAME.encode_utf16().chain(Some(0)).collect();

        let pipe_handle = unsafe {
            CreateNamedPipeW(
                windows::core::PCWSTR(pipe_name.as_ptr()),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                1,                // Max instances
                8192,             // Out buffer size
                8192,             // In buffer size
                0,                // Default timeout
                Some(null_mut()), // Default security
            )
        };

        if pipe_handle.is_invalid() {
            error!("Failed to create macro named pipe");
            std::thread::sleep(std::time::Duration::from_secs(1));
            continue;
        }

        info!("Macro pipe created, waiting for Settings connection...");

        // Wait for Settings to connect
        unsafe {
            match ConnectNamedPipe(pipe_handle, Some(null_mut())) {
                Ok(_) => {
                    info!("Settings connected to Macro pipe");
                }
                Err(e) => {
                    let error_code = e.code().0 as u32;
                    if error_code != ERROR_PIPE_CONNECTED.0 {
                        warn!("ConnectNamedPipe error: {}", e);
                        let _ = CloseHandle(pipe_handle);
                        continue;
                    }
                }
            }
        }

        // Read messages from Settings
        loop {
            let mut buffer = [0u8; 8192];
            let mut bytes_read = 0u32;

            let read_result = unsafe {
                ReadFile(
                    pipe_handle,
                    Some(&mut buffer),
                    Some(&mut bytes_read),
                    None,
                )
            };

            match read_result {
                Ok(_) if bytes_read > 0 => {
                    // Deserialize and process message
                    match bincode::deserialize::<SettingsToMacro>(&buffer[..bytes_read as usize]) {
                        Ok(message) => {
                            debug!("Received IPC message: {:?}", message);
                            process_message(&state, message);
                        }
                        Err(e) => {
                            error!("Failed to deserialize IPC message: {}", e);
                        }
                    }
                }
                Ok(_) => {
                    // No data, pipe might be closing
                    debug!("Empty read from pipe");
                }
                Err(e) => {
                    let error_code = e.code().0 as u32;
                    if error_code == ERROR_BROKEN_PIPE.0 || error_code == ERROR_NO_DATA.0 {
                        info!("Settings disconnected from Macro pipe");
                        break;
                    }
                    error!("ReadFile error: {}", e);
                    break;
                }
            }
        }

        // Cleanup pipe
        unsafe {
            let _ = DisconnectNamedPipe(pipe_handle);
            let _ = CloseHandle(pipe_handle);
        }

        info!("Macro pipe closed, recreating...");
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// Process a message from Settings
fn process_message(state: &Arc<Mutex<MacroAppState>>, message: SettingsToMacro) {
    match message {
        SettingsToMacro::ConfigUpdated(config) => {
            info!("Macro config updated: {} macros", config.macros.len());
            let mut state_guard = state.lock().unwrap();
            state_guard.config = config;
        }
        SettingsToMacro::SetEnabled(enabled) => {
            info!("Macro execution enabled: {}", enabled);
            let mut state_guard = state.lock().unwrap();
            state_guard.enabled = enabled;
        }
        SettingsToMacro::Shutdown => {
            info!("Shutdown requested");
            std::process::exit(0);
        }
    }
}

#[cfg(not(windows))]
pub fn run_ipc_listener(_state: Arc<Mutex<MacroAppState>>) -> Result<()> {
    anyhow::bail!("Macro IPC is only supported on Windows")
}
