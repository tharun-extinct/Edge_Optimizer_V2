//! Edge Optimizer Core Library
//!
//! Shared functionality for all Edge Optimizer processes
//!
//! Architecture:
//! - Runner process owns the system tray (uses tray_icon module)
//! - Settings process owns all UI windows (uses gui, flyout modules)
//! - IPC communication via named pipes (ipc module)

pub mod common_apps;
pub mod config;
pub mod crosshair_overlay;
pub mod flyout;
pub mod gui;
pub mod image_picker;
pub mod ipc;
pub mod process;
pub mod profile;
pub mod tray_flyout; // Legacy, may be removed
pub mod tray_icon; // New minimal tray manager for Runner

/// Re-export startup flags from settings for GUI
pub use crate::gui::GuiFlags;

/// Startup flags parsed from command line (used by Settings)
#[derive(Debug, Default, Clone)]
pub struct StartupFlags {
    /// Show flyout immediately on startup (triggered by Runner)
    pub show_flyout: bool,
    /// Bring main window to front (triggered by Runner)
    pub bring_to_front: bool,
    /// Flyout-only mode: Start with main window hidden, only show flyout
    /// Used when Runner spawns Settings for single-click tray action
    pub flyout_only: bool,
}
