# IPC contracts

## Outcome

Runner, WinUI 3, workers, and the broker communicate through versioned, bounded, validated cross-language messages with OS-backed peer identity.

## Current verified status

**Status:** Partial

Current named-pipe communication uses Rust Serde/Bincode. A bounded C# compatibility codec now connects WinUI to Runner for state, saves, live process snapshots, cleanup intents, activation, and orchestration results. Runner accepts connections non-blockingly. Protobuf, authoritative generated bindings, explicit length framing, and authenticated broker identity are not implemented.

## Architecture dependencies

- [IPC and protocol boundary](../architecture.md#ipc-and-protocol-boundary)
- [Privilege and identity](../architecture.md#privilege-and-identity)

## Feature-specific implications

The `.proto` schema will be authoritative for wire representation. Serialized auth fields are informational only; peer identity comes from the named-pipe token and ACL.

## Related blueprints

### Required

None.

### Impact checks

- [Settings client](settings-client.md) — C# bindings and client transport.
- [Privileged broker](privileged-broker.md) — authenticated privileged endpoint.
- [Crosshair overlay](crosshair-overlay.md) — WinUI/Runner state and Runner/worker lifecycle messages.
- [Macro automation](macro-automation.md) — edit, record, playback, cancellation, and worker events.
- [System Tweaks](system-tweaks.md) — snapshots, activation plans, commands, and structured results.

## Relevant implementation and tests

- `crates/core/src/ipc.rs` — transitional Settings/Runner pipe.
- `crates/core/src/engine_ipc.rs` — transitional Runner/Engine pipe.
- `crates/core/src/orchestration.rs` — transitional envelope and operations.

## Acceptance criteria

- [ ] Generate Rust and C# bindings from one Protobuf schema.
- [ ] Enforce protocol version and maximum frame length before decoding.
- [ ] Reject unknown or malformed privileged operations.
- [ ] Verify broker peer identity from Windows, not message claims.
- [ ] Bound idempotency retention by time and size.

## Remaining gaps

The complete target contract remains to be implemented before the active WinUI Settings client can enable Runner-backed behavior.
