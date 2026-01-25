/// Inter-Process Communication between Settings and Runner processes
/// Uses Windows Named Pipes for cross-process communication
use crate::profile::Profile;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::ptr::null_mut;
use std::time::Duration;

#[cfg(windows)]
use windows::Win32::{Foundation::*, Storage::FileSystem::*, System::Pipes::*};

/// Named pipe path for IPC
#[allow(dead_code)]
pub const PIPE_NAME: &str = r"\\.\pipe\EdgeOptimizerIPC";

/// Messages from Settings to Runner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuiToTray {
    /// Update profiles list
    ProfilesUpdated(Vec<Profile>),
    /// Active profile changed
    ActiveProfileChanged(Option<String>),
    /// Overlay visibility changed
    OverlayVisibilityChanged(bool),
    /// Request tray to exit
    Shutdown,
}

/// Messages from Runner to Settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrayToGui {
    /// User selected a profile from tray
    ActivateProfile(String),
    /// User deactivated profile from tray
    DeactivateProfile,
    /// User toggled overlay from tray
    ToggleOverlay,
    /// User requested to open settings/GUI
    OpenSettings,
    /// User requested exit
    Exit,
}

/// Named Pipe Server (Runner side)
/// Receives messages from Settings and sends messages to Settings
#[cfg(windows)]
#[allow(dead_code)]
pub struct NamedPipeServer {
    pipe_handle: HANDLE,
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
                PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
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

            Ok(Self { pipe_handle })
        }
    }

    /// Wait for a client to connect (blocking)
    pub fn wait_for_connection(&self) -> Result<()> {
        unsafe {
            let result = ConnectNamedPipe(self.pipe_handle, Some(null_mut()));
            match result {
                Ok(_) => {
                    tracing::info!("Client connected to named pipe");
                    Ok(())
                }
                Err(e) => {
                    // ERROR_PIPE_CONNECTED means client already connected
                    let error_code = e.code().0 as u32;
                    if error_code == ERROR_PIPE_CONNECTED.0 {
                        tracing::info!("Client already connected to named pipe");
                        Ok(())
                    } else {
                        Err(anyhow::anyhow!("ConnectNamedPipe failed: {}", e))
                    }
                }
            }
        }
    }

    /// Try to receive a message (non-blocking)
    pub fn try_recv(&self) -> Result<Option<GuiToTray>> {
        let mut buffer = [0u8; 8192];
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
                    if error_code == ERROR_NO_DATA.0 {
                        return Ok(None); // No data available
                    }
                    Err(anyhow::anyhow!("ReadFile failed: {}", e))
                }
            }
        }
    }

    /// Send a message to Settings
    pub fn send(&self, message: &TrayToGui) -> Result<()> {
        let data = bincode::serialize(message).context("Failed to serialize TrayToGui message")?;

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
pub struct NamedPipeClient {
    pipe_handle: HANDLE,
}

#[cfg(windows)]
#[allow(dead_code)]
impl NamedPipeClient {
    /// Connect to the named pipe server (Runner)
    pub fn connect() -> Result<Self> {
        let pipe_name: Vec<u16> = PIPE_NAME.encode_utf16().chain(Some(0)).collect();

        unsafe {
            // Try to connect with timeout
            for _ in 0..10 {
                let pipe_handle = CreateFileW(
                    windows::core::PCWSTR(pipe_name.as_ptr()),
                    (FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0).into(),
                    FILE_SHARE_NONE,
                    None,
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    HANDLE::default(),
                )?;

                if !pipe_handle.is_invalid() {
                    tracing::info!("Connected to named pipe: {}", PIPE_NAME);
                    return Ok(Self { pipe_handle });
                }

                std::thread::sleep(Duration::from_millis(100));
            }

            anyhow::bail!("Failed to connect to named pipe after retries");
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
        let mut buffer = [0u8; 8192];
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
