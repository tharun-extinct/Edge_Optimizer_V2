//! Crosshair overlay - runs as a completely separate window process
//! Uses pure Windows API for transparent, click-through, always-on-top display

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::path::Path;

/// Thread-safe handle to control the overlay
pub struct OverlayHandle {
    running: Arc<AtomicBool>,
    _handle: JoinHandle<()>,
}

impl OverlayHandle {
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Drop for OverlayHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Start a crosshair overlay. Returns immediately, overlay runs in background thread.
pub fn start_overlay(
    image_path: String,
    x_offset: i32,
    y_offset: i32,
) -> Result<OverlayHandle, String> {
    // Verify image exists
    if !Path::new(&image_path).exists() {
        return Err(format!("Crosshair image not found: {}", image_path));
    }
    
    // Pre-load the image
    let img = image::open(&image_path)
        .map_err(|e| format!("Failed to load crosshair image: {}", e))?;
    
    let rgba = img.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    
    // Pre-convert to BGRA
    let mut bgra_pixels: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
    for pixel in rgba.pixels() {
        bgra_pixels.push(pixel[2]); // B
        bgra_pixels.push(pixel[1]); // G
        bgra_pixels.push(pixel[0]); // R
        bgra_pixels.push(pixel[3]); // A
    }
    
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    
    println!("[Crosshair] Starting overlay thread...");
    println!("[Crosshair] Image: {}x{}, Offset: ({}, {})", width, height, x_offset, y_offset);
    
    let handle = thread::spawn(move || {
        #[cfg(windows)]
        unsafe {
            overlay_thread_main(bgra_pixels, width, height, x_offset, y_offset, running_clone);
        }
        
        #[cfg(not(windows))]
        {
            let _ = (bgra_pixels, width, height, x_offset, y_offset, running_clone);
            eprintln!("[Crosshair] Windows only!");
        }
    });
    
    Ok(OverlayHandle {
        running,
        _handle: handle,
    })
}

#[cfg(windows)]
unsafe fn overlay_thread_main(
    pixels: Vec<u8>,
    img_width: u32,
    img_height: u32,
    x_offset: i32,
    y_offset: i32,
    running: Arc<AtomicBool>,
) {
    use std::mem::zeroed;
    use std::ptr::null_mut;
    
    use windows::Win32::Foundation::{COLORREF, HWND, HINSTANCE};
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, DeleteDC,
        DeleteObject, SelectObject, UpdateWindow, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        GetDC, ReleaseDC,
    };
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DispatchMessageW, PeekMessageW,
        RegisterClassExW, ShowWindow, WNDCLASSEXW, CS_HREDRAW, CS_VREDRAW,
        WS_EX_LAYERED, WS_EX_TRANSPARENT, WS_EX_TOPMOST, WS_EX_TOOLWINDOW,
        WS_POPUP, SW_SHOWNA, MSG, PM_REMOVE,
        GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN, SetLayeredWindowAttributes,
        LWA_COLORKEY, SetWindowPos, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE,
        DestroyWindow,
    };
    use windows::Win32::Graphics::Gdi::InvalidateRect;
    use windows::core::PCWSTR;
    
    // ===== SCREEN SIZE =====
    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let screen_h = GetSystemMetrics(SM_CYSCREEN);
    
    // ===== CALCULATE CENTER POSITION =====
    let win_x = (screen_w / 2) - (img_width as i32 / 2) + x_offset;
    let win_y = (screen_h / 2) - (img_height as i32 / 2) + y_offset;
    
    println!("[Crosshair] Screen: {}x{}", screen_w, screen_h);
    println!("[Crosshair] Window position: ({}, {})", win_x, win_y);
    
    // ===== CREATE UNIQUE CLASS NAME =====
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let class_name_str = format!("XhairOverlay_{}\0", timestamp);
    let class_name: Vec<u16> = class_name_str.encode_utf16().collect();
    
    // ===== GET HINSTANCE =====
    let hinstance = match GetModuleHandleW(PCWSTR::null()) {
        Ok(h) => HINSTANCE(h.0),
        Err(_) => {
            eprintln!("[Crosshair] Failed to get module handle");
            return;
        }
    };
    
    // ===== CREATE BITMAP DC AND BITMAP =====
    let screen_dc = GetDC(HWND::default());
    let mem_dc = CreateCompatibleDC(screen_dc);
    ReleaseDC(HWND::default(), screen_dc);
    
    let bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: img_width as i32,
            biHeight: -(img_height as i32), // Negative = top-down bitmap
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0 as u32,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [zeroed(); 1],
    };
    
    let mut bits_ptr: *mut std::ffi::c_void = null_mut();
    let hbitmap = match CreateDIBSection(mem_dc, &bmi, DIB_RGB_COLORS, &mut bits_ptr, None, 0) {
        Ok(bmp) => bmp,
        Err(e) => {
            eprintln!("[Crosshair] Failed to create DIB: {:?}", e);
            DeleteDC(mem_dc);
            return;
        }
    };
    
    if bits_ptr.is_null() {
        eprintln!("[Crosshair] DIB bits pointer is null!");
        DeleteObject(hbitmap);
        DeleteDC(mem_dc);
        return;
    }
    
    // ===== COPY PIXELS WITH MAGENTA TRANSPARENCY =====
    let dst = std::slice::from_raw_parts_mut(
        bits_ptr as *mut u8,
        (img_width * img_height * 4) as usize,
    );
    
    // Magenta color for transparency (RGB 255,0,255 -> BGR 255,0,255)
    for (i, chunk) in pixels.chunks(4).enumerate() {
        let idx = i * 4;
        let alpha = chunk[3];
        
        if alpha < 128 {
            // Transparent -> magenta (color key will make this see-through)
            dst[idx + 0] = 255; // B
            dst[idx + 1] = 0;   // G
            dst[idx + 2] = 255; // R
            dst[idx + 3] = 255; // A
        } else {
            // Opaque -> copy original BGRA
            dst[idx + 0] = chunk[0]; // B
            dst[idx + 1] = chunk[1]; // G
            dst[idx + 2] = chunk[2]; // R
            dst[idx + 3] = chunk[3]; // A
        }
    }
    
    println!("[Crosshair] Bitmap created and filled");
    
    // Select bitmap into DC
    let old_obj = SelectObject(mem_dc, hbitmap);
    
    // ===== STORE GLOBALS FOR WM_PAINT =====
    GLOBAL_OVERLAY = Some(GlobalOverlay {
        mem_dc,
        img_width,
        img_height,
    });
    
    // ===== REGISTER WINDOW CLASS =====
    let wcex = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(crosshair_wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: Default::default(),
        hCursor: Default::default(),
        hbrBackground: Default::default(),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        hIconSm: Default::default(),
    };
    
    let atom = RegisterClassExW(&wcex);
    if atom == 0 {
        eprintln!("[Crosshair] Failed to register window class");
        SelectObject(mem_dc, old_obj);
        DeleteObject(hbitmap);
        DeleteDC(mem_dc);
        GLOBAL_OVERLAY = None;
        return;
    }
    
    println!("[Crosshair] Window class registered");
    
    // ===== CREATE OVERLAY WINDOW =====
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
        eprintln!("[Crosshair] Failed to create window!");
        SelectObject(mem_dc, old_obj);
        DeleteObject(hbitmap);
        DeleteDC(mem_dc);
        GLOBAL_OVERLAY = None;
        return;
    }
    
    println!("[Crosshair] Window created: {:?}", hwnd.0);
    
    // ===== SET TRANSPARENCY COLOR KEY =====
    // Magenta (FF00FF) in COLORREF is 0x00FF00FF (BGR order)
    let magenta = COLORREF(0x00FF00FF);
    if let Err(e) = SetLayeredWindowAttributes(hwnd, magenta, 0, LWA_COLORKEY) {
        eprintln!("[Crosshair] SetLayeredWindowAttributes failed: {:?}", e);
    } else {
        println!("[Crosshair] Transparency set (magenta key)");
    }
    
    // ===== SHOW WINDOW =====
    let _ = ShowWindow(hwnd, SW_SHOWNA);
    let _ = UpdateWindow(hwnd);
    let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
    
    println!("[Crosshair] Window shown - crosshair should be visible!");
    
    // ===== MESSAGE LOOP =====
    let mut msg: MSG = zeroed();
    let mut counter = 0u32;
    
    while running.load(Ordering::SeqCst) {
        // Non-blocking message peek
        while PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE).as_bool() {
            if msg.message == 0x0012 { // WM_QUIT
                running.store(false, Ordering::SeqCst);
                break;
            }
            let _ = DispatchMessageW(&msg);
        }
        
        // Periodically keep on top and refresh
        counter = counter.wrapping_add(1);
        if counter % 20 == 0 {
            let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
            let _ = InvalidateRect(hwnd, None, false);
        }
        
        thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
    }
    
    // ===== CLEANUP =====
    println!("[Crosshair] Cleaning up...");
    let _ = DestroyWindow(hwnd);
    SelectObject(mem_dc, old_obj);
    let _ = DeleteObject(hbitmap);
    let _ = DeleteDC(mem_dc);
    GLOBAL_OVERLAY = None;
    println!("[Crosshair] Overlay stopped");
}

// Global storage for bitmap DC (used by WM_PAINT)
#[cfg(windows)]
struct GlobalOverlay {
    mem_dc: windows::Win32::Graphics::Gdi::HDC,
    img_width: u32,
    img_height: u32,
}

#[cfg(windows)]
static mut GLOBAL_OVERLAY: Option<GlobalOverlay> = None;

#[cfg(windows)]
unsafe extern "system" fn crosshair_wnd_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::LRESULT;
    use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, BitBlt, PAINTSTRUCT, SRCCOPY};
    use windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, PostQuitMessage};
    use std::mem::zeroed;
    
    const WM_PAINT_VAL: u32 = 0x000F;
    const WM_DESTROY_VAL: u32 = 0x0002;
    
    match msg {
        WM_PAINT_VAL => {
            let mut ps: PAINTSTRUCT = zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            
            if let Some(ref ov) = GLOBAL_OVERLAY {
                let _ = BitBlt(
                    hdc,
                    0, 0,
                    ov.img_width as i32,
                    ov.img_height as i32,
                    ov.mem_dc,
                    0, 0,
                    SRCCOPY,
                );
            }
            
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_DESTROY_VAL => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
