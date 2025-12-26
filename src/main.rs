#![windows_subsystem = "windows"]

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
    // Run GUI application
    gui::run()?;
    Ok(())
}
