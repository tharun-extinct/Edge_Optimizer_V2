use std::ptr:: null_mut;
use std:: mem;
use winapi::um::winuser::*;
use winapi::um:: wingdi::*;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use winapi::um::dwmapi::*;

const WS_EX_TOPMOST: u32 = 0x00000008;
const WS_EX_LAYERED: u32 = 0x00080000;
const WS_EX_TRANSPARENT: u32 = 0x00000020;
const WS_EX_NOACTIVATE: u32 = 0x08000000;

pub struct CrosshairOverlay {
    hwnd: HWND,
    size: i32,
    x: i32,
    y:  i32,
}

impl CrosshairOverlay {
    pub fn new(size: i32) -> Result<Self, String> {
        let hwnd = unsafe { Self::create_overlay_window(size)? };
        
        let (screen_width, screen_height) = Self::get_screen_dimensions();
        let x = (screen_width - size) / 2;
        let y = (screen_height - size) / 2;

        Ok(CrosshairOverlay {
            hwnd,
            size,
            x,
            y,
        })
    }

    unsafe fn create_overlay_window(size: i32) -> Result<HWND, String> {
        let class_name = Self::to_wstring("CrosshairOverlay");
        
        // Register window class
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self:: window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: GetModuleHandleW(null_mut()),
            hIcon: null_mut(),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
            lpszClassName: class_name. as_ptr(),
        };

        if RegisterClassW(&wc) == 0 {
            return Err("Failed to register window class". to_string());
        }

        let (screen_width, screen_height) = Self::get_screen_dimensions();
        let x = (screen_width - size) / 2;
        let y = (screen_height - size) / 2;

        // Create the window with proper flags
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE,
            class_name.as_ptr(),
            Self::to_wstring("Crosshair").as_ptr(),
            WS_POPUP,
            x,
            y,
            size,
            size,
            null_mut(),
            null_mut(),
            GetModuleHandleW(null_mut()),
            null_mut(),
        );

        if hwnd. is_null() {
            return Err("Failed to create window". to_string());
        }

        // Make the window transparent and always on top
        SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA);
        
        // Extended window style to stay on top of fullscreen apps
        let mut margins = MARGINS {
            cxLeftWidth: -1,
            cxRightWidth: -1,
            cyTopHeight: -1,
            cyBottomHeight: -1,
        };
        DwmExtendFrameIntoClientArea(hwnd, &mut margins);

        // Force the window to be topmost
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        Ok(hwnd)
    }

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg:  UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_PAINT => {
                let mut ps:  PAINTSTRUCT = mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                
                // Draw crosshair
                Self::draw_crosshair(hdc, hwnd);
                
                EndPaint(hwnd, &ps);
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    unsafe fn draw_crosshair(hdc: HDC, hwnd: HWND) {
        let mut rect:  RECT = mem::zeroed();
        GetClientRect(hwnd, &mut rect);
        
        let center_x = (rect.right - rect.left) / 2;
        let center_y = (rect.bottom - rect.top) / 2;
        
        // Create a green pen
        let pen = CreatePen(PS_SOLID, 2, RGB(0, 255, 0));
        let old_pen = SelectObject(hdc, pen as *mut _);
        
        // Draw horizontal line
        MoveToEx(hdc, center_x - 10, center_y, null_mut());
        LineTo(hdc, center_x + 10, center_y);
        
        // Draw vertical line
        MoveToEx(hdc, center_x, center_y - 10, null_mut());
        LineTo(hdc, center_x, center_y + 10);
        
        // Draw center dot
        let brush = CreateSolidBrush(RGB(0, 255, 0));
        let old_brush = SelectObject(hdc, brush as *mut _);
        Ellipse(hdc, center_x - 2, center_y - 2, center_x + 2, center_y + 2);
        
        SelectObject(hdc, old_brush);
        SelectObject(hdc, old_pen);
        DeleteObject(brush as *mut _);
        DeleteObject(pen as *mut _);
    }

    fn get_screen_dimensions() -> (i32, i32) {
        unsafe {
            (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN))
        }
    }

    fn to_wstring(s: &str) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;
        use std::ffi::OsStr;
        OsStr::new(s)
            .encode_wide()
            .chain(Some(0))
            .collect()
    }

    pub fn run_message_loop(&self) -> Result<(), String> {
        unsafe {
            let mut msg:  MSG = mem::zeroed();
            
            // Continuously ensure window stays on top
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    // Re-assert topmost status periodically for fullscreen games
                    SetWindowPos(
                        msg.hwnd,
                        HWND_TOPMOST,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                    );
                }
            });

            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        Ok(())
    }

    pub fn update_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
        unsafe {
            SetWindowPos(
                self.hwnd,
                HWND_TOPMOST,
                x,
                y,
                self.size,
                self.size,
                SWP_NOACTIVATE,
            );
        }
    }
}

// Required structs for DWM API
#[repr(C)]
struct MARGINS {
    cxLeftWidth: i32,
    cxRightWidth: i32,
    cyTopHeight: i32,
    cyBottomHeight:  i32,
}