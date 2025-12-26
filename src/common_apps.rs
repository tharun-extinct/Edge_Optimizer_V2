/// Common applications selector for process management
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonApp {
    pub name: String,
    pub executable: String,
    pub icon: Option<String>, // Path to icon or emoji
}

/// List of commonly used applications that users might want to kill
pub const COMMON_APPS: &[(&str, &str)] = &[
    // Communication
    ("Discord", "Discord.exe"),
    ("Discord Canary", "DiscordCanary.exe"),
    ("Telegram", "Telegram.exe"),
    ("Slack", "slack.exe"),
    ("Zoom", "Zoom.exe"),
    
    // Streaming & Recording
    ("OBS Studio", "obs64.exe"),
    ("OBS Studio (32-bit)", "obs32.exe"),
    ("XSplit Broadcaster", "XSplitBroadcaster.exe"),
    ("Streamlabs OBS", "Streamlabs OBS.exe"),
    ("Twitch Studio", "TwitchStudio.exe"),
    
    // Media & Music
    ("Spotify", "Spotify.exe"),
    ("YouTube Music", "YouTubeMusic.exe"),
    ("VLC Media Player", "vlc.exe"),
    ("Foobar2000", "foobar2000.exe"),
    
    // Gaming
    ("Steam", "Steam.exe"),
    ("Epic Games Launcher", "EpicGamesLauncher.exe"),
    ("GOG Galaxy", "GalaxyClient.exe"),
    ("Battle.net", "Battle.net.exe"),
    ("Ubisoft Connect", "UbisoftConnect.exe"),
    
    // Browser
    ("Chrome", "chrome.exe"),
    ("Firefox", "firefox.exe"),
    ("Edge", "msedge.exe"),
    
    // Cloud & Synchronization
    ("OneDrive", "OneDrive.exe"),
    ("Dropbox", "Dropbox.exe"),
    ("Google Drive", "GoogleDriveFS.exe"),
    ("iCloud", "iCloudServices.exe"),
    
    // Antivirus & System
    ("Windows Defender", "MsMpEng.exe"),
    ("Norton", "NortonLifeLock.exe"),
    ("McAfee", "McShield.exe"),
    
    // Development
    ("Visual Studio Code", "Code.exe"),
    ("Visual Studio", "devenv.exe"),
    ("IntelliJ IDEA", "idea64.exe"),
    ("Jetbrains Client", "jetbrains-client.exe"),
    
    // Accessibility
    ("Windows 11 Game Bar", "GameBarFTDesktopComp.exe"),
];

pub fn get_common_apps() -> Vec<CommonApp> {
    COMMON_APPS
        .iter()
        .map(|(name, executable)| CommonApp {
            name: name.to_string(),
            executable: executable.to_string(),
            icon: None,
        })
        .collect()
}

pub fn find_app_by_executable(executable: &str) -> Option<&'static (&'static str, &'static str)> {
    COMMON_APPS
        .iter()
        .find(|(_, exe)| exe.eq_ignore_ascii_case(executable))
}
