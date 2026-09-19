use crate::macro_config::MacroConfig;
use crate::orchestration::{Envelope, RunnerToSettingsEvent, SettingsToRunnerCommand};
/// Inter-Process Communication between Settings and Runner processes
/// Uses Windows Named Pipes for cross-process communication
use crate::profile::Profile;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

#[cfg(windows)]
use windows::Win32::{Foundation::*, Storage::FileSystem::*, System::Pipes::*};

/// Named pipe path for IPC (Settings <-> Runner)
#[allow(dead_code)]
pub const PIPE_NAME: &str = r"\\.\pipe\EdgeOptimizerIPC";
const MAX_SETTINGS_MESSAGE_BYTES: usize = 1024 * 1024;

/// Named pipe path for Macro IPC (Settings <-> Macro)
#[allow(dead_code)]
pub const MACRO_PIPE_NAME: &str = r"\\.\pipe\EdgeOptimizerMacroIPC";

/// Messages from Settings to Runner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuiToTray {
    /// Request the authoritative state snapshot from Runner.
    RequestState,
    /// Update profiles list
    ProfilesUpdated(Vec<Profile>),
    /// Active profile changed
    ActiveProfileChanged(Option<String>),
    /// Overlay visibility changed
    OverlayVisibilityChanged(bool),
    /// Request tray to exit
    Shutdown,
    /// Versioned orchestration command envelope
    Orchestration(Envelope<SettingsToRunnerCommand>),
    /// Request a read-only process list for the System Tweaks UI.
    RequestProcessSnapshot,
}

/// Messages from Runner to Settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrayToGui {
    /// Authoritative state loaded from Runner's SQLite database.
    StateSnapshot {
        profiles: Vec<Profile>,
        active_profile: Option<String>,
        overlay_visible: bool,
    },
    /// User selected a profile from tray
    ActivateProfile(String),
    /// User deactivated profile from tray
    DeactivateProfile,
    /// User toggled overlay from tray
    ToggleOverlay,
    /// User requested to open settings/GUI
    OpenSettings,
    /// User single-clicked tray icon - show flyout window
    ShowFlyout,
    /// User clicked away or toggled - hide flyout window
    HideFlyout,
    /// User double-clicked tray icon - bring main window to front
    BringMainToFront,
    /// User requested exit
    Exit,
    /// Versioned orchestration event envelope
    OrchestrationEvent(Envelope<RunnerToSettingsEvent>),
    /// Read-only process metrics; process selection remains profile state.
    ProcessSnapshot(Vec<ProcessSnapshotEntry>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSnapshotEntry {
    pub name: String,
    pub cpu_percent: f32,
    pub memory_kb: u64,
}

/// Named Pipe Server (Runner side)
/// Receives messages from Settings and sends messages to Settings
#[cfg(windows)]
#[allow(dead_code)]
pub struct NamedPipeServer {
    pipe_handle: HANDLE,
    connected: AtomicBool,
}

#[cfg(windows)]
#[allow(dead_code)]
impl NamedPipeServer {
    /// Create a new named pipe server (Runner side)
    pub fn new() -> Result<Self> {
        use std::ptr::null_mut;

        let pipe_name: Vec<u16> = PIPE_NAME.encode_utf16().chain(Some(0)).collect();

        unsafe {
            let pipe_handle = CreateNamedPipeW(
                windows::core::PCWSTR(pipe_name.as_ptr()),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_NOWAIT,
                1,                // Max instances
                8192,             // Out buffer size
                8192,             // In buffer size
                0,                // Default timeout
                Some(null_mut()), // Default security
            );

            if pipe_handle.is_invalid() {
                anyhow::bail!("Failed to create named pipe");
            }

            tracing::info!("Named pipe server created: {}", PIPE_NAME);

            Ok(Self {
                pipe_handle,
                connected: AtomicBool::new(false),
            })
        }
    }

    /// Accept a pending Settings connection without blocking Runner's UI loop.
    pub fn try_accept(&self) -> Result<bool> {
        if self.connected.load(Ordering::Acquire) {
            return Ok(true);
        }

        unsafe {
            match ConnectNamedPipe(self.pipe_handle, None) {
                Ok(_) => {
                    tracing::info!("Client connected to named pipe");
                    self.connected.store(true, Ordering::Release);
                    Ok(true)
                }
                Err(error) => match error.code().0 as u32 {
                    code if code == ERROR_PIPE_CONNECTED.0 => {
                        tracing::info!("Client already connected to named pipe");
                        self.connected.store(true, Ordering::Release);
                        Ok(true)
                    }
                    code if code == ERROR_PIPE_LISTENING.0 || code == ERROR_NO_DATA.0 => Ok(false),
                    _ => Err(anyhow::anyhow!("ConnectNamedPipe failed: {}", error)),
                },
            }
        }
    }

    /// Try to receive a message (non-blocking)
    pub fn try_recv(&self) -> Result<Option<GuiToTray>> {
        if !self.connected.load(Ordering::Acquire) {
            return Ok(None);
        }
        let mut buffer = vec![0u8; MAX_SETTINGS_MESSAGE_BYTES];
        let mut bytes_read = 0u32;

        unsafe {
            match ReadFile(
                self.pipe_handle,
                Some(&mut buffer),
                Some(&mut bytes_read),
                None,
            ) {
                Ok(_) => {
                    if bytes_read == 0 {
                        return Ok(None);
                    }

                    let message: GuiToTray = bincode::deserialize(&buffer[..bytes_read as usize])
                        .context("Failed to deserialize GuiToTray message")?;

                    Ok(Some(message))
                }
                Err(e) => {
                    let error_code = e.code().0 as u32;
                    if error_code == ERROR_NO_DATA.0 || error_code == ERROR_PIPE_LISTENING.0 {
                        return Ok(None); // No data available
                    }
                    if error_code == ERROR_BROKEN_PIPE.0 {
                        let _ = DisconnectNamedPipe(self.pipe_handle);
                        self.connected.store(false, Ordering::Release);
                        return Err(anyhow::anyhow!("Settings disconnected"));
                    }
                    Err(anyhow::anyhow!("ReadFile failed: {}", e))
                }
            }
        }
    }

    /// Send a message to Settings
    pub fn send(&self, message: &TrayToGui) -> Result<()> {
        if !self.connected.load(Ordering::Acquire) {
            anyhow::bail!("Settings is not connected");
        }
        let data = bincode::serialize(message).context("Failed to serialize TrayToGui message")?;
        if data.len() > MAX_SETTINGS_MESSAGE_BYTES {
            anyhow::bail!("Settings response exceeds 1 MiB");
        }

        let mut bytes_written = 0u32;

        unsafe {
            WriteFile(
                self.pipe_handle,
                Some(&data),
                Some(&mut bytes_written),
                None,
            )
            .context("WriteFile failed")?;

            let _ = FlushFileBuffers(self.pipe_handle);
        }

        Ok(())
    }
}

#[cfg(windows)]
impl Drop for NamedPipeServer {
    fn drop(&mut self) {
        unsafe {
            let _ = DisconnectNamedPipe(self.pipe_handle);
            let _ = CloseHandle(self.pipe_handle);
        }
        tracing::info!("Named pipe server closed");
    }
}

/// Named Pipe Client (Settings side)
/// Connects to Runner and exchanges messages
#[cfg(windows)]
#[allow(dead_code)]
#[derive(Debug)]
pub struct NamedPipeClient {
    pipe_handle: HANDLE,
}

#[cfg(windows)]
#[allow(dead_code)]
impl NamedPipeClient {
    /// Connect to the named pipe server (Runner) with exponential backoff
    pub fn connect() -> Result<Self> {
        Self::connect_with_timeout(Duration::from_secs(3))
    }

    /// Connect to the named pipe server with custom timeout
    pub fn connect_with_timeout(timeout: Duration) -> Result<Self> {
        let pipe_name: Vec<u16> = PIPE_NAME.encode_utf16().chain(Some(0)).collect();
        let start = std::time::Instant::now();
        let mut attempt = 0u32;

        unsafe {
            // Try to connect with exponential backoff
            while start.elapsed() < timeout {
                attempt += 1;
                let pipe_handle = CreateFileW(
                    windows::core::PCWSTR(pipe_name.as_ptr()),
                    (FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0).into(),
                    FILE_SHARE_NONE,
                    None,
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    HANDLE::default(),
                );

                let pipe_handle = match pipe_handle {
                    Ok(h) => h,
                    Err(_) => {
                        // Exponential backoff: 50ms, 100ms, 200ms, ... capped at 500ms
                        let delay = Duration::from_millis((50 * (1 << attempt.min(4))) as u64);
                        std::thread::sleep(delay);
                        continue;
                    }
                };

                if !pipe_handle.is_invalid() {
                    tracing::info!(
                        "Connected to named pipe: {} (attempt {})",
                        PIPE_NAME,
                        attempt
                    );
                    return Ok(Self { pipe_handle });
                }

                // Exponential backoff
                let delay = Duration::from_millis((50 * (1 << attempt.min(4))) as u64);
                std::thread::sleep(delay);
            }

            anyhow::bail!("Failed to connect to named pipe after {:?}", timeout);
        }
    }

    /// Try to connect without blocking (single attempt)
    pub fn try_connect() -> Result<Option<Self>> {
        let pipe_name: Vec<u16> = PIPE_NAME.encode_utf16().chain(Some(0)).collect();

        unsafe {
            let pipe_handle = CreateFileW(
                windows::core::PCWSTR(pipe_name.as_ptr()),
                (FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0).into(),
                FILE_SHARE_NONE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                HANDLE::default(),
            );

            match pipe_handle {
                Ok(h) if !h.is_invalid() => {
                    tracing::info!("Connected to named pipe: {}", PIPE_NAME);
                    Ok(Some(Self { pipe_handle: h }))
                }
                _ => Ok(None), // Not available yet
            }
        }
    }

    /// Send a message to Runner
    pub fn send(&self, message: &GuiToTray) -> Result<()> {
        let data = bincode::serialize(message).context("Failed to serialize GuiToTray message")?;

        let mut bytes_written = 0u32;

        unsafe {
            WriteFile(
                self.pipe_handle,
                Some(&data),
                Some(&mut bytes_written),
                None,
            )
            .context("WriteFile failed")?;

            let _ = FlushFileBuffers(self.pipe_handle);
        }

        Ok(())
    }

    /// Try to receive a message (non-blocking)
    pub fn try_recv(&self) -> Result<Option<TrayToGui>> {
        let mut buffer = vec![0u8; MAX_SETTINGS_MESSAGE_BYTES];
        let mut bytes_read = 0u32;

        unsafe {
            match ReadFile(
                self.pipe_handle,
                Some(&mut buffer),
                Some(&mut bytes_read),
                None,
            ) {
                Ok(_) => {
                    if bytes_read == 0 {
                        return Ok(None);
                    }

                    let message: TrayToGui = bincode::deserialize(&buffer[..bytes_read as usize])
                        .context("Failed to deserialize TrayToGui message")?;

                    Ok(Some(message))
                }
                Err(e) => {
                    let error_code = e.code().0 as u32;
                    if error_code == ERROR_NO_DATA.0 {
                        return Ok(None); // No data available
                    }
                    Err(anyhow::anyhow!("ReadFile failed: {}", e))
                }
            }
        }
    }
}

#[cfg(windows)]
impl Drop for NamedPipeClient {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.pipe_handle);
        }
        tracing::info!("Named pipe client closed");
    }
}

// Legacy std::sync::mpsc compatibility types for non-Windows or migration
use std::sync::mpsc::{Receiver, Sender};

/// Channels held by the GUI side (legacy - will be removed)
#[allow(dead_code)]
pub struct GuiChannels {
    pub to_tray: Sender<GuiToTray>,
    pub from_tray: Receiver<TrayToGui>,
}

/// Channels held by the Tray side (legacy - will be removed)
pub struct TrayChannels {
    pub from_gui: Receiver<GuiToTray>,
    pub to_gui: Sender<TrayToGui>,
}

/// Transitional Runner-to-Macro worker protocol. Runner owns this worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunnerToMacroCommand {
    ConfigUpdated(MacroConfig),
    SetEnabled(bool),
    Shutdown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestration::{AuthContext, Envelope, SettingsToRunnerCommand};
    use crate::profile::create_profile;

    #[test]
    fn settings_state_request_round_trips_without_opening_a_pipe() {
        // Verifies the transitional state request can be encoded and decoded entirely in memory.
        let encoded = bincode::serialize(&GuiToTray::RequestState).unwrap();
        let decoded: GuiToTray = bincode::deserialize(&encoded).unwrap();
        assert!(matches!(decoded, GuiToTray::RequestState));
    }

    #[test]
    fn runner_state_snapshot_round_trips_without_opening_a_pipe() {
        // Verifies authoritative profile and activation state survive transitional serialization.
        let message = TrayToGui::StateSnapshot {
            profiles: vec![create_profile("Gaming".into())],
            active_profile: Some("Gaming".into()),
            overlay_visible: true,
        };
        let encoded = bincode::serialize(&message).unwrap();
        let decoded: TrayToGui = bincode::deserialize(&encoded).unwrap();
        assert!(matches!(
            decoded,
            TrayToGui::StateSnapshot {
                profiles,
                active_profile: Some(active),
                overlay_visible: true,
            } if profiles.len() == 1 && active == "Gaming"
        ));
    }

    #[test]
    fn orchestration_envelope_preserves_command_correlation() {
        // Verifies a settings command retains its request ID across the current IPC message wrapper.
        let envelope = Envelope::with_request_id(
            "settings",
            AuthContext::InteractiveUser,
            "request-88",
            SettingsToRunnerCommand::OpenFlyout,
        );
        let encoded = bincode::serialize(&GuiToTray::Orchestration(envelope)).unwrap();
        let decoded: GuiToTray = bincode::deserialize(&encoded).unwrap();
        assert!(matches!(
            decoded,
            GuiToTray::Orchestration(Envelope {
                request_id,
                payload: SettingsToRunnerCommand::OpenFlyout,
                ..
            }) if request_id == "request-88"
        ));
    }

    #[test]
    fn process_snapshot_round_trips_without_enumerating_live_processes() {
        // Verifies process metrics cross the transitional pipe using deterministic fixture data only.
        let message = TrayToGui::ProcessSnapshot(vec![ProcessSnapshotEntry {
            name: "fixture.exe".into(),
            cpu_percent: 1.5,
            memory_kb: 2048,
        }]);
        let encoded = bincode::serialize(&message).unwrap();
        let decoded: TrayToGui = bincode::deserialize(&encoded).unwrap();
        assert!(matches!(
            decoded,
            TrayToGui::ProcessSnapshot(entries)
                if entries.len() == 1 && entries[0].name == "fixture.exe"
        ));
    }

    #[test]
    fn macro_worker_configuration_round_trips() {
        // Verifies Runner and the Macro worker share the same activation-time configuration representation.
        let command = RunnerToMacroCommand::ConfigUpdated(Default::default());
        let encoded = bincode::serialize(&command).unwrap();
        let decoded: RunnerToMacroCommand = bincode::deserialize(&encoded).unwrap();
        assert!(matches!(
            decoded,
            RunnerToMacroCommand::ConfigUpdated(config) if config.macros.is_empty()
        ));
    }
}
