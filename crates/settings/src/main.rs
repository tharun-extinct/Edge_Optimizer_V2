//! EdgeOptimizer.Settings - Main GUI Process
//!
//! This process manages:
//! - Main settings window (full GUI application)
//! - Flyout window (quick access flyout)
//! - IPC communication with Runner process (receives commands)
//! - Profile management and system optimization
//!
//! Architecture:
//! - Runner owns the system tray and sends IPC messages to us
//! - Settings owns all UI windows (MainWindow, FlyoutWindow)
//! - We listen for ShowFlyout/BringMainToFront IPC messages from Runner

// #![windows_subsystem = "windows"]  // Temporarily disabled for debugging

use edge_optimizer_core::gui;
use edge_optimizer_core::ipc::NamedPipeClient;
use edge_optimizer_core::StartupFlags;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("EdgeOptimizer.Settings starting...");

    // Parse command line arguments
    let flags = parse_args();
    tracing::info!("Startup flags: {:?}", flags);

    // Connect to Runner's IPC pipe (non-blocking, Runner may not be running)
    let ipc_client = match NamedPipeClient::try_connect() {
        Ok(Some(client)) => {
            tracing::info!("Connected to Runner IPC pipe");
            Some(client)
        }
        Ok(None) => {
            tracing::info!("Runner not available yet, will retry in GUI");
            None
        }
        Err(e) => {
            tracing::warn!("Failed to connect to Runner IPC: {}", e);
            None
        }
    };

    // Run the GUI application with IPC client and startup flags
    gui::run_with_ipc(ipc_client, flags)?;

    Ok(())
}

/// Parse command line arguments for startup flags
fn parse_args() -> StartupFlags {
    let args: Vec<String> = std::env::args().collect();
    let mut flags = StartupFlags::default();

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--show-flyout" => flags.show_flyout = true,
            "--bring-to-front" => flags.bring_to_front = true,
            "--flyout-only" => {
                flags.flyout_only = true;
                flags.show_flyout = true; // flyout-only implies show flyout
            }
            _ => {}
        }
    }

    flags
}
