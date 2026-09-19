# Crosshair overlay

## Outcome

A player can configure one PNG crosshair per profile, preview its position, and have Runner own a click-through, unprivileged overlay worker for the active profile.

## Current verified status

**Status:** Partial

Code inspection on 2026-09-05 confirms that Rust profiles carry an image path, offsets, and an enabled flag; Rust validates decodable 100×100 images; Runner starts the standalone overlay during successful profile activation; and the Win32 worker renders a centered, transparent, click-through, topmost window. The WinUI preview has profile-scoped enable, replace, remove, reset, center, and bounded movement logic with unit tests.

WinUI now hydrates and saves the profile's crosshair path, enablement, and offsets through Runner's transitional compatibility pipe. Activation flows through Runner, which starts and owns the current overlay worker handle. The current launcher still uses command-line worker arguments rather than a versioned Runner/worker protocol. Interactive overlay behavior has not been verified in hosted CI.

## Architecture dependencies

- [Component boundaries](../architecture.md#component-boundaries)
- [State ownership and persistence](../architecture.md#state-ownership-and-persistence)
- [IPC and protocol boundary](../architecture.md#ipc-and-protocol-boundary)
- [Failure and recovery](../architecture.md#failure-and-recovery)

## Feature-specific implications

### Component boundaries

Settings edits and previews are presentation concerns. Runner owns the active overlay lifecycle and starts or stops the unprivileged Crosshair worker; neither Settings nor the worker opens durable state.

### State ownership and persistence

Crosshair enablement, asset identity, and offsets are profile-scoped Runner state. Selected images must ultimately be copied into managed storage instead of retaining an arbitrary user path.

### IPC and protocol boundary

WinUI sends validated intent to Runner, and Runner sends explicit start, update, stop, acknowledgement, and error messages to the worker. The current command-line launch arguments are transitional, not the target contract.

### Failure and recovery

A missing or invalid image, worker disconnect, or render failure must surface as a crosshair-specific failure without blocking unrelated profile settings or triggering a privileged fallback.

## Related blueprints

### Required

- [Profile persistence](profile-persistence.md) — owns profile state and future managed crosshair assets.
- [IPC contracts](ipc-contracts.md) — defines WinUI/Runner and Runner/worker messages and framing.

### Impact checks

- [Settings client](settings-client.md) — owns the WinUI 3 navigation, bindings, preview presentation, and accessibility surface.

## Relevant implementation and tests

- `crates/core/src/profile.rs` — transitional profile fields and validation.
- `crates/core/src/image_picker.rs` — PNG decoding and 100×100 dimension validation with disposable fixtures.
- `crates/core/src/crosshair_overlay.rs` — transitional detached-process launcher and stop behavior.
- `crates/runner/src/main.rs` — activation-time overlay orchestration and result summary.
- `crates/crosshair/src/main.rs` — Win32 layered-window rendering and click-through behavior.
- `apps/EdgeOptimizer.Settings.Core/ViewModels/CrosshairViewModel.cs` — profile-scoped edit and preview logic saved through Runner.
- `apps/EdgeOptimizer.Settings.WinUI/Services/TransitionalBincodeRunnerClient.cs` — temporary WinUI/Runner compatibility transport.
- `tests/EdgeOptimizer.Settings.Core.Tests/CrosshairViewModelTests.cs` — movement, bounds, file selection, removal, hiding, and reset tests.

## Acceptance criteria

- [x] Represent enabled state, image reference, and X/Y offsets per Rust profile.
- [x] Reject missing, undecodable, or incorrectly sized images in the current Rust image-selection validation path.
- [ ] Repeat asset validation at the Runner/worker boundary before overlay use.
- [x] Preserve independent WinUI 3 crosshair preview state when switching profiles.
- [x] Center and move the preview deterministically within its current supported bounds.
- [ ] Define one coordinate range and screen/DPI interpretation shared by Rust, WinUI 3, and the worker.
- [ ] Copy selected images into managed storage and persist stable asset identities.
- [x] Route WinUI 3 profile saves and activation through Runner.
- [ ] Replace direct command-line lifecycle control with versioned worker messages.
- [ ] Return correlated start, update, stop, acknowledgement, and render-error results.
- [ ] Handle missing assets and worker restart without silently resetting valid profile state.
- [ ] Verify click-through, topmost, DPI, fullscreen, and multi-monitor behavior in Windows integration tests.

## Remaining gaps

Managed assets, generated contracts, versioned worker updates, shared coordinate semantics, and Windows visual/integration verification remain planned. The current worker receives activation-time command-line configuration and must be replaced by the target lifecycle contract.
