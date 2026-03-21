use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;

#[cfg(windows)]
use windows::Win32::{Foundation::*, Storage::FileSystem::*, System::Pipes::*};

pub const ENGINE_PIPE_NAME: &str = r"\\.\pipe\EdgeOptimizerEngineIPC";

#[cfg(windows)]
pub struct EnginePipeServer {
    pipe_handle: HANDLE,
}

#[cfg(windows)]
impl EnginePipeServer {
    pub fn new(pipe_name: &str) -> Result<Self> {
        let pipe_name_wide: Vec<u16> = pipe_name.encode_utf16().chain(Some(0)).collect();

        unsafe {
            let pipe_handle = CreateNamedPipeW(
                windows::core::PCWSTR(pipe_name_wide.as_ptr()),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                1,
                64 * 1024,
                64 * 1024,
                0,
                Some(std::ptr::null_mut()),
            );

            if pipe_handle.is_invalid() {
                anyhow::bail!("failed to create engine pipe server")
            }

            Ok(Self { pipe_handle })
        }
    }

    pub fn new_default() -> Result<Self> {
        Self::new(ENGINE_PIPE_NAME)
    }

    pub fn wait_for_client(&self) -> Result<()> {
        unsafe {
            match ConnectNamedPipe(self.pipe_handle, Some(std::ptr::null_mut())) {
                Ok(_) => Ok(()),
                Err(e) => {
                    let error_code = e.code().0 as u32;
                    if error_code == ERROR_PIPE_CONNECTED.0 {
                        Ok(())
                    } else {
                        Err(anyhow::anyhow!("ConnectNamedPipe failed: {}", e))
                    }
                }
            }
        }
    }

    pub fn recv<T: DeserializeOwned>(&self) -> Result<Option<T>> {
        let mut buffer = vec![0u8; 64 * 1024];
        let mut bytes_read = 0u32;

        unsafe {
            match ReadFile(
                self.pipe_handle,
                Some(buffer.as_mut_slice()),
                Some(&mut bytes_read),
                None,
            ) {
                Ok(_) => {
                    if bytes_read == 0 {
                        return Ok(None);
                    }
                    let data = &buffer[..bytes_read as usize];
                    let msg = bincode::deserialize::<T>(data)
                        .context("failed to deserialize engine pipe payload")?;
                    Ok(Some(msg))
                }
                Err(e) => {
                    let code = e.code().0 as u32;
                    if code == ERROR_BROKEN_PIPE.0 || code == ERROR_PIPE_NOT_CONNECTED.0 {
                        return Ok(None);
                    }
                    Err(anyhow::anyhow!("ReadFile failed: {}", e))
                }
            }
        }
    }

    pub fn send<T: Serialize>(&self, message: &T) -> Result<()> {
        let data =
            bincode::serialize(message).context("failed to serialize engine pipe payload")?;
        let mut bytes_written = 0u32;

        unsafe {
            WriteFile(
                self.pipe_handle,
                Some(data.as_slice()),
                Some(&mut bytes_written),
                None,
            )
            .context("WriteFile failed")?;
            let _ = FlushFileBuffers(self.pipe_handle);
        }

        Ok(())
    }

    pub fn disconnect(&self) {
        unsafe {
            let _ = DisconnectNamedPipe(self.pipe_handle);
        }
    }
}

#[cfg(windows)]
impl Drop for EnginePipeServer {
    fn drop(&mut self) {
        unsafe {
            let _ = DisconnectNamedPipe(self.pipe_handle);
            let _ = CloseHandle(self.pipe_handle);
        }
    }
}

#[cfg(windows)]
pub struct EnginePipeClient {
    pipe_handle: HANDLE,
}

#[cfg(windows)]
impl EnginePipeClient {
    pub fn connect(pipe_name: &str, timeout: Duration) -> Result<Self> {
        let pipe_name_wide: Vec<u16> = pipe_name.encode_utf16().chain(Some(0)).collect();
        let start = std::time::Instant::now();
        let mut delay_ms = 50u64;

        unsafe {
            while start.elapsed() <= timeout {
                let pipe_handle = CreateFileW(
                    windows::core::PCWSTR(pipe_name_wide.as_ptr()),
                    (FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0).into(),
                    FILE_SHARE_NONE,
                    None,
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    HANDLE::default(),
                );

                match pipe_handle {
                    Ok(h) if !h.is_invalid() => return Ok(Self { pipe_handle: h }),
                    _ => {
                        std::thread::sleep(Duration::from_millis(delay_ms));
                        delay_ms = (delay_ms * 2).min(500);
                    }
                }
            }
        }

        anyhow::bail!(
            "timed out connecting to engine pipe after {:?} ({})",
            timeout,
            pipe_name
        )
    }

    pub fn connect_default(timeout: Duration) -> Result<Self> {
        Self::connect(ENGINE_PIPE_NAME, timeout)
    }

    pub fn send<T: Serialize>(&self, message: &T) -> Result<()> {
        let data =
            bincode::serialize(message).context("failed to serialize engine client payload")?;
        let mut bytes_written = 0u32;

        unsafe {
            WriteFile(
                self.pipe_handle,
                Some(data.as_slice()),
                Some(&mut bytes_written),
                None,
            )
            .context("WriteFile failed")?;
            let _ = FlushFileBuffers(self.pipe_handle);
        }

        Ok(())
    }

    pub fn recv<T: DeserializeOwned>(&self) -> Result<Option<T>> {
        let mut buffer = vec![0u8; 64 * 1024];
        let mut bytes_read = 0u32;

        unsafe {
            match ReadFile(
                self.pipe_handle,
                Some(buffer.as_mut_slice()),
                Some(&mut bytes_read),
                None,
            ) {
                Ok(_) => {
                    if bytes_read == 0 {
                        return Ok(None);
                    }
                    let data = &buffer[..bytes_read as usize];
                    let msg = bincode::deserialize::<T>(data)
                        .context("failed to deserialize engine client payload")?;
                    Ok(Some(msg))
                }
                Err(e) => {
                    let code = e.code().0 as u32;
                    if code == ERROR_BROKEN_PIPE.0 || code == ERROR_PIPE_NOT_CONNECTED.0 {
                        return Ok(None);
                    }
                    Err(anyhow::anyhow!("ReadFile failed: {}", e))
                }
            }
        }
    }
}

#[cfg(windows)]
impl Drop for EnginePipeClient {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.pipe_handle);
        }
    }
}

#[cfg(not(windows))]
pub struct EnginePipeServer;
#[cfg(not(windows))]
pub struct EnginePipeClient;

#[cfg(not(windows))]
impl EnginePipeServer {
    pub fn new(_pipe_name: &str) -> Result<Self> {
        anyhow::bail!("engine pipe server is only supported on windows")
    }

    pub fn new_default() -> Result<Self> {
        anyhow::bail!("engine pipe server is only supported on windows")
    }

    pub fn wait_for_client(&self) -> Result<()> {
        anyhow::bail!("engine pipe server is only supported on windows")
    }

    pub fn recv<T: DeserializeOwned>(&self) -> Result<Option<T>> {
        anyhow::bail!("engine pipe server is only supported on windows")
    }

    pub fn send<T: Serialize>(&self, _message: &T) -> Result<()> {
        anyhow::bail!("engine pipe server is only supported on windows")
    }

    pub fn disconnect(&self) {}
}

#[cfg(not(windows))]
impl EnginePipeClient {
    pub fn connect(_pipe_name: &str, _timeout: Duration) -> Result<Self> {
        anyhow::bail!("engine pipe client is only supported on windows")
    }

    pub fn connect_default(_timeout: Duration) -> Result<Self> {
        anyhow::bail!("engine pipe client is only supported on windows")
    }

    pub fn send<T: Serialize>(&self, _message: &T) -> Result<()> {
        anyhow::bail!("engine pipe client is only supported on windows")
    }

    pub fn recv<T: DeserializeOwned>(&self) -> Result<Option<T>> {
        anyhow::bail!("engine pipe client is only supported on windows")
    }
}
