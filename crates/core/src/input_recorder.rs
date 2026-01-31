//! Input Recorder Module
//!
//! Uses Windows low-level keyboard hooks to capture ONLY keyboard events for macro recording.
//! Mouse events are NOT recorded - they must be inserted manually via the Insert Event menu.
//! Runs in a background thread with its own Windows message pump.

use crate::macro_config::MacroAction;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;
use tracing::{debug, info, warn, error};

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, SetWindowsHookExW, 
    TranslateMessage, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, 
    MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

#[cfg(target_os = "windows")]
use std::cell::RefCell;

#[cfg(target_os = "windows")]
thread_local! {
    static HOOK_TX: RefCell<Option<Sender<MacroAction>>> = const { RefCell::new(None) };
    static HOOK_RECORDING: RefCell<bool> = const { RefCell::new(false) };
    static HOOK_LAST_TIME: RefCell<Instant> = RefCell::new(Instant::now());
}

/// Converts Windows virtual key code to a string representation
#[cfg(target_os = "windows")]
fn vk_to_string(vk: u32) -> String {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    
    match VIRTUAL_KEY(vk as u16) {
        VK_BACK => "Backspace".to_string(),
        VK_TAB => "Tab".to_string(),
        VK_RETURN => "Enter".to_string(),
        VK_SHIFT => "Shift".to_string(),
        VK_CONTROL => "Ctrl".to_string(),
        VK_MENU => "Alt".to_string(),
        VK_PAUSE => "Pause".to_string(),
        VK_CAPITAL => "CapsLock".to_string(),
        VK_ESCAPE => "Esc".to_string(),
        VK_SPACE => "Space".to_string(),
        VK_PRIOR => "PageUp".to_string(),
        VK_NEXT => "PageDown".to_string(),
        VK_END => "End".to_string(),
        VK_HOME => "Home".to_string(),
        VK_LEFT => "Left".to_string(),
        VK_UP => "Up".to_string(),
        VK_RIGHT => "Right".to_string(),
        VK_DOWN => "Down".to_string(),
        VK_INSERT => "Insert".to_string(),
        VK_DELETE => "Delete".to_string(),
        // Numbers
        VK_0 => "0".to_string(),
        VK_1 => "1".to_string(),
        VK_2 => "2".to_string(),
        VK_3 => "3".to_string(),
        VK_4 => "4".to_string(),
        VK_5 => "5".to_string(),
        VK_6 => "6".to_string(),
        VK_7 => "7".to_string(),
        VK_8 => "8".to_string(),
        VK_9 => "9".to_string(),
        // Letters
        VK_A => "A".to_string(),
        VK_B => "B".to_string(),
        VK_C => "C".to_string(),
        VK_D => "D".to_string(),
        VK_E => "E".to_string(),
        VK_F => "F".to_string(),
        VK_G => "G".to_string(),
        VK_H => "H".to_string(),
        VK_I => "I".to_string(),
        VK_J => "J".to_string(),
        VK_K => "K".to_string(),
        VK_L => "L".to_string(),
        VK_M => "M".to_string(),
        VK_N => "N".to_string(),
        VK_O => "O".to_string(),
        VK_P => "P".to_string(),
        VK_Q => "Q".to_string(),
        VK_R => "R".to_string(),
        VK_S => "S".to_string(),
        VK_T => "T".to_string(),
        VK_U => "U".to_string(),
        VK_V => "V".to_string(),
        VK_W => "W".to_string(),
        VK_X => "X".to_string(),
        VK_Y => "Y".to_string(),
        VK_Z => "Z".to_string(),
        // Numpad
        VK_NUMPAD0 => "Num0".to_string(),
        VK_NUMPAD1 => "Num1".to_string(),
        VK_NUMPAD2 => "Num2".to_string(),
        VK_NUMPAD3 => "Num3".to_string(),
        VK_NUMPAD4 => "Num4".to_string(),
        VK_NUMPAD5 => "Num5".to_string(),
        VK_NUMPAD6 => "Num6".to_string(),
        VK_NUMPAD7 => "Num7".to_string(),
        VK_NUMPAD8 => "Num8".to_string(),
        VK_NUMPAD9 => "Num9".to_string(),
        VK_MULTIPLY => "Num*".to_string(),
        VK_ADD => "Num+".to_string(),
        VK_SUBTRACT => "Num-".to_string(),
        VK_DECIMAL => "NumDel".to_string(),
        VK_DIVIDE => "Num/".to_string(),
        // Function keys
        VK_F1 => "F1".to_string(),
        VK_F2 => "F2".to_string(),
        VK_F3 => "F3".to_string(),
        VK_F4 => "F4".to_string(),
        VK_F5 => "F5".to_string(),
        VK_F6 => "F6".to_string(),
        VK_F7 => "F7".to_string(),
        VK_F8 => "F8".to_string(),
        VK_F9 => "F9".to_string(),
        VK_F10 => "F10".to_string(),
        VK_F11 => "F11".to_string(),
        VK_F12 => "F12".to_string(),
        VK_NUMLOCK => "NumLock".to_string(),
        VK_SCROLL => "ScrollLock".to_string(),
        VK_LSHIFT => "Shift".to_string(),
        VK_RSHIFT => "Shift".to_string(),
        VK_LCONTROL => "Ctrl".to_string(),
        VK_RCONTROL => "Ctrl".to_string(),
        VK_LMENU => "Alt".to_string(),
        VK_RMENU => "AltGr".to_string(),
        VK_LWIN => "Win".to_string(),
        VK_RWIN => "Win".to_string(),
        // Punctuation
        VK_OEM_1 => ";".to_string(),
        VK_OEM_PLUS => "=".to_string(),
        VK_OEM_COMMA => ",".to_string(),
        VK_OEM_MINUS => "-".to_string(),
        VK_OEM_PERIOD => ".".to_string(),
        VK_OEM_2 => "/".to_string(),
        VK_OEM_3 => "`".to_string(),
        VK_OEM_4 => "[".to_string(),
        VK_OEM_5 => "\\".to_string(),
        VK_OEM_6 => "]".to_string(),
        VK_OEM_7 => "'".to_string(),
        VK_SNAPSHOT => "PrintScreen".to_string(),
        _ => format!("Key{}", vk),
    }
}

/// Low-level keyboard hook callback
#[cfg(target_os = "windows")]
unsafe extern "system" fn keyboard_hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    use windows::Win32::UI::WindowsAndMessaging::HC_ACTION;
    
    if code == HC_ACTION as i32 {
        let kb_struct = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        let vk_code = kb_struct.vkCode;
        
        // Determine if it's a key press or release
        let is_press = matches!(wparam.0 as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
        let is_release = matches!(wparam.0 as u32, WM_KEYUP | WM_SYSKEYUP);
        
        if is_press || is_release {
            HOOK_RECORDING.with(|recording| {
                if *recording.borrow() {
                    HOOK_TX.with(|tx_cell| {
                        if let Some(ref tx) = *tx_cell.borrow() {
                            // Calculate delay
                            HOOK_LAST_TIME.with(|last_time| {
                                let now = Instant::now();
                                let delay_ms = now.duration_since(*last_time.borrow()).as_millis() as u64;
                                
                                // Add delay if more than 10ms since last event
                                if delay_ms > 10 {
                                    let _ = tx.send(MacroAction::Delay { ms: delay_ms });
                                }
                                
                                *last_time.borrow_mut() = now;
                            });
                            
                            let key_str = vk_to_string(vk_code);
                            
                            let action = if is_press {
                                debug!("[InputRecorder] KeyPress: {}", key_str);
                                MacroAction::KeyPress { key: key_str, delay_ms: 0 }
                            } else {
                                debug!("[InputRecorder] KeyRelease: {}", key_str);
                                MacroAction::KeyRelease { key: key_str, delay_ms: 0 }
                            };
                            
                            if let Err(e) = tx.send(action) {
                                warn!("[InputRecorder] Failed to send action: {}", e);
                            }
                        }
                    });
                }
            });
        }
    }
    
    // Always pass to next hook (don't block input)
    CallNextHookEx(HHOOK::default(), code, wparam, lparam)
}

/// Input recorder that captures ONLY keyboard events in a background thread.
/// Mouse events are NOT recorded - they must be inserted manually via the Insert Event menu.
pub struct InputRecorder {
    is_recording: Arc<AtomicBool>,
    receiver: Option<Receiver<MacroAction>>,
    stop_signal: Option<Sender<()>>,
    _thread_handle: Option<JoinHandle<()>>,
}

impl InputRecorder {
    /// Create a new input recorder (not started)
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            receiver: None,
            stop_signal: None,
            _thread_handle: None,
        }
    }

    /// Start recording keyboard events only (no mouse events)
    #[cfg(target_os = "windows")]
    pub fn start_recording(&mut self) {
        if self.is_recording.load(Ordering::SeqCst) {
            info!("[InputRecorder] Already recording, ignoring start request");
            return;
        }

        info!("[InputRecorder] Starting keyboard recording (Windows hooks)...");

        let (tx, rx) = channel::<MacroAction>();
        let (stop_tx, stop_rx) = channel::<()>();
        
        self.receiver = Some(rx);
        self.stop_signal = Some(stop_tx);

        let is_recording = self.is_recording.clone();
        is_recording.store(true, Ordering::SeqCst);

        // Spawn the listener thread with Windows message pump
        let handle = thread::spawn(move || {
            info!("[InputRecorder] Listener thread started (Windows)");
            
            // Set up thread-local storage for the hook callback
            HOOK_TX.with(|cell| {
                *cell.borrow_mut() = Some(tx);
            });
            HOOK_RECORDING.with(|cell| {
                *cell.borrow_mut() = true;
            });
            HOOK_LAST_TIME.with(|cell| {
                *cell.borrow_mut() = Instant::now();
            });
            
            // Install the keyboard hook
            let hook = unsafe {
                SetWindowsHookExW(
                    WH_KEYBOARD_LL,
                    Some(keyboard_hook_proc),
                    None,
                    0,
                )
            };
            
            match hook {
                Ok(h) => {
                    info!("[InputRecorder] Keyboard hook installed successfully");
                    
                    // Run message pump - this is REQUIRED for low-level hooks to work on Windows
                    let mut msg = MSG::default();
                    loop {
                        // Check if we should stop
                        if stop_rx.try_recv().is_ok() {
                            info!("[InputRecorder] Stop signal received");
                            break;
                        }
                        
                        // Process messages with a timeout (non-blocking peek)
                        unsafe {
                            // Use GetMessage which blocks, but we check stop_rx periodically
                            // Actually, use PeekMessage to avoid blocking indefinitely
                            use windows::Win32::UI::WindowsAndMessaging::{PeekMessageW, PM_REMOVE};
                            
                            if PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                                TranslateMessage(&msg);
                                DispatchMessageW(&msg);
                            } else {
                                // No message, sleep a bit to avoid busy loop
                                std::thread::sleep(std::time::Duration::from_millis(10));
                            }
                        }
                    }
                    
                    // Unhook
                    unsafe {
                        let _ = UnhookWindowsHookEx(h);
                    }
                    info!("[InputRecorder] Keyboard hook removed");
                }
                Err(e) => {
                    error!("[InputRecorder] Failed to install keyboard hook: {:?}", e);
                }
            }
            
            // Clean up thread-local storage
            HOOK_RECORDING.with(|cell| {
                *cell.borrow_mut() = false;
            });
            HOOK_TX.with(|cell| {
                *cell.borrow_mut() = None;
            });
            
            info!("[InputRecorder] Listener thread ending");
        });

        self._thread_handle = Some(handle);
        info!("[InputRecorder] Recording started, listener thread spawned");
    }

    /// Start recording (non-Windows stub)
    #[cfg(not(target_os = "windows"))]
    pub fn start_recording(&mut self) {
        warn!("[InputRecorder] Keyboard recording not supported on this platform");
    }

    /// Stop recording and return collected actions
    pub fn stop_recording(&mut self) -> Vec<MacroAction> {
        info!("[InputRecorder] Stopping recording...");
        self.is_recording.store(false, Ordering::SeqCst);
        
        // Signal the thread to stop
        if let Some(ref stop_tx) = self.stop_signal {
            let _ = stop_tx.send(());
        }
        
        // Give the thread a moment to process remaining events
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        let mut actions = Vec::new();
        if let Some(ref rx) = self.receiver {
            // Drain all pending events
            while let Ok(action) = rx.try_recv() {
                actions.push(action);
            }
        }

        info!("[InputRecorder] Collected {} actions", actions.len());

        // Clean up
        self.receiver = None;
        self.stop_signal = None;
        
        // Optimize: remove very small delays
        Self::optimize_actions(actions)
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    /// Poll for new recorded actions (non-blocking)
    pub fn poll_actions(&self) -> Vec<MacroAction> {
        let mut actions = Vec::new();
        if let Some(ref rx) = self.receiver {
            while let Ok(action) = rx.try_recv() {
                actions.push(action);
            }
        }
        actions
    }

    /// Optimize recorded actions by merging consecutive small delays
    fn optimize_actions(actions: Vec<MacroAction>) -> Vec<MacroAction> {
        let mut optimized = Vec::new();
        let mut pending_delay: u64 = 0;

        for action in actions {
            match action {
                MacroAction::Delay { ms } => {
                    // Accumulate delays
                    pending_delay += ms;
                }
                other => {
                    // Flush pending delay if > 50ms
                    if pending_delay > 50 {
                        optimized.push(MacroAction::Delay { ms: pending_delay });
                    }
                    pending_delay = 0;
                    optimized.push(other);
                }
            }
        }

        // Flush any remaining delay
        if pending_delay > 50 {
            optimized.push(MacroAction::Delay { ms: pending_delay });
        }

        optimized
    }
}

impl Default for InputRecorder {
    fn default() -> Self {
        Self::new()
    }
}
