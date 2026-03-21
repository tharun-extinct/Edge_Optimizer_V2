//! EdgeOptimizer_EngineCtl - CLI control client for EngineSvc

use anyhow::{Context, Result};
use edge_optimizer_core::engine_ipc::EnginePipeClient;
use edge_optimizer_core::orchestration::{
    AuthContext, CleanupKind, EngineToRunnerEvent, Envelope, RunnerToEngineCommand,
};
use std::time::Duration;

fn main() -> Result<()> {
    tracing_subscriber::fmt().with_target(false).init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        anyhow::bail!("missing command")
    }

    let command = parse_command(&args[1..])?;
    let request_id = edge_optimizer_core::orchestration::next_request_id("engine-ctl");
    let envelope = Envelope::with_request_id(
        "engine-ctl",
        AuthContext::ScheduledTask,
        request_id,
        command,
    );

    let client = EnginePipeClient::connect_default(Duration::from_secs(5))
        .context("failed to connect to EdgeOptimizer_EngineSvc")?;

    client
        .send(&envelope)
        .context("failed to send command to engine")?;
    let response: Option<Envelope<EngineToRunnerEvent>> = client
        .recv()
        .context("failed to receive response from engine")?;

    let Some(response) = response else {
        anyhow::bail!("engine disconnected before returning a response");
    };

    match response.payload {
        EngineToRunnerEvent::Result(result) => {
            if result.success {
                println!("SUCCESS: {}", result.summary);
                Ok(())
            } else {
                anyhow::bail!("FAILED: {}", result.summary)
            }
        }
        EngineToRunnerEvent::Ack { message } => {
            println!("ACK: {}", message);
            Ok(())
        }
        EngineToRunnerEvent::Capabilities {
            cleanup_kinds,
            supports_process_kill,
        } => {
            let labels: Vec<&str> = cleanup_kinds.iter().map(CleanupKind::as_str).collect();
            println!(
                "CAPABILITIES: cleanup=[{}], process_kill={}",
                labels.join(","),
                supports_process_kill
            );
            Ok(())
        }
        EngineToRunnerEvent::Pong => {
            println!("PONG");
            Ok(())
        }
        EngineToRunnerEvent::Error { message, .. } => anyhow::bail!("ERROR: {}", message),
        EngineToRunnerEvent::Progress { message } => {
            println!("PROGRESS: {}", message);
            Ok(())
        }
    }
}

fn parse_command(args: &[String]) -> Result<RunnerToEngineCommand> {
    match args[0].as_str() {
        "ping" => Ok(RunnerToEngineCommand::Ping),
        "capabilities" => Ok(RunnerToEngineCommand::GetCapabilities),
        "cleanup" => {
            let kind = args.get(1).ok_or_else(|| {
                anyhow::anyhow!("cleanup requires a kind: recycle-bin|browser-cache")
            })?;
            let cleanup_kind = match kind.as_str() {
                "recycle-bin" => CleanupKind::RecycleBin,
                "browser-cache" => CleanupKind::BrowserCache,
                _ => anyhow::bail!("unsupported cleanup kind: {}", kind),
            };
            Ok(RunnerToEngineCommand::RunCleanup { cleanup_kind })
        }
        "kill" => {
            if args.len() < 2 {
                anyhow::bail!("kill requires at least one process name")
            }
            Ok(RunnerToEngineCommand::KillProcesses {
                processes: args[1..].to_vec(),
            })
        }
        _ => {
            print_usage();
            anyhow::bail!("unknown command: {}", args[0])
        }
    }
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  EdgeOptimizer_EngineCtl ping");
    eprintln!("  EdgeOptimizer_EngineCtl capabilities");
    eprintln!("  EdgeOptimizer_EngineCtl cleanup recycle-bin");
    eprintln!("  EdgeOptimizer_EngineCtl cleanup browser-cache");
    eprintln!("  EdgeOptimizer_EngineCtl kill <process1.exe> <process2.exe>");
}
