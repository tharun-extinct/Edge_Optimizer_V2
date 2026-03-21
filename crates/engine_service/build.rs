//! Build script for EdgeOptimizer.EngineSvc

fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set("FileDescription", "EdgeOptimizer.EngineSvc");
        res.set("ProductName", "Edge Optimizer");
        res.set("InternalName", "EdgeOptimizer.EngineSvc");
        res.set("OriginalFilename", "EdgeOptimizer_EngineSvc.exe");
        res.set("CompanyName", "Edge Optimizer");
        let _ = res.compile();
    }
}
