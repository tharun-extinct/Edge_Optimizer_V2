//! EdgeOptimizer.Macro - Keyboard/Mouse Macro Execution Process
//!
//! This process runs separately from the Settings UI and handles:
//! - Global hotkey listening for macro triggers
//! - Macro execution with precise timing
//! - IPC communication with Settings for configuration updates
//!
//! Architecture:
//! - Receives macro configurations from Settings via IPC
//! - Listens for registered hotkeys using Windows global hooks
//! - Executes macro sequences when triggered
//! - Runs a Win32 message loop for hotkey events

#![windows_subsystem = "windows"]

mod executor;
mod hotkey_manager;
mod ipc_handler;

use anyhow::Result;
use edge_optimizer_core::macro_config::MacroConfig;
use std::sync::{Arc, Mutex};
use tracing::{error, info};

/// Application state shared across threads
pub struct MacroAppState {
    /// Current macro configuration (from active profile)
    pub config: MacroConfig,
    /// Whether macro execution is enabled
    pub enabled: bool,
    /// Currently executing macro (prevents re-entry)
    pub executing: bool,
}

impl Default for MacroAppState {
    fn default() -> Self {
        Self {
            config: MacroConfig::default(),
            enabled: true,
            executing: false,
        }
    }
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .init();

    info!("EdgeOptimizer.Macro starting...");

    // Create shared application state
    let state = Arc::new(Mutex::new(MacroAppState::default()));

    // Start IPC listener thread (receives config from Settings)
    let ipc_state = Arc::clone(&state);
    std::thread::spawn(move || {
        if let Err(e) = ipc_handler::run_ipc_listener(ipc_state) {
            error!("IPC listener error: {}", e);
        }
    });

    // Run the main hotkey listener loop (Win32 message pump)
    // This blocks and processes global hotkey events
    if let Err(e) = hotkey_manager::run_hotkey_loop(state) {
        error!("Hotkey loop error: {}", e);
        return Err(e);
    }

    info!("EdgeOptimizer.Macro shutting down");
    Ok(())
}
