/// Minimal System Tray Icon Manager for Runner process
///
/// This module provides a lightweight tray icon with context menu.
/// It does NOT handle flyout windows - those are owned by the Settings process.
/// Runner sends IPC messages to Settings to trigger flyout/window actions.
use anyhow::{anyhow, Result};
use tray_icon::menu::{Menu, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

/// Load application icon from favicon.ico file
fn load_app_icon() -> Result<Icon> {
    // Try multiple paths
    let paths_to_try = vec![
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("favicon.ico"))),
        Some(std::path::PathBuf::from("favicon.ico")),
        Some(std::path::PathBuf::from(
            "X:\\AI_and_Automation\\EdgeOptimizer\\favicon.ico",
        )),
    ];

    for path_opt in paths_to_try {
        if let Some(path) = path_opt {
            if path.exists() {
                let icon_data = std::fs::read(&path)
                    .map_err(|e| anyhow!("Failed to read favicon.ico: {}", e))?;

                // Decode with image crate
                let img = image::load_from_memory(&icon_data)
                    .map_err(|e| anyhow!("Failed to decode icon: {}", e))?;

                let img = img.resize_exact(16, 16, image::imageops::FilterType::Lanczos3);
                let rgba = img.to_rgba8();

                return Icon::from_rgba(rgba.into_raw(), 16, 16)
                    .map_err(|e| anyhow!("Failed to create icon from image: {:?}", e));
            }
        }
    }

    // Fallback: green square
    let icon_rgba: Vec<u8> = (0..16 * 16)
        .flat_map(|_| vec![0x00, 0xAA, 0x00, 0xFF])
        .collect();
    Icon::from_rgba(icon_rgba, 16, 16)
        .map_err(|e| anyhow!("Failed to create fallback icon: {:?}", e))
}

/// Minimal tray icon manager for Runner process
/// Only handles icon display and context menu - NO flyout window
pub struct TrayIconManager {
    #[allow(dead_code)]
    tray_icon: TrayIcon,
    active_profile: Option<String>,
    pub menu_item_settings: MenuId,
    pub menu_item_docs: MenuId,
    pub menu_item_bug_report: MenuId,
    pub menu_item_exit: MenuId,
}

impl TrayIconManager {
    /// Create a new tray icon manager
    pub fn new(active_profile: Option<String>) -> Result<Self> {
        let tooltip = if let Some(ref name) = active_profile {
            format!("Edge Optimizer - {}", name)
        } else {
            "Edge Optimizer - Inactive".to_string()
        };

        tracing::info!("Creating tray icon");

        let icon = load_app_icon()?;
        tracing::debug!("Icon loaded");

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

        tracing::info!("Tray icon created successfully with context menu");

        Ok(Self {
            tray_icon,
            active_profile,
            menu_item_settings,
            menu_item_docs,
            menu_item_bug_report,
            menu_item_exit,
        })
    }

    /// Update tooltip based on active profile
    pub fn set_active_profile(&mut self, active: Option<String>) {
        self.active_profile = active;
        let tooltip = if let Some(ref name) = self.active_profile {
            format!("Edge Optimizer - {}", name)
        } else {
            "Edge Optimizer - Inactive".to_string()
        };
        let _ = self.tray_icon.set_tooltip(Some(&tooltip));
    }

    /// Get current active profile name
    pub fn active_profile(&self) -> Option<&String> {
        self.active_profile.as_ref()
    }
}
