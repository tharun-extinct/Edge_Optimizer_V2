# Settings client

## Outcome

An on-demand, unprivileged WinUI client edits profiles and presents Runner state while consuming no memory when closed.

## Current verified status

**Status:** Partial

WinUI 3 is the active Settings UI and is launched on demand by Runner from the packaged `EdgeOptimizer.Settings.WinUI.exe`. It restores the full profile-scoped Dashboard, Crosshair, Macros, and System Tweaks surfaces and uses a transitional Bincode compatibility client to hydrate Runner state, save profile collections, request live processes and cleanup, and activate profiles. Runner's pipe accepts the client non-blockingly. A Windows CI job tests UI-independent logic, compiles WinUI XAML, and publishes a self-contained client artifact. Generated Protobuf bindings and runtime UI smoke automation remain planned.

## Architecture dependencies

- [Component boundaries](../architecture.md#component-boundaries)
- [IPC and protocol boundary](../architecture.md#ipc-and-protocol-boundary)

## Feature-specific implications

WinUI 3 never owns durable state or privileged operations. It requests a snapshot from Runner, submits validated commands, and exits completely when closed.

## Related blueprints

### Required

- [IPC contracts](ipc-contracts.md) — generated C# types are required before retiring the transitional compatibility transport.

### Impact checks

- [Profile persistence](profile-persistence.md) — profile CRUD and startup hydration must remain Runner-owned.
- [Crosshair overlay](crosshair-overlay.md) — WinUI owns preview presentation but not overlay lifecycle or assets.
- [Macro automation](macro-automation.md) — WinUI owns editing presentation but not hooks or input execution.
- [System Tweaks](system-tweaks.md) — WinUI 3 presents choices without performing cleanup, termination, or machine changes.

## Relevant implementation and tests

- `apps/EdgeOptimizer.Settings.Core` — UI-independent models, contracts, and view-model logic.
- `apps/EdgeOptimizer.Settings.WinUI` — active WinUI presentation client.
- `crates/runner/src/main.rs` — launches the packaged WinUI client.

## Acceptance criteria

- [ ] Build on .NET 10 LTS.
- [x] Hydrate profiles and active state from Runner through the transitional compatibility transport.
- [x] Perform profile collection saves through Runner; generated versioned bindings remain planned.
- [ ] Exit fully when the window closes.
- [x] Runner launches the packaged WinUI Settings client.
- [x] Save supported profile state, activate profiles, and request live process/cleanup operations through Runner's transitional transport.
- [x] Include WinUI 3 build and logic tests in CI.

## Remaining gaps

Generated Protobuf bindings, golden cross-language fixtures, profile rename validation, safe macro recording/test playback, and interactive Windows UI automation remain planned. The Bincode compatibility client is transitional and must be removed after the shared generated contract lands. GitHub Actions is the build/test authority because the local .NET 10 SDK is unavailable.
