#![windows_subsystem = "windows"]

mod config;
mod overlay;
mod process;
mod profile;
mod tray;

use anyhow::Result;
use config::{get_data_directory, load_config, save_config, AppConfig};
use overlay::OverlayWindow;
use process::kill_processes;
use profile::{load_profiles, Profile};
use std::fs;
use std::time::UNIX_EPOCH;
use tray::TrayManager;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

/// Application state
struct AppState {
    config: AppConfig,
    profiles: Vec<Profile>,
    active_profile_index: Option<usize>,
    overlay: Option<OverlayWindow>,
    tray: TrayManager,
    profiles_last_modified: Option<u64>,
}

impl AppState {
    /// Get the currently active profile
    fn get_active_profile(&self) -> Option<&Profile> {
        self.active_profile_index
            .and_then(|idx| self.profiles.get(idx))
    }

    /// Find profile index by name
    fn find_profile_index(&self, name: &str) -> Option<usize> {
        self.profiles.iter().position(|p| p.name == name)
    }
}

fn main() -> Result<()> {
    // Initialize application
    println!("Starting Gaming Optimizer...");

    // Load configuration
    let config = load_config().unwrap_or_default();

    // Get data directory
    let data_dir = get_data_directory()?;

    // Load profiles
    let profiles = load_profiles(&data_dir).unwrap_or_default();

    // Find active profile index
    let active_profile_index = config
        .active_profile
        .as_ref()
        .and_then(|name| profiles.iter().position(|p| &p.name == name));

    // Get active profile name for tray
    let active_profile_name = active_profile_index.map(|idx| profiles[idx].name.as_str());

    // Create event loop
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    // Create system tray
    let tray = TrayManager::new(&profiles, active_profile_name)?;

    // Create initial state
    let mut state = AppState {
        config,
        profiles,
        active_profile_index,
        overlay: None,
        tray,
        profiles_last_modified: get_profiles_modified_time(&data_dir),
    };

    // Start with no active profile (as per design decision)
    state.config.active_profile = None;
    state.config.overlay_visible = false;
    state.active_profile_index = None;

    println!("Gaming Optimizer started. Check system tray.");

    // Main event loop
    let _ = event_loop.run(move |event, event_loop_target| {
        event_loop_target.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent { event, window_id } => {
                // Handle overlay window events
                if let Some(ref mut overlay) = state.overlay {
                    if window_id == overlay.window().id() {
                        match event {
                            WindowEvent::CloseRequested => {
                                // Don't close, just hide
                                overlay.hide();
                                state.config.overlay_visible = false;
                                let _ = save_config(&state.config);
                            }
                            WindowEvent::Resized(new_size) => {
                                let _ = overlay.on_resize(new_size);
                            }
                            _ => {}
                        }
                    }
                }
            }

            Event::AboutToWait => {
                // Check for profile file updates
                check_and_reload_profiles(&mut state, &data_dir);

                // Poll tray events
                if let Some(tray_event) = state.tray.poll_events(&state.profiles) {
                    handle_tray_event(tray_event, &mut state, &data_dir);
                }
            }

            Event::LoopExiting => {
                // Clean up
                if let Some(ref mut overlay) = state.overlay {
                    overlay.hide();
                }
                let _ = save_config(&state.config);
            }

            _ => {}
        }
    });

    Ok(())
}

/// Handle tray menu events
fn handle_tray_event(
    event: tray::TrayEvent,
    state: &mut AppState,
    data_dir: &std::path::Path,
) {
    match event {
        tray::TrayEvent::ProfileSelected(profile_name) => {
            activate_profile(state, &profile_name, data_dir);
        }

        tray::TrayEvent::ProfileDeactivated => {
            deactivate_profile(state);
        }

        tray::TrayEvent::OverlayToggled => {
            toggle_overlay(state);
        }

        tray::TrayEvent::OpenSettings => {
            open_settings(data_dir);
        }

        tray::TrayEvent::Exit => {
            // Clean shutdown
            if let Some(ref mut overlay) = state.overlay {
                overlay.hide();
            }
            let _ = save_config(&state.config);
            std::process::exit(0);
        }
    }
}

/// Activate a gaming profile
fn activate_profile(
    state: &mut AppState,
    profile_name: &str,
    _data_dir: &std::path::Path,
) {
    // Find profile
    let profile_idx = match state.find_profile_index(profile_name) {
        Some(idx) => idx,
        None => {
            eprintln!("Profile not found: {}", profile_name);
            return;
        }
    };

    let profile = &state.profiles[profile_idx];

    // Kill processes
    println!("Activating profile: {}", profile_name);
    let report = kill_processes(&profile.processes_to_kill);

    // Log kill report
    if !report.killed.is_empty() {
        println!("Killed processes: {:?}", report.killed);
    }
    if !report.failed.is_empty() {
        eprintln!("Failed to kill processes: {:?}", report.failed);
    }
    if !report.not_found.is_empty() {
        println!("Processes not found: {:?}", report.not_found);
    }
    if !report.blocklist_skipped.is_empty() {
        println!("Protected processes skipped: {:?}", report.blocklist_skipped);
    }

    // Update overlay if profile has crosshair configured
    if let Some(ref image_path) = profile.crosshair_image_path {
        if profile.overlay_enabled {
            match OverlayWindow::new(
                image_path,
                profile.crosshair_x_offset,
                profile.crosshair_y_offset,
            ) {
                Ok((mut overlay, _overlay_event_loop)) => {
                    let _ = overlay.show();
                    state.overlay = Some(overlay);
                    state.config.overlay_visible = true;
                    println!("Overlay shown with crosshair: {}", image_path);
                }
                Err(e) => {
                    eprintln!("Failed to create overlay: {}", e);
                    state.overlay = None;
                    state.config.overlay_visible = false;
                }
            }
        }
    } else {
        // No crosshair configured, hide overlay
        if let Some(ref mut overlay) = state.overlay {
            overlay.hide();
        }
        state.config.overlay_visible = false;
    }

    // Update state
    state.active_profile_index = Some(profile_idx);
    state.config.active_profile = Some(profile_name.to_string());

    // Update tray
    let _ = state.tray.set_active_profile(Some(profile_name));
    let has_overlay = state.overlay.is_some();
    let _ = state
        .tray
        .set_overlay_visible(state.config.overlay_visible, has_overlay);

    // Save config
    let _ = save_config(&state.config);
}

/// Deactivate current profile
fn deactivate_profile(state: &mut AppState) {
    println!("Deactivating profile");

    // Hide overlay
    if let Some(ref mut overlay) = state.overlay {
        overlay.hide();
    }

    // Update state
    state.active_profile_index = None;
    state.config.active_profile = None;
    state.config.overlay_visible = false;

    // Update tray
    let _ = state.tray.set_active_profile(None);
    let _ = state.tray.set_overlay_visible(false, false);

    // Save config
    let _ = save_config(&state.config);
}

/// Toggle overlay visibility
fn toggle_overlay(state: &mut AppState) {
    if let Some(ref mut overlay) = state.overlay {
        if overlay.is_visible() {
            overlay.hide();
            state.config.overlay_visible = false;
            println!("Overlay hidden");
        } else {
            let _ = overlay.show();
            state.config.overlay_visible = true;
            println!("Overlay shown");
        }

        let _ = state
            .tray
            .set_overlay_visible(state.config.overlay_visible, true);
        let _ = save_config(&state.config);
    }
}

/// Open settings folder in File Explorer
fn open_settings(data_dir: &std::path::Path) {
    println!("Opening settings folder: {:?}", data_dir);

    // Create directory if it doesn't exist
    let _ = fs::create_dir_all(data_dir);

    // Open in File Explorer (Windows)
    #[cfg(windows)]
    {
        use std::process::Command;
        let _ = Command::new("explorer.exe")
            .arg(data_dir.to_string_lossy().to_string())
            .spawn();
    }

    #[cfg(not(windows))]
    {
        eprintln!("Settings folder: {:?}", data_dir);
    }
}

/// Get the modification time of profiles.json
fn get_profiles_modified_time(data_dir: &std::path::Path) -> Option<u64> {
    let profiles_path = data_dir.join("profiles.json");
    fs::metadata(profiles_path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
}

/// Check if profiles.json has been modified and reload if needed
fn check_and_reload_profiles(state: &mut AppState, data_dir: &std::path::Path) {
    let current_modified = get_profiles_modified_time(data_dir);

    if current_modified != state.profiles_last_modified {
        println!("Profiles file modified, reloading...");

        // Reload profiles
        if let Ok(new_profiles) = load_profiles(data_dir) {
            state.profiles = new_profiles;
            state.profiles_last_modified = current_modified;

            // Update tray menu
            let active_name = state.active_profile_index
                .and_then(|idx| state.profiles.get(idx))
                .map(|p| p.name.as_str());
            let _ = state.tray.update_profiles(&state.profiles, active_name);

            println!("Profiles reloaded successfully");
        }
    }
}
