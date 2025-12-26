// #![windows_subsystem = "windows"]  // Temporarily disabled for debugging

mod config;
mod overlay;
mod process;
mod profile;
mod tray;
mod gui;
mod ipc;
mod common_apps;
mod image_picker;

use anyhow::Result;

fn main() -> Result<()> {
    // Run GUI application directly (tray integration disabled for now)
    // The tray-icon crate requires Windows message pump which conflicts with ICED
    gui::run()?;
    
    Ok(())
}
