/// Tray flyout menu with GDI+ rendering
/// 
/// This module implements a modern-looking flyout menu that spawns from the system tray
/// using Win32 layered windows with GDI+ for anti-aliased rendering and DWM for shadows.

use std::mem;
use std::ptr::null_mut;
use std::sync::mpsc::Sender;
use windows::core::PCWSTR;
use windows::Win32::{
    Foundation::*,
    Graphics::{
        Dwm::*,
        Gdi::*,
        GdiPlus::*,
    },
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::*,
};

use crate::ipc::TrayToGui;
use crate::profile::Profile;

const WINDOW_CLASS: &str = "TrayFlyoutWindowClass";
const FLYOUT_WIDTH: i32 = 386;  // Match PowerToys
const FLYOUT_HEIGHT: i32 = 486;  // Match PowerToys
const ITEM_HEIGHT: i32 = 60;     // Taller items
const PADDING: i32 = 16;

/// Flyout window handle (actual state stored in GWLP_USERDATA for thread safety)
pub struct FlyoutWindow {
    hwnd: HWND,
}

/// Internal state stored in GWLP_USERDATA
struct FlyoutState {
    hwnd: HWND,
    profiles: Vec<Profile>,
    active_profile: Option<String>,
    hover_index: Option<usize>,
    to_gui_tx: Sender<TrayToGui>,
    gdiplus_token: usize,
}

/// Menu item for rendering (internal use)
#[derive(Clone)]
#[allow(dead_code)]
struct MenuItem {
    name: String,
    is_active: bool,
}

impl FlyoutWindow {
    /// Create and show the flyout window near the tray icon
    pub fn new(
        _tray_rect: RECT,
        profiles: Vec<Profile>,
        active_profile: Option<String>,
        to_gui_tx: Sender<TrayToGui>,
    ) -> anyhow::Result<Self> {
        unsafe {
            // Initialize GDI+
            let startup_input = GdiplusStartupInput {
                GdiplusVersion: 1,
                DebugEventCallback: 0,
                SuppressBackgroundThread: FALSE.into(),
                SuppressExternalCodecs: FALSE.into(),
            };
            let mut gdiplus_token: usize = 0;
            let mut output = GdiplusStartupOutput::default();
            let status = GdiplusStartup(&mut gdiplus_token, &startup_input, &mut output);
            if status.0 != 0 {
                anyhow::bail!("Failed to initialize GDI+: {}", status.0);
            }

            // Register window class
            let hinstance = GetModuleHandleW(None)?;
            let class_name = WINDOW_CLASS.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
            
            let wc = WNDCLASSEXW {
                cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wndproc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance.into(),
                hIcon: HICON::default(),
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                hbrBackground: HBRUSH::default(),
                lpszMenuName: PCWSTR::null(),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                hIconSm: HICON::default(),
            };

            RegisterClassExW(&wc);

            // Use fixed dimensions like PowerToys
            let window_height = FLYOUT_HEIGHT;

            // Calculate position - appear above the tray icon in bottom-right
            let screen_width = GetSystemMetrics(SM_CXSCREEN);
            let screen_height = GetSystemMetrics(SM_CYSCREEN);
            
            // Position: right side of screen, above taskbar (like PowerToys)
            let margin = 12; // PowerToys uses 12px margin
            let final_x = screen_width - FLYOUT_WIDTH - margin;
            let final_y = screen_height - window_height - 60; // 60px above bottom (for taskbar)
            
            println!("[FLYOUT] Screen: {}x{}, Position: ({}, {}), Size: {}x{}", 
                screen_width, screen_height, final_x, final_y, FLYOUT_WIDTH, window_height);

            // Create layered window at the correct position
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                PCWSTR(class_name.as_ptr()),
                PCWSTR::null(),
                WS_POPUP,
                final_x,
                final_y,
                FLYOUT_WIDTH,
                window_height,
                HWND::default(),
                HMENU::default(),
                hinstance,
                None,
            );
            
            if hwnd == HWND::default() {
                anyhow::bail!("Failed to create flyout window");
            }

            // Enable DWM shadow
            let policy = DWMNCRENDERINGPOLICY(DWMNCRP_ENABLED.0);
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_NCRENDERING_POLICY,
                &policy as *const _ as *const _,
                mem::size_of::<DWMNCRENDERINGPOLICY>() as u32,
            )?;

            let state = FlyoutState {
                hwnd,
                profiles,
                active_profile,
                hover_index: None,
                to_gui_tx,
                gdiplus_token,
            };

            // Store state in GWLP_USERDATA using Box for proper ownership management
            let state_box = Box::new(state);
            let state_ptr = Box::into_raw(state_box);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);

            // Get reference for initial render (safe - we control lifetime via WM_DESTROY)
            let state_ref = &*state_ptr;

            // Initial render
            FlyoutState::render(state_ref)?;

            // Show and activate window so user can interact
            ShowWindow(hwnd, SW_SHOW);
            use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;
            SetForegroundWindow(hwnd);
            
            // Return lightweight handle
            anyhow::Ok(Self { hwnd })
        }
    }

    /// Show the flyout window
    pub fn show(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
        }
    }

    /// Hide the flyout window
    pub fn hide(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    /// Update profiles list
    pub fn update_profiles(&mut self, profiles: Vec<Profile>, active: Option<String>) -> anyhow::Result<()> {
        unsafe {
            if let Some(state) = Self::get_state(self.hwnd) {
                state.profiles = profiles;
                state.active_profile = active;
                FlyoutState::render(state)?;
            }
            anyhow::Ok(())
        }
    }

    /// Get state reference from window data (for internal use)
    unsafe fn get_state(hwnd: HWND) -> Option<&'static mut FlyoutState> {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        if ptr != 0 {
            Some(&mut *(ptr as *mut FlyoutState))
        } else {
            None
        }
    }

    /// Window procedure
    unsafe extern "system" fn wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_MOUSEMOVE => {
                let state = FlyoutWindow::get_state(hwnd);
                if let Some(state) = state {
                    let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                    let x = (lparam.0 & 0xFFFF) as i16 as i32;
                    
                    // Items start at y=90 (below title and subtitle)
                    let items_start_y = 90;
                    let item_index = (y - items_start_y) / ITEM_HEIGHT;
                    
                    // Check if mouse is in the item area
                    if y >= items_start_y && x >= PADDING && x < (FLYOUT_WIDTH - PADDING) 
                        && item_index >= 0 && (item_index as usize) < state.profiles.len() 
                    {
                        if state.hover_index != Some(item_index as usize) {
                            state.hover_index = Some(item_index as usize);
                            let _ = FlyoutState::render(state);
                        }
                    } else if state.hover_index.is_some() {
                        state.hover_index = None;
                        let _ = FlyoutState::render(state);
                    }
                }
                LRESULT(0)
            }
            WM_LBUTTONDOWN => {
                let state = FlyoutWindow::get_state(hwnd);
                if let Some(state) = state {
                    if let Some(index) = state.hover_index {
                        if let Some(profile) = state.profiles.get(index) {
                            println!("[FLYOUT] Activating profile: {}", profile.name);
                            // Send activation request to main app
                            let _ = state.to_gui_tx.send(TrayToGui::ActivateProfile(profile.name.clone()));
                            // Close flyout
                            let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                        }
                    }
                }
                LRESULT(0)
            }
            WM_KILLFOCUS => {
                // Don't auto-close on focus loss - let user interact
                LRESULT(0)
            }
            WM_ACTIVATE => {
                // Close flyout when deactivated (clicked outside)
                if wparam.0 == 0 {
                    let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                }
                LRESULT(0)
            }
            WM_CLOSE => {
                let _ = DestroyWindow(hwnd);
                LRESULT(0)
            }
            WM_DESTROY => {
                let state = FlyoutWindow::get_state(hwnd);
                if let Some(state) = state {
                    // Cleanup GDI+
                    GdiplusShutdown(state.gdiplus_token);
                    
                    // CRITICAL: Reclaim Box ownership and drop to free memory
                    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
                    if ptr != 0 {
                        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0); // Clear pointer first
                        let _ = Box::from_raw(ptr as *mut FlyoutState); // Drop the Box
                    }
                }
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

impl Drop for FlyoutWindow {
    fn drop(&mut self) {
        unsafe {
            if self.hwnd != HWND::default() {
                let _ = DestroyWindow(self.hwnd);
            }
        }
    }
}

impl FlyoutState {
    /// Render the flyout menu with GDI+
    unsafe fn render(&self) -> anyhow::Result<()> {
        let screen_dc = GetDC(None);
        let mem_dc = CreateCompatibleDC(screen_dc);

        let window_height = FLYOUT_HEIGHT;

        // Create DIB for layered window
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: FLYOUT_WIDTH,
                biHeight: -window_height, // Top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0 as u32,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD::default()],
        };

        let mut bits: *mut core::ffi::c_void = null_mut();
        let hbitmap = CreateDIBSection(
            mem_dc,
            &bmi,
            DIB_RGB_COLORS,
            &mut bits,
            None,
            0,
        )?;

        SelectObject(mem_dc, hbitmap);

        // Create GDI+ Graphics object
        let mut graphics: *mut GpGraphics = null_mut();
        let status = GdipCreateFromHDC(mem_dc, &mut graphics);
        if status.0 != 0 {
            return Err(anyhow::anyhow!("Failed to create GDI+ graphics context"));
        }
        GdipSetSmoothingMode(graphics, SmoothingMode(4)); // SmoothingModeAntiAlias
        GdipSetTextRenderingHint(graphics, TextRenderingHint(5)); // TextRenderingHintClearTypeGridFit

        // Clear with semi-transparent dark background
        let mut brush_bg: *mut GpSolidFill = null_mut();
        GdipCreateSolidFill(0xF0_1E_1E_1E, &mut brush_bg); // ARGB
        GdipFillRectangleI(
            graphics,
            brush_bg as *mut GpBrush,
            0,
            0,
            FLYOUT_WIDTH,
            window_height,
        );

        // Draw rounded background rectangle
        let mut path: *mut GpPath = null_mut();
        GdipCreatePath(FillModeWinding, &mut path);
        Self::add_rounded_rectangle(
            path,
            4.0,
            4.0,
            (FLYOUT_WIDTH - 8) as f32,
            (window_height - 8) as f32,
            8.0,
        );
        GdipFillPath(graphics, brush_bg as *mut GpBrush, path);

        // Create font
        let font_family_name = "Segoe UI\0".encode_utf16().collect::<Vec<u16>>();
        let mut font_family: *mut GpFontFamily = null_mut();
        GdipCreateFontFamilyFromName(PCWSTR(font_family_name.as_ptr()), null_mut(), &mut font_family);
        
        // Title font (larger, semibold)
        let mut title_font: *mut GpFont = null_mut();
        GdipCreateFont(font_family, 18.0, FontStyle(1).0, Unit(2), &mut title_font); // Bold
        
        // Regular font for items
        let mut font: *mut GpFont = null_mut();
        GdipCreateFont(font_family, 14.0, FontStyle(0).0, Unit(2), &mut font);

        // Draw title "Gaming Profiles"
        let mut brush_title: *mut GpSolidFill = null_mut();
        GdipCreateSolidFill(0xFF_FF_FF_FF, &mut brush_title);
        
        let title = "Gaming Profiles\0".encode_utf16().collect::<Vec<u16>>();
        let title_rect = RectF {
            X: PADDING as f32,
            Y: PADDING as f32,
            Width: (FLYOUT_WIDTH - PADDING * 2) as f32,
            Height: 40.0,
        };
        
        let mut string_format: *mut GpStringFormat = null_mut();
        GdipCreateStringFormat(0, 0, &mut string_format);
        GdipSetStringFormatAlign(string_format, StringAlignmentNear);
        GdipSetStringFormatLineAlign(string_format, StringAlignmentNear);
        
        GdipDrawString(
            graphics,
            PCWSTR(title.as_ptr()),
            title.len() as i32 - 1,
            title_font,
            &title_rect,
            string_format,
            brush_title as *mut GpBrush,
        );
        GdipDeleteBrush(brush_title as *mut GpBrush);
        
        // Draw separator line under title
        let mut pen_sep: *mut GpPen = null_mut();
        GdipCreatePen1(0x40_FF_FF_FF, 1.0, UnitPixel, &mut pen_sep);
        GdipDrawLineI(graphics, pen_sep, PADDING, 50, FLYOUT_WIDTH - PADDING, 50);
        GdipDeletePen(pen_sep);
        
        // Subtitle "Select a profile to activate"
        let mut brush_subtitle: *mut GpSolidFill = null_mut();
        GdipCreateSolidFill(0x80_FF_FF_FF, &mut brush_subtitle);
        
        let subtitle = "Click to activate a profile\0".encode_utf16().collect::<Vec<u16>>();
        let subtitle_rect = RectF {
            X: PADDING as f32,
            Y: 56.0,
            Width: (FLYOUT_WIDTH - PADDING * 2) as f32,
            Height: 24.0,
        };
        
        let mut small_font: *mut GpFont = null_mut();
        GdipCreateFont(font_family, 11.0, FontStyle(0).0, Unit(2), &mut small_font);
        
        GdipDrawString(
            graphics,
            PCWSTR(subtitle.as_ptr()),
            subtitle.len() as i32 - 1,
            small_font,
            &subtitle_rect,
            string_format,
            brush_subtitle as *mut GpBrush,
        );
        GdipDeleteBrush(brush_subtitle as *mut GpBrush);
        
        // Profile items start below subtitle
        let items_start_y = 90;

        // Draw profile items
        for (i, profile) in self.profiles.iter().enumerate() {
            let y = items_start_y + i as i32 * ITEM_HEIGHT;
            let is_hover = self.hover_index == Some(i);
            let is_active = self.active_profile.as_ref() == Some(&profile.name);

            // Item background (rounded rectangle for hover)
            if is_hover {
                let mut brush_hover: *mut GpSolidFill = null_mut();
                GdipCreateSolidFill(0x40_FF_FF_FF, &mut brush_hover);
                
                let mut hover_path: *mut GpPath = null_mut();
                GdipCreatePath(FillModeWinding, &mut hover_path);
                Self::add_rounded_rectangle(
                    hover_path,
                    PADDING as f32,
                    y as f32,
                    (FLYOUT_WIDTH - PADDING * 2) as f32,
                    (ITEM_HEIGHT - 4) as f32,
                    6.0,
                );
                GdipFillPath(graphics, brush_hover as *mut GpBrush, hover_path);
                GdipDeletePath(hover_path);
                GdipDeleteBrush(brush_hover as *mut GpBrush);
            }

            // Profile name text
            let mut brush_text: *mut GpSolidFill = null_mut();
            GdipCreateSolidFill(0xFF_FF_FF_FF, &mut brush_text);
            
            let text = profile.name.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
            let rect = RectF {
                X: (PADDING + 12) as f32,
                Y: (y + 8) as f32,
                Width: (FLYOUT_WIDTH - PADDING * 2 - 50) as f32,
                Height: 24.0,
            };

            GdipDrawString(
                graphics,
                PCWSTR(text.as_ptr()),
                text.len() as i32 - 1,
                font,
                &rect,
                string_format,
                brush_text as *mut GpBrush,
            );
            
            // Profile description (processes to kill count)
            let desc = format!("{} processes to manage\0", profile.processes_to_kill.len());
            let desc_utf16: Vec<u16> = desc.encode_utf16().collect();
            let desc_rect = RectF {
                X: (PADDING + 12) as f32,
                Y: (y + 30) as f32,
                Width: (FLYOUT_WIDTH - PADDING * 2 - 50) as f32,
                Height: 20.0,
            };
            
            let mut brush_desc: *mut GpSolidFill = null_mut();
            GdipCreateSolidFill(0x80_FF_FF_FF, &mut brush_desc);
            
            GdipDrawString(
                graphics,
                PCWSTR(desc_utf16.as_ptr()),
                desc_utf16.len() as i32 - 1,
                small_font,
                &desc_rect,
                string_format,
                brush_desc as *mut GpBrush,
            );
            GdipDeleteBrush(brush_desc as *mut GpBrush);

            // Active indicator (checkmark or "Active" badge)
            if is_active {
                let mut brush_active: *mut GpSolidFill = null_mut();
                GdipCreateSolidFill(0xFF_4C_AF_50, &mut brush_active); // Green
                
                let badge_x = FLYOUT_WIDTH - PADDING - 60;
                let badge_y = y + ITEM_HEIGHT / 2 - 10;
                
                // Draw "Active" text
                let active_text = "Active\0".encode_utf16().collect::<Vec<u16>>();
                let active_rect = RectF {
                    X: badge_x as f32,
                    Y: badge_y as f32,
                    Width: 50.0,
                    Height: 20.0,
                };
                GdipDrawString(
                    graphics,
                    PCWSTR(active_text.as_ptr()),
                    active_text.len() as i32 - 1,
                    small_font,
                    &active_rect,
                    string_format,
                    brush_active as *mut GpBrush,
                );
                GdipDeleteBrush(brush_active as *mut GpBrush);
            }

            GdipDeleteBrush(brush_text as *mut GpBrush);
        }
        
        // Draw "No profiles" message if empty
        if self.profiles.is_empty() {
            let mut brush_empty: *mut GpSolidFill = null_mut();
            GdipCreateSolidFill(0x80_FF_FF_FF, &mut brush_empty);
            
            let empty_text = "No gaming profiles configured\0".encode_utf16().collect::<Vec<u16>>();
            let empty_rect = RectF {
                X: PADDING as f32,
                Y: (window_height / 2 - 20) as f32,
                Width: (FLYOUT_WIDTH - PADDING * 2) as f32,
                Height: 40.0,
            };
            
            let mut center_format: *mut GpStringFormat = null_mut();
            GdipCreateStringFormat(0, 0, &mut center_format);
            GdipSetStringFormatAlign(center_format, StringAlignmentCenter);
            GdipSetStringFormatLineAlign(center_format, StringAlignmentCenter);
            
            GdipDrawString(
                graphics,
                PCWSTR(empty_text.as_ptr()),
                empty_text.len() as i32 - 1,
                font,
                &empty_rect,
                center_format,
                brush_empty as *mut GpBrush,
            );
            GdipDeleteBrush(brush_empty as *mut GpBrush);
            GdipDeleteStringFormat(center_format);
        }

        // Cleanup GDI+ resources
        GdipDeleteFont(font);
        GdipDeleteFont(title_font);
        GdipDeleteFont(small_font);
        GdipDeleteFontFamily(font_family);
        GdipDeletePath(path);
        GdipDeleteBrush(brush_bg as *mut GpBrush);
        GdipDeleteStringFormat(string_format);
        GdipDeleteGraphics(graphics);

        // Premultiply alpha for layered window
        Self::premultiply_alpha(bits as *mut u8, FLYOUT_WIDTH, window_height);

        // Update layered window (don't pass win_pos, use current window position)
        let win_size = SIZE {
            cx: FLYOUT_WIDTH,
            cy: window_height,
        };
        let src_pos = POINT { x: 0, y: 0 };
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };

        UpdateLayeredWindow(
            self.hwnd,
            screen_dc,
            None,  // Use current window position
            Some(&win_size),
            mem_dc,
            Some(&src_pos),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        )?;

        // Cleanup
        DeleteObject(hbitmap);
        DeleteDC(mem_dc);
        ReleaseDC(None, screen_dc);

        anyhow::Ok(())
    }

    /// Add rounded rectangle path to GDI+ path
    unsafe fn add_rounded_rectangle(
        path: *mut GpPath,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
    ) {
        // Top-left arc
        GdipAddPathArcI(path, x as i32, y as i32, (radius * 2.0) as i32, (radius * 2.0) as i32, 180.0, 90.0);
        // Top line
        GdipAddPathLineI(path, (x + radius) as i32, y as i32, (x + width - radius) as i32, y as i32);
        // Top-right arc
        GdipAddPathArcI(path, (x + width - radius * 2.0) as i32, y as i32, (radius * 2.0) as i32, (radius * 2.0) as i32, 270.0, 90.0);
        // Right line
        GdipAddPathLineI(path, (x + width) as i32, (y + radius) as i32, (x + width) as i32, (y + height - radius) as i32);
        // Bottom-right arc
        GdipAddPathArcI(path, (x + width - radius * 2.0) as i32, (y + height - radius * 2.0) as i32, (radius * 2.0) as i32, (radius * 2.0) as i32, 0.0, 90.0);
        // Bottom line
        GdipAddPathLineI(path, (x + width - radius) as i32, (y + height) as i32, (x + radius) as i32, (y + height) as i32);
        // Bottom-left arc
        GdipAddPathArcI(path, x as i32, (y + height - radius * 2.0) as i32, (radius * 2.0) as i32, (radius * 2.0) as i32, 90.0, 90.0);
        // Left line
        GdipAddPathLineI(path, x as i32, (y + height - radius) as i32, x as i32, (y + radius) as i32);
        
        GdipClosePathFigure(path);
    }

    /// Draw checkmark symbol
    unsafe fn draw_checkmark(graphics: *mut GpGraphics, x: i32, y: i32) {
        let mut pen: *mut GpPen = null_mut();
        GdipCreatePen1(0xFF_4C_AF_50, 2.5, Unit(2), &mut pen); // Green checkmark
        
        // Draw checkmark path
        let points = [
            Point { X: x - 6, Y: y },
            Point { X: x - 2, Y: y + 5 },
            Point { X: x + 6, Y: y - 5 },
        ];
        
        GdipDrawLinesI(graphics, pen, points.as_ptr(), 3);
        GdipDeletePen(pen);
    }

    /// Premultiply alpha for proper blending
    unsafe fn premultiply_alpha(bits: *mut u8, width: i32, height: i32) {
        let pixel_count = (width * height) as usize;
        for i in 0..pixel_count {
            let offset = i * 4;
            let b = *bits.add(offset);
            let g = *bits.add(offset + 1);
            let r = *bits.add(offset + 2);
            let a = *bits.add(offset + 3);

            if a > 0 && a < 255 {
                *bits.add(offset) = ((b as u16 * a as u16) / 255) as u8;
                *bits.add(offset + 1) = ((g as u16 * a as u16) / 255) as u8;
                *bits.add(offset + 2) = ((r as u16 * a as u16) / 255) as u8;
            }
        }
    }
}
