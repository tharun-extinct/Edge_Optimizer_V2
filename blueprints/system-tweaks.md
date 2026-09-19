# System Tweaks

## Outcome

A player can configure profile-scoped optimization options, understand their safety and privilege requirements, preview the activation plan, and receive structured results when Runner applies explicitly authorized operations.

## Current verified status

**Status:** Partial

Code inspection on 2026-09-05 confirms Rust profile storage for selected process names and a fan-speed flag, process-name normalization and protected-name tests, Runner-to-Engine command routing with fake-operation tests, and transitional Recycle Bin and browser-cache commands. The WinUI preview provides profile-scoped process selection, filtering, fan and cleanup toggles, selection totals, and restore-default behavior with unit tests.

WinUI saves supported profile fields through Runner and requests a live read-only process snapshot. The compatibility client can encode explicit cleanup intents, but the cleanup controls remain disabled because the transitional EngineSvc would execute them in its SYSTEM environment; the target architecture requires user-specific cleanup in the verified interactive-user context. Recycle Bin and browser-cache selection toggles still do not exist in the Rust profile contract. `fan_speed_max` is stored but is not applied by Engine command dispatch. PID-level safety validation is not implemented.

## Architecture dependencies

- [Component boundaries](../architecture.md#component-boundaries)
- [State ownership and persistence](../architecture.md#state-ownership-and-persistence)
- [IPC and protocol boundary](../architecture.md#ipc-and-protocol-boundary)
- [Privilege and identity](../architecture.md#privilege-and-identity)
- [Process safety](../architecture.md#process-safety)
- [Failure and recovery](../architecture.md#failure-and-recovery)

## Feature-specific implications

### Component boundaries

Settings presents choices and validation feedback but does not enumerate or terminate processes, clear user data, change fan policy, or open durable state. Runner builds the activation plan, executes user-context work, and delegates only allowlisted machine-level operations to the broker.

### State ownership and persistence

Profile-scoped selections are durable only through Runner. Restoring a profile repopulates the UI but must not terminate processes, clean data, or apply fan policy until the user issues a new explicit activation command.

### IPC and protocol boundary

Process snapshots, activation previews, save commands, explicit cleanup intents, and structured per-operation results cross the WinUI/Runner contract. Runner/broker requests remain bounded, versioned, correlated, and independently authorized.

### Privilege and identity

Each tweak must be classified as presentation-only, interactive-user, or machine-level. Recycle Bin and browser-cache cleanup are user-specific and must not derive paths from a SYSTEM profile. The broker receives only the minimal machine-level allowlist.

### Process safety

UI filtering is advisory. Runner and the final execution boundary must normalize targets and reject ambiguous, protected, critical, or changed process identities immediately before termination.

### Failure and recovery

Partial results remain visible by operation. Failure of cleanup, process termination, or fan policy must not be reported as full activation success, silently retried with broader privilege, or corrupt the last valid profile state.

## Related blueprints

### Required

- [Profile persistence](profile-persistence.md) — owns durable profile selections and startup restoration semantics.
- [IPC contracts](ipc-contracts.md) — carries process snapshots, activation plans, commands, and results.
- [Process safety](process-safety.md) — defines normalization and final process validation.
- [Privileged broker](privileged-broker.md) — owns allowlisted machine-level execution and verified identity.

### Impact checks

- [Settings client](settings-client.md) — owns WinUI presentation, accessibility, navigation, and unavailable-service states.

## Relevant implementation and tests

- `crates/core/src/profile.rs` — selected processes and transitional fan-speed profile flag.
- `crates/core/src/process.rs` — process discovery, normalization, protected-name policy, termination, and safe unit tests.
- `crates/core/src/orchestration.rs` — cleanup and activation messages plus structured operation results.
- `crates/core/src/engine_commands.rs` — injectable Engine command routing and fake-operation tests.
- `crates/runner/src/main.rs` — activation, cleanup routing, persistence, and Engine state transitions.
- `crates/engine_service/src/main.rs` — transitional process and cleanup execution.
- `apps/EdgeOptimizer.Settings.Core/ViewModels/SystemTweaksViewModel.cs` — profile selections, live process presentation, and cleanup intent logic.
- `tests/EdgeOptimizer.Settings.Core.Tests/SystemTweaksViewModelTests.cs` — filtering, totals, safe defaults, and profile isolation tests.

## Acceptance criteria

- [x] Store selected process names and the transitional fan flag per Rust profile.
- [x] Normalize protected process names across casing, whitespace, and `.exe` forms.
- [x] Route process and cleanup intents through injectable Engine decision logic for safe hosted tests.
- [x] Keep WinUI process/toggle preview state independent between profiles and restore safe defaults deterministically.
- [x] Replace fixture processes with a Runner-provided read-only snapshot.
- [ ] Add structured protected/ambiguous selection reasons.
- [ ] Enable explicit Recycle Bin and browser-cache cleanup only after Runner executes them in the verified interactive-user context.
- [ ] Align WinUI 3 and Rust profile contracts for every supported tweak; do not imply persistence for preview-only toggles.
- [ ] Define supported fan-policy hardware, authorization, apply, rollback, and unavailable behavior before enabling it.
- [ ] Classify each operation by execution context and move user cleanup out of the SYSTEM environment.
- [ ] Validate PID identity and Windows critical/protected state immediately before termination.
- [ ] Preview the exact activation plan and return killed, missing, skipped, failed, cleaned, applied, and rolled-back outcomes.
- [ ] Require explicit user intent for cleanup and machine mutations; startup restoration performs no side effects.
- [ ] Verify privileged and destructive behavior only in an isolated Windows environment, never on hosted CI or a developer machine.

## Remaining gaps

Runner-backed WinUI 3 state, a unified tweak schema, live process snapshots, executable fan policy, correct user-context cleanup, PID validation, rollback semantics, authenticated broker execution, and Windows integration coverage remain planned. Until those contracts exist, the WinUI 3 page must continue to identify unavailable actions as preview-only.
