use crate::profile::Profile;
use anyhow::{anyhow, Result};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{TrayIcon, TrayIconBuilder};

/// System tray manager
pub struct TrayManager {
    tray_icon: TrayIcon,
    menu: Menu,
    profile_submenu: Submenu,
    overlay_toggle: MenuItem,
    settings_item: MenuItem,
    exit_item: MenuItem,
}

/// Events that can be triggered from the tray menu
#[derive(Debug, Clone, PartialEq)]
pub enum TrayEvent {
    ProfileSelected(String),
    ProfileDeactivated,
    OverlayToggled,
    OpenSettings,
    Exit,
}

impl TrayManager {
    /// Create a new system tray icon with menu
    pub fn new(profiles: &[Profile], active_profile: Option<&str>) -> Result<Self> {
        // Create menu items
        let menu = Menu::new();

        // Title item (non-clickable)
        let title = MenuItem::new("Gaming Optimizer", false, None);
        menu.append(&title)
            .map_err(|e| anyhow!("Failed to add title item: {}", e))?;

        // Separator
        menu.append(&PredefinedMenuItem::separator())
            .map_err(|e| anyhow!("Failed to add separator: {}", e))?;

        // Profiles submenu
        let profile_submenu = Submenu::new("Profiles", true);
        Self::populate_profile_submenu(&profile_submenu, profiles, active_profile)?;
        menu.append(&profile_submenu)
            .map_err(|e| anyhow!("Failed to add profiles submenu: {}", e))?;

        // Separator
        menu.append(&PredefinedMenuItem::separator())
            .map_err(|e| anyhow!("Failed to add separator: {}", e))?;

        // Overlay toggle (initially disabled)
        let overlay_toggle = MenuItem::new("Overlay Visible", active_profile.is_some(), None);
        menu.append(&overlay_toggle)
            .map_err(|e| anyhow!("Failed to add overlay toggle: {}", e))?;

        // Separator
        menu.append(&PredefinedMenuItem::separator())
            .map_err(|e| anyhow!("Failed to add separator: {}", e))?;

        // Settings
        let settings_item = MenuItem::new("⚙ Settings", true, None);
        menu.append(&settings_item)
            .map_err(|e| anyhow!("Failed to add settings item: {}", e))?;

        // Exit
        let exit_item = MenuItem::new("❌ Exit", true, None);
        menu.append(&exit_item)
            .map_err(|e| anyhow!("Failed to add exit item: {}", e))?;

        // Create tray icon
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_tooltip("Gaming Optimizer - Inactive")
            .build()
            .map_err(|e| anyhow!("Failed to create tray icon: {}", e))?;

        Ok(TrayManager {
            tray_icon,
            menu,
            profile_submenu,
            overlay_toggle,
            settings_item,
            exit_item,
        })
    }

    /// Populate the profiles submenu with current profiles
    fn populate_profile_submenu(
        submenu: &Submenu,
        profiles: &[Profile],
        active_profile: Option<&str>,
    ) -> Result<()> {
        // Clear existing items
        // Note: tray-icon doesn't have a clear method, so we create a new submenu each time

        if profiles.is_empty() {
            let no_profiles = MenuItem::new("(No profiles - open Settings)", false, None);
            submenu
                .append(&no_profiles)
                .map_err(|e| anyhow!("Failed to add no profiles item: {}", e))?;
        } else {
            // Add each profile
            for profile in profiles {
                let is_active = active_profile == Some(&profile.name);
                let label = if is_active {
                    format!("✓ {}", profile.name)
                } else {
                    profile.name.clone()
                };
                let item = MenuItem::new(label, true, None);
                submenu
                    .append(&item)
                    .map_err(|e| anyhow!("Failed to add profile item: {}", e))?;
            }

            // Add separator
            submenu
                .append(&PredefinedMenuItem::separator())
                .map_err(|e| anyhow!("Failed to add separator: {}", e))?;

            // Add "(None)" option to deactivate profile
            let none_item = MenuItem::new("(None)", true, None);
            submenu
                .append(&none_item)
                .map_err(|e| anyhow!("Failed to add none item: {}", e))?;
        }

        Ok(())
    }

    /// Update the tray menu with new profiles list
    pub fn update_profiles(&mut self, profiles: &[Profile], active_profile: Option<&str>) -> Result<()> {
        // Create new profiles submenu
        let new_submenu = Submenu::new("Profiles", true);
        Self::populate_profile_submenu(&new_submenu, profiles, active_profile)?;

        // Replace old submenu in menu
        // Note: This is a limitation - tray-icon doesn't support dynamic menu updates well
        // In a real implementation, you might need to rebuild the entire menu
        self.profile_submenu = new_submenu;

        Ok(())
    }

    /// Update tooltip to show active profile
    pub fn set_active_profile(&mut self, profile_name: Option<&str>) -> Result<()> {
        let tooltip = if let Some(name) = profile_name {
            format!("Gaming Optimizer - {}", name)
        } else {
            "Gaming Optimizer - Inactive".to_string()
        };

        self.tray_icon.set_tooltip(Some(tooltip))
            .map_err(|e| anyhow!("Failed to set tooltip: {}", e))?;

        Ok(())
    }

    /// Update overlay toggle state
    pub fn set_overlay_visible(&mut self, visible: bool, enabled: bool) -> Result<()> {
        self.overlay_toggle.set_enabled(enabled);

        // Update text to show checkmark when visible
        let text = if visible {
            "☑ Overlay Visible"
        } else {
            "☐ Overlay Visible"
        };
        self.overlay_toggle.set_text(text);

        Ok(())
    }

    /// Poll for menu events and convert to TrayEvent
    pub fn poll_events(&self, profiles: &[Profile]) -> Option<TrayEvent> {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            return self.handle_menu_event(event, profiles);
        }
        None
    }

    /// Handle a menu event and convert to TrayEvent
    fn handle_menu_event(&self, event: MenuEvent, profiles: &[Profile]) -> Option<TrayEvent> {
        let event_id = event.id;

        // Check if it's a profile item
        for profile in profiles {
            // This is a simplified check - in reality, we'd need to store item IDs
            // For now, we'll use the text matching approach
            if self.is_profile_item(&event_id, &profile.name) {
                return Some(TrayEvent::ProfileSelected(profile.name.clone()));
            }
        }

        // Check for "(None)" deactivation
        if self.is_none_item(&event_id) {
            return Some(TrayEvent::ProfileDeactivated);
        }

        // Check overlay toggle
        if event_id == self.overlay_toggle.id() {
            return Some(TrayEvent::OverlayToggled);
        }

        // Check settings
        if event_id == self.settings_item.id() {
            return Some(TrayEvent::OpenSettings);
        }

        // Check exit
        if event_id == self.exit_item.id() {
            return Some(TrayEvent::Exit);
        }

        None
    }

    /// Check if event ID corresponds to a profile item
    fn is_profile_item(&self, _event_id: &tray_icon::menu::MenuId, _profile_name: &str) -> bool {
        // This is a placeholder - proper implementation would track menu item IDs
        // For MVP, this will be handled differently in main.rs
        false
    }

    /// Check if event ID corresponds to the "(None)" item
    fn is_none_item(&self, _event_id: &tray_icon::menu::MenuId) -> bool {
        // This is a placeholder - proper implementation would track menu item IDs
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile;

    #[test]
    fn test_tray_event_equality() {
        let event1 = TrayEvent::Exit;
        let event2 = TrayEvent::Exit;
        assert_eq!(event1, event2);

        let event3 = TrayEvent::ProfileSelected("Test".to_string());
        let event4 = TrayEvent::ProfileSelected("Test".to_string());
        assert_eq!(event3, event4);
    }
}
