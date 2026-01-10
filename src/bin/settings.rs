//! EdgeOptimizer.Settings - Main GUI Process
//! 
//! This process manages:
//! - Main settings window (full GUI application)
//! - Flyout window (quick access flyout)
//! - IPC communication with Runner process
//! - Profile management and system optimization

// #![windows_subsystem = "windows"]  // Temporarily disabled for debugging

// Import modules from the workspace
use gaming_optimizer::gui;

fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    tracing::info!("EdgeOptimizer.Settings starting...");
    
    // Run the GUI application
    gui::run()?;
    
    Ok(())
}
