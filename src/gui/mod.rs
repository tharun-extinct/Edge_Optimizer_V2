/// ICED GUI Application Module with System Tray Integration
mod profile_editor;
pub mod styles;

use iced::{
    executor, Application, Command, Element, Settings, Length, Alignment, Theme, Subscription,
    widget::{Container, Column, Row, Text, Button, Scrollable, Checkbox, TextInput, Space, Toggler},
};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use crate::profile::Profile;
use crate::common_apps::COMMON_APPS;
use crate::config::get_data_directory;
use crate::profile::{load_profiles, save_profiles};
use crate::image_picker::{open_image_picker, validate_crosshair_image};
use crate::process::{list_processes, kill_processes, ProcessInfo};
use crate::crosshair_overlay::{self, OverlayHandle};
use crate::tray_flyout::TrayFlyoutManager;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use std::time::Instant;
use once_cell::sync::Lazy;
use tray_icon::{TrayIconEvent, MouseButton, MouseButtonState};
use tray_icon::menu::MenuEvent;

/// Global channel for tray icon events
static TRAY_EVENT_RX: Lazy<Mutex<Option<Receiver<TrayIconEvent>>>> = Lazy::new(|| Mutex::new(None));

/// Global channel for menu events
static MENU_EVENT_RX: Lazy<Mutex<Option<Receiver<MenuEvent>>>> = Lazy::new(|| Mutex::new(None));

/// Global sender for profile activations from flyout
static FLYOUT_PROFILE_RX: Lazy<Mutex<Option<Receiver<String>>>> = Lazy::new(|| Mutex::new(None));

/// Track click timing for double-click detection
static LAST_CLICK_TIME: Lazy<Mutex<Option<Instant>>> = Lazy::new(|| Mutex::new(None));
static PENDING_SINGLE_CLICK: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

/// Store menu item IDs for checking exit
static MENU_EXIT_ID: Lazy<Mutex<Option<tray_icon::menu::MenuId>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug, Clone)]
pub enum Message {
    // Profile management
    ProfileNameChanged(String),
    ProfileSelected(usize),
    NewProfile,
    SaveProfile,
    DeleteProfile,
    ActivateProfile,
    
    // Process selection
    ProcessToggled(String, bool),
    RefreshProcesses,
    ProcessFilterChanged(String),
    
    // Crosshair settings
    CrosshairOffsetXChanged(String),
    CrosshairOffsetYChanged(String),
    CrosshairMoveUp,
    CrosshairMoveDown,
    CrosshairMoveLeft,
    CrosshairMoveRight,
    CrosshairCenter,
    OverlayEnabledToggled(bool),
    SelectImage,
    ClearImage,
    
    // Fan control
    FanSpeedMaxToggled(bool),
    
    // Tray events
    TrayTick,
    TrayProfileSelected(String),
    TrayDeactivate,
    TrayExit,
}

pub struct GameOptimizer {
    profiles: Vec<Profile>,
    selected_profile_index: Option<usize>,
    
    // Current editing state
    edit_name: String,
    edit_x_offset: String,
    edit_y_offset: String,
    edit_image_path: Option<String>,
    edit_overlay_enabled: bool,
    edit_fan_speed_max: bool,
    
    // Process selection (executable name -> selected)
    process_selection: HashMap<String, bool>,
    
    // Live system processes
    running_processes: Vec<ProcessInfo>,
    process_filter: String,
    
    // Status message
    status_message: String,
    
    // Data directory
    data_dir: Option<std::path::PathBuf>,
    
    // Active profile
    active_profile_name: Option<String>,
    
    // Crosshair overlay handle
    overlay_handle: Option<OverlayHandle>,
    
    // Tray manager (kept in app state since TrayIcon is !Send)
    tray_manager: Option<TrayFlyoutManager>,
}

/// Tray action to be processed by the app
#[derive(Debug, Clone)]
enum TrayAction {
    ShowFlyout,
    HideFlyout,
    ProfileSelected(String),
    Exit,
    None,
}

/// Process tray events - returns action for the app to handle
fn process_tray_events() -> TrayAction {
    // IMPORTANT: Pump Windows messages for tray icon to work
    // iced's winit doesn't process these by default
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::*;
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            // Don't process WM_QUIT here - let iced handle shutdown
            if msg.message == WM_QUIT {
                println!("[GUI] WM_QUIT received in message pump - ignoring");
                continue;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    
    // Check for profile activation from flyout
    if let Ok(guard) = FLYOUT_PROFILE_RX.lock() {
        if let Some(ref rx) = *guard {
            if let Ok(profile_name) = rx.try_recv() {
                println!("[GUI] Profile activated from flyout: {}", profile_name);
                return TrayAction::ProfileSelected(profile_name);
            }
        }
    }
    
    // Check for menu events (right-click context menu)
    if let Ok(guard) = MENU_EVENT_RX.lock() {
        if let Some(ref rx) = *guard {
            if let Ok(event) = rx.try_recv() {
                println!("[GUI] Menu event received: {:?}", event);
                // Check if it's the exit item
                if let Ok(exit_guard) = MENU_EXIT_ID.lock() {
                    if let Some(ref exit_id) = *exit_guard {
                        if event.id == *exit_id {
                            return TrayAction::Exit;
                        }
                    }
                }
            }
        }
    }
    
    // Check for tray icon click events
    if let Ok(guard) = TRAY_EVENT_RX.lock() {
        if let Some(ref rx) = *guard {
            if let Ok(event) = rx.try_recv() {
                match event {
                    TrayIconEvent::Click { button, button_state, .. } => {
                        if button == MouseButton::Left && button_state == MouseButtonState::Up {
                            let now = Instant::now();
                            
                            // Check for double-click
                            let is_double_click = if let Ok(guard) = LAST_CLICK_TIME.lock() {
                                if let Some(last_time) = *guard {
                                    now.duration_since(last_time).as_millis() < 500
                                } else {
                                    false
                                }
                            } else {
                                false
                            };
                            
                            if is_double_click {
                                // Double-click - clear state
                                if let Ok(mut guard) = LAST_CLICK_TIME.lock() {
                                    *guard = None;
                                }
                                if let Ok(mut guard) = PENDING_SINGLE_CLICK.lock() {
                                    *guard = false;
                                }
                                println!("[GUI] Double-click detected - GUI already open");
                                // GUI is already open, nothing to do
                            } else {
                                // First click - start timer
                                if let Ok(mut guard) = LAST_CLICK_TIME.lock() {
                                    *guard = Some(now);
                                }
                                if let Ok(mut guard) = PENDING_SINGLE_CLICK.lock() {
                                    *guard = true;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    // Check if single-click timer expired (show flyout)
    let should_toggle_flyout = if let Ok(guard) = PENDING_SINGLE_CLICK.lock() {
        if *guard {
            if let Ok(time_guard) = LAST_CLICK_TIME.lock() {
                if let Some(last_time) = *time_guard {
                    Instant::now().duration_since(last_time).as_millis() >= 500
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    
    if should_toggle_flyout {
        // Clear pending state
        if let Ok(mut guard) = PENDING_SINGLE_CLICK.lock() {
            *guard = false;
        }
        return TrayAction::ShowFlyout;
    }
    
    TrayAction::None
}

impl GameOptimizer {
    fn load_profiles_from_disk(&mut self) {
        if let Some(ref data_dir) = self.data_dir {
            match load_profiles(data_dir) {
                Ok(profiles) => {
                    self.profiles = profiles;
                    self.status_message = format!("Loaded {} profiles", self.profiles.len());
                }
                Err(e) => {
                    self.status_message = format!("Failed to load profiles: {}", e);
                }
            }
        }
    }
    
    fn save_profiles_to_disk(&mut self) {
        if let Some(ref data_dir) = self.data_dir {
            match save_profiles(&self.profiles, data_dir) {
                Ok(_) => {
                    self.status_message = "Profiles saved successfully".to_string();
                }
                Err(e) => {
                    self.status_message = format!("Failed to save profiles: {}", e);
                }
            }
        }
    }
    
    fn refresh_running_processes(&mut self) {
        self.running_processes = list_processes();
        self.running_processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    }
    
    fn clear_edit_form(&mut self) {
        self.edit_name = String::new();
        self.edit_x_offset = "0".to_string();
        self.edit_y_offset = "0".to_string();
        self.edit_image_path = None;
        self.edit_overlay_enabled = false;
        self.edit_fan_speed_max = false;
        self.process_selection.clear();
        self.selected_profile_index = None;
    }
    
    fn load_profile_to_edit(&mut self, index: usize) {
        if let Some(profile) = self.profiles.get(index) {
            self.edit_name = profile.name.clone();
            self.edit_x_offset = profile.crosshair_x_offset.to_string();
            self.edit_y_offset = profile.crosshair_y_offset.to_string();
            self.edit_image_path = profile.crosshair_image_path.clone();
            self.edit_overlay_enabled = profile.overlay_enabled;
            self.edit_fan_speed_max = profile.fan_speed_max;
            
            self.process_selection.clear();
            for proc in &profile.processes_to_kill {
                self.process_selection.insert(proc.clone(), true);
            }
            
            self.selected_profile_index = Some(index);
        }
    }
    
    fn get_selected_processes(&self) -> Vec<String> {
        self.process_selection
            .iter()
            .filter(|(_, &selected)| selected)
            .map(|(name, _)| name.clone())
            .collect()
    }
    
    fn activate_profile_by_name(&mut self, name: &str) {
        if let Some(index) = self.profiles.iter().position(|p| p.name == name) {
            self.selected_profile_index = Some(index);
            self.load_profile_to_edit(index);
            self.activate_current_profile();
        }
    }
    
    fn activate_current_profile(&mut self) {
        if let Some(index) = self.selected_profile_index {
            if let Some(profile) = self.profiles.get(index) {
                let profile_name = profile.name.clone();
                let processes = profile.processes_to_kill.clone();
                let fan_max = profile.fan_speed_max;
                let overlay_enabled = profile.overlay_enabled;
                let image_path = profile.crosshair_image_path.clone();
                let x_offset = profile.crosshair_x_offset;
                let y_offset = profile.crosshair_y_offset;
                
                let report = kill_processes(&processes);
                
                let mut status_parts = Vec::new();
                
                if !report.killed.is_empty() {
                    status_parts.push(format!("Killed: {}", report.killed.join(", ")));
                }
                if !report.not_found.is_empty() {
                    status_parts.push(format!("Not running: {}", report.not_found.join(", ")));
                }
                if !report.blocklist_skipped.is_empty() {
                    status_parts.push(format!("Protected: {}", report.blocklist_skipped.join(", ")));
                }
                
                self.active_profile_name = Some(profile_name.clone());
                
                if fan_max {
                    status_parts.push("Fan: MAX".to_string());
                }
                
                // Handle crosshair overlay
                // First, stop any existing overlay
                if let Some(ref mut handle) = self.overlay_handle {
                    handle.stop();
                }
                self.overlay_handle = None;
                
                // Start new overlay if enabled and image path exists
                if overlay_enabled {
                    if let Some(ref path) = image_path {
                        match crosshair_overlay::start_overlay(path.clone(), x_offset, y_offset) {
                            Ok(handle) => {
                                self.overlay_handle = Some(handle);
                                status_parts.push("🎯 Crosshair ON".to_string());
                            }
                            Err(e) => {
                                status_parts.push(format!("Crosshair error: {}", e));
                            }
                        }
                    } else {
                        status_parts.push("Crosshair: No image".to_string());
                    }
                }
                
                if status_parts.is_empty() {
                    self.status_message = format!("✅ Profile '{}' activated!", profile_name);
                } else {
                    self.status_message = format!("✅ Profile '{}' activated! {}", profile_name, status_parts.join(" | "));
                }
                
                self.refresh_running_processes();
                
                // Update tray with new active profile
                self.update_tray();
            }
        } else {
            self.status_message = "⚠️ No profile selected to activate".to_string();
        }
    }
    
    fn deactivate_profile(&mut self) {
        self.active_profile_name = None;
        
        // Stop overlay when deactivating
        if let Some(ref mut handle) = self.overlay_handle {
            handle.stop();
        }
        self.overlay_handle = None;
        
        self.status_message = "Profile deactivated".to_string();
        self.update_tray();
    }
    
    /// Update the live crosshair overlay with new offsets (restarts if running)
    fn update_live_overlay(&mut self) {
        // Only update if we have an active overlay
        if self.overlay_handle.is_some() {
            // Stop existing overlay
            if let Some(ref handle) = self.overlay_handle {
                handle.stop();
            }
            self.overlay_handle = None;
            
            // Restart with new offsets if we have an image
            if self.edit_overlay_enabled {
                if let Some(ref path) = self.edit_image_path {
                    let x_offset: i32 = self.edit_x_offset.parse().unwrap_or(0);
                    let y_offset: i32 = self.edit_y_offset.parse().unwrap_or(0);
                    
                    match crosshair_overlay::start_overlay(path.clone(), x_offset, y_offset) {
                        Ok(handle) => {
                            self.overlay_handle = Some(handle);
                        }
                        Err(e) => {
                            self.status_message = format!("Crosshair error: {}", e);
                        }
                    }
                }
            }
        }
    }
    
    fn update_tray(&mut self) {
        // Update tray with current profiles
        if let Some(ref mut tray) = self.tray_manager {
            tray.update_profiles(self.profiles.clone());
            tray.set_active_profile(self.active_profile_name.clone());
        }
    }
    
    fn toggle_flyout(&mut self) {
        if let Some(ref mut tray) = self.tray_manager {
            if tray.is_flyout_visible() {
                tray.hide_flyout();
            } else {
                if let Err(e) = tray.show_flyout() {
                    eprintln!("[GUI] Failed to show flyout: {}", e);
                }
            }
        }
    }
}

impl Application for GameOptimizer {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let data_dir = get_data_directory().ok();
        let mut app = GameOptimizer {
            profiles: Vec::new(),
            selected_profile_index: None,
            edit_name: String::new(),
            edit_x_offset: "0".to_string(),
            edit_y_offset: "0".to_string(),
            edit_image_path: None,
            edit_overlay_enabled: false,
            edit_fan_speed_max: false,
            process_selection: HashMap::new(),
            running_processes: Vec::new(),
            process_filter: String::new(),
            status_message: "Welcome to Gaming Optimizer".to_string(),
            data_dir,
            active_profile_name: None,
            overlay_handle: None,
            tray_manager: None,  // Will be set by run() via Flags if we change approach
        };
        app.load_profiles_from_disk();
        app.refresh_running_processes();
        
        // Create tray manager on main thread (inside iced's new)
        let app_config = crate::config::load_config();
        match TrayFlyoutManager::new_with_channels(app.profiles.clone(), app_config.active_profile) {
            Ok((tray, event_rx, menu_rx, profile_rx)) => {
                // Store the exit menu ID
                if let Ok(mut guard) = MENU_EXIT_ID.lock() {
                    *guard = Some(tray.menu_item_exit.clone());
                }
                // Store channels in globals
                if let Ok(mut guard) = TRAY_EVENT_RX.lock() {
                    *guard = Some(event_rx);
                }
                if let Ok(mut guard) = MENU_EVENT_RX.lock() {
                    *guard = Some(menu_rx);
                }
                if let Ok(mut guard) = FLYOUT_PROFILE_RX.lock() {
                    *guard = Some(profile_rx);
                }
                app.tray_manager = Some(tray);
                println!("[GUI] Tray manager created successfully");
            }
            Err(e) => {
                eprintln!("[GUI] Failed to create tray: {}", e);
            }
        }
        
        (app, Command::none())
    }

    fn title(&self) -> String {
        String::from("Gaming Optimizer - Profile Manager")
    }

    fn subscription(&self) -> Subscription<Message> {
        // Poll for tray events (faster polling for responsive click detection)
        struct TrayPoller;
        
        iced::subscription::unfold(
            std::any::TypeId::of::<TrayPoller>(),
            (),
            |_| async move {
                std::thread::sleep(Duration::from_millis(50)); // 50ms for responsive clicks
                (Message::TrayTick, ())
            }
        )
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::TrayTick => {
                // Process tray events (clicks, menu, flyout profile selection)
                match process_tray_events() {
                    TrayAction::ShowFlyout => {
                        self.toggle_flyout();
                    }
                    TrayAction::ProfileSelected(name) => {
                        return self.update(Message::TrayProfileSelected(name));
                    }
                    TrayAction::Exit => {
                        return self.update(Message::TrayExit);
                    }
                    _ => {}
                }
            }
            
            Message::TrayProfileSelected(name) => {
                self.activate_profile_by_name(&name);
            }
            
            Message::TrayDeactivate => {
                self.deactivate_profile();
            }
            
            Message::TrayExit => {
                // Clean exit
                std::process::exit(0);
            }
            
            Message::ProfileNameChanged(name) => {
                self.edit_name = name;
            }
            
            Message::ProfileSelected(index) => {
                self.load_profile_to_edit(index);
                self.status_message = format!("Editing profile: {}", self.edit_name);
            }
            
            Message::NewProfile => {
                self.clear_edit_form();
                self.status_message = "Creating new profile".to_string();
            }
            
            Message::SaveProfile => {
                if self.edit_name.trim().is_empty() {
                    self.status_message = "❌ Error: Profile name cannot be empty".to_string();
                    return Command::none();
                }
                
                let x_offset = self.edit_x_offset.parse().unwrap_or(0);
                let y_offset = self.edit_y_offset.parse().unwrap_or(0);
                
                let profile = Profile {
                    name: self.edit_name.clone(),
                    processes_to_kill: self.get_selected_processes(),
                    crosshair_image_path: self.edit_image_path.clone(),
                    crosshair_x_offset: x_offset,
                    crosshair_y_offset: y_offset,
                    overlay_enabled: self.edit_overlay_enabled,
                    fan_speed_max: self.edit_fan_speed_max,
                };
                
                if let Some(index) = self.selected_profile_index {
                    self.profiles[index] = profile;
                    self.status_message = format!("✅ Updated profile: {}", self.edit_name);
                } else {
                    self.profiles.push(profile);
                    self.selected_profile_index = Some(self.profiles.len() - 1);
                    self.status_message = format!("✅ Created profile: {}", self.edit_name);
                }
                
                self.save_profiles_to_disk();
                self.update_tray();
            }
            
            Message::DeleteProfile => {
                if let Some(index) = self.selected_profile_index {
                    let name = self.profiles[index].name.clone();
                    self.profiles.remove(index);
                    self.clear_edit_form();
                    self.save_profiles_to_disk();
                    self.update_tray();
                    self.status_message = format!("🗑️ Deleted profile: {}", name);
                }
            }
            
            Message::ActivateProfile => {
                self.activate_current_profile();
            }
            
            Message::ProcessToggled(process, enabled) => {
                self.process_selection.insert(process, enabled);
            }
            
            Message::RefreshProcesses => {
                self.refresh_running_processes();
                self.status_message = format!("🔄 Refreshed: {} processes found", self.running_processes.len());
            }
            
            Message::ProcessFilterChanged(filter) => {
                self.process_filter = filter;
            }
            
            Message::CrosshairOffsetXChanged(value) => {
                self.edit_x_offset = value;
            }
            
            Message::CrosshairOffsetYChanged(value) => {
                self.edit_y_offset = value;
            }
            
            Message::CrosshairMoveUp => {
                let current: i32 = self.edit_y_offset.parse().unwrap_or(0);
                self.edit_y_offset = (current - 1).to_string();
                self.update_live_overlay();
            }
            
            Message::CrosshairMoveDown => {
                let current: i32 = self.edit_y_offset.parse().unwrap_or(0);
                self.edit_y_offset = (current + 1).to_string();
                self.update_live_overlay();
            }
            
            Message::CrosshairMoveLeft => {
                let current: i32 = self.edit_x_offset.parse().unwrap_or(0);
                self.edit_x_offset = (current - 1).to_string();
                self.update_live_overlay();
            }
            
            Message::CrosshairMoveRight => {
                let current: i32 = self.edit_x_offset.parse().unwrap_or(0);
                self.edit_x_offset = (current + 1).to_string();
                self.update_live_overlay();
            }
            
            Message::CrosshairCenter => {
                self.edit_x_offset = "0".to_string();
                self.edit_y_offset = "0".to_string();
                self.status_message = "Crosshair centered".to_string();
                self.update_live_overlay();
            }
            
            Message::OverlayEnabledToggled(enabled) => {
                self.edit_overlay_enabled = enabled;
            }
            
            Message::FanSpeedMaxToggled(enabled) => {
                self.edit_fan_speed_max = enabled;
            }
            
            Message::SelectImage => {
                match open_image_picker() {
                    Ok(path) => {
                        match validate_crosshair_image(&path) {
                            Ok(_) => {
                                let path_str = path.to_string_lossy().to_string();
                                self.edit_image_path = Some(path_str.clone());
                                self.status_message = format!("📁 Selected image: {}", path_str);
                            }
                            Err(e) => {
                                self.status_message = format!("❌ Invalid image: {}", e);
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
            
            Message::ClearImage => {
                self.edit_image_path = None;
                self.status_message = "Cleared crosshair image".to_string();
            }
        }
        
        Command::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Left panel - Profile list
        let mut profile_list = Column::new()
            .spacing(5)
            .padding(10)
            .push(Text::new("📋 Profiles").size(20))
            .push(Space::new(Length::Fill, Length::Fixed(10.0)));
        
        for (i, profile) in self.profiles.iter().enumerate() {
            let is_selected = self.selected_profile_index == Some(i);
            let is_active = self.active_profile_name.as_ref() == Some(&profile.name);
            
            let label = if is_active {
                format!("🟢 {}", profile.name)
            } else if is_selected {
                format!("▶ {}", profile.name)
            } else {
                profile.name.clone()
            };
            
            profile_list = profile_list.push(
                Button::new(Text::new(label))
                    .on_press(Message::ProfileSelected(i))
                    .width(Length::Fill)
                    .padding(8)
            );
        }
        
        profile_list = profile_list
            .push(Space::new(Length::Fill, Length::Fixed(10.0)))
            .push(
                Button::new(Text::new("+ New Profile"))
                    .on_press(Message::NewProfile)
                    .width(Length::Fill)
                    .padding(10)
            );
        
        let left_panel = Container::new(
            Scrollable::new(profile_list)
        )
        .width(Length::Fixed(200.0))
        .height(Length::Fill)
        .padding(10);
        
        // Right panel - Edit form
        let edit_section = Column::new()
            .spacing(15)
            .padding(20)
            .push(Text::new("✏️ Edit Profile").size(24))
            
            .push(Text::new("Profile Name"))
            .push(
                TextInput::new("Enter profile name...", &self.edit_name)
                    .on_input(Message::ProfileNameChanged)
                    .padding(10)
                    .width(Length::Fill)
            )
            
            .push(Space::new(Length::Fill, Length::Fixed(10.0)))
            
            .push(
                Row::new()
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(Text::new("🌀 Fan Speed").size(18))
                    .push(
                        Toggler::new(
                            Some("Set to MAX when active".to_string()),
                            self.edit_fan_speed_max,
                            Message::FanSpeedMaxToggled
                        )
                        .width(Length::Shrink)
                    )
            )
            
            .push(Space::new(Length::Fill, Length::Fixed(10.0)))
            
            .push(
                Row::new()
                    .spacing(10)
                    .align_items(Alignment::Center)
                    .push(Text::new("🔪 Processes to Kill").size(18))
                    .push(
                        Button::new(Text::new("🔄 Refresh"))
                            .on_press(Message::RefreshProcesses)
                            .padding(5)
                    )
            )
            .push(Text::new("Select running applications to close when activating:").size(12))
            .push(
                TextInput::new("Filter processes...", &self.process_filter)
                    .on_input(Message::ProcessFilterChanged)
                    .padding(8)
                    .width(Length::Fill)
            )
            .push(self.render_process_selector())
            
            .push(Space::new(Length::Fill, Length::Fixed(10.0)))
            
            .push(Text::new("🎯 Crosshair Overlay").size(18))
            .push(Text::new("Crosshair will be centered on screen. Use arrows for pixel-perfect adjustment.").size(12))
            
            // Image selection row
            .push(
                Row::new()
                    .spacing(10)
                    .align_items(Alignment::Center)
                    .push(
                        Button::new(Text::new("📁 Select Image"))
                            .on_press(Message::SelectImage)
                            .padding(10)
                    )
                    .push(
                        if self.edit_image_path.is_some() {
                            Button::new(Text::new("❌ Clear"))
                                .on_press(Message::ClearImage)
                                .padding(10)
                        } else {
                            Button::new(Text::new("❌ Clear")).padding(10)
                        }
                    )
                    .push(
                        if let Some(ref path) = self.edit_image_path {
                            Text::new(format!("✓ {}", path.split('\\').last().unwrap_or(path))).size(12)
                        } else {
                            Text::new("No image (100x100 PNG recommended)").size(12)
                        }
                    )
            )
            
            // Crosshair adjustment box
            .push(
                Container::new(
                    Column::new()
                        .spacing(5)
                        .align_items(Alignment::Center)
                        .push(Text::new("Position Adjustment").size(14))
                        .push(
                            Row::new()
                                .spacing(10)
                                .align_items(Alignment::Center)
                                .push(Space::new(Length::Fixed(40.0), Length::Shrink))
                                .push(
                                    Button::new(Text::new("▲").size(16))
                                        .on_press(Message::CrosshairMoveUp)
                                        .padding(8)
                                        .width(Length::Fixed(40.0))
                                )
                                .push(Space::new(Length::Fixed(40.0), Length::Shrink))
                        )
                        .push(
                            Row::new()
                                .spacing(5)
                                .align_items(Alignment::Center)
                                .push(
                                    Button::new(Text::new("◀").size(16))
                                        .on_press(Message::CrosshairMoveLeft)
                                        .padding(8)
                                        .width(Length::Fixed(40.0))
                                )
                                .push(
                                    Button::new(Text::new("⊙").size(14))
                                        .on_press(Message::CrosshairCenter)
                                        .padding(8)
                                        .width(Length::Fixed(50.0))
                                )
                                .push(
                                    Button::new(Text::new("▶").size(16))
                                        .on_press(Message::CrosshairMoveRight)
                                        .padding(8)
                                        .width(Length::Fixed(40.0))
                                )
                        )
                        .push(
                            Row::new()
                                .spacing(10)
                                .align_items(Alignment::Center)
                                .push(Space::new(Length::Fixed(40.0), Length::Shrink))
                                .push(
                                    Button::new(Text::new("▼").size(16))
                                        .on_press(Message::CrosshairMoveDown)
                                        .padding(8)
                                        .width(Length::Fixed(40.0))
                                )
                                .push(Space::new(Length::Fixed(40.0), Length::Shrink))
                        )
                        .push(
                            Text::new(format!("Offset: X={}, Y={}", self.edit_x_offset, self.edit_y_offset)).size(12)
                        )
                )
                .padding(15)
                .width(Length::Fixed(200.0))
            )
            
            // Manual offset input (for precise values)
            .push(
                Row::new()
                    .spacing(15)
                    .align_items(Alignment::Center)
                    .push(Text::new("Manual:").size(12))
                    .push(
                        Row::new()
                            .spacing(5)
                            .align_items(Alignment::Center)
                            .push(Text::new("X").size(12))
                            .push(
                                TextInput::new("0", &self.edit_x_offset)
                                    .on_input(Message::CrosshairOffsetXChanged)
                                    .width(Length::Fixed(60.0))
                                    .padding(5)
                            )
                    )
                    .push(
                        Row::new()
                            .spacing(5)
                            .align_items(Alignment::Center)
                            .push(Text::new("Y").size(12))
                            .push(
                                TextInput::new("0", &self.edit_y_offset)
                                    .on_input(Message::CrosshairOffsetYChanged)
                                    .width(Length::Fixed(60.0))
                                    .padding(5)
                            )
                    )
            )
            
            .push(
                Checkbox::new("Enable crosshair overlay", self.edit_overlay_enabled)
                    .on_toggle(Message::OverlayEnabledToggled)
            )
            
            .push(Space::new(Length::Fill, Length::Fixed(20.0)))
            
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        Button::new(Text::new("💾 Save Profile"))
                            .on_press(Message::SaveProfile)
                            .padding(12)
                    )
                    .push(
                        if self.selected_profile_index.is_some() {
                            Button::new(Text::new("🗑️ Delete"))
                                .on_press(Message::DeleteProfile)
                                .padding(12)
                        } else {
                            Button::new(Text::new("🗑️ Delete")).padding(12)
                        }
                    )
                    .push(
                        if self.selected_profile_index.is_some() {
                            Button::new(Text::new("⚡ ACTIVATE"))
                                .on_press(Message::ActivateProfile)
                                .padding(12)
                        } else {
                            Button::new(Text::new("⚡ ACTIVATE")).padding(12)
                        }
                    )
            );
        
        let right_panel = Container::new(
            Scrollable::new(edit_section)
        )
        .width(Length::Fill)
        .height(Length::Fill);
        
        let content = Column::new()
            .push(
                Row::new()
                    .push(left_panel)
                    .push(right_panel)
                    .height(Length::FillPortion(9))
            )
            .push(
                Container::new(
                    Row::new()
                        .spacing(20)
                        .push(Text::new(&self.status_message).size(14))
                        .push(Space::new(Length::Fill, Length::Shrink))
                        .push(
                            if let Some(ref name) = self.active_profile_name {
                                Text::new(format!("🟢 Active: {} | 📌 Tray", name)).size(14)
                            } else {
                                Text::new("No active profile | 📌 Tray").size(14)
                            }
                        )
                )
                .width(Length::Fill)
                .padding(10)
                .height(Length::FillPortion(1))
            );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl GameOptimizer {
    fn render_process_selector(&self) -> Element<Message> {
        let filter_lower = self.process_filter.to_lowercase();
        
        let mut seen: HashSet<String> = HashSet::new();
        let mut processes_to_show: Vec<(&str, &str, Option<f32>, Option<u64>)> = Vec::new();
        
        for proc in &self.running_processes {
            let name_lower = proc.name.to_lowercase();
            if !seen.contains(&name_lower) {
                if filter_lower.is_empty() || name_lower.contains(&filter_lower) {
                    seen.insert(name_lower);
                    processes_to_show.push((
                        &proc.name,
                        &proc.name,
                        Some(proc.cpu_percent),
                        Some(proc.memory_kb)
                    ));
                }
            }
        }
        
        for (name, exe) in COMMON_APPS.iter() {
            let exe_lower = exe.to_lowercase();
            if !seen.contains(&exe_lower) {
                if self.process_selection.get(*exe).copied().unwrap_or(false) {
                    if filter_lower.is_empty() || exe_lower.contains(&filter_lower) || name.to_lowercase().contains(&filter_lower) {
                        seen.insert(exe_lower);
                        processes_to_show.push((name, exe, None, None));
                    }
                }
            }
        }
        
        processes_to_show.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        
        let mut grid = Column::new().spacing(3);
        
        if processes_to_show.is_empty() {
            grid = grid.push(Text::new("No processes found matching filter").size(12));
        } else {
            for (display_name, exe_name, cpu, mem) in processes_to_show.iter().take(50) {
                let is_selected = self.process_selection.get(*exe_name).copied().unwrap_or(false);
                let exe_string = exe_name.to_string();
                
                let info = match (cpu, mem) {
                    (Some(c), Some(m)) => format!("{} - CPU: {:.1}% | {} MB", display_name, c, m / 1024),
                    _ => format!("{} (not running)", display_name),
                };
                
                grid = grid.push(
                    Checkbox::new(info, is_selected)
                        .on_toggle(move |checked| Message::ProcessToggled(exe_string.clone(), checked))
                        .width(Length::Fill)
                );
            }
            
            if processes_to_show.len() > 50 {
                grid = grid.push(
                    Text::new(format!("... and {} more (use filter)", processes_to_show.len() - 50)).size(12)
                );
            }
        }
        
        Container::new(
            Scrollable::new(grid).height(Length::Fixed(200.0))
        )
        .width(Length::Fill)
        .into()
    }
}

pub fn run() -> iced::Result {
    println!("[GUI] Starting GUI with integrated tray...");
    
    // Tray is created inside Application::new() on main thread
    let result = GameOptimizer::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(1000.0, 750.0),
            min_size: Some(iced::Size::new(900.0, 650.0)),
            ..Default::default()
        },
        ..Default::default()
    });
    
    println!("[GUI] Iced returned: {:?}", result);
    result
}
