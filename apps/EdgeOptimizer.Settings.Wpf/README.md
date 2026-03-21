# EdgeOptimizer.Settings.Wpf

Initial WPF (.NET 8) shell for the Settings UI.

Current state:
- The Rust Runner + Engine service own optimization and cleanup execution.
- This WPF app is the future orchestration UI client.

Next integration step:
- Add an IPC client in this app to send versioned orchestration envelopes to Runner.
