use sysinfo::System;

/// Information about a running process
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    #[allow(dead_code)]
    pub pid: u32,
    pub name: String,
    pub memory_kb: u64,
    pub cpu_percent: f32,
}

/// Report of process killing operation
#[derive(Debug, Clone)]
pub struct KillReport {
    pub killed: Vec<String>,
    pub failed: Vec<String>,
    pub not_found: Vec<String>,
    pub blocklist_skipped: Vec<String>,
}

impl KillReport {
    fn new() -> Self {
        KillReport {
            killed: Vec::new(),
            failed: Vec::new(),
            not_found: Vec::new(),
            blocklist_skipped: Vec::new(),
        }
    }
}

/// Critical Windows processes that cannot be killed
/// Killing these could crash the system or cause serious instability
const PROTECTED_PROCESSES: &[&str] = &[
    "csrss.exe",    // Client Server Runtime
    "dwm.exe",      // Desktop Window Manager
    "explorer.exe", // Windows Explorer (shell)
    "lsass.exe",    // Local Security Authority
    "services.exe", // Services Control Manager
    "smss.exe",     // Session Manager
    "system",       // System process
    "wininit.exe",  // Windows Init
    "winlogon.exe", // Windows Logon
    "svchost.exe",  // Service Host (critical services)
];

/// Check if a process name is in the protected list (case-insensitive)
fn is_protected(process_name: &str) -> bool {
    let name_lower = process_name.to_lowercase();
    PROTECTED_PROCESSES
        .iter()
        .any(|protected| protected.to_lowercase() == name_lower)
}

/// Normalize process name for matching (case-insensitive, strips .exe if present)
fn normalize_process_name(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.ends_with(".exe") {
        lower[..lower.len() - 4].to_string()
    } else {
        lower
    }
}

/// List all running processes
pub fn list_processes() -> Vec<ProcessInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut processes = Vec::new();

    for (pid, process) in sys.processes() {
        processes.push(ProcessInfo {
            pid: pid.as_u32(),
            name: process.name().to_string(),
            memory_kb: process.memory() / 1024,
            cpu_percent: process.cpu_usage(),
        });
    }

    // Sort by name for easier viewing
    processes.sort_by(|a, b| a.name.cmp(&b.name));

    processes
}

/// Kill processes by name
/// Returns a detailed report of what happened
pub fn kill_processes(process_names: &[String]) -> KillReport {
    let mut report = KillReport::new();
    let mut sys = System::new_all();
    sys.refresh_all();

    for target_name in process_names {
        let target_normalized = normalize_process_name(target_name);

        // Check if process is protected
        if is_protected(&target_normalized) || is_protected(target_name) {
            report.blocklist_skipped.push(target_name.clone());
            continue;
        }

        // Find all processes matching this name
        let mut found_any = false;
        let mut killed_any = false;
        let mut failed_any = false;

        for (_pid, process) in sys.processes() {
            let process_name = process.name();
            let process_normalized = normalize_process_name(process_name);

            // Match either with or without .exe extension
            if process_normalized == target_normalized
                || process_name.to_lowercase() == target_name.to_lowercase()
            {
                found_any = true;

                // Attempt to kill the process
                if process.kill() {
                    killed_any = true;
                } else {
                    failed_any = true;
                }
            }
        }

        // Record result for this process name
        if killed_any && !failed_any {
            report.killed.push(target_name.clone());
        } else if killed_any && failed_any {
            // Some instances killed, some failed
            report.killed.push(format!("{} (partial)", target_name));
            report.failed.push(format!("{} (partial)", target_name));
        } else if failed_any {
            report.failed.push(target_name.clone());
        } else if !found_any {
            report.not_found.push(target_name.clone());
        }
    }

    // Refresh system info after killing
    sys.refresh_all();

    report
}

/// Check if a process name would be blocked by the safety blocklist
#[allow(dead_code)]
pub fn would_be_protected(process_name: &str) -> bool {
    is_protected(process_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_process_name() {
        assert_eq!(normalize_process_name("notepad.exe"), "notepad");
        assert_eq!(normalize_process_name("Notepad.exe"), "notepad");
        assert_eq!(normalize_process_name("NOTEPAD.EXE"), "notepad");
        assert_eq!(normalize_process_name("notepad"), "notepad");
    }

    #[test]
    fn test_is_protected() {
        assert!(is_protected("csrss.exe"));
        assert!(is_protected("CSRSS.EXE"));
        assert!(is_protected("explorer.exe"));
        assert!(is_protected("Explorer.exe"));
        assert!(!is_protected("notepad.exe"));
        assert!(!is_protected("chrome.exe"));
    }

    #[test]
    fn test_would_be_protected() {
        assert!(would_be_protected("dwm.exe"));
        assert!(would_be_protected("DWM.exe"));
        assert!(!would_be_protected("discord.exe"));
    }

    #[test]
    fn test_list_processes() {
        let processes = list_processes();
        // Should return at least some processes on any system
        assert!(!processes.is_empty());
    }

    #[test]
    fn test_kill_report_new() {
        let report = KillReport::new();
        assert!(report.killed.is_empty());
        assert!(report.failed.is_empty());
        assert!(report.not_found.is_empty());
        assert!(report.blocklist_skipped.is_empty());
    }
}
