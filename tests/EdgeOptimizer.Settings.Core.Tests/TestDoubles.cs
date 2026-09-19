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
    public List<ProfileWorkspace> ActivatedProfiles { get; } = new();
    public List<string> CleanupRequests { get; } = new();
    public int ProcessSnapshotRequests { get; private set; }
    public event EventHandler<bool>? ConnectionChanged;
    public event EventHandler<RunnerSnapshot>? SnapshotReceived;
    public event EventHandler<string>? StatusReceived;
    public event EventHandler<IReadOnlyList<ProcessItem>>? ProcessSnapshotReceived;
    public event EventHandler<string?>? ActiveProfileChanged;
    public event EventHandler<RunnerWindowCommand>? WindowCommandReceived;

    public Task StartAsync(CancellationToken cancellationToken = default) => Task.CompletedTask;

    public Task SaveProfilesAsync(IReadOnlyList<ProfileWorkspace> profiles, CancellationToken cancellationToken = default)
    {
        SavedProfiles.AddRange(profiles);
        return Task.CompletedTask;
    }

    public Task SetActiveProfileAsync(string? profileName, CancellationToken cancellationToken = default) => Task.CompletedTask;
    public Task SetOverlayVisibilityAsync(bool visible, CancellationToken cancellationToken = default) => Task.CompletedTask;
    public Task ActivateProfileAsync(ProfileWorkspace profile, CancellationToken cancellationToken = default) { ActivatedProfiles.Add(profile); return Task.CompletedTask; }
    public Task RequestCleanupAsync(string cleanupKind, CancellationToken cancellationToken = default) { CleanupRequests.Add(cleanupKind); return Task.CompletedTask; }
    public Task RequestProcessSnapshotAsync(CancellationToken cancellationToken = default) { ProcessSnapshotRequests++; return Task.CompletedTask; }

    public void RaiseSnapshot(RunnerSnapshot snapshot) => SnapshotReceived?.Invoke(this, snapshot);
    public void RaiseActiveProfile(string? name) => ActiveProfileChanged?.Invoke(this, name);
}
