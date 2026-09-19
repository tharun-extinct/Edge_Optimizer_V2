using System.Collections.ObjectModel;
using System.Windows.Input;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using EdgeOptimizer.Settings.Core.Models;
using EdgeOptimizer.Settings.Core.Services;

namespace EdgeOptimizer.Settings.Core.ViewModels;

public sealed class MainWindowViewModel : ObservableObject
{
    private readonly IRunnerClient _runnerClient;
    private ProfileWorkspace? _selectedProfile;
    private object? _currentPage;
    private string _currentPageLabel = "Dashboard";
    private string _currentPageTitle = "Gaming dashboard";
    private string _statusMessage = "Connecting to Runner…";

    public MainWindowViewModel(IFilePicker filePicker, IRunnerClient runnerClient)
    {
        _runnerClient = runnerClient;
        Dashboard = new DashboardViewModel(NavigateTo);
        Crosshair = new CrosshairViewModel(filePicker, SaveProfilesAsync);
        Macros = new MacrosViewModel(SaveProfilesAsync);
        SystemTweaks = new SystemTweaksViewModel(SaveProfilesAsync, RequestCleanupAsync, RequestProcessSnapshotAsync);
        NavigateCommand = new RelayCommand<string>(NavigateTo);
        NewProfileCommand = new RelayCommand(NewProfile);
        DuplicateProfileCommand = new RelayCommand(DuplicateProfile, () => SelectedProfile is not null);
        DeleteProfileCommand = new AsyncRelayCommand(DeleteProfileAsync, () => SelectedProfile is not null);
        SaveCommand = new AsyncRelayCommand(SaveProfilesAsync);
        ActivateProfileCommand = new AsyncRelayCommand(ActivateProfileAsync, () => SelectedProfile is not null && ActivationEnabled);

        _runnerClient.ConnectionChanged += (_, connected) =>
        {
            OnPropertyChanged(nameof(ActivationEnabled));
            NotifyCommandState();
            StatusMessage = connected ? "Connected to Runner. Loading profiles…" : "Runner is unavailable. Changes cannot be saved.";
        };
        _runnerClient.SnapshotReceived += (_, snapshot) => ApplySnapshot(snapshot);
        _runnerClient.StatusReceived += (_, status) => StatusMessage = status;
        _runnerClient.ProcessSnapshotReceived += (_, processes) => SystemTweaks.ApplyProcessSnapshot(processes);

        Profiles.Add(new ProfileWorkspace("Fortnite", true));
        Profiles.Add(new ProfileWorkspace("Valorant", false));
        Profiles.Add(new ProfileWorkspace("Shadow of Tomb Raider", false));
        SelectedProfile = Profiles[0];
        NavigateTo("Dashboard");
    }

    public ObservableCollection<ProfileWorkspace> Profiles { get; } = new();
    public DashboardViewModel Dashboard { get; }
    public CrosshairViewModel Crosshair { get; }
    public MacrosViewModel Macros { get; }
    public SystemTweaksViewModel SystemTweaks { get; }
    public ICommand NavigateCommand { get; }
    public ICommand NewProfileCommand { get; }
    public ICommand DuplicateProfileCommand { get; }
    public ICommand DeleteProfileCommand { get; }
    public ICommand SaveCommand { get; }
    public ICommand ActivateProfileCommand { get; }

    public ProfileWorkspace? SelectedProfile
    {
        get => _selectedProfile;
        set
        {
            if (!SetProperty(ref _selectedProfile, value) || value is null) return;
            Dashboard.LoadProfile(value);
            Crosshair.LoadProfile(value);
            Macros.LoadProfile(value);
            SystemTweaks.LoadProfile(value);
            StatusMessage = $"Loaded {value.Name}.";
            NotifyCommandState();
        }
    }

    public object? CurrentPage { get => _currentPage; private set => SetProperty(ref _currentPage, value); }
    public string CurrentPageLabel { get => _currentPageLabel; private set => SetProperty(ref _currentPageLabel, value); }
    public string CurrentPageTitle { get => _currentPageTitle; private set => SetProperty(ref _currentPageTitle, value); }
    public string StatusMessage { get => _statusMessage; private set => SetProperty(ref _statusMessage, value); }
    public bool ActivationEnabled => _runnerClient.IsConnected;
    public bool IsDashboardSelected => CurrentPageLabel == "Dashboard";
    public bool IsCrosshairSelected => CurrentPageLabel == "Crosshair";
    public bool IsMacrosSelected => CurrentPageLabel == "Macros";
    public bool IsSystemTweaksSelected => CurrentPageLabel == "System Tweaks";

    public async Task InitializeAsync(CancellationToken cancellationToken = default)
    {
        try { await _runnerClient.StartAsync(cancellationToken); }
        catch (Exception error) { StatusMessage = $"Could not connect to Runner: {error.Message}"; }
    }

    public void NavigateTo(string? page)
    {
        switch (page)
        {
            case "Crosshair": CurrentPage = Crosshair; CurrentPageLabel = "Crosshair"; CurrentPageTitle = "Crosshair overlay"; break;
            case "Macros": CurrentPage = Macros; CurrentPageLabel = "Macros"; CurrentPageTitle = "Macro editor"; break;
            case "SystemTweaks": CurrentPage = SystemTweaks; CurrentPageLabel = "System Tweaks"; CurrentPageTitle = "System tweaks"; break;
            default: CurrentPage = Dashboard; CurrentPageLabel = "Dashboard"; CurrentPageTitle = "Gaming dashboard"; break;
        }
        OnPropertyChanged(nameof(IsDashboardSelected));
        OnPropertyChanged(nameof(IsCrosshairSelected));
        OnPropertyChanged(nameof(IsMacrosSelected));
        OnPropertyChanged(nameof(IsSystemTweaksSelected));
    }

    private void ApplySnapshot(RunnerSnapshot snapshot)
    {
        Profiles.Clear();
        foreach (var profile in snapshot.Profiles) Profiles.Add(profile);
        if (Profiles.Count == 0) Profiles.Add(new ProfileWorkspace("New profile", false));
        foreach (var profile in Profiles)
            profile.IsActive = string.Equals(profile.Name, snapshot.ActiveProfileName, StringComparison.OrdinalIgnoreCase);
        SelectedProfile = Profiles.FirstOrDefault(profile => profile.IsActive) ?? Profiles[0];
        StatusMessage = $"Loaded {snapshot.Profiles.Count} profile(s) from Runner.";
    }

    private void NewProfile()
    {
        var profile = new ProfileWorkspace(GetUniqueProfileName("New profile"), false);
        Profiles.Add(profile);
        SelectedProfile = profile;
        StatusMessage = "New profile created. Save changes to persist it.";
    }

    private void DuplicateProfile()
    {
        if (SelectedProfile is null) return;
        var copy = ProfileWorkspaceCopy.Create(SelectedProfile, GetUniqueProfileName($"{SelectedProfile.Name} copy"));
        Profiles.Add(copy);
        SelectedProfile = copy;
    }

    private async Task DeleteProfileAsync()
    {
        if (SelectedProfile is null) return;
        var index = Profiles.IndexOf(SelectedProfile);
        Profiles.Remove(SelectedProfile);
        SelectedProfile = Profiles.Count == 0 ? null : Profiles[Math.Clamp(index, 0, Profiles.Count - 1)];
        await SaveProfilesAsync();
    }

    private async Task SaveProfilesAsync()
    {
        if (!_runnerClient.IsConnected) { StatusMessage = "Runner is unavailable; changes were not saved."; return; }
        await _runnerClient.SaveProfilesAsync(Profiles.ToArray());
        StatusMessage = "Changes saved to Runner.";
    }

    private async Task ActivateProfileAsync()
    {
        if (SelectedProfile is null || !_runnerClient.IsConnected) return;
        await SaveProfilesAsync();
        await _runnerClient.ActivateProfileAsync(SelectedProfile);
        StatusMessage = $"Activating {SelectedProfile.Name}…";
    }

    private Task RequestCleanupAsync(string kind) => _runnerClient.RequestCleanupAsync(kind);
    private Task RequestProcessSnapshotAsync() => _runnerClient.RequestProcessSnapshotAsync();

    private string GetUniqueProfileName(string root)
    {
        var candidate = root;
        var suffix = 2;
        while (Profiles.Any(profile => string.Equals(profile.Name, candidate, StringComparison.OrdinalIgnoreCase)))
            candidate = $"{root} {suffix++}";
        return candidate;
    }

    private void NotifyCommandState()
    {
        ((RelayCommand)DuplicateProfileCommand).NotifyCanExecuteChanged();
        ((AsyncRelayCommand)DeleteProfileCommand).NotifyCanExecuteChanged();
        ((AsyncRelayCommand)ActivateProfileCommand).NotifyCanExecuteChanged();
    }
}

internal static class ProfileWorkspaceCopy
{
    public static ProfileWorkspace Create(ProfileWorkspace source, string name)
    {
        var copy = new ProfileWorkspace(name, false)
        {
            OverlayEnabled = source.OverlayEnabled,
            CrosshairImageName = source.CrosshairImageName,
            CrosshairImagePath = source.CrosshairImagePath,
            CrosshairXOffset = source.CrosshairXOffset,
            CrosshairYOffset = source.CrosshairYOffset,
            FanBoostEnabled = source.FanBoostEnabled,
            RecycleBinEnabled = source.RecycleBinEnabled,
            BrowserCacheEnabled = source.BrowserCacheEnabled,
        };
        copy.Macros.Clear();
        foreach (var macro in source.Macros)
            copy.Macros.Add(new MacroDefinition(macro.Name, macro.Shortcut, macro.Steps.ToArray()) { IsEnabled = macro.IsEnabled, RepeatMode = macro.RepeatMode, RepeatCount = macro.RepeatCount, StopKey = macro.StopKey });
        copy.Processes.Clear();
        foreach (var process in source.Processes)
            copy.Processes.Add(new ProcessItem(process.Name, process.Cpu, process.Memory, process.IsSelected));
        return copy;
    }
}
