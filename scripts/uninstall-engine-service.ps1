param(
    [string]$TaskName = "EdgeOptimizerEngineSvcHost"
)

Stop-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue
Write-Host "Task '$TaskName' removed."
