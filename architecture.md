# Edge Optimizer Architecture

## Purpose and authority

This document defines cross-cutting contracts for the iterative architecture migration. Current code and tests are authoritative for implemented behavior; blueprints distinguish verified behavior from planned work.

## Component boundaries

- **Settings UI:** an unprivileged, on-demand WinUI presentation client. It is packaged beside Runner as `EdgeOptimizer.Settings.WinUI.exe`. Runner starts it on demand; a transitional Bincode compatibility client currently restores hydration, profile saves, process snapshots, cleanup intents, and activation while the generated Protobuf contract remains planned.
- **Runner:** the per-user startup agent, tray owner, orchestration authority, process owner, and sole durable-state owner.
- **Crosshair and Macro workers:** unprivileged native workers started and stopped by Runner.
- **Privileged Broker:** a minimal Windows SCM service that performs only allowlisted machine-level operations requested by an authenticated Runner. The current scheduled-task EngineSvc is transitional.

Dependencies point from clients and workers toward versioned contracts. Presentation clients never call privileged Windows operations or open Runner's database.

## State ownership and persistence

Runner exclusively owns `%LOCALAPPDATA%\EdgeOptimizer\state.db`. SQLite schema changes use `PRAGMA user_version` migrations. Profile-list replacement and active-profile changes are transactional. An active profile must reference an existing profile, and deleting that profile clears activation.

At startup Runner restores profiles, active-profile identity, and UI state. Restoration does not replay optimization side effects: process termination, cleanup, and privileged changes require a new explicit command. When the database is empty, Runner may import the legacy `%APPDATA%\GamingOptimizer` JSON files once.

Crosshair files will be copied into an application-managed assets directory and referenced by asset identity. Portable import/export will use versioned JSON. These asset and export rules are planned, not yet implemented.

## IPC and protocol boundary

Settings communicates only with Runner; Runner communicates with workers and the Privileged Broker. Every protocol has an explicit version, bounded message size, validation, and generated Rust/C# types. Protobuf over per-user Windows named pipes is the target contract.

The current Rust endpoints still use Serde/Bincode. This is transitional and must not become the WinUI 3 contract. Serialized identity or `AuthContext` fields are claims, not authentication evidence.

## Privilege and identity

The Privileged Broker exposes a minimal allowlist. Its named pipe has an explicit security descriptor, verifies the connecting token/SID, and authorizes each operation independently. User-specific cleanup executes in Runner's interactive-user context; the broker must not infer a user profile from its SYSTEM environment.

The current EngineSvc scheduled task and default-security named pipe do not yet enforce this contract.

## Process safety

Protected process names are normalized identically for configuration input and discovered executables before comparison. Extensionless, mixed-case, and surrounding-whitespace forms must be blocked. Before termination, the broker will also validate process identity and critical/protected status from the target PID. Name normalization is implemented; PID-level broker validation is planned.

## Failure and recovery

Persistence failures are surfaced and must not silently reset valid state. A schema newer than the binary supports is rejected. Database updates use transactions, and Runner checkpoints/copies the last valid database before each state mutation. Automated recovery from that backup is planned.

IPC failures degrade the affected capability and never authorize a privileged fallback. Duplicate request tracking must be bounded by time and size; the current in-memory unbounded cache is transitional.

## Verification boundaries

- State-store unit tests verify schema creation, restart restoration, referential activation, and invalid activation.
- Process-safety unit tests verify all accepted name forms.
- IPC contract tests must round-trip generated messages in both Rust and C# before WinUI 3 enables Runner-backed behavior.
- Broker integration tests must cover standard-user access, unauthorized clients, malformed frames, and service restart.
- Windows UI and service behavior require Windows integration tests; documentation alone never marks them implemented.

## Feature blueprints

See [the blueprint manifest](blueprints/README.md).
