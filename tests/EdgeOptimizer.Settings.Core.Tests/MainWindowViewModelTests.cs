using EdgeOptimizer.Settings.Core.ViewModels;
using EdgeOptimizer.Settings.Core.Models;
using EdgeOptimizer.Settings.Core.Services;
using CommunityToolkit.Mvvm.Input;

namespace EdgeOptimizer.Settings.Core.Tests;

public sealed class MainWindowViewModelTests
{
    [Fact]
    public void NavigationPreservesSelectedProfile()
    {
        // Verifies switching feature pages never loses the user's selected profile context.
        var viewModel = CreateViewModel();
        var selected = viewModel.Profiles[1];
        viewModel.SelectedProfile = selected;

        foreach (var page in new[] { "Dashboard", "Crosshair", "Macros", "SystemTweaks" })
        {
            viewModel.NavigateTo(page);
            Assert.Same(selected, viewModel.SelectedProfile);
        }
    }

    [Fact]
    public void SelectingProfileLoadsEveryWorkspace()
    {
        // Verifies Crosshair, Macros, System Tweaks, and Dashboard receive the newly selected profile state.
        var viewModel = CreateViewModel();
        var selected = viewModel.Profiles[1];
        selected.CrosshairXOffset = 17;
        selected.FanBoostEnabled = false;

        viewModel.SelectedProfile = selected;

        Assert.Same(selected, viewModel.Dashboard.SelectedProfile);
        Assert.Equal(17, viewModel.Crosshair.XOffset);
        Assert.Same(selected.Macros[0], viewModel.Macros.SelectedMacro);
        Assert.False(viewModel.SystemTweaks.FanBoostEnabled);
    }

    [Fact]
    public void DashboardQuickActionNavigatesToRequestedWorkspace()
    {
        // Verifies a Dashboard quick action routes through the shell navigation contract.
        var viewModel = CreateViewModel();

        viewModel.Dashboard.NavigateCommand.Execute("Macros");

        Assert.Equal("Macros", viewModel.CurrentPageLabel);
        Assert.Same(viewModel.Macros, viewModel.CurrentPage);
    }

    [Fact]
    public void NewProfileBecomesSelectedAndProfileScoped()
    {
        // Verifies a new temporary profile is selected and loaded by every workspace.
        var viewModel = CreateViewModel();

        viewModel.NewProfileCommand.Execute(null);

        Assert.Equal(4, viewModel.Profiles.Count);
        Assert.Same(viewModel.Profiles[^1], viewModel.SelectedProfile);
        Assert.Same(viewModel.SelectedProfile, viewModel.Dashboard.SelectedProfile);
    }

    [Fact]
    public void ActivationReflectsRunnerAvailability()
    {
        // Verifies profile activation stays disabled until a Runner client reports a connection.
        Assert.False(CreateViewModel().ActivationEnabled);
        Assert.True(new MainWindowViewModel(new FakeFilePicker(null), new FakeRunnerClient(true)).ActivationEnabled);
    }

    [Fact]
    public void RunnerSnapshotReplacesPreviewProfilesAndRestoresActivation()
    {
        // Verifies Runner hydration replaces placeholder data and marks the authoritative active profile.
        var runner = new FakeRunnerClient(true);
        var viewModel = new MainWindowViewModel(new FakeFilePicker(null), runner);
        runner.RaiseSnapshot(new RunnerSnapshot(
            new[] { new ProfileWorkspace("Hydrated", false), new ProfileWorkspace("Active", false) },
            "Active",
            true));

        Assert.Equal(2, viewModel.Profiles.Count);
        Assert.Equal("Active", viewModel.SelectedProfile?.Name);
        Assert.True(viewModel.SelectedProfile?.IsActive);
    }

    [Fact]
    public async Task ActivationSavesThenRequestsRunnerOrchestration()
    {
        // Verifies activation persists the current collection before asking Runner to start workers and optimization.
        var runner = new FakeRunnerClient(true);
        var viewModel = new MainWindowViewModel(new FakeFilePicker(null), runner);

        await ((IAsyncRelayCommand)viewModel.ActivateProfileCommand).ExecuteAsync(null);

        Assert.Equal(viewModel.Profiles.Count, runner.SavedProfiles.Count);
        Assert.Same(viewModel.SelectedProfile, Assert.Single(runner.ActivatedProfiles));
    }

    private static MainWindowViewModel CreateViewModel() =>
        new(new FakeFilePicker(null), new FakeRunnerClient());
}
