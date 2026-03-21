//! EdgeOptimizer.Runner - System Tray Orchestrator
//!
//! Responsibilities:
//! - Own tray icon/menu and user interaction loop
//! - Forward optimization/cleanup intents from Settings to Engine service
//! - Relay engine state/results back to Settings
//! - Keep Settings process optional/on-demand

#![windows_subsystem = "windows"]

use anyhow::{Context, Result};
use edge_optimizer_core::{
    config,
    crosshair_overlay::{self, OverlayHandle},
    engine_ipc::EnginePipeClient,
    ipc::{GuiToTray, NamedPipeServer, TrayToGui},
    orchestration::{
        AuthContext, CleanupKind, EngineState, EngineToRunnerEvent, Envelope, IdempotencyCache,
        OperationResult, RunnerToEngineCommand, RunnerToSettingsEvent, SettingsToRunnerCommand,
    },
    profile::Profile,
    tray_icon::TrayIconManager,
};
use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};
use tray_icon::menu::MenuEvent;
use tray_icon::{MouseButton, MouseButtonState, TrayIconEvent};
use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::*;

struct SingleInstanceGuard {
    lock_path: PathBuf,
}

impl SingleInstanceGuard {
    fn acquire() -> Result<Self> {
        let lock_path = std::env::temp_dir().join("EdgeOptimizer.Runner.lock");
        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&lock_path)
        {
            Ok(_file) => Ok(Self { lock_path }),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                anyhow::bail!("EdgeOptimizer.Runner is already running")
            }
            Err(e) => Err(anyhow::anyhow!(
                "failed to create runner lock file {:?}: {}",
                lock_path,
                e
            )),
        }
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.lock_path);
    }
}

enum UiEvent {
    Tray(TrayIconEvent),
    Menu(MenuEvent),
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("EdgeOptimizer.Runner starting...");

    let _instance_guard = match SingleInstanceGuard::acquire() {
        Ok(guard) => guard,
        Err(e) => {
            tracing::warn!("{}", e);
            return Ok(());
        }
    };

    let app_config = config::load_config();
    let mut tray = TrayIconManager::new(app_config.active_profile.clone())
        .context("failed to create tray icon manager")?;

    let pipe_server = NamedPipeServer::new().context("failed to create named pipe server")?;

    let (ui_tx, ui_rx) = mpsc::channel::<UiEvent>();
    let tray_tx = ui_tx.clone();
    let menu_tx = ui_tx;

    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = tray_tx.send(UiEvent::Tray(event));
    }));

    MenuEvent::set_event_handler(Some(move |event| {
        let _ = menu_tx.send(UiEvent::Menu(event));
    }));

    let mut settings_connected = false;
    let mut engine_state = EngineState::Starting;
    let mut idempotency = IdempotencyCache::default();
    let mut overlay_handle: Option<OverlayHandle> = None;
    let mut should_exit = false;

    let mut last_click_time: Option<Instant> = None;
    let mut pending_single_click = false;
    let mut last_engine_probe = Instant::now() - Duration::from_secs(30);

    unsafe {
        let mut msg = MSG::default();
        while !should_exit {
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    should_exit = true;
                    break;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let timeout = if pending_single_click {
                Duration::from_millis(20)
            } else {
                Duration::from_millis(200)
            };

            match ui_rx.recv_timeout(timeout) {
                Ok(UiEvent::Tray(event)) => handle_tray_event(
                    event,
                    &pipe_server,
                    &mut settings_connected,
                    &mut pending_single_click,
                    &mut last_click_time,
                ),
                Ok(UiEvent::Menu(event)) => {
                    should_exit |=
                        handle_menu_event(event, &mut tray, &pipe_server, &mut settings_connected)?;
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => should_exit = true,
            }

            if pending_single_click {
                if let Some(last_time) = last_click_time {
                    if Instant::now().duration_since(last_time) >= Duration::from_millis(500) {
                        pending_single_click = false;
                        if settings_connected {
                            if let Err(e) = pipe_server.send(&TrayToGui::ShowFlyout) {
                                tracing::warn!("failed to send ShowFlyout via IPC: {}", e);
                                settings_connected = false;
                                let _ = spawn_settings_window(Some("--flyout-only"));
                            }
                        } else {
                            let _ = spawn_settings_window(Some("--flyout-only"));
                        }
                    }
                }
            }

            match pipe_server.try_recv() {
                Ok(Some(msg)) => {
                    if !settings_connected {
                        settings_connected = true;
                        send_runner_event(
                            &pipe_server,
                            &mut settings_connected,
                            RunnerToSettingsEvent::EngineState(engine_state.clone()),
                        );
                    }
                    should_exit |= handle_settings_message(
                        msg,
                        &pipe_server,
                        &mut settings_connected,
                        &mut engine_state,
                        &mut idempotency,
                        &mut overlay_handle,
                        &mut tray,
                    )?;
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!("error reading settings IPC: {}", e);
                    settings_connected = false;
                }
            }

            if last_engine_probe.elapsed() >= Duration::from_secs(5) {
                let probe_req = edge_optimizer_core::orchestration::next_request_id("runner-ping");
                let next_state = match call_engine(&probe_req, RunnerToEngineCommand::Ping) {
                    Ok(EngineToRunnerEvent::Pong) => EngineState::Ready,
                    Ok(EngineToRunnerEvent::Error { .. }) => EngineState::Degraded,
                    Ok(_) => EngineState::Ready,
                    Err(_) => EngineState::Disconnected,
                };
                set_engine_state(
                    &pipe_server,
                    &mut settings_connected,
                    &mut engine_state,
                    next_state,
                );
                last_engine_probe = Instant::now();
            }
        }
    }

    if let Some(handle) = overlay_handle.take() {
        handle.stop();
    }

    tracing::info!("EdgeOptimizer.Runner shutdown complete");
    Ok(())
}

fn handle_tray_event(
    event: TrayIconEvent,
    pipe_server: &NamedPipeServer,
    settings_connected: &mut bool,
    pending_single_click: &mut bool,
    last_click_time: &mut Option<Instant>,
) {
    if let TrayIconEvent::Click {
        button,
        button_state,
        ..
    } = event
    {
        if button == MouseButton::Left && button_state == MouseButtonState::Up {
            let now = Instant::now();
            let is_double_click = last_click_time
                .map(|last| now.duration_since(last) < Duration::from_millis(500))
                .unwrap_or(false);

            if is_double_click {
                *pending_single_click = false;
                *last_click_time = None;
                if *settings_connected {
                    if let Err(e) = pipe_server.send(&TrayToGui::BringMainToFront) {
                        tracing::warn!("failed to send BringMainToFront via IPC: {}", e);
                        *settings_connected = false;
                        let _ = spawn_settings_window(None);
                    }
                } else {
                    let _ = spawn_settings_window(None);
                }
            } else {
                *last_click_time = Some(now);
                *pending_single_click = true;
            }
        }
    }
}

fn handle_menu_event(
    event: MenuEvent,
    tray: &mut TrayIconManager,
    pipe_server: &NamedPipeServer,
    settings_connected: &mut bool,
) -> Result<bool> {
    if event.id == tray.menu_item_settings {
        if *settings_connected {
            if let Err(e) = pipe_server.send(&TrayToGui::BringMainToFront) {
                tracing::warn!("failed to send BringMainToFront via IPC: {}", e);
                *settings_connected = false;
                spawn_settings_window(None)?;
            }
        } else {
            spawn_settings_window(None)?;
        }
        return Ok(false);
    }
    if event.id == tray.menu_item_docs {
        let _ = open::that("https://github.com/yourusername/EdgeOptimizer#readme");
        return Ok(false);
    }
    if event.id == tray.menu_item_bug_report {
        let _ = open::that("https://github.com/yourusername/EdgeOptimizer/issues/new");
        return Ok(false);
    }
    if event.id == tray.menu_item_exit {
        if *settings_connected {
            let _ = pipe_server.send(&TrayToGui::Exit);
        }
        return Ok(true);
    }
    Ok(false)
}

fn handle_settings_message(
    msg: GuiToTray,
    pipe_server: &NamedPipeServer,
    settings_connected: &mut bool,
    engine_state: &mut EngineState,
    idempotency: &mut IdempotencyCache,
    overlay_handle: &mut Option<OverlayHandle>,
    tray: &mut TrayIconManager,
) -> Result<bool> {
    match msg {
        GuiToTray::ProfilesUpdated(_profiles) => {}
        GuiToTray::ActiveProfileChanged(active) => {
            tray.set_active_profile(active);
        }
        GuiToTray::OverlayVisibilityChanged(_visible) => {}
        GuiToTray::Shutdown => return Ok(true),
        GuiToTray::Orchestration(env) => {
            if !idempotency.check_and_insert(&env.request_id) {
                send_runner_event(
                    pipe_server,
                    settings_connected,
                    RunnerToSettingsEvent::Ack {
                        message: format!("Duplicate request ignored: {}", env.request_id),
                    },
                );
                return Ok(false);
            }

            process_orchestration_command(
                env,
                pipe_server,
                settings_connected,
                engine_state,
                overlay_handle,
            )?;
        }
    }
    Ok(false)
}

fn process_orchestration_command(
    env: Envelope<SettingsToRunnerCommand>,
    pipe_server: &NamedPipeServer,
    settings_connected: &mut bool,
    engine_state: &mut EngineState,
    overlay_handle: &mut Option<OverlayHandle>,
) -> Result<()> {
    match env.payload {
        SettingsToRunnerCommand::ActivateProfile { profile, .. }
        | SettingsToRunnerCommand::RequestOptimization { profile, .. } => {
            set_engine_state(
                pipe_server,
                settings_connected,
                engine_state,
                EngineState::Starting,
            );
            let event = optimize_profile(env.request_id, profile, overlay_handle);
            if matches!(event, RunnerToSettingsEvent::OptimizationResult(ref r) if r.success) {
                set_engine_state(
                    pipe_server,
                    settings_connected,
                    engine_state,
                    EngineState::Ready,
                );
            } else {
                set_engine_state(
                    pipe_server,
                    settings_connected,
                    engine_state,
                    EngineState::Degraded,
                );
            }
            send_runner_event(pipe_server, settings_connected, event);
        }
        SettingsToRunnerCommand::RequestCleanup { cleanup_kind } => {
            let event = request_cleanup(env.request_id, cleanup_kind);
            if matches!(event, RunnerToSettingsEvent::CleanupResult(ref r) if r.success) {
                set_engine_state(
                    pipe_server,
                    settings_connected,
                    engine_state,
                    EngineState::Ready,
                );
            } else {
                set_engine_state(
                    pipe_server,
                    settings_connected,
                    engine_state,
                    EngineState::Degraded,
                );
            }
            send_runner_event(pipe_server, settings_connected, event);
        }
        SettingsToRunnerCommand::PreviewImpact { profile } => {
            let message = format!(
                "Preview: {} process(es) selected for optimization",
                profile.processes_to_kill.len()
            );
            send_runner_event(
                pipe_server,
                settings_connected,
                RunnerToSettingsEvent::Ack { message },
            );
        }
        SettingsToRunnerCommand::OpenFlyout => {
            if let Err(e) = pipe_server.send(&TrayToGui::ShowFlyout) {
                tracing::warn!("failed to send ShowFlyout to settings: {}", e);
                *settings_connected = false;
            }
        }
        SettingsToRunnerCommand::OpenSettings => {
            if let Err(e) = pipe_server.send(&TrayToGui::BringMainToFront) {
                tracing::warn!("failed to send BringMainToFront to settings: {}", e);
                *settings_connected = false;
            }
        }
    }
    Ok(())
}

fn optimize_profile(
    request_id: String,
    profile: Profile,
    overlay_handle: &mut Option<OverlayHandle>,
) -> RunnerToSettingsEvent {
    let command = RunnerToEngineCommand::ApplyProfile {
        profile: profile.clone(),
    };

    let mut result = match call_engine(&request_id, command) {
        Ok(EngineToRunnerEvent::Result(result)) => result,
        Ok(EngineToRunnerEvent::Error { message, .. }) => {
            return RunnerToSettingsEvent::UserActionRequired {
                reason: format!("Engine rejected optimization: {}", message),
            };
        }
        Ok(other) => {
            return RunnerToSettingsEvent::UserActionRequired {
                reason: format!("Unexpected engine response: {:?}", other),
            };
        }
        Err(e) => {
            return RunnerToSettingsEvent::UserActionRequired {
                reason: format!(
                    "Engine unavailable. Ensure EdgeOptimizer_EngineSvc is running. {}",
                    e
                ),
            };
        }
    };

    if let Some(handle) = overlay_handle.take() {
        handle.stop();
    }

    if profile.overlay_enabled {
        match &profile.crosshair_image_path {
            Some(path) => match crosshair_overlay::start_overlay(
                path.clone(),
                profile.crosshair_x_offset,
                profile.crosshair_y_offset,
            ) {
                Ok(handle) => {
                    *overlay_handle = Some(handle);
                    result.summary = format!("{} | crosshair=on", result.summary);
                }
                Err(e) => {
                    result.success = false;
                    result.summary = format!("{} | crosshair_error={}", result.summary, e);
                }
            },
            None => {
                result.summary = format!("{} | crosshair=skipped(no-image)", result.summary);
            }
        }
    } else {
        result.summary = format!("{} | crosshair=off", result.summary);
    }

    RunnerToSettingsEvent::OptimizationResult(result)
}

fn request_cleanup(request_id: String, cleanup_kind: CleanupKind) -> RunnerToSettingsEvent {
    let command = RunnerToEngineCommand::RunCleanup {
        cleanup_kind: cleanup_kind.clone(),
    };

    match call_engine(&request_id, command) {
        Ok(EngineToRunnerEvent::Result(result)) => RunnerToSettingsEvent::CleanupResult(result),
        Ok(EngineToRunnerEvent::Error { message, .. }) => {
            RunnerToSettingsEvent::CleanupResult(OperationResult::error(
                request_id,
                format!("cleanup failed ({}): {}", cleanup_kind.as_str(), message),
            ))
        }
        Ok(other) => RunnerToSettingsEvent::CleanupResult(OperationResult::error(
            request_id,
            format!("unexpected engine response: {:?}", other),
        )),
        Err(e) => RunnerToSettingsEvent::CleanupResult(OperationResult::error(
            request_id,
            format!("engine unavailable: {}", e),
        )),
    }
}

fn call_engine(request_id: &str, command: RunnerToEngineCommand) -> Result<EngineToRunnerEvent> {
    let client = EnginePipeClient::connect_default(Duration::from_secs(2))
        .context("failed to connect to engine pipe")?;

    let request = Envelope::with_request_id(
        "edge-runner",
        AuthContext::RunnerService,
        request_id.to_string(),
        command,
    );

    client
        .send(&request)
        .context("failed to send command to engine")?;
    let response: Option<Envelope<EngineToRunnerEvent>> =
        client.recv().context("failed to receive engine response")?;

    response
        .map(|env| env.payload)
        .ok_or_else(|| anyhow::anyhow!("engine disconnected without response"))
}

fn send_runner_event(
    pipe_server: &NamedPipeServer,
    settings_connected: &mut bool,
    event: RunnerToSettingsEvent,
) {
    if !*settings_connected {
        return;
    }
    let env = Envelope::new("edge-runner", AuthContext::RunnerService, event);
    if let Err(e) = pipe_server.send(&TrayToGui::OrchestrationEvent(env)) {
        tracing::warn!("failed to send runner event to settings: {}", e);
        *settings_connected = false;
    }
}

fn set_engine_state(
    pipe_server: &NamedPipeServer,
    settings_connected: &mut bool,
    current: &mut EngineState,
    next: EngineState,
) {
    if *current == next {
        return;
    }
    *current = next.clone();
    tracing::info!("Engine state -> {:?}", next);
    send_runner_event(
        pipe_server,
        settings_connected,
        RunnerToSettingsEvent::EngineState(next),
    );
}

fn spawn_settings_window(flag: Option<&str>) -> Result<()> {
    if bring_existing_settings_to_front() {
        return Ok(());
    }

    let exe_dir = std::env::current_exe()?
        .parent()
        .context("failed to get executable directory")?
        .to_path_buf();

    let settings_exe = exe_dir.join("EdgeOptimizer_Settings.exe");
    let alt_exe = exe_dir.join("edge_optimizer_settings.exe");
    let target = if settings_exe.exists() {
        settings_exe
    } else if alt_exe.exists() {
        alt_exe
    } else {
        anyhow::bail!("Settings executable not found in {:?}", exe_dir);
    };

    let mut cmd = Command::new(&target);
    if let Some(f) = flag {
        cmd.arg(f);
    }
    cmd.spawn().context("failed to spawn Settings process")?;
    Ok(())
}

fn bring_existing_settings_to_front() -> bool {
    unsafe {
        let title: Vec<u16> = "Edge Optimizer - Profile Manager\0"
            .encode_utf16()
            .collect();
        let hwnd = FindWindowW(None, PCWSTR(title.as_ptr()));

        if hwnd != HWND::default() {
            if IsIconic(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            }
            let _ = SetForegroundWindow(hwnd);
            let _ = BringWindowToTop(hwnd);
            return true;
        }
        false
    }
}
