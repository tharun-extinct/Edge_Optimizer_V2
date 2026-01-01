// #![windows_subsystem = "windows"]  // Temporarily disabled for debugging

mod config;
mod overlay;
mod process;
mod profile;
mod tray;
mod tray_flyout;
mod gui;
mod ipc;
mod common_apps;
mod image_picker;
mod crosshair_overlay;
mod flyout;

use anyhow::Result;

fn main() -> Result<()> {
    // Check command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "--tray-only" {
        // Run in tray-only mode (no GUI)
        run_tray_only()?;
    } else {
        // Run full GUI application with integrated tray
        gui::run()?;
    }
    
    Ok(())
}

/// Run in tray-only mode without GUI
fn run_tray_only() -> Result<()> {
    // Load configuration
    let app_config = config::load_config();
    
    // Load profiles  
    let data_dir = config::get_data_directory()?;
    let profiles = profile::load_profiles(&data_dir)?;
    
    // Create IPC channels using std::sync::mpsc
    let (gui_to_tray_tx, gui_to_tray_rx) = std::sync::mpsc::channel();
    let (tray_to_gui_tx, tray_to_gui_rx) = std::sync::mpsc::channel();
    
    let channels = ipc::TrayChannels {
        to_gui: tray_to_gui_tx,
        from_gui: gui_to_tray_rx,
    };
    
    // Start tray thread with flyout
    tray_flyout::run_tray_flyout_thread(
        channels,
        profiles,
        app_config.active_profile,
    );
    
    // Keep main thread alive
    loop {
        // Check for messages from tray
        if let Ok(msg) = tray_to_gui_rx.recv_timeout(std::time::Duration::from_millis(100)) {
            match msg {
                ipc::TrayToGui::ActivateProfile(name) => {
                    println!("Activating profile: {}", name);
                    // TODO: Implement profile activation logic
                }
                ipc::TrayToGui::Exit => {
                    println!("Exiting...");
                    break;
                }
                _ => {}
            }
        }
    }
    
    Ok(())
}
