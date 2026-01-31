//! Input Listener
//!
//! High-level API for listening to keyboard and mouse events.

use crate::input_hooks::{
    install_keyboard_hook, install_mouse_hook, is_keyboard_hook_installed,
    is_mouse_hook_installed, uninstall_keyboard_hook, uninstall_mouse_hook,
};
use crate::types::{KeyboardData, MouseData};
use crossbeam_channel::{bounded, Receiver};
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tracing::{debug, error, info};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, TranslateMessage, MSG,
};

/// Configuration for InputListener
#[derive(Debug, Clone)]
pub struct ListenerConfig {
    /// Enable keyboard hook
    pub keyboard: bool,
    /// Enable mouse hook
    pub mouse: bool,
    /// Channel buffer size
    pub buffer_size: usize,
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self {
            keyboard: true,
            mouse: false,
            buffer_size: 256,
        }
    }
}

impl ListenerConfig {
    /// Create config for keyboard only
    pub fn keyboard_only() -> Self {
        Self {
            keyboard: true,
            mouse: false,
            buffer_size: 256,
        }
    }

    /// Create config for mouse only
    pub fn mouse_only() -> Self {
        Self {
            keyboard: false,
            mouse: true,
            buffer_size: 256,
        }
    }

    /// Create config for both keyboard and mouse
    pub fn all() -> Self {
        Self {
            keyboard: true,
            mouse: true,
            buffer_size: 256,
        }
    }
}

/// Input event listener with high-level API
pub struct InputListener {
    /// Keyboard event receiver
    pub keyboard_rx: Option<Receiver<KeyboardData>>,
    /// Mouse event receiver
    pub mouse_rx: Option<Receiver<MouseData>>,
    /// Message loop thread handle
    message_thread: Option<JoinHandle<()>>,
    /// Flag to signal shutdown
    running: Arc<Mutex<bool>>,
}

impl InputListener {
    /// Create a new input listener with the given configuration
    pub fn new(config: ListenerConfig) -> Result<Self, String> {
        let mut listener = InputListener {
            keyboard_rx: None,
            mouse_rx: None,
            message_thread: None,
            running: Arc::new(Mutex::new(false)),
        };

        // Create channels and install hooks
        if config.keyboard {
            let (tx, rx) = bounded(config.buffer_size);
            install_keyboard_hook(tx)?;
            listener.keyboard_rx = Some(rx);
        }

        if config.mouse {
            let (tx, rx) = bounded(config.buffer_size);
            install_mouse_hook(tx)?;
            listener.mouse_rx = Some(rx);
        }

        // Start the message loop thread
        *listener.running.lock() = true;
        let running = listener.running.clone();
        
        listener.message_thread = Some(thread::spawn(move || {
            info!("Input listener message loop started");
            run_message_loop(running);
            info!("Input listener message loop ended");
        }));

        Ok(listener)
    }

    /// Create a keyboard-only listener
    pub fn keyboard() -> Result<Self, String> {
        Self::new(ListenerConfig::keyboard_only())
    }

    /// Create a mouse-only listener
    pub fn mouse() -> Result<Self, String> {
        Self::new(ListenerConfig::mouse_only())
    }

    /// Create a listener for both keyboard and mouse
    pub fn all() -> Result<Self, String> {
        Self::new(ListenerConfig::all())
    }

    /// Stop listening and clean up
    pub fn stop(&mut self) {
        debug!("Stopping input listener");
        *self.running.lock() = false;

        // Uninstall hooks
        if is_keyboard_hook_installed() {
            uninstall_keyboard_hook();
        }
        if is_mouse_hook_installed() {
            uninstall_mouse_hook();
        }

        // Wait for message thread to finish
        if let Some(handle) = self.message_thread.take() {
            let _ = handle.join();
        }
    }

    /// Check if listener is active
    pub fn is_running(&self) -> bool {
        *self.running.lock()
    }
}

impl Drop for InputListener {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Run the Windows message loop (required for low-level hooks)
fn run_message_loop(running: Arc<Mutex<bool>>) {
    let mut msg = MSG::default();

    loop {
        // Check if we should stop
        if !*running.lock() {
            break;
        }

        // Process messages with timeout
        unsafe {
            let result = GetMessageW(&mut msg, None, 0, 0);

            match result.0 {
                -1 => {
                    error!("GetMessage error");
                    break;
                }
                0 => {
                    // WM_QUIT received
                    debug!("WM_QUIT received");
                    break;
                }
                _ => {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
    }
}
