/// Inter-Process Communication between GUI and System Tray
use serde::{Deserialize, Serialize};
use crate::profile::Profile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcMessage {
    /// GUI -> Tray: Profile was created or updated
    ProfileUpdated(Profile),
    /// GUI -> Tray: Profile was deleted
    ProfileDeleted(String),
    /// GUI -> Tray: Activate a profile
    ActivateProfile(String),
    /// GUI -> Tray: Deactivate current profile
    DeactivateProfile,
    /// GUI -> Tray: Toggle overlay visibility
    ToggleOverlay,
    /// Tray -> GUI: Request current profiles list
    RequestProfiles,
    /// Tray -> GUI: Send current profiles list
    ProfilesList(Vec<Profile>),
    /// Tray -> GUI: Current active profile changed
    ActiveProfileChanged(Option<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppMode {
    GUI,
    Tray,
}
