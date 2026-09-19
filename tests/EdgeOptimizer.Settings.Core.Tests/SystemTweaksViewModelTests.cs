using EdgeOptimizer.Settings.Core.Models;
using EdgeOptimizer.Settings.Core.ViewModels;

namespace EdgeOptimizer.Settings.Core.Tests;

public sealed class SystemTweaksViewModelTests
{
    [Fact]
    public void ProcessSearchAndSelectionSummaryAreDeterministic()
    {
        // Verifies filtering is case-insensitive and selected-process totals reflect model changes.
        var profile = new ProfileWorkspace("Test", false);
        var viewModel = new SystemTweaksViewModel();
        viewModel.LoadProfile(profile);
        viewModel.ProcessFilter = "CHROME";
        Assert.Equal("chrome.exe", Assert.Single(viewModel.FilteredProcesses).Name);

        profile.Processes[0].IsSelected = false;
        Assert.Equal("2 selected", viewModel.SelectionSummary);
    }

    [Fact]
    public void RestoreDefaultsClearsTogglesAndProcesses()
    {
        // Verifies restoring safe defaults disables all tweak options and process selections.
        var profile = new ProfileWorkspace("Test", false);
        var viewModel = new SystemTweaksViewModel();
        viewModel.LoadProfile(profile);

        viewModel.RestoreDefaultsCommand.Execute(null);

        Assert.False(viewModel.FanBoostEnabled);
        Assert.False(viewModel.RecycleBinEnabled);
        Assert.False(viewModel.BrowserCacheEnabled);
        Assert.All(profile.Processes, process => Assert.False(process.IsSelected));
        Assert.Equal("0 selected", viewModel.SelectionSummary);
    }

    [Fact]
    public void SwitchingProfilesUsesIndependentTweakState()
    {
        // Verifies profile selection swaps tweak values without leaking changes between profiles.
        var first = new ProfileWorkspace("First", false);
        var second = new ProfileWorkspace("Second", false) { FanBoostEnabled = false };
        var viewModel = new SystemTweaksViewModel();
        viewModel.LoadProfile(first);
        Assert.True(viewModel.FanBoostEnabled);
        viewModel.LoadProfile(second);
        Assert.False(viewModel.FanBoostEnabled);
        Assert.True(first.FanBoostEnabled);
    }

    [Fact]
    public void ProcessSnapshotPreservesSelectionsAndUpdatesMetrics()
    {
        // Verifies a Runner process refresh keeps selected names while replacing stale CPU and memory values.
        var profile = new ProfileWorkspace("Test", false);
        var viewModel = new SystemTweaksViewModel();
        viewModel.LoadProfile(profile);

        viewModel.ApplyProcessSnapshot(new[]
        {
            new ProcessItem("Discord.exe", "2.0%", "200 MB", false),
            new ProcessItem("game.exe", "8.0%", "900 MB", false),
        });

        Assert.True(profile.Processes.Single(process => process.Name == "Discord.exe").IsSelected);
        Assert.False(profile.Processes.Single(process => process.Name == "game.exe").IsSelected);
        Assert.Equal("1 selected", viewModel.SelectionSummary);
    }
}
