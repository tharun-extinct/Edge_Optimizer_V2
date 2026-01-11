//! EdgeOptimizer.Settings - Main GUI Process
//! 
//! This process manages:
//! - Main settings window (full GUI application)
//! - Flyout window (quick access flyout)
//! - IPC communication with Runner process
//! - Profile management and system optimization

// #![windows_subsystem = "windows"]  // Temporarily disabled for debugging

use std::process::{Command, Stdio};
use std::path::Path;

// Import modules from the workspace
use gaming_optimizer::gui;

fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    tracing::info!("EdgeOptimizer.Settings starting...");
    
    // Spawn the Runner process (manages system tray)
    spawn_runner_process();
    
    // Run the GUI application
    gui::run()?;
    
    Ok(())
}

/// Spawn the EdgeOptimizer.Runner process to manage the system tray
fn spawn_runner_process() {
    // Find the runner executable next to this executable
    if let Ok(exe_path) = std::env::current_exe() {
        let exe_dir = exe_path.parent().unwrap_or(Path::new("."));
        let runner_path = exe_dir.join("edge_optimizer_runner.exe");
        
        if runner_path.exists() {
            tracing::info!("Spawning Runner process: {:?}", runner_path);
            match Command::new(&runner_path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(child) => {
                    tracing::info!("Runner process spawned with PID: {}", child.id());
                }
                Err(e) => {
                    tracing::error!("Failed to spawn Runner process: {}", e);
                }
            }
        } else {
            tracing::warn!("Runner executable not found at {:?}", runner_path);
        }
    }
}
