# EdgeOptimizer.Settings.WinUI

WinUI 3 (.NET 10 LTS) presentation client for Edge Optimizer.

Current state:
- The Rust Runner + Engine service own optimization and cleanup execution.
- The WinUI client now includes the shared profile-scoped shell and preview implementations of Dashboard, Crosshair, Macros, and System Tweaks.
- The client connects to Runner's transitional Bincode named pipe, hydrates Runner-owned profiles, saves edits, requests live process snapshots and cleanup, and activates profiles.
- Runner starts the unprivileged crosshair and macro workers for the active profile. The shared Protobuf contract remains the required replacement for this compatibility transport.
- Recording, safe test playback, managed crosshair assets, and fan-policy execution remain unavailable and are identified as such in the UI.
- The window exits fully when closed and does not open Runner's database or perform privileged operations.

Next integration step:
- Generate C# protocol types from the shared Protobuf contract.
- Request the authoritative profile/active-state snapshot from Runner.
- Send profile edits and orchestration commands to Runner; never open `state.db` directly.
