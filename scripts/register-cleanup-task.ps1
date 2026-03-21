param(
    [string]$TaskName = "EdgeOptimizerCleanup",
    [ValidateSet("recycle-bin", "browser-cache")]
    [string]$CleanupKind = "browser-cache",
    [string]$At = "03:00",
    [string]$EngineCtlPath = "$PSScriptRoot\..\target\release\EdgeOptimizer_EngineCtl.exe"
)

$resolvedPath = Resolve-Path $EngineCtlPath -ErrorAction Stop
$action = New-ScheduledTaskAction -Execute $resolvedPath.Path -Argument "cleanup $CleanupKind"
$trigger = New-ScheduledTaskTrigger -Daily -At $At
$principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest

Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger -Principal $principal -Force
Write-Host "Scheduled task '$TaskName' registered for cleanup kind '$CleanupKind' at $At"