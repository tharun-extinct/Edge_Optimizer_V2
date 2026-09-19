using System.ComponentModel;
using EdgeOptimizer.Settings.Core.ViewModels;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media.Imaging;
using Windows.Storage;

namespace EdgeOptimizer.Settings.WinUI.Views;

public partial class CrosshairView : UserControl
{
    private CrosshairViewModel? _viewModel;

    public CrosshairView()
    {
        InitializeComponent();
        DataContextChanged += OnDataContextChanged;
        Unloaded += (_, _) =>
        {
            if (_viewModel is not null) _viewModel.PropertyChanged -= ViewModelPropertyChanged;
        };
    }

    private void OnDataContextChanged(FrameworkElement sender, DataContextChangedEventArgs args)
    {
        if (_viewModel is not null) _viewModel.PropertyChanged -= ViewModelPropertyChanged;
        _viewModel = args.NewValue as CrosshairViewModel;
        if (_viewModel is not null) _viewModel.PropertyChanged += ViewModelPropertyChanged;
        _ = LoadPreviewAsync();
    }

    private void ViewModelPropertyChanged(object? sender, PropertyChangedEventArgs args)
    {
        if (args.PropertyName == nameof(CrosshairViewModel.ImagePath)) _ = LoadPreviewAsync();
    }

    private async Task LoadPreviewAsync()
    {
        var path = _viewModel?.ImagePath;
        if (string.IsNullOrWhiteSpace(path))
        {
            PreviewImage.Source = null;
            FallbackCrosshair.Visibility = Visibility.Visible;
            return;
        }

        try
        {
            var file = await StorageFile.GetFileFromPathAsync(path);
            await using var stream = await file.OpenStreamForReadAsync();
            var bitmap = new BitmapImage();
            await bitmap.SetSourceAsync(stream.AsRandomAccessStream());
            PreviewImage.Source = bitmap;
            FallbackCrosshair.Visibility = Visibility.Collapsed;
        }
        catch
        {
            PreviewImage.Source = null;
            FallbackCrosshair.Visibility = Visibility.Visible;
        }
    }
}
