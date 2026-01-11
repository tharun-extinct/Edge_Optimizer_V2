//! Build script to embed Windows resource metadata into executables
//! This sets the application name shown in Task Manager

fn main() {
    #[cfg(windows)]
    {
        // Get the binary name being built
        let target = std::env::var("CARGO_BIN_NAME").unwrap_or_default();
        
        let mut res = winresource::WindowsResource::new();
        
        // Set common metadata
        res.set("ProductName", "Edge Optimizer");
        res.set("CompanyName", "Edge Optimizer");
        res.set("LegalCopyright", "Copyright © 2026");
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));
        
        // Set binary-specific metadata (this shows in Task Manager)
        match target.as_str() {
            "edge_optimizer_settings" => {
                res.set("FileDescription", "EdgeOptimizer.Settings");
                res.set("InternalName", "EdgeOptimizer.Settings");
                res.set("OriginalFilename", "EdgeOptimizer.Settings.exe");
            }
            "edge_optimizer_runner" => {
                res.set("FileDescription", "EdgeOptimizer.Runner");
                res.set("InternalName", "EdgeOptimizer.Runner");
                res.set("OriginalFilename", "EdgeOptimizer.Runner.exe");
            }
            "edge_optimizer_crosshair" => {
                res.set("FileDescription", "EdgeOptimizer.Crosshair");
                res.set("InternalName", "EdgeOptimizer.Crosshair");
                res.set("OriginalFilename", "EdgeOptimizer.Crosshair.exe");
            }
            _ => {
                res.set("FileDescription", "Edge Optimizer");
                res.set("InternalName", "EdgeOptimizer");
            }
        }
        
        // Compile the resource
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resources: {}", e);
        }
    }
}
