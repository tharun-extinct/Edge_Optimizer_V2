use anyhow::{anyhow, Result};
use image::GenericImageView;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::path::Path;
use std::rc::Rc;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Fullscreen, Window, WindowBuilder, WindowLevel};

/// Crosshair overlay window manager
pub struct OverlayWindow {
    window: Rc<Window>,
    surface: Surface<Rc<Window>, Rc<Window>>,
    crosshair_data: Vec<u32>,
    crosshair_width: u32,
    crosshair_height: u32,
    x_offset: i32,
    y_offset: i32,
    is_visible: bool,
}

impl OverlayWindow {
    /// Create a new overlay window with the specified crosshair image
    pub fn new(
        image_path: &str,
        x_offset: i32,
        y_offset: i32,
    ) -> Result<(Self, EventLoop<()>)> {
        // Load and validate crosshair image
        let (crosshair_data, width, height) = Self::load_crosshair_image(image_path)?;

        // Create event loop for overlay window
        let event_loop = EventLoop::new()
            .map_err(|e| anyhow!("Failed to create event loop: {}", e))?;

        // Create fullscreen transparent window
        let window = WindowBuilder::new()
            .with_fullscreen(Some(Fullscreen::Borderless(None)))
            .with_transparent(true)
            .with_decorations(false)
            .with_title("Gaming Optimizer Overlay")
            .build(&event_loop)
            .map_err(|e| anyhow!("Failed to create overlay window: {}", e))?;

        // Set window properties
        window.set_window_level(WindowLevel::AlwaysOnTop);
        window
            .set_cursor_hittest(false)
            .map_err(|e| anyhow!("Failed to enable click-through: {}", e))?;

        let window_rc = Rc::new(window);

        // Create softbuffer context and surface
        let context = Context::new(window_rc.clone())
            .map_err(|e| anyhow!("Failed to create softbuffer context: {}", e))?;

        let surface = Surface::new(&context, window_rc.clone())
            .map_err(|e| anyhow!("Failed to create softbuffer surface: {}", e))?;

        let mut overlay = OverlayWindow {
            window: window_rc,
            surface,
            crosshair_data,
            crosshair_width: width,
            crosshair_height: height,
            x_offset,
            y_offset,
            is_visible: false,
        };

        // Initial render
        overlay.render()?;

        Ok((overlay, event_loop))
    }

    /// Load crosshair image from PNG file
    /// Validates that image is exactly 100x100 pixels
    fn load_crosshair_image(path: &str) -> Result<(Vec<u32>, u32, u32)> {
        let img = image::open(Path::new(path))
            .map_err(|e| anyhow!("Failed to load crosshair image: {}", e))?;

        let (width, height) = img.dimensions();

        // Validate dimensions (must be exactly 100x100)
        if width != 100 || height != 100 {
            return Err(anyhow!(
                "Crosshair image must be exactly 100x100 pixels (got {}x{})",
                width,
                height
            ));
        }

        // Convert to RGBA8
        let rgba = img.to_rgba8();

        // Convert RGBA to ARGB32 format for softbuffer
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for pixel in rgba.pixels() {
            let r = pixel[0] as u32;
            let g = pixel[1] as u32;
            let b = pixel[2] as u32;
            let a = pixel[3] as u32;

            // ARGB format: 0xAARRGGBB
            let argb = (a << 24) | (r << 16) | (g << 8) | b;
            pixels.push(argb);
        }

        Ok((pixels, width, height))
    }

    /// Render the crosshair to the window
    fn render(&mut self) -> Result<()> {
        let window_size = self.window.inner_size();
        let width = window_size.width;
        let height = window_size.height;

        if width == 0 || height == 0 {
            return Ok(());
        }

        // Resize surface if needed
        self.surface
            .resize(
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            )
            .map_err(|e| anyhow!("Failed to resize surface: {}", e))?;

        // Get buffer
        let mut buffer = self
            .surface
            .buffer_mut()
            .map_err(|e| anyhow!("Failed to get buffer: {}", e))?;

        // Fill with transparent pixels
        for pixel in buffer.iter_mut() {
            *pixel = 0x00000000; // Fully transparent
        }

        // Calculate crosshair position (centered with offset)
        let crosshair_x = ((width as i32) / 2) - (self.crosshair_width as i32 / 2) + self.x_offset;
        let crosshair_y =
            ((height as i32) / 2) - (self.crosshair_height as i32 / 2) + self.y_offset;

        // Blit crosshair image to buffer
        for y in 0..self.crosshair_height {
            for x in 0..self.crosshair_width {
                let src_idx = (y * self.crosshair_width + x) as usize;
                let dst_x = crosshair_x + x as i32;
                let dst_y = crosshair_y + y as i32;

                // Check bounds
                if dst_x >= 0 && dst_x < width as i32 && dst_y >= 0 && dst_y < height as i32 {
                    let dst_idx = (dst_y as u32 * width + dst_x as u32) as usize;
                    if dst_idx < buffer.len() && src_idx < self.crosshair_data.len() {
                        buffer[dst_idx] = self.crosshair_data[src_idx];
                    }
                }
            }
        }

        // Present buffer
        buffer
            .present()
            .map_err(|e| anyhow!("Failed to present buffer: {}", e))?;

        Ok(())
    }

    /// Show the overlay window
    pub fn show(&mut self) -> Result<()> {
        self.window.set_visible(true);
        self.is_visible = true;
        self.render()?;
        Ok(())
    }

    /// Hide the overlay window
    pub fn hide(&mut self) {
        self.window.set_visible(false);
        self.is_visible = false;
    }

    /// Update the overlay with a new crosshair image and offsets
    pub fn update(&mut self, image_path: &str, x_offset: i32, y_offset: i32) -> Result<()> {
        // Load new crosshair
        let (crosshair_data, width, height) = Self::load_crosshair_image(image_path)?;

        self.crosshair_data = crosshair_data;
        self.crosshair_width = width;
        self.crosshair_height = height;
        self.x_offset = x_offset;
        self.y_offset = y_offset;

        // Re-render if visible
        if self.is_visible {
            self.render()?;
        }

        Ok(())
    }

    /// Check if overlay is currently visible
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Get the window reference
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Handle window resize event
    pub fn on_resize(&mut self, new_size: PhysicalSize<u32>) -> Result<()> {
        if new_size.width > 0 && new_size.height > 0 && self.is_visible {
            self.render()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_crosshair_image_invalid_size() {
        // This test requires a test image file, which we don't have in the repo
        // In a real implementation, you would create test images for validation
    }
}
