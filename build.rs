//! Build script to embed Windows resource metadata into executables
//! This sets the application name shown in Task Manager

fn main() {
    #[cfg(windows)]
    {
        use std::path::Path;
        use std::env;
        
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let out_dir = env::var("OUT_DIR").unwrap();
        
        // Determine which binary based on OUT_DIR path
        let is_settings = out_dir.contains("edge_optimizer_settings");
        let is_runner = out_dir.contains("edge_optimizer_runner");
        let is_crosshair = out_dir.contains("edge_optimizer_crosshair");
        
        let rc_file = if is_settings {
            "resources/settings.rc"
        } else if is_runner {
            "resources/runner.rc"
        } else if is_crosshair {
            "resources/crosshair.rc"
        } else {
            // Default/library build - use settings
            "resources/settings.rc"
        };
        
        println!("cargo:warning=Using resource file: {}", rc_file);
        
        let rc_path = Path::new(&manifest_dir).join(rc_file);
        
        if rc_path.exists() {
            let mut res = winresource::WindowsResource::new();
            res.set_resource_file(rc_path.to_str().unwrap());
            
            if let Err(e) = res.compile() {
                println!("cargo:warning=Failed to compile Windows resources: {}", e);
            }
        } else {
            println!("cargo:warning=Resource file not found: {:?}", rc_path);
        }
    }
}
