//! Build script for EdgeOptimizer.Crosshair
//! Embeds Windows resource metadata

fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set("FileDescription", "EdgeOptimizer.Crosshair");
        res.set("ProductName", "Edge Optimizer");
        res.set("InternalName", "EdgeOptimizer.Crosshair");
        res.set("OriginalFilename", "EdgeOptimizer.Crosshair.exe");
        res.set("CompanyName", "Edge Optimizer");
        res.set("LegalCopyright", "Copyright © 2026");
        
        if let Err(e) = res.compile() {
            println!("cargo:warning=Failed to compile Windows resources: {}", e);
        }
    }
}
