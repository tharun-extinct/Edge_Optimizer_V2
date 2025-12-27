//! Standalone crosshair overlay - works over fullscreen games
//! Uses DWM composition like Xbox Game Bar, Discord, and NVIDIA overlays
//! Usage: crosshair.exe <image_path> <x_offset> <y_offset>

#![windows_subsystem = "windows"]

use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 4 {
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
    
    // Convert to BGRA (premultiplied alpha for UpdateLayeredWindow)
    let mut bgra_pixels: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
    for pixel in rgba.pixels() {
        let a = pixel[3] as f32 / 255.0;
        // Premultiply alpha for proper blending
        bgra_pixels.push((pixel[2] as f32 * a) as u8); // B
        bgra_pixels.push((pixel[1] as f32 * a) as u8); // G
        bgra_pixels.push((pixel[0] as f32 * a) as u8); // R
        bgra_pixels.push(pixel[3]);                     // A
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
    
    use windows::Win32::Foundation::{COLORREF, HWND, HINSTANCE, POINT, SIZE};
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject,
        GetDC, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER,
        BI_RGB, DIB_RGB_COLORS, AC_SRC_ALPHA, AC_SRC_OVER, BLENDFUNCTION,
    };
    use windows::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea;
    use windows::Win32::UI::Controls::MARGINS;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DispatchMessageW, PeekMessageW,
        GetSystemMetrics, RegisterClassExW, SetWindowPos, ShowWindow,
        UpdateLayeredWindow, CS_HREDRAW, CS_VREDRAW, HWND_TOPMOST,
        MSG, PM_REMOVE, SM_CXSCREEN, SM_CYSCREEN, SWP_NOMOVE, SWP_NOSIZE,
        SWP_NOACTIVATE, SW_SHOWNA, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_TOOLWINDOW,
        WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_EX_NOACTIVATE, WS_POPUP,
        ULW_ALPHA,
    };
    use windows::core::PCWSTR;
    
    // Screen dimensions
    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let screen_h = GetSystemMetrics(SM_CYSCREEN);
    
    // Calculate centered position
    let win_x = (screen_w / 2) - (img_width as i32 / 2) + x_offset;
    let win_y = (screen_h / 2) - (img_height as i32 / 2) + y_offset;
    
    // Unique class name
    let class_name: Vec<u16> = "CrosshairDWMOverlay\0".encode_utf16().collect();
    
    let hinstance = match GetModuleHandleW(PCWSTR::null()) {
        Ok(h) => HINSTANCE(h.0),
        Err(_) => return,
    };
    
    // Create bitmap with alpha channel
    let screen_dc = GetDC(HWND::default());
    let mem_dc = CreateCompatibleDC(screen_dc);
    
    let bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: img_width as i32,
            biHeight: -(img_height as i32), // Top-down
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
            ReleaseDC(HWND::default(), screen_dc);
            DeleteDC(mem_dc);
            return;
        }
    };
    
    if bits_ptr.is_null() {
        ReleaseDC(HWND::default(), screen_dc);
        let _ = DeleteObject(hbitmap);
        let _ = DeleteDC(mem_dc);
        return;
    }
    
    // Copy premultiplied alpha pixels
    let dst = std::slice::from_raw_parts_mut(bits_ptr as *mut u8, (img_width * img_height * 4) as usize);
    dst.copy_from_slice(&pixels);
    
    let old_obj = SelectObject(mem_dc, hbitmap);
    
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
        ReleaseDC(HWND::default(), screen_dc);
        let _ = DeleteObject(hbitmap);
        let _ = DeleteDC(mem_dc);
        return;
    }
    
    // Create window with all necessary extended styles
    let hwnd = CreateWindowExW(
        WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
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
        ReleaseDC(HWND::default(), screen_dc);
        let _ = DeleteObject(hbitmap);
        let _ = DeleteDC(mem_dc);
        return;
    }
    
    // ===== DWM MAGIC - This is how Xbox Game Bar works =====
    // Extend frame into client area with -1 margins
    // This makes the window part of DWM composition
    let margins = MARGINS {
        cxLeftWidth: -1,
        cxRightWidth: -1,
        cyTopHeight: -1,
        cyBottomHeight: -1,
    };
    let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);
    
    // Use UpdateLayeredWindow with per-pixel alpha for proper transparency
    let blend = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER as u8,
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: AC_SRC_ALPHA as u8,
    };
    
    let size = SIZE {
        cx: img_width as i32,
        cy: img_height as i32,
    };
    
    let src_point = POINT { x: 0, y: 0 };
    let win_point = POINT { x: win_x, y: win_y };
    
    // Update the layered window with our bitmap
    let _ = UpdateLayeredWindow(
        hwnd,
        screen_dc,
        Some(&win_point),
        Some(&size),
        mem_dc,
        Some(&src_point),
        COLORREF(0), // Unused when ULW_ALPHA is set
        Some(&blend),
        ULW_ALPHA,
    );
    
    ReleaseDC(HWND::default(), screen_dc);
    
    // Force topmost
    let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
    
    // Show window without activating
    let _ = ShowWindow(hwnd, SW_SHOWNA);
    
    // Store for cleanup
    GLOBAL_HWND = Some(hwnd);
    
    // Message loop with periodic topmost refresh
    let mut msg: MSG = zeroed();
    let mut counter: u32 = 0;
    
    loop {
        // Process messages (non-blocking)
        while PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE).as_bool() {
            if msg.message == 0x0012 { // WM_QUIT
                // Cleanup
                SelectObject(mem_dc, old_obj);
                let _ = DeleteObject(hbitmap);
                let _ = DeleteDC(mem_dc);
                GLOBAL_HWND = None;
                return;
            }
            let _ = DispatchMessageW(&msg);
        }
        
        // Every ~100ms, re-assert topmost (fights fullscreen games)
        counter = counter.wrapping_add(1);
        if counter % 6 == 0 {
            let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        }
        
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

#[cfg(windows)]
static mut GLOBAL_HWND: Option<windows::Win32::Foundation::HWND> = None;

#[cfg(windows)]
unsafe extern "system" fn wnd_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::LRESULT;
    use windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, PostQuitMessage};
    
    const WM_DESTROY: u32 = 0x0002;
    const WM_NCHITTEST: u32 = 0x0084;
    const HTTRANSPARENT: i32 = -1;
    
    match msg {
        WM_NCHITTEST => {
            // Make window completely click-through
            LRESULT(HTTRANSPARENT as isize)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
