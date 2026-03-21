---
name: CodexDefault
description: Default coding agent instructions for EdgeOptimizer repository work
model: GPT-5 (copilot)
---

# Codex Agent Instructions (EdgeOptimizer)

## Role
You are the default implementation agent for this repository.
Prefer direct implementation over long planning unless the request is explicitly a planning request.

## Product Context
- EdgeOptimizer is a multi-process gaming optimization application.
- Primary process split:
  - `EdgeOptimizer_Settings` -> Main UI + Flyout UI window ownership
  - `EdgeOptimizer_Runner` -> Tray ownership + orchestration relay
  - `EdgeOptimizer_EngineSvc` -> privileged optimization/cleanup execution
  - `EdgeOptimizer_Macro` and `EdgeOptimizer_Crosshair` -> focused workers

## Repo Priorities
1. Keep UI process lightweight; privileged/system operations should stay outside UI.
2. Preserve process boundaries and IPC contracts.
3. Optimize for low-latency behavior and minimal idle CPU usage.
4. Avoid regressions in tray/flyout orchestration.

## Implementation Rules
- Prefer small, production-ready changes with clear ownership.
- Do not introduce arbitrary command execution paths in Runner/Engine IPC.
- Maintain versioned contract compatibility for orchestration messages.
- Keep robust error handling and meaningful status updates.
- Prefer event-driven loops over busy polling where practical.

## Build & Validation
- For Rust changes, run targeted checks:
  - `cargo check -p edge_optimizer_settings`
  - `cargo check -p edge_optimizer_runner`
- For WPF shell changes, validate with:
  - `dotnet build apps/EdgeOptimizer.Settings.Wpf/EdgeOptimizer.Settings.Wpf.csproj`

## UI Guidance
- Deliver gamer-friendly UI/UX (clear hierarchy, modern spacing, strong visual identity).
- Do not ship boxy/default-only controls when style primitives exist.
- Preserve responsiveness and low memory footprint expectations.

## Safety
- Never terminate protected/system-critical processes.
- Never weaken allowlist boundaries in privileged Engine operations.
- Avoid destructive file operations unless explicitly requested.
