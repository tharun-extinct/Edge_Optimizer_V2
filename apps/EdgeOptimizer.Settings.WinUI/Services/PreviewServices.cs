using EdgeOptimizer.Settings.Core.Services;
using Windows.Storage.Pickers;

namespace EdgeOptimizer.Settings.WinUI.Services;

public sealed class WinUIFilePicker : IFilePicker
{
    public async Task<string?> PickPngAsync(CancellationToken cancellationToken = default)
    {
        var window = App.Window ?? throw new InvalidOperationException("The main window is not available.");
        var picker = new FileOpenPicker { SuggestedStartLocation = PickerLocationId.PicturesLibrary, ViewMode = PickerViewMode.Thumbnail };
        picker.FileTypeFilter.Add(".png");
        WinRT.Interop.InitializeWithWindow.Initialize(picker, WinRT.Interop.WindowNative.GetWindowHandle(window));
        var file = await picker.PickSingleFileAsync().AsTask(cancellationToken);
        return file?.Path;
    }
}
