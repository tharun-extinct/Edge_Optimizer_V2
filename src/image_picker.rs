/// Windows native file dialog for image selection
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use image::GenericImageView;

/// Open Windows file dialog to select a PNG file
#[cfg(windows)]
pub fn open_image_picker() -> Result<PathBuf> {
    use rfd::FileDialog;
    
    let file = FileDialog::new()
        .add_filter("PNG Image", &["png"])
        .add_filter("All Files", &["*"])
        .pick_file();

    file.ok_or_else(|| anyhow!("No file selected"))
}

#[cfg(not(windows))]
pub fn open_image_picker() -> Result<PathBuf> {
    Err(anyhow!("File picker only supported on Windows"))
}

/// Validate that the selected image is 100x100 pixels
pub fn validate_crosshair_image(path: &PathBuf) -> Result<()> {
    let reader = image::io::Reader::open(path)
        .map_err(|e| anyhow!("Failed to open image: {}", e))?;
    
    let image = reader.decode()
        .map_err(|e| anyhow!("Failed to decode image: {}", e))?;
    
    let (width, height) = image.dimensions();
    
    if width != 100 || height != 100 {
        return Err(anyhow!(
            "Invalid image dimensions: {}x{} (expected 100x100)",
            width, height
        ));
    }
    
    Ok(())
}

/// Load and convert image to RGBA8 for preview/rendering
pub fn load_crosshair_image(path: &PathBuf) -> Result<(Vec<u32>, u32, u32)> {
    validate_crosshair_image(path)?;
    
    let reader = image::io::Reader::open(path)
        .map_err(|e| anyhow!("Failed to open image: {}", e))?;
    
    let image = reader.decode()
        .map_err(|e| anyhow!("Failed to decode image: {}", e))?;
    
    let rgba_image = image.to_rgba8();
    let (width, height) = rgba_image.dimensions();
    
    // Convert RGBA8 to ARGB32 (u32) format for softbuffer
    let pixels: Vec<u32> = rgba_image
        .chunks_exact(4)
        .map(|chunk| {
            let r = chunk[0] as u32;
            let g = chunk[1] as u32;
            let b = chunk[2] as u32;
            let a = chunk[3] as u32;
            (a << 24) | (r << 16) | (g << 8) | b
        })
        .collect();
    
    Ok((pixels, width, height))
}
