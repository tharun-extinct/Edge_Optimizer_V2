using EdgeOptimizer.Settings.Core.Models;

namespace EdgeOptimizer.Settings.Core.Services;

public interface IFilePicker
{
    Task<string?> PickPngAsync(CancellationToken cancellationToken = default);
}

public interface IRunnerClient
{
    bool IsConnected { get; }
    event EventHandler<bool>? ConnectionChanged;
    event EventHandler<RunnerSnapshot>? SnapshotReceived;
    event EventHandler<string>? StatusReceived;
    event EventHandler<IReadOnlyList<ProcessItem>>? ProcessSnapshotReceived;
    event EventHandler<RunnerWindowCommand>? WindowCommandReceived;

    Task StartAsync(CancellationToken cancellationToken = default);
    Task SaveProfilesAsync(IReadOnlyList<ProfileWorkspace> profiles, CancellationToken cancellationToken = default);
    Task SetActiveProfileAsync(string? profileName, CancellationToken cancellationToken = default);
    Task SetOverlayVisibilityAsync(bool visible, CancellationToken cancellationToken = default);
    Task ActivateProfileAsync(ProfileWorkspace profile, CancellationToken cancellationToken = default);
    Task RequestCleanupAsync(string cleanupKind, CancellationToken cancellationToken = default);
    Task RequestProcessSnapshotAsync(CancellationToken cancellationToken = default);
}

public sealed record RunnerSnapshot(
    IReadOnlyList<ProfileWorkspace> Profiles,
    string? ActiveProfileName,
    bool OverlayVisible);

public enum RunnerWindowCommand
{
    Show,
    Hide,
    BringToFront,
    Exit,
}

public interface IProcessSource
{
    IReadOnlyList<ProcessItem> GetProcesses();
}

public interface ICleanupClient
{
    Task RequestCleanupAsync(string cleanupKind, CancellationToken cancellationToken = default);
}

public interface IMacroClient
{
    Task TestMacroAsync(MacroDefinition macro, CancellationToken cancellationToken = default);
}

public interface IOverlayClient
{
    Task PreviewAsync(ProfileWorkspace profile, CancellationToken cancellationToken = default);
}
