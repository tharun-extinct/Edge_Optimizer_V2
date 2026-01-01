/// System tray with flyout menu integration
/// 
/// This module provides a simplified tray icon that spawns a custom flyout window
/// instead of using native OS context menus.

use crate::flyout::FlyoutWindow;
use crate::ipc::{TrayChannels, GuiToTray};
use crate::profile::Profile;
use anyhow::{anyhow, Result};
use std::sync::mpsc::{Sender, TryRecvError, Receiver};
use std::time::Instant;
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState, Icon, menu::MenuEvent};
use tray_icon::menu::{Menu, MenuItem, MenuId, PredefinedMenuItem};

/// Load application icon from favicon.ico file
fn load_app_icon() -> Result<Icon> {
    let icon_path = std::path::Path::new("favicon.ico");
    
    if !icon_path.exists() {
        anyhow::bail!("favicon.ico not found in project root!");
    }
    
    let icon_data = std::fs::read(icon_path)
        .map_err(|e| anyhow!("Failed to read favicon.ico: {}", e))?;
    
    // Try direct loading first
    Icon::from_rgba(icon_data.clone(), 16, 16)
        .or_else(|_| {
            // If direct loading fails, decode with image crate
            let img = image::load_from_memory(&icon_data)
                .map_err(|e| anyhow!("Failed to decode icon: {}", e))?;
            
            let img = img.resize_exact(16, 16, image::imageops::FilterType::Lanczos3);
            let rgba = img.to_rgba8();
            
            Icon::from_rgba(rgba.into_raw(), 16, 16)
                .map_err(|e| anyhow!("Failed to create icon from image: {:?}", e))
        })
}

/// Simplified tray manager that works with flyout
pub struct TrayFlyoutManager {
    tray_icon: TrayIcon,
    flyout: Option<FlyoutWindow>,
    profiles: Vec<Profile>,
    active_profile: Option<String>,
    menu_item_settings: MenuId,
    menu_item_docs: MenuId,
    menu_item_bug_report: MenuId,
    menu_item_exit: MenuId,
    last_click_time: Option<Instant>,
    pending_single_click: bool,
}

impl TrayFlyoutManager {
    /// Create a new tray icon without menu
    pub fn new(profiles: Vec<Profile>, active_profile: Option<String>) -> Result<Self> {
        let tooltip = if let Some(ref name) = active_profile {
            format!("Gaming Optimizer - {}", name)
        } else {
            "Gaming Optimizer - Inactive".to_string()
        };

        println!("[TRAY] Creating tray icon with {} profiles", profiles.len());
        
        let icon = load_app_icon()?;
        println!("[TRAY] Icon loaded");
        
        // Create context menu (appears on right-click)
        let menu = Menu::new();
        let settings_item = MenuItem::new("Open Settings", true, None);
        let docs_item = MenuItem::new("Documentation", true, None);
        let bug_item = MenuItem::new("Report Bug", true, None);
        let separator = PredefinedMenuItem::separator();
        let exit_item = MenuItem::new("Exit", true, None);
        
        menu.append(&settings_item)
            .map_err(|e| anyhow!("Failed to add settings item: {}", e))?;
        menu.append(&docs_item)
            .map_err(|e| anyhow!("Failed to add docs item: {}", e))?;
        menu.append(&bug_item)
            .map_err(|e| anyhow!("Failed to add bug report item: {}", e))?;
        menu.append(&separator)
            .map_err(|e| anyhow!("Failed to add separator: {}", e))?;
        menu.append(&exit_item)
            .map_err(|e| anyhow!("Failed to add exit item: {}", e))?;
        
        // Store menu IDs for event handling
        let menu_item_settings = settings_item.id().clone();
        let menu_item_docs = docs_item.id().clone();
        let menu_item_bug_report = bug_item.id().clone();
        let menu_item_exit = exit_item.id().clone();
        
        let tray_icon = TrayIconBuilder::new()
            .with_tooltip(&tooltip)
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .build()
            .map_err(|e| anyhow!("Failed to create tray icon: {}", e))?;
        
        println!("[TRAY] Tray icon created successfully with context menu");

        Ok(TrayFlyoutManager {
            tray_icon,
            flyout: None,
            profiles,
            active_profile,
            menu_item_settings,
            menu_item_docs,
            menu_item_bug_report,
            menu_item_exit,
            last_click_time: None,
            pending_single_click: false,
        })
    }

    /// Show the flyout menu
    fn show_flyout(&mut self, to_gui_tx: &Sender<crate::ipc::TrayToGui>) -> Result<()> {
        println!("[FLYOUT] Attempting to show flyout menu");
        
        // Close existing flyout if any
        self.flyout = None;

        // Get tray icon rect for positioning
        let tray_rect = if let Some(rect) = self.tray_icon.rect() {
            println!("[FLYOUT] Tray icon position: {:?}, size: {:?}", rect.position, rect.size);
            windows::Win32::Foundation::RECT {
                left: rect.position.x as i32,
                top: rect.position.y as i32,
                right: (rect.position.x as i32 + rect.size.width as i32),
                bottom: (rect.position.y as i32 + rect.size.height as i32),
            }
        } else {
            println!("[FLYOUT] Warning: Could not get tray rect, using screen corner");
            // Fallback to lower-right corner
            use windows::Win32::UI::WindowsAndMessaging::*;
            unsafe {
                let screen_width = GetSystemMetrics(SM_CXSCREEN);
                let screen_height = GetSystemMetrics(SM_CYSCREEN);
                windows::Win32::Foundation::RECT {
                    left: screen_width - 100,
                    top: screen_height - 50,
                    right: screen_width,
                    bottom: screen_height,
                }
            }
        };

        // Create and show flyout
        println!("[FLYOUT] Creating flyout window with {} profiles", self.profiles.len());
        let flyout = FlyoutWindow::new(
            tray_rect,
            self.profiles.clone(),
            self.active_profile.clone(),
            to_gui_tx.clone(),
        )?;

        println!("[FLYOUT] Showing flyout window");
        flyout.show();
        self.flyout = Some(flyout);
        println!("[FLYOUT] Flyout displayed successfully");

        anyhow::Ok(())
    }

    /// Hide the flyout menu
    fn hide_flyout(&mut self) {
        self.flyout = None;
    }

    /// Update tooltip based on active profile
    fn update_tooltip(&mut self) {
        let tooltip = if let Some(ref name) = self.active_profile {
            format!("Gaming Optimizer - {}", name)
        } else {
            "Gaming Optimizer - Inactive".to_string()
        };
        
        self.tray_icon.set_tooltip(Some(&tooltip));
    }

    /// Update profiles list
    pub fn update_profiles(&mut self, profiles: Vec<Profile>) {
        self.profiles = profiles;
        if let Some(ref mut flyout) = self.flyout {
            let _ = flyout.update_profiles(self.profiles.clone(), self.active_profile.clone());
        }
    }

    /// Set active profile
    pub fn set_active_profile(&mut self, active: Option<String>) {
        self.active_profile = active;
        self.update_tooltip();
        if let Some(ref mut flyout) = self.flyout {
            let _ = flyout.update_profiles(self.profiles.clone(), self.active_profile.clone());
        }
    }
}

/// Run the tray with flyout on the main thread
pub fn run_tray_flyout_thread(
    channels: TrayChannels,
    initial_profiles: Vec<Profile>,
    active_profile: Option<String>,
) {
    use windows::Win32::UI::WindowsAndMessaging::*;
    
    println!("[TRAY] Starting tray flyout on main thread");
    
    // Create the tray manager
    let mut tray = match TrayFlyoutManager::new(initial_profiles, active_profile) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[TRAY] Failed to create tray: {}", e);
            return;
        }
    };

    println!("[TRAY] Setting up event handler");
    
    // Create channels for tray icon and menu events
    let (event_tx, event_rx): (Sender<TrayIconEvent>, Receiver<TrayIconEvent>) = std::sync::mpsc::channel();
    let (menu_tx, menu_rx): (Sender<MenuEvent>, Receiver<MenuEvent>) = std::sync::mpsc::channel();
    
    // Set up event handler to forward events to our channel
    TrayIconEvent::set_event_handler(Some(move |event| {
        println!("[TRAY] *** EVENT HANDLER CALLED: {:?} ***", event);
        let _ = event_tx.send(event);
    }));
    
    // Set up menu event handler
    MenuEvent::set_event_handler(Some(move |event| {
        println!("[MENU] *** MENU EVENT: {:?} ***", event);
        let _ = menu_tx.send(event);
    }));

    println!("[TRAY] Event handler set, entering Windows message loop");

    // Windows message loop - required for tray icon events
    unsafe {
        let mut msg = MSG::default();
        loop {
            // Process Windows messages (this enables tray icon events)
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    println!("[TRAY] WM_QUIT received, exiting");
                    return;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            // Check for tray icon events
            match event_rx.try_recv() {
                Ok(event) => {
                    println!("[TRAY] Processing event: {:?}", event);
                    match event {
                        TrayIconEvent::Click { button, button_state, .. } => {
                            println!("[TRAY] Click - button: {:?}, state: {:?}", button, button_state);
                            
                            if button == MouseButton::Left && button_state == MouseButtonState::Up {
                                let now = Instant::now();
                                
                                // Check for double-click (within 500ms of last click)
                                if let Some(last_time) = tray.last_click_time {
                                    if now.duration_since(last_time).as_millis() < 500 {
                                        // Double-click detected!
                                        println!("[TRAY] DOUBLE CLICK - opening full GUI");
                                        tray.pending_single_click = false;
                                        tray.last_click_time = None;
                                        
                                        // Send message to open GUI
                                        let _ = channels.to_gui.send(crate::ipc::TrayToGui::OpenSettings);
                                        continue;
                                    }
                                }
                                
                                // First click - start timer for single-click
                                println!("[TRAY] First click detected, waiting for potential double-click");
                                tray.last_click_time = Some(now);
                                tray.pending_single_click = true;
                            }
                        }
                        _ => {}
                    }
                }
                Err(_) => {}
            }
            
            // Check if single-click timer expired (500ms passed)
            if tray.pending_single_click {
                if let Some(last_time) = tray.last_click_time {
                    if Instant::now().duration_since(last_time).as_millis() >= 500 {
                        // Single click confirmed - show flyout
                        println!("[TRAY] Single click confirmed - toggling flyout");
                        tray.pending_single_click = false;
                        
                        if tray.flyout.is_some() {
                            println!("[TRAY] Hiding existing flyout");
                            tray.hide_flyout();
                        } else {
                            println!("[TRAY] Showing new flyout");
                            if let Err(e) = tray.show_flyout(&channels.to_gui) {
                                eprintln!("[TRAY] Failed to show flyout: {}", e);
                            }
                        }
                    }
                }
            }
            
            // Check for menu events
            match menu_rx.try_recv() {
                Ok(event) => {
                    println!("[MENU] Processing menu event: {:?}", event);
                    if event.id == tray.menu_item_settings {
                        println!("[MENU] Open Settings clicked");
                        let _ = channels.to_gui.send(crate::ipc::TrayToGui::OpenSettings);
                    } else if event.id == tray.menu_item_docs {
                        println!("[MENU] Documentation clicked");
                        // Open documentation URL
                        if let Err(e) = open::that("https://github.com/yourusername/gaming_optimizer#readme") {
                            eprintln!("[MENU] Failed to open documentation: {}", e);
                        }
                    } else if event.id == tray.menu_item_bug_report {
                        println!("[MENU] Report Bug clicked");
                        // Open GitHub issues page
                        if let Err(e) = open::that("https://github.com/yourusername/gaming_optimizer/issues/new") {
                            eprintln!("[MENU] Failed to open bug report page: {}", e);
                        }
                    } else if event.id == tray.menu_item_exit {
                        println!("[MENU] Exit clicked");
                        let _ = channels.to_gui.send(crate::ipc::TrayToGui::Exit);
                        break;
                    }
                }
                Err(_) => {}
            }

            // Check for messages from GUI
            match channels.from_gui.try_recv() {
                Ok(msg) => match msg {
                    GuiToTray::ProfilesUpdated(new_profiles) => {
                        println!("[TRAY] Received ProfilesUpdated");
                        tray.update_profiles(new_profiles);
                    }
                    GuiToTray::ActiveProfileChanged(new_active) => {
                        println!("[TRAY] Received ActiveProfileChanged");
                        tray.set_active_profile(new_active);
                    }
                    GuiToTray::OverlayVisibilityChanged(_visible) => {
                        // Not used in flyout mode
                    }
                    GuiToTray::Shutdown => {
                        println!("[TRAY] Received shutdown signal");
                        break;
                    }
                },
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    println!("[TRAY] Channel disconnected, exiting");
                    break;
                }
            }

            // Small sleep to avoid busy-waiting
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    
    println!("[TRAY] Tray thread exiting");
}
