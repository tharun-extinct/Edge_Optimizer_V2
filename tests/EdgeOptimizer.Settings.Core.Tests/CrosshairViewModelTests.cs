using EdgeOptimizer.Settings.Core.Models;
using EdgeOptimizer.Settings.Core.ViewModels;

namespace EdgeOptimizer.Settings.Core.Tests;

public sealed class CrosshairViewModelTests
{
    [Fact]
    public void DirectionalMovementAndCenterUpdateCoordinates()
    {
        // Verifies movement changes one axis at a time and centering resets both axes.
        var viewModel = CreateViewModel(new FakeFilePicker(null));
        viewModel.MoveCommand.Execute("Right");
        viewModel.MoveCommand.Execute("Down");
        Assert.Equal((1, 1), (viewModel.XOffset, viewModel.YOffset));

        viewModel.CenterCommand.Execute(null);
        Assert.Equal((0, 0), (viewModel.XOffset, viewModel.YOffset));
    }

    [Fact]
    public void CoordinatesAreClampedToSupportedRange()
    {
        // Verifies manual offsets cannot exceed the preview's safe coordinate limits.
        var viewModel = CreateViewModel(new FakeFilePicker(null));
        viewModel.XOffset = 900;
        viewModel.YOffset = -900;
        Assert.Equal((500, -500), (viewModel.XOffset, viewModel.YOffset));
    }

    [Fact]
    public async Task FilePickerCancellationLeavesImageUnchangedAsync()
    {
        // Verifies cancelling image selection preserves the current crosshair asset.
        var viewModel = CreateViewModel(new FakeFilePicker(null));
        await viewModel.ReplaceImageCommand.ExecuteAsync(null);
        Assert.Equal("dot-crosshair.png", viewModel.ImageName);
    }

    [Fact]
    public async Task ReplaceRemoveAndResetUpdatePreviewStateAsync()
    {
        // Verifies image replacement, removal, hiding, and reset mutate only the selected profile preview.
        var viewModel = CreateViewModel(new FakeFilePicker(@"C:\fixtures\precision.png"));
        await viewModel.ReplaceImageCommand.ExecuteAsync(null);
        Assert.Equal("precision.png", viewModel.ImageName);

        viewModel.RemoveImageCommand.Execute(null);
        Assert.Equal("No image selected", viewModel.ImageName);
        viewModel.HidePreviewCommand.Execute(null);
        Assert.False(viewModel.OverlayEnabled);

        viewModel.ResetCommand.Execute(null);
        Assert.True(viewModel.OverlayEnabled);
        Assert.Equal("dot-crosshair.png", viewModel.ImageName);
    }

    private static CrosshairViewModel CreateViewModel(FakeFilePicker picker)
    {
        var viewModel = new CrosshairViewModel(picker);
        viewModel.LoadProfile(new ProfileWorkspace("Test", false));
        return viewModel;
    }
}
