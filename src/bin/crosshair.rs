//! Standalone crosshair overlay process
//! This runs as a completely separate process from the main application
//! Usage: crosshair.exe <image_path> <x_offset> <y_offset>

#![windows_subsystem = "windows"]

use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 4 {
        // Silent fail for GUI app - no console output
        return;
    }
    
    let image_path = &args[1];
    let x_offset: i32 = args[2].parse().unwrap_or(0);
    let y_offset: i32 = args[3].parse().unwrap_or(0);
    
    if !Path::new(image_path).exists() {
        return;
    }
    
    // Load image
    let img = match image::open(image_path) {
        Ok(img) => img,
        Err(_) => return,
    };
    
    let rgba = img.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    
    // Convert to BGRA
    let mut bgra_pixels: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
    for pixel in rgba.pixels() {
        bgra_pixels.push(pixel[2]); // B
        bgra_pixels.push(pixel[1]); // G
        bgra_pixels.push(pixel[0]); // R
        bgra_pixels.push(pixel[3]); // A
    }
    
    #[cfg(windows)]
    unsafe {
        run_overlay(bgra_pixels, width, height, x_offset, y_offset);
    }
}

#[cfg(windows)]
unsafe fn run_overlay(
    pixels: Vec<u8>,
    img_width: u32,
    img_height: u32,
    x_offset: i32,
    y_offset: i32,
) {
    use std::mem::zeroed;
    use std::ptr::null_mut;
    
    use windows::Win32::Foundation::{COLORREF, HWND, HINSTANCE, LPARAM, LRESULT, WPARAM};
    use windows::Win32::Graphics::Gdi::{
        BeginPaint, BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC,
        DeleteObject, EndPaint, GetDC, ReleaseDC, SelectObject, UpdateWindow,
        BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HDC, PAINTSTRUCT, SRCCOPY,
    };
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetSystemMetrics,
        PostQuitMessage, RegisterClassExW, SetLayeredWindowAttributes, SetWindowPos,
        ShowWindow, CS_HREDRAW, CS_VREDRAW, HWND_TOPMOST, LWA_COLORKEY, MSG, SM_CXSCREEN,
        SM_CYSCREEN, SWP_NOMOVE, SWP_NOSIZE, SW_SHOWNA, WNDCLASSEXW, WS_EX_LAYERED,
        WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
    };
    use windows::core::PCWSTR;
    
    // Screen dimensions
    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let screen_h = GetSystemMetrics(SM_CYSCREEN);
    
    // Calculate centered position
    let win_x = (screen_w / 2) - (img_width as i32 / 2) + x_offset;
    let win_y = (screen_h / 2) - (img_height as i32 / 2) + y_offset;
    
    // Unique class name
    let class_name: Vec<u16> = "CrosshairOverlayStandalone\0".encode_utf16().collect();
    
    let hinstance = match GetModuleHandleW(PCWSTR::null()) {
        Ok(h) => HINSTANCE(h.0),
        Err(_) => return,
    };
    
    // Create bitmap
    let screen_dc = GetDC(HWND::default());
    let mem_dc = CreateCompatibleDC(screen_dc);
    ReleaseDC(HWND::default(), screen_dc);
    
    let bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: img_width as i32,
            biHeight: -(img_height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0 as u32,
            ..zeroed()
        },
        bmiColors: [zeroed(); 1],
    };
    
    let mut bits_ptr: *mut std::ffi::c_void = null_mut();
    let hbitmap = match CreateDIBSection(mem_dc, &bmi, DIB_RGB_COLORS, &mut bits_ptr, None, 0) {
        Ok(bmp) => bmp,
        Err(_) => {
            DeleteDC(mem_dc);
            return;
        }
    };
    
    if bits_ptr.is_null() {
        let _ = DeleteObject(hbitmap);
        let _ = DeleteDC(mem_dc);
        return;
    }
    
    // Copy pixels with magenta for transparency
    let dst = std::slice::from_raw_parts_mut(bits_ptr as *mut u8, (img_width * img_height * 4) as usize);
    for (i, chunk) in pixels.chunks(4).enumerate() {
        let idx = i * 4;
        if chunk[3] < 128 {
            // Transparent -> magenta
            dst[idx] = 255;     // B
            dst[idx + 1] = 0;   // G
            dst[idx + 2] = 255; // R
            dst[idx + 3] = 255; // A
        } else {
            dst[idx] = chunk[0];     // B
            dst[idx + 1] = chunk[1]; // G
            dst[idx + 2] = chunk[2]; // R
            dst[idx + 3] = chunk[3]; // A
        }
    }
    
    let old_obj = SelectObject(mem_dc, hbitmap);
    
    // Store globally for WM_PAINT
    GLOBAL_DC = Some(mem_dc);
    GLOBAL_WIDTH = img_width;
    GLOBAL_HEIGHT = img_height;
    
    // Register window class
    let wcex = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        hInstance: hinstance,
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..zeroed()
    };
    
    if RegisterClassExW(&wcex) == 0 {
        SelectObject(mem_dc, old_obj);
        let _ = DeleteObject(hbitmap);
        let _ = DeleteDC(mem_dc);
        return;
    }
    
    // Create window
    let hwnd = CreateWindowExW(
        WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
        PCWSTR(class_name.as_ptr()),
        PCWSTR::null(),
        WS_POPUP,
        win_x,
        win_y,
        img_width as i32,
        img_height as i32,
        HWND::default(),
        None,
        hinstance,
        None,
    );
    
    if hwnd.0 == 0 {
        SelectObject(mem_dc, old_obj);
        let _ = DeleteObject(hbitmap);
        let _ = DeleteDC(mem_dc);
        return;
    }
    
    // Set magenta as transparent
    let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0x00FF00FF), 0, LWA_COLORKEY);
    
    // Show window
    let _ = ShowWindow(hwnd, SW_SHOWNA);
    let _ = UpdateWindow(hwnd);
    let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
    
    // Message loop - runs forever until killed
    let mut msg: MSG = zeroed();
    while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
        let _ = DispatchMessageW(&msg);
    }
    
    // Cleanup
    SelectObject(mem_dc, old_obj);
    let _ = DeleteObject(hbitmap);
    let _ = DeleteDC(mem_dc);
    GLOBAL_DC = None;
}

#[cfg(windows)]
static mut GLOBAL_DC: Option<windows::Win32::Graphics::Gdi::HDC> = None;
#[cfg(windows)]
static mut GLOBAL_WIDTH: u32 = 0;
#[cfg(windows)]
static mut GLOBAL_HEIGHT: u32 = 0;

#[cfg(windows)]
unsafe extern "system" fn wnd_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::LRESULT;
    use windows::Win32::Graphics::Gdi::{BeginPaint, BitBlt, EndPaint, PAINTSTRUCT, SRCCOPY};
    use windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, PostQuitMessage};
    use std::mem::zeroed;
    
    const WM_PAINT: u32 = 0x000F;
    const WM_DESTROY: u32 = 0x0002;
    
    match msg {
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            
            if let Some(src_dc) = GLOBAL_DC {
                let _ = BitBlt(hdc, 0, 0, GLOBAL_WIDTH as i32, GLOBAL_HEIGHT as i32, src_dc, 0, 0, SRCCOPY);
            }
            
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
