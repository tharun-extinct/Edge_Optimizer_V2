/// ICED GUI Application Module
///
/// Architecture:
/// - This GUI is owned by the Settings process
/// - Runner process owns the system tray and sends IPC messages
/// - We receive ShowFlyout/BringMainToFront commands via IPC from Runner
pub mod macro_editor;
mod profile_editor;
pub mod styles;

use crate::common_apps::COMMON_APPS;
use crate::config::get_data_directory;
use crate::crosshair_overlay::{self, OverlayHandle};
use crate::flyout::FlyoutWindow;
use crate::image_picker::{open_image_picker, validate_crosshair_image};
use crate::ipc::{GuiToTray, NamedPipeClient, TrayToGui};
use crate::macro_config::MacroConfig;
use crate::process::{kill_processes, list_processes, ProcessInfo};
use crate::profile::Profile;
use crate::profile::{load_profiles, save_profiles};
use iced::{
    executor,
    widget::{
        Button, Checkbox, Column, Container, Row, Scrollable, Space, Text, TextInput, Toggler,
    },
    Alignment, Application, Command, Element, Length, Settings, Subscription, Theme,
};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{self, Receiver};
use std::sync::Mutex;
use std::time::Duration;

/// Global channel for IPC messages from Runner
static IPC_MESSAGE_RX: Lazy<Mutex<Option<Receiver<TrayToGui>>>> = Lazy::new(|| Mutex::new(None));

/// Global sender for profile activations from flyout (flyout → GUI)
static FLYOUT_PROFILE_RX: Lazy<Mutex<Option<Receiver<String>>>> = Lazy::new(|| Mutex::new(None));

/// Startup flags for the GUI application
#[derive(Debug, Default, Clone)]
pub struct GuiFlags {
    /// Show flyout immediately on startup
    pub show_flyout: bool,
    /// Bring main window to front
    pub bring_to_front: bool,
    /// Flyout-only mode: main window starts hidden
    pub flyout_only: bool,
    /// IPC client (will be moved into the listener thread)
    pub ipc_client: Option<std::sync::Arc<Mutex<NamedPipeClient>>>,
}

/// Application pages for navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Profiles,
    Macros,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    NavigateTo(Page),

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

    // Macro editor
    MacroMessage(macro_editor::MacroMessage),
    SaveMacros,

    // IPC events from Runner
    IpcTick,
    IpcShowFlyout,
    IpcHideFlyout,
    IpcBringToFront,
    IpcExit,

    // Flyout events
    FlyoutProfileSelected(String),
    #[allow(dead_code)]
    FlyoutDeactivate,
    
    // Recording tick for polling recorded actions
    RecordingTick,
}

pub struct GameOptimizer {
    // Current page
    current_page: Page,

    profiles: Vec<Profile>,
    selected_profile_index: Option<usize>,

    // Macro editor state
    macro_editor_state: macro_editor::MacroEditorState,
    
    // Input recorder for macro recording
    input_recorder: crate::input_recorder::InputRecorder,

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

    // Flyout window (owned by Settings, triggered by IPC from Runner)
    flyout_window: Option<FlyoutWindow>,

    // IPC client for sending messages to Runner
    ipc_client: Option<std::sync::Arc<Mutex<NamedPipeClient>>>,

    // Startup flags
    pending_show_flyout: bool,
}

/// Process IPC messages from Runner - returns action for the app to handle
fn process_ipc_messages() -> Option<Message> {
    // Check for IPC messages from Runner
    if let Ok(guard) = IPC_MESSAGE_RX.lock() {
        if let Some(ref rx) = *guard {
            if let Ok(msg) = rx.try_recv() {
                return match msg {
                    TrayToGui::ShowFlyout => Some(Message::IpcShowFlyout),
                    TrayToGui::HideFlyout => Some(Message::IpcHideFlyout),
                    TrayToGui::BringMainToFront => Some(Message::IpcBringToFront),
                    TrayToGui::Exit => Some(Message::IpcExit),
                    TrayToGui::ActivateProfile(name) => Some(Message::FlyoutProfileSelected(name)),
                    TrayToGui::OpenSettings => Some(Message::IpcBringToFront),
                    _ => None,
                };
            }
        }
    }

    // Check for profile activation from flyout
    if let Ok(guard) = FLYOUT_PROFILE_RX.lock() {
        if let Some(ref rx) = *guard {
            if let Ok(profile_name) = rx.try_recv() {
                println!("[GUI] Profile activated from flyout: {}", profile_name);
                return Some(Message::FlyoutProfileSelected(profile_name));
            }
        }
    }

    None
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
        self.running_processes
            .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
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
                    status_parts.push(format!(
                        "Protected: {}",
                        report.blocklist_skipped.join(", ")
                    ));
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
                    self.status_message = format!(
                        "✅ Profile '{}' activated! {}",
                        profile_name,
                        status_parts.join(" | ")
                    );
                }

                self.refresh_running_processes();

                // Update tray with new active profile
                self.notify_runner_profile_changed();
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
        self.notify_runner_profile_changed();
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

    /// Send profile change notification to Runner via IPC
    fn notify_runner_profile_changed(&mut self) {
        if let Some(ref client) = self.ipc_client {
            if let Ok(client) = client.lock() {
                let msg = GuiToTray::ActiveProfileChanged(self.active_profile_name.clone());
                if let Err(e) = client.send(&msg) {
                    eprintln!("[GUI] Failed to notify Runner of profile change: {}", e);
                }
            }
        }
    }

    /// Show the flyout window (owned by Settings, triggered by IPC from Runner)
    fn show_flyout(&mut self) {
        println!("[GUI] Showing flyout window");

        // Close existing flyout if any
        self.flyout_window = None;

        // Get screen position for flyout (near taskbar)
        let tray_rect = unsafe {
            use windows::Win32::UI::WindowsAndMessaging::*;
            let screen_width = GetSystemMetrics(SM_CXSCREEN);
            let screen_height = GetSystemMetrics(SM_CYSCREEN);
            windows::Win32::Foundation::RECT {
                left: screen_width - 100,
                top: screen_height - 50,
                right: screen_width,
                bottom: screen_height,
            }
        };

        // Create IPC sender for flyout → GUI profile selection
        let (tx, rx) = mpsc::channel::<crate::ipc::TrayToGui>();

        // Store receivers
        let profile_tx = {
            let (ptx, prx) = mpsc::channel::<String>();
            if let Ok(mut guard) = FLYOUT_PROFILE_RX.lock() {
                *guard = Some(prx);
            }
            ptx
        };

        // Forward TrayToGui::ActivateProfile to String channel
        std::thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                if let crate::ipc::TrayToGui::ActivateProfile(name) = msg {
                    let _ = profile_tx.send(name);
                }
            }
        });

        // Create flyout window
        match FlyoutWindow::new(
            tray_rect,
            self.profiles.clone(),
            self.active_profile_name.clone(),
            tx,
        ) {
            Ok(flyout) => {
                flyout.show();
                self.flyout_window = Some(flyout);
                println!("[GUI] Flyout displayed successfully");
            }
            Err(e) => {
                eprintln!("[GUI] Failed to create flyout: {}", e);
            }
        }
    }

    /// Hide the flyout window
    fn hide_flyout(&mut self) {
        self.flyout_window = None;
    }

    /// Toggle flyout visibility
    #[allow(dead_code)]
    fn toggle_flyout(&mut self) {
        if self.flyout_window.is_some() {
            self.hide_flyout();
        } else {
            self.show_flyout();
        }
    }

    /// Bring main window to front using Win32 API
    fn bring_to_front(&self) {
        println!("[GUI] BringMainToFront requested");
        
        // Use Win32 APIs to find and bring our window to front
        unsafe {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::*;
            
            // Find window by class or enumerate to find ours
            // iced windows typically have the title we set
            let title: Vec<u16> = "Edge Optimizer - Profile Manager\0".encode_utf16().collect();
            let hwnd = FindWindowW(None, windows::core::PCWSTR(title.as_ptr()));
            
            if hwnd != HWND::default() {
                println!("[GUI] Found window, bringing to front");
                
                // Restore if minimized
                if IsIconic(hwnd).as_bool() {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                }
                
                // Bring to foreground
                let _ = SetForegroundWindow(hwnd);
                
                // Also try BringWindowToTop for good measure
                let _ = BringWindowToTop(hwnd);
            } else {
                println!("[GUI] Window not found by title, trying alternate method");
                // Window is likely already in focus since we're running
            }
        }
    }
}

impl Application for GameOptimizer {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = GuiFlags;

    fn new(flags: GuiFlags) -> (Self, Command<Message>) {
        let data_dir = get_data_directory().ok();
        let mut app = GameOptimizer {
            current_page: Page::default(),
            profiles: Vec::new(),
            selected_profile_index: None,
            macro_editor_state: macro_editor::MacroEditorState::default(),
            input_recorder: crate::input_recorder::InputRecorder::new(),
            edit_name: String::new(),
            edit_x_offset: "0".to_string(),
            edit_y_offset: "0".to_string(),
            edit_image_path: None,
            edit_overlay_enabled: false,
            edit_fan_speed_max: false,
            process_selection: HashMap::new(),
            running_processes: Vec::new(),
            process_filter: String::new(),
            status_message: "Welcome to Edge Optimizer".to_string(),
            data_dir,
            active_profile_name: None,
            overlay_handle: None,
            flyout_window: None,
            ipc_client: flags.ipc_client.clone(),
            pending_show_flyout: flags.show_flyout,
        };
        app.load_profiles_from_disk();
        app.refresh_running_processes();

        println!(
            "[GUI] Application initialized, pending_show_flyout={}",
            app.pending_show_flyout
        );

        // Return initial command to show flyout if requested
        let cmd = if flags.show_flyout {
            Command::perform(async {}, |_| Message::IpcShowFlyout)
        } else {
            Command::none()
        };

        (app, cmd)
    }

    fn title(&self) -> String {
        String::from("Edge Optimizer - Profile Manager")
    }

    fn subscription(&self) -> Subscription<Message> {
        // Poll for IPC messages from Runner
        struct IpcPoller;

        let ipc_sub = iced::subscription::unfold(std::any::TypeId::of::<IpcPoller>(), (), |_| async move {
            std::thread::sleep(Duration::from_millis(50)); // 50ms for responsive IPC
            (Message::IpcTick, ())
        });

        // Poll for recorded actions when recording
        struct RecordingPoller;
        
        let recording_sub = if self.macro_editor_state.is_recording {
            iced::subscription::unfold(std::any::TypeId::of::<RecordingPoller>(), (), |_| async move {
                std::thread::sleep(Duration::from_millis(100)); // 100ms for recording
                (Message::RecordingTick, ())
            })
        } else {
            Subscription::none()
        };

        Subscription::batch([ipc_sub, recording_sub])
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::NavigateTo(page) => {
                self.current_page = page;
                // When switching to Macros page, load macros from selected profile
                if page == Page::Macros {
                    if let Some(index) = self.selected_profile_index {
                        if let Some(profile) = self.profiles.get(index) {
                            self.macro_editor_state = macro_editor::MacroEditorState::new(
                                profile.macros.macros.clone()
                            );
                        }
                    } else {
                        self.macro_editor_state = macro_editor::MacroEditorState::default();
                    }
                }
            }

            Message::MacroMessage(macro_msg) => {
                // Handle start/stop recording specially
                match &macro_msg {
                    macro_editor::MacroMessage::StartRecording => {
                        self.macro_editor_state.update(macro_msg);
                        self.input_recorder.start_recording();
                        self.status_message = "🔴 Recording... Press keys/mouse to capture".to_string();
                    }
                    macro_editor::MacroMessage::StopRecording => {
                        let recorded_actions = self.input_recorder.stop_recording();
                        // Add recorded actions to current macro
                        if let Some(m) = self.macro_editor_state.current_macro_mut() {
                            m.actions.extend(recorded_actions);
                        }
                        self.macro_editor_state.update(macro_msg);
                        self.status_message = "⏹ Recording stopped".to_string();
                    }
                    _ => {
                        self.macro_editor_state.update(macro_msg);
                    }
                }
            }
            
            Message::RecordingTick => {
                // Poll for new recorded actions
                if self.macro_editor_state.is_recording {
                    let actions = self.input_recorder.poll_actions();
                    for action in actions {
                        self.macro_editor_state.update(
                            macro_editor::MacroMessage::RecordedAction(action)
                        );
                    }
                }
            }

            Message::SaveMacros => {
                // Save macros back to the selected profile
                if let Some(index) = self.selected_profile_index {
                    if let Some(profile) = self.profiles.get_mut(index) {
                        profile.macros = MacroConfig {
                            macros: self.macro_editor_state.macros.clone(),
                        };
                        self.save_profiles_to_disk();
                        self.status_message = "✅ Macros saved".to_string();
                    }
                } else {
                    self.status_message = "⚠️ Select a profile first to save macros".to_string();
                }
            }

            Message::IpcTick => {
                // Process IPC messages from Runner
                if let Some(ipc_msg) = process_ipc_messages() {
                    return self.update(ipc_msg);
                }
            }

            Message::IpcShowFlyout => {
                self.show_flyout();
            }

            Message::IpcHideFlyout => {
                self.hide_flyout();
            }

            Message::IpcBringToFront => {
                self.bring_to_front();
            }

            Message::IpcExit => {
                // Clean exit
                std::process::exit(0);
            }

            Message::FlyoutProfileSelected(name) => {
                self.activate_profile_by_name(&name);
                self.hide_flyout(); // Close flyout after selection
            }

            Message::FlyoutDeactivate => {
                self.deactivate_profile();
                self.hide_flyout();
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

                // Preserve existing macros if updating, or create default for new
                let existing_macros = self.selected_profile_index
                    .and_then(|i| self.profiles.get(i))
                    .map(|p| p.macros.clone())
                    .unwrap_or_default();

                let profile = Profile {
                    name: self.edit_name.clone(),
                    processes_to_kill: self.get_selected_processes(),
                    crosshair_image_path: self.edit_image_path.clone(),
                    crosshair_x_offset: x_offset,
                    crosshair_y_offset: y_offset,
                    overlay_enabled: self.edit_overlay_enabled,
                    fan_speed_max: self.edit_fan_speed_max,
                    macros: existing_macros,
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
                self.notify_runner_profile_changed();
            }

            Message::DeleteProfile => {
                if let Some(index) = self.selected_profile_index {
                    let name = self.profiles[index].name.clone();
                    self.profiles.remove(index);
                    self.clear_edit_form();
                    self.save_profiles_to_disk();
                    self.notify_runner_profile_changed();
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
                self.status_message = format!(
                    "🔄 Refreshed: {} processes found",
                    self.running_processes.len()
                );
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

            Message::SelectImage => match open_image_picker() {
                Ok(path) => match validate_crosshair_image(&path) {
                    Ok(_) => {
                        let path_str = path.to_string_lossy().to_string();
                        self.edit_image_path = Some(path_str.clone());
                        self.status_message = format!("📁 Selected image: {}", path_str);
                    }
                    Err(e) => {
                        self.status_message = format!("❌ Invalid image: {}", e);
                    }
                },
                Err(_) => {}
            },

            Message::ClearImage => {
                self.edit_image_path = None;
                self.status_message = "Cleared crosshair image".to_string();
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // LEFT SIDEBAR: Profiles + Macros section
        let mut sidebar = Column::new()
            .spacing(5)
            .padding(10)
            .width(Length::Fixed(200.0))
            .push(Text::new("📋 Profiles").size(18));

        // Profile list
        for (i, profile) in self.profiles.iter().enumerate() {
            let is_selected = self.selected_profile_index == Some(i) && self.current_page == Page::Profiles;
            let is_active = self.active_profile_name.as_ref() == Some(&profile.name);

            let label = if is_active {
                format!("🟢 {}", profile.name)
            } else if is_selected {
                format!("▶ {}", profile.name)
            } else {
                profile.name.clone()
            };

            sidebar = sidebar.push(
                Button::new(Text::new(label).size(13))
                    .on_press(Message::ProfileSelected(i))
                    .width(Length::Fill)
                    .padding(6),
            );
        }

        sidebar = sidebar
            .push(Space::new(Length::Fill, Length::Fixed(5.0)))
            .push(
                Button::new(Text::new("+ New Profile").size(12))
                    .on_press(Message::NewProfile)
                    .width(Length::Fill)
                    .padding(8),
            );

        // Macros section in sidebar
        sidebar = sidebar
            .push(Space::new(Length::Fill, Length::Fixed(20.0)))
            .push(Text::new("🎮 Macros").size(18))
            .push(Space::new(Length::Fill, Length::Fixed(5.0)))
            .push(
                Button::new(
                    Text::new(if self.current_page == Page::Macros { "▶ Macro Editor" } else { "  Macro Editor" })
                        .size(13)
                )
                .on_press(Message::NavigateTo(Page::Macros))
                .width(Length::Fill)
                .padding(6),
            );

        let left_panel = Container::new(Scrollable::new(sidebar))
            .height(Length::Fill);

        // MAIN CONTENT based on current page
        let main_content: Element<'_, Message> = match self.current_page {
            Page::Profiles => self.render_profile_editor(),
            Page::Macros => self.render_macros_page(),
        };

        // Status bar
        let status_bar = Container::new(
            Row::new()
                .spacing(20)
                .push(Text::new(&self.status_message).size(14))
                .push(Space::new(Length::Fill, Length::Shrink))
                .push(if let Some(ref name) = self.active_profile_name {
                    Text::new(format!("🟢 Active: {} | 📌 Tray", name)).size(14)
                } else {
                    Text::new("No active profile | 📌 Tray").size(14)
                }),
        )
        .width(Length::Fill)
        .padding(10)
        .height(Length::Fixed(40.0));

        let content = Column::new()
            .push(
                Row::new()
                    .push(left_panel)
                    .push(main_content)
                    .height(Length::Fill),
            )
            .push(status_bar);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl GameOptimizer {
    /// Render the Profile Editor (main content area for profiles page)
    fn render_profile_editor(&self) -> Element<'_, Message> {
        // Profile Edit form
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

        Container::new(Scrollable::new(edit_section))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }

    /// Render the Macros page
    fn render_macros_page(&self) -> Element<'_, Message> {
        let profile_name = self.selected_profile_index
            .and_then(|i| self.profiles.get(i))
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "No profile selected".to_string());

        let header = Column::new()
            .spacing(10)
            .push(Text::new("🎮 Gaming Macros").size(24))
            .push(Text::new(format!("Profile: {}", profile_name)).size(14))
            .push(Space::new(Length::Fill, Length::Fixed(10.0)));

        let macro_editor = self.macro_editor_state.view()
            .map(Message::MacroMessage);

        let save_button = Button::new(Text::new("💾 Save Macros"))
            .on_press(Message::SaveMacros)
            .padding(12);

        let content = Column::new()
            .spacing(15)
            .padding(20)
            .push(header)
            .push(macro_editor)
            .push(Space::new(Length::Fill, Length::Fixed(10.0)))
            .push(save_button);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn render_process_selector(&self) -> Element<'_, Message> {
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
                        Some(proc.memory_kb),
                    ));
                }
            }
        }

        for (name, exe) in COMMON_APPS.iter() {
            let exe_lower = exe.to_lowercase();
            if !seen.contains(&exe_lower) {
                if self.process_selection.get(*exe).copied().unwrap_or(false) {
                    if filter_lower.is_empty()
                        || exe_lower.contains(&filter_lower)
                        || name.to_lowercase().contains(&filter_lower)
                    {
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
                let is_selected = self
                    .process_selection
                    .get(*exe_name)
                    .copied()
                    .unwrap_or(false);
                let exe_string = exe_name.to_string();

                let info = match (cpu, mem) {
                    (Some(c), Some(m)) => {
                        format!("{} - CPU: {:.1}% | {} MB", display_name, c, m / 1024)
                    }
                    _ => format!("{} (not running)", display_name),
                };

                grid = grid.push(
                    Checkbox::new(info, is_selected)
                        .on_toggle(move |checked| {
                            Message::ProcessToggled(exe_string.clone(), checked)
                        })
                        .width(Length::Fill),
                );
            }

            if processes_to_show.len() > 50 {
                grid = grid.push(
                    Text::new(format!(
                        "... and {} more (use filter)",
                        processes_to_show.len() - 50
                    ))
                    .size(12),
                );
            }
        }

        Container::new(Scrollable::new(grid).height(Length::Fixed(200.0)))
            .width(Length::Fill)
            .into()
    }
}

pub fn run() -> iced::Result {
    println!("[GUI] Starting GUI (standalone mode, no IPC)...");
    run_with_ipc(None, crate::StartupFlags::default())
}

/// Run GUI with IPC client and startup flags
/// Called by Settings main.rs
pub fn run_with_ipc(
    ipc_client: Option<NamedPipeClient>,
    startup_flags: crate::StartupFlags,
) -> iced::Result {
    println!("[GUI] Starting GUI with IPC support...");

    // Wrap IPC client in Arc<Mutex> for thread-safe sharing
    let ipc_arc = ipc_client.map(|c| std::sync::Arc::new(Mutex::new(c)));

    // If we have an IPC client, start a listener thread
    if let Some(ref client_arc) = ipc_arc {
        let client_clone = client_arc.clone();
        let (tx, rx) = mpsc::channel::<TrayToGui>();

        // Store the IPC receiver globally
        if let Ok(mut guard) = IPC_MESSAGE_RX.lock() {
            *guard = Some(rx);
        }

        // Start IPC listener thread
        std::thread::spawn(move || {
            println!("[IPC-LISTENER] Started listening for Runner messages");
            loop {
                // Try to receive IPC messages
                if let Ok(client) = client_clone.lock() {
                    match client.try_recv() {
                        Ok(Some(msg)) => {
                            println!("[IPC-LISTENER] Received: {:?}", msg);
                            if tx.send(msg).is_err() {
                                println!("[IPC-LISTENER] GUI channel closed, exiting");
                                break;
                            }
                        }
                        Ok(None) => {
                            // No message available
                        }
                        Err(e) => {
                            eprintln!("[IPC-LISTENER] Error receiving: {}", e);
                        }
                    }
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        });
    }

    // Prepare flags for the application
    let flags = GuiFlags {
        show_flyout: startup_flags.show_flyout,
        bring_to_front: startup_flags.bring_to_front,
        flyout_only: startup_flags.flyout_only,
        ipc_client: ipc_arc,
    };

    // In flyout-only mode, start with main window hidden
    let window_visible = !startup_flags.flyout_only;
    println!("[GUI] Flyout-only mode: {}, window visible: {}", startup_flags.flyout_only, window_visible);

    let result = GameOptimizer::run(Settings {
        flags,
        window: iced::window::Settings {
            size: iced::Size::new(1000.0, 750.0),
            min_size: Some(iced::Size::new(900.0, 650.0)),
            visible: window_visible,
            ..Default::default()
        },
        ..Default::default()
    });

    println!("[GUI] Iced returned: {:?}", result);
    result
}
