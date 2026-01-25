//! EdgeOptimizer.Runner - System Tray Process
//!
//! This process manages:
//! - System tray icon with context menu (right-click)
//! - IPC communication with Settings process via named pipes
//! - Win32 message loop for tray icon events
//!
//! Architecture:
//! - Runner owns the tray icon and sends IPC messages to Settings
//! - Settings owns the Flyout and MainWindow, listens for IPC
//! - On single-click: Runner sends ShowFlyout via IPC
//! - On double-click: Runner sends BringMainToFront via IPC (or spawns Settings if not running)

#![windows_subsystem = "windows"]

use anyhow::{Context, Result};
use std::process::Command;
use std::time::{Duration, Instant};

// Import from core library crate
use edge_optimizer_core::{
    config,
    ipc::{GuiToTray, NamedPipeServer, TrayToGui},
    tray_icon::TrayIconManager,
};

use tray_icon::menu::MenuEvent;
use tray_icon::{MouseButton, MouseButtonState, TrayIconEvent};
use windows::Win32::UI::WindowsAndMessaging::*;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("EdgeOptimizer.Runner starting...");

    // Load configuration for active profile tooltip
    let app_config = config::load_config();

    // Create minimal tray icon manager (no flyout - that's owned by Settings)
    let mut tray = TrayIconManager::new(app_config.active_profile.clone())
        .context("Failed to create tray icon manager")?;

    tracing::info!("Tray icon created");

    // Initialize named pipe server for IPC with Settings process
    let pipe_server = NamedPipeServer::new().context("Failed to create named pipe server")?;

    tracing::info!("Named pipe server created, waiting for Settings to connect...");

    // Track whether Settings is connected (for fallback spawning)
    let mut settings_connected = false;

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
                                // Double-click: Bring Settings main window to front
                                tracing::info!(
                                    "Double-click detected - bringing Settings to front"
                                );
                                pending_single_click = false;
                                last_click_time = None;

                                // Send IPC to Settings, fallback to spawning if not connected
                                if settings_connected {
                                    if let Err(e) = pipe_server.send(&TrayToGui::BringMainToFront) {
                                        tracing::warn!(
                                            "Failed to send BringMainToFront via IPC: {}",
                                            e
                                        );
                                        settings_connected = false;
                                        // Fallback: spawn Settings
                                        if let Err(e) = spawn_settings_window(None) {
                                            tracing::error!(
                                                "Failed to spawn Settings window: {}",
                                                e
                                            );
                                        }
                                    }
                                } else {
                                    // Settings not connected, spawn it
                                    if let Err(e) = spawn_settings_window(None) {
                                        tracing::error!("Failed to spawn Settings window: {}", e);
                                    }
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
                        // Single-click confirmed: Send ShowFlyout via IPC
                        tracing::info!("Single-click confirmed - sending ShowFlyout via IPC");
                        pending_single_click = false;

                        // Send IPC to Settings to show flyout
                        if settings_connected {
                            if let Err(e) = pipe_server.send(&TrayToGui::ShowFlyout) {
                                tracing::warn!("Failed to send ShowFlyout via IPC: {}", e);
                                settings_connected = false;
                                // Fallback: spawn Settings in flyout-only mode (hidden main window)
                                if let Err(e) = spawn_settings_window(Some("--flyout-only")) {
                                    tracing::error!("Failed to spawn Settings with flyout: {}", e);
                                }
                            }
                        } else {
                            // Settings not connected, spawn it in flyout-only mode
                            if let Err(e) = spawn_settings_window(Some("--flyout-only")) {
                                tracing::error!("Failed to spawn Settings with flyout: {}", e);
                            }
                        }
                    }
                }
            }

            // Process menu events (right-click context menu)
            if let Ok(event) = menu_rx.try_recv() {
                if event.id == tray.menu_item_settings {
                    tracing::info!("Settings menu clicked - opening Settings window");
                    // Send IPC or spawn
                    if settings_connected {
                        if let Err(e) = pipe_server.send(&TrayToGui::BringMainToFront) {
                            tracing::warn!("Failed to send BringMainToFront via IPC: {}", e);
                            settings_connected = false;
                            if let Err(e) = spawn_settings_window(None) {
                                tracing::error!("Failed to spawn Settings window: {}", e);
                            }
                        }
                    } else {
                        if let Err(e) = spawn_settings_window(None) {
                            tracing::error!("Failed to spawn Settings window: {}", e);
                        }
                    }
                } else if event.id == tray.menu_item_docs {
                    tracing::info!("Documentation menu clicked");
                    let _ = open::that("https://github.com/yourusername/EdgeOptimizer#readme");
                } else if event.id == tray.menu_item_bug_report {
                    tracing::info!("Bug report menu clicked");
                    let _ = open::that("https://github.com/yourusername/EdgeOptimizer/issues/new");
                } else if event.id == tray.menu_item_exit {
                    tracing::info!("Exit menu clicked");
                    // Send shutdown to Settings if connected
                    if settings_connected {
                        let _ = pipe_server.send(&TrayToGui::Exit);
                    }
                    return Ok(());
                }
            }

            // Poll named pipe for messages from Settings process
            match pipe_server.try_recv() {
                Ok(Some(msg)) => {
                    // If we received a message, Settings is connected
                    if !settings_connected {
                        tracing::info!("Settings process connected to IPC");
                        settings_connected = true;
                    }
                    match msg {
                        GuiToTray::ActiveProfileChanged(new_active) => {
                            tracing::info!(
                                "Received ActiveProfileChanged from Settings: {:?}",
                                new_active
                            );
                            tray.set_active_profile(new_active);
                        }
                        GuiToTray::ProfilesUpdated(_profiles) => {
                            tracing::info!("Received ProfilesUpdated from Settings");
                            // TrayIconManager doesn't need profiles, just tooltip
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
                    settings_connected = false;
                }
            }

            // Small sleep to avoid busy-waiting
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

/// Spawn the Settings window process
/// Optional flag can be passed (e.g., "--show-flyout" to immediately show flyout)
fn spawn_settings_window(flag: Option<&str>) -> Result<()> {
    // First check if Settings window is already running
    if bring_existing_settings_to_front() {
        tracing::info!("Settings window already exists, brought to front");
        return Ok(());
    }

    let exe_dir = std::env::current_exe()?
        .parent()
        .context("Failed to get executable directory")?
        .to_path_buf();

    let settings_exe = exe_dir.join("EdgeOptimizer_Settings.exe");

    if !settings_exe.exists() {
        // Try alternate casing
        let alt_exe = exe_dir.join("edge_optimizer_settings.exe");
        if alt_exe.exists() {
            let mut cmd = Command::new(&alt_exe);
            if let Some(f) = flag {
                cmd.arg(f);
            }
            cmd.spawn().context("Failed to spawn Settings process")?;
            tracing::info!("Settings process spawned: {:?} {:?}", alt_exe, flag);
            return Ok(());
        }
        anyhow::bail!("Settings executable not found: {:?}", settings_exe);
    }

    let mut cmd = Command::new(&settings_exe);
    if let Some(f) = flag {
        cmd.arg(f);
    }
    cmd.spawn().context("Failed to spawn Settings process")?;

    tracing::info!("Settings process spawned: {:?} {:?}", settings_exe, flag);

    Ok(())
}

/// Try to find and bring existing Settings window to front
/// Returns true if window was found and brought to front
fn bring_existing_settings_to_front() -> bool {
    unsafe {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::*;

        // Try to find Settings window by title
        let title: Vec<u16> = "Edge Optimizer - Profile Manager\0"
            .encode_utf16()
            .collect();
        let hwnd = FindWindowW(None, windows::core::PCWSTR(title.as_ptr()));

        if hwnd != HWND::default() {
            tracing::info!("Found existing Settings window, bringing to front");

            // Restore if minimized
            if IsIconic(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            }

            // Bring to foreground
            let _ = SetForegroundWindow(hwnd);
            let _ = BringWindowToTop(hwnd);

            return true;
        }

        false
    }
}
