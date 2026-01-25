//! EdgeOptimizer.Runner - System Tray Process
//!
//! This process manages:
//! - System tray icon with context menu (right-click)
//! - Flyout window display (left single-click)
//! - Settings window spawn (left double-click)
//! - IPC communication with Settings process via named pipes
//! - Win32 message loop for tray icon events

#![windows_subsystem = "windows"]

use anyhow::{Context, Result};
use std::process::Command;
use std::time::{Duration, Instant};

// Import from core library crate
use edge_optimizer_core::{
    config,
    ipc::{GuiToTray, NamedPipeServer},
    profile,
    tray_flyout::TrayFlyoutManager,
};

use tray_icon::menu::MenuEvent;
use tray_icon::{MouseButton, MouseButtonState, TrayIconEvent};
use windows::Win32::UI::WindowsAndMessaging::*;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("EdgeOptimizer.Runner starting...");

    // Load configuration
    let app_config = config::load_config();

    // Load profiles
    let data_dir = config::get_data_directory()?;
    let profiles = profile::load_profiles(&data_dir)?;

    tracing::info!("Loaded {} profiles", profiles.len());

    // Create tray manager
    let mut tray = TrayFlyoutManager::new(profiles, app_config.active_profile.clone())
        .context("Failed to create tray manager")?;

    tracing::info!("Tray manager created");

    // Initialize named pipe server for IPC with Settings process
    let pipe_server = NamedPipeServer::new().context("Failed to create named pipe server")?;

    tracing::info!("Named pipe server created, waiting for Settings to connect...");

    // Note: We don't block waiting for connection here, we'll check for messages in the loop

    // Set up event handlers for tray icon and menu
    let (event_tx, event_rx) = std::sync::mpsc::channel::<TrayIconEvent>();
    let (menu_tx, menu_rx) = std::sync::mpsc::channel::<MenuEvent>();

    TrayIconEvent::set_event_handler(Some(move |event| {
        tracing::debug!("Tray event: {:?}", event);
        let _ = event_tx.send(event);
    }));

    MenuEvent::set_event_handler(Some(move |event| {
        tracing::debug!("Menu event: {:?}", event);
        let _ = menu_tx.send(event);
    }));

    tracing::info!("Event handlers set, entering message loop");

    // Click timing state for double-click detection
    let mut last_click_time: Option<Instant> = None;
    let mut pending_single_click = false;

    // Main Win32 message loop
    unsafe {
        let mut msg = MSG::default();
        loop {
            // Pump Windows messages (required for tray icon events)
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    tracing::info!("WM_QUIT received, exiting");
                    return Ok(());
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            // Process tray icon events
            if let Ok(event) = event_rx.try_recv() {
                match event {
                    TrayIconEvent::Click {
                        button,
                        button_state,
                        ..
                    } => {
                        if button == MouseButton::Left && button_state == MouseButtonState::Up {
                            let now = Instant::now();

                            // Check for double-click (within 500ms)
                            let is_double_click = if let Some(last_time) = last_click_time {
                                now.duration_since(last_time).as_millis() < 500
                            } else {
                                false
                            };

                            if is_double_click {
                                // Double-click: Open full Settings window
                                tracing::info!("Double-click detected - spawning Settings window");
                                pending_single_click = false;
                                last_click_time = None;

                                // Spawn Settings process
                                if let Err(e) = spawn_settings_window() {
                                    tracing::error!("Failed to spawn Settings window: {}", e);
                                }
                            } else {
                                // First click: Start timer for single-click
                                tracing::debug!(
                                    "First click detected, waiting for potential double-click"
                                );
                                last_click_time = Some(now);
                                pending_single_click = true;
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Check if single-click timer expired (500ms)
            if pending_single_click {
                if let Some(last_time) = last_click_time {
                    if Instant::now().duration_since(last_time).as_millis() >= 500 {
                        // Single-click confirmed: Toggle flyout
                        tracing::info!("Single-click confirmed - toggling flyout");
                        pending_single_click = false;

                        if tray.is_flyout_visible() {
                            tray.hide_flyout();
                        } else {
                            if let Err(e) = tray.show_flyout() {
                                tracing::error!("Failed to show flyout: {}", e);
                            }
                        }
                    }
                }
            }

            // Process menu events (right-click context menu)
            if let Ok(event) = menu_rx.try_recv() {
                if event.id == tray.menu_item_settings {
                    tracing::info!("Settings menu clicked - spawning Settings window");
                    if let Err(e) = spawn_settings_window() {
                        tracing::error!("Failed to spawn Settings window: {}", e);
                    }
                } else if event.id == tray.menu_item_docs {
                    tracing::info!("Documentation menu clicked");
                    let _ = open::that("https://github.com/yourusername/gaming_optimizer#readme");
                } else if event.id == tray.menu_item_bug_report {
                    tracing::info!("Bug report menu clicked");
                    let _ =
                        open::that("https://github.com/yourusername/gaming_optimizer/issues/new");
                } else if event.id == tray.menu_item_exit {
                    tracing::info!("Exit menu clicked");
                    // TODO: Send shutdown via IPC to Settings if running
                    return Ok(());
                }
            }

            // Poll named pipe for messages from Settings process
            match pipe_server.try_recv() {
                Ok(Some(msg)) => {
                    match msg {
                        GuiToTray::ProfilesUpdated(new_profiles) => {
                            tracing::info!("Received ProfilesUpdated from Settings");
                            tray.update_profiles(new_profiles);
                        }
                        GuiToTray::ActiveProfileChanged(new_active) => {
                            tracing::info!(
                                "Received ActiveProfileChanged from Settings: {:?}",
                                new_active
                            );
                            tray.set_active_profile(new_active);
                        }
                        GuiToTray::OverlayVisibilityChanged(_visible) => {
                            // Not used in Runner
                        }
                        GuiToTray::Shutdown => {
                            tracing::info!("Received shutdown signal from Settings");
                            return Ok(());
                        }
                    }
                }
                Ok(None) => {
                    // No messages available
                }
                Err(e) => {
                    tracing::warn!("Error reading from named pipe: {}", e);
                }
            }

            // Small sleep to avoid busy-waiting
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

/// Spawn the Settings window process
fn spawn_settings_window() -> Result<()> {
    let exe_dir = std::env::current_exe()?
        .parent()
        .context("Failed to get executable directory")?
        .to_path_buf();

    let settings_exe = exe_dir.join("edge_optimizer_settings.exe");

    if !settings_exe.exists() {
        anyhow::bail!("Settings executable not found: {:?}", settings_exe);
    }

    Command::new(&settings_exe)
        .spawn()
        .context("Failed to spawn Settings process")?;

    tracing::info!("Settings process spawned: {:?}", settings_exe);

    Ok(())
}
