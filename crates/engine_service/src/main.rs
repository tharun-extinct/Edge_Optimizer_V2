//! EdgeOptimizer.EngineSvc - privileged optimization worker

// #![windows_subsystem = "windows"]

use anyhow::{Context, Result};
use edge_optimizer_core::engine_ipc::EnginePipeServer;
use edge_optimizer_core::orchestration::{
    AuthContext, CleanupKind, EngineToRunnerEvent, Envelope, IdempotencyCache, OperationResult,
    RunnerToEngineCommand,
};
use edge_optimizer_core::process::kill_processes;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    tracing::info!("EdgeOptimizer.EngineSvc starting");

    let server = EnginePipeServer::new_default().context("failed to create engine pipe server")?;
    let mut idempotency = IdempotencyCache::default();

    loop {
        server
            .wait_for_client()
            .context("failed waiting for runner connection")?;
        tracing::info!("Runner connected to engine pipe");

        loop {
            let request: Option<Envelope<RunnerToEngineCommand>> =
                server.recv().context("failed receiving engine request")?;

            let Some(request) = request else {
                tracing::info!("Runner disconnected from engine pipe");
                server.disconnect();
                break;
            };

            let response_payload = if !idempotency.check_and_insert(&request.request_id) {
                EngineToRunnerEvent::Ack {
                    message: format!("Duplicate request ignored: {}", request.request_id),
                }
            } else {
                handle_command(&request)
            };

            let response =
                request.respond("edge-engine", AuthContext::RunnerService, response_payload);
            if let Err(e) = server.send(&response) {
                tracing::warn!("failed to send engine response: {}", e);
                server.disconnect();
                break;
            }
        }
    }
}

fn handle_command(request: &Envelope<RunnerToEngineCommand>) -> EngineToRunnerEvent {
    match &request.payload {
        RunnerToEngineCommand::Ping => EngineToRunnerEvent::Pong,
        RunnerToEngineCommand::GetCapabilities => EngineToRunnerEvent::Capabilities {
            cleanup_kinds: vec![CleanupKind::RecycleBin, CleanupKind::BrowserCache],
            supports_process_kill: true,
        },
        RunnerToEngineCommand::KillProcesses { processes } => {
            let report = kill_processes(processes);
            EngineToRunnerEvent::Result(OperationResult::from_kill_report(
                request.request_id.clone(),
                report,
            ))
        }
        RunnerToEngineCommand::ApplyProfile { profile } => {
            let report = kill_processes(&profile.processes_to_kill);
            let mut result = OperationResult::from_kill_report(request.request_id.clone(), report);
            result.summary = format!("profile={} {}", profile.name, result.summary);
            EngineToRunnerEvent::Result(result)
        }
        RunnerToEngineCommand::RunCleanup { cleanup_kind } => {
            let result = run_cleanup(request.request_id.clone(), cleanup_kind.clone());
            EngineToRunnerEvent::Result(result)
        }
    }
}

fn run_cleanup(request_id: String, cleanup_kind: CleanupKind) -> OperationResult {
    match cleanup_kind {
        CleanupKind::RecycleBin => match clear_recycle_bin() {
            Ok(summary) => OperationResult {
                request_id,
                success: true,
                summary,
                ..OperationResult::default()
            },
            Err(e) => {
                OperationResult::error(request_id, format!("recycle-bin cleanup failed: {}", e))
            }
        },
        CleanupKind::BrowserCache => match clear_browser_cache() {
            Ok(summary) => OperationResult {
                request_id,
                success: true,
                summary,
                ..OperationResult::default()
            },
            Err(e) => {
                OperationResult::error(request_id, format!("browser-cache cleanup failed: {}", e))
            }
        },
    }
}

fn clear_recycle_bin() -> Result<String> {
    let status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Clear-RecycleBin -Force -ErrorAction Stop",
        ])
        .status()
        .context("failed to invoke Clear-RecycleBin")?;

    if status.success() {
        Ok("Recycle Bin cleaned".to_string())
    } else {
        anyhow::bail!("Clear-RecycleBin exited with status {}", status)
    }
}

fn clear_browser_cache() -> Result<String> {
    let local_app_data = env::var_os("LOCALAPPDATA")
        .ok_or_else(|| anyhow::anyhow!("LOCALAPPDATA is unavailable"))?;
    let base = PathBuf::from(local_app_data);

    let cache_paths = [
        base.join("Google")
            .join("Chrome")
            .join("User Data")
            .join("Default")
            .join("Cache"),
        base.join("Google")
            .join("Chrome")
            .join("User Data")
            .join("Default")
            .join("Code Cache"),
        base.join("Microsoft")
            .join("Edge")
            .join("User Data")
            .join("Default")
            .join("Cache"),
        base.join("Microsoft")
            .join("Edge")
            .join("User Data")
            .join("Default")
            .join("Code Cache"),
    ];

    let mut cleaned = 0usize;
    for path in cache_paths {
        if path.exists() {
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).with_context(|| format!("failed to recreate {:?}", path))?;
            cleaned += 1;
        }
    }

    Ok(format!(
        "Browser cache cleanup complete ({} path(s) cleaned)",
        cleaned
    ))
}
