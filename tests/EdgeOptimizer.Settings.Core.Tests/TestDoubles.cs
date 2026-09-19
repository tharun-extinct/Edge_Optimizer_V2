using EdgeOptimizer.Settings.Core.Models;
using EdgeOptimizer.Settings.Core.Services;

namespace EdgeOptimizer.Settings.Core.Tests;

internal sealed class FakeFilePicker(string? path) : IFilePicker
{
    public Task<string?> PickPngAsync(CancellationToken cancellationToken = default) => Task.FromResult(path);
}

internal sealed class FakeRunnerClient(bool connected = false) : IRunnerClient
{
    public bool IsConnected => connected;
    public List<ProfileWorkspace> SavedProfiles { get; } = new();
    public event EventHandler<bool>? ConnectionChanged;
    public event EventHandler<RunnerSnapshot>? SnapshotReceived;
    public event EventHandler<string>? StatusReceived;
    public event EventHandler<RunnerWindowCommand>? WindowCommandReceived;

    public Task StartAsync(CancellationToken cancellationToken = default) => Task.CompletedTask;

    public Task SaveProfilesAsync(IReadOnlyList<ProfileWorkspace> profiles, CancellationToken cancellationToken = default)
    {
        SavedProfiles.AddRange(profiles);
        return Task.CompletedTask;
    }

    public Task SetActiveProfileAsync(string? profileName, CancellationToken cancellationToken = default) => Task.CompletedTask;
    public Task SetOverlayVisibilityAsync(bool visible, CancellationToken cancellationToken = default) => Task.CompletedTask;
    public Task ActivateProfileAsync(ProfileWorkspace profile, CancellationToken cancellationToken = default) => Task.CompletedTask;
    public Task RequestCleanupAsync(string cleanupKind, CancellationToken cancellationToken = default) => Task.CompletedTask;
}
