//! Crosshair overlay launcher - spawns crosshair as a separate detached process
//! The crosshair process runs independently and survives even if main app closes

use std::path::Path;
use std::process::{Command, Stdio};

/// Handle to track the crosshair process
pub struct OverlayHandle {
    process_name: String,
}

impl OverlayHandle {
    /// Kill all crosshair processes
    pub fn stop(&self) {
        #[cfg(windows)]
        {
            // Kill the crosshair process by name
            let _ = Command::new("taskkill")
                .args(["/F", "/IM", &self.process_name])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
        }
    }
}

impl Drop for OverlayHandle {
    fn drop(&mut self) {
        // Don't stop on drop - let it run independently!
        // User explicitly needs to call stop() or use the tray menu
    }
}

/// Start crosshair as a completely separate process
/// The crosshair will continue running even if the main app closes
pub fn start_overlay(
    image_path: String,
    x_offset: i32,
    y_offset: i32,
) -> Result<OverlayHandle, String> {
    // Validate image exists
    if !Path::new(&image_path).exists() {
        return Err(format!("Image not found: {}", image_path));
    }
    
    // Find the crosshair executable (should be next to the main exe)
    let crosshair_exe = get_crosshair_exe_path()?;
    
    println!("[Crosshair] Starting separate process: {}", crosshair_exe.display());
    println!("[Crosshair] Image: {}, Offset: ({}, {})", image_path, x_offset, y_offset);
    
    // Kill any existing crosshair process first
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "edge_optimizer_crosshair.exe"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    
    // Spawn crosshair as detached process
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        Command::new(&crosshair_exe)
            .arg(&image_path)
            .arg(x_offset.to_string())
            .arg(y_offset.to_string())
            .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to spawn crosshair process: {}", e))?;
    }
    
    #[cfg(not(windows))]
    {
        Command::new(&crosshair_exe)
            .arg(&image_path)
            .arg(x_offset.to_string())
            .arg(y_offset.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to spawn crosshair process: {}", e))?;
    }
    
    println!("[Crosshair] Process started successfully!");
    
    Ok(OverlayHandle {
        process_name: "edge_optimizer_crosshair.exe".to_string(),
    })
}

/// Kill all running crosshair processes (can be called without a handle)
#[allow(dead_code)]
pub fn kill_all_crosshairs() {
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "edge_optimizer_crosshair.exe"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

/// Find the edge_optimizer_crosshair.exe path
fn get_crosshair_exe_path() -> Result<std::path::PathBuf, String> {
    // Try to find edge_optimizer_crosshair.exe next to the main executable
    if let Ok(exe_path) = std::env::current_exe() {
        let exe_dir = exe_path.parent().unwrap_or(Path::new("."));
        
        // Check same directory
        let crosshair_path = exe_dir.join("edge_optimizer_crosshair.exe");
        if crosshair_path.exists() {
            return Ok(crosshair_path);
        }
        
        // Check release directory (for development)
        let release_path = exe_dir.join("target").join("release").join("edge_optimizer_crosshair.exe");
        if release_path.exists() {
            return Ok(release_path);
        }
    }
    
    // Try current directory
    let current_dir = std::env::current_dir().unwrap_or_default();
    let local_path = current_dir.join("edge_optimizer_crosshair.exe");
    if local_path.exists() {
        return Ok(local_path);
    }
    
    // Try target/release (development)
    let dev_path = current_dir.join("target").join("release").join("edge_optimizer_crosshair.exe");
    if dev_path.exists() {
        return Ok(dev_path);
    }
    
    Err("edge_optimizer_crosshair.exe not found. Make sure it's in the same directory as the main app.".to_string())
}
