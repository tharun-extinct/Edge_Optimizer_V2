param(
    [string]$TaskName = "EdgeOptimizerEngineSvcHost",
    [string]$BinaryPath = "$PSScriptRoot\..\target\release\EdgeOptimizer_EngineSvc.exe"
)

$resolved = Resolve-Path $BinaryPath -ErrorAction Stop
$action = New-ScheduledTaskAction -Execute $resolved.Path -Argument "--console"
$trigger = New-ScheduledTaskTrigger -AtStartup
$principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest

Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger -Principal $principal -Force
Start-ScheduledTask -TaskName $TaskName
Write-Host "Task '$TaskName' registered and started (service-host mode)."
