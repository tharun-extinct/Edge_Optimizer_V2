//! Test crosshair image generator

use image::{Rgba, RgbaImage};
use std::path::Path;

fn main() {
    create_test_crosshair("test_crosshair.png");
    println!("Created test_crosshair.png");
}

fn create_test_crosshair(path: &str) {
    let size = 32u32;
    let mut img = RgbaImage::new(size, size);
    
    // Fill with transparent
    for pixel in img.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 0]);
    }
    
    let center = size as i32 / 2;
    let red = Rgba([255, 0, 0, 255]);
    let green = Rgba([0, 255, 0, 255]);
    
    // Draw crosshair
    // Horizontal line
    for x in 0..size as i32 {
        if x == center || x == center - 1 || x == center + 1 {
            continue; // Gap in center
        }
        img.put_pixel(x as u32, center as u32, green);
        img.put_pixel(x as u32, (center - 1) as u32, green);
    }
    
    // Vertical line
    for y in 0..size as i32 {
        if y == center || y == center - 1 || y == center + 1 {
            continue; // Gap in center
        }
        img.put_pixel(center as u32, y as u32, green);
        img.put_pixel((center - 1) as u32, y as u32, green);
    }
    
    // Center dot (red)
    img.put_pixel(center as u32, center as u32, red);
    img.put_pixel((center - 1) as u32, center as u32, red);
    img.put_pixel((center + 1) as u32, center as u32, red);
    img.put_pixel(center as u32, (center - 1) as u32, red);
    img.put_pixel(center as u32, (center + 1) as u32, red);
    
    img.save(Path::new(path)).expect("Failed to save image");
}
