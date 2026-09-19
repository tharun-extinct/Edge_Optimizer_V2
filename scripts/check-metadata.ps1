# Post-build script to set correct Windows resource metadata for each binary
# Run this after 'cargo build --release'

param(
    [string]$TargetDir = ".\target\release"
)

# Define the metadata for each binary
$binaries = @{
    "EdgeOptimizer.Settings.WinUI.exe" = @{
        FileDescription = "EdgeOptimizer.Settings.WinUI"
        ProductName = "Edge Optimizer"
        InternalName = "EdgeOptimizer.Settings.WinUI"
    }
    "edge_optimizer_runner.exe" = @{
        FileDescription = "EdgeOptimizer.Runner"
        ProductName = "Edge Optimizer"
        InternalName = "EdgeOptimizer.Runner"
    }
    "edge_optimizer_crosshair.exe" = @{
        FileDescription = "EdgeOptimizer.Crosshair"
        ProductName = "Edge Optimizer"
        InternalName = "EdgeOptimizer.Crosshair"
    }
    "EdgeOptimizer_Macro.exe" = @{
        FileDescription = "EdgeOptimizer.Macro"
        ProductName = "Edge Optimizer"
        InternalName = "EdgeOptimizer.Macro"
    }
}

Write-Host "Checking binary metadata..." -ForegroundColor Cyan

foreach ($binary in $binaries.Keys) {
    $path = Join-Path $TargetDir $binary
    if (Test-Path $path) {
        $info = (Get-Item $path).VersionInfo
        Write-Host "`n$binary :" -ForegroundColor Yellow
        Write-Host "  FileDescription: $($info.FileDescription)"
        Write-Host "  InternalName: $($info.InternalName)"
        Write-Host "  ProductName: $($info.ProductName)"
        
        $expected = $binaries[$binary]
        if ($info.FileDescription -eq $expected.FileDescription) {
            Write-Host "  ✓ Metadata correct" -ForegroundColor Green
        } else {
            Write-Host "  ✗ Expected: $($expected.FileDescription)" -ForegroundColor Red
        }
    } else {
        Write-Host "`n$binary : NOT FOUND" -ForegroundColor Red
    }
}
