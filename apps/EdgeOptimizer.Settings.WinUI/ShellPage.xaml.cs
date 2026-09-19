using System.ComponentModel;
using EdgeOptimizer.Settings.Core.ViewModels;
using EdgeOptimizer.Settings.WinUI.Views;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace EdgeOptimizer.Settings.WinUI;

public sealed partial class ShellPage : Page
{
    private readonly MainWindowViewModel _viewModel;
    public UIElement DragRegion => TitleBarDragRegion;

    public ShellPage(MainWindowViewModel viewModel)
    {
        InitializeComponent();
        _viewModel = viewModel;
        DataContext = viewModel;
        _viewModel.PropertyChanged += ViewModelPropertyChanged;
        RenderPage(_viewModel.CurrentPageLabel);
    }

    private void OnNavigateClick(object sender, RoutedEventArgs args)
    {
        if (sender is Button { Tag: string target }) _viewModel.NavigateTo(target);
    }

    private void ViewModelPropertyChanged(object? sender, PropertyChangedEventArgs args)
    {
        if (args.PropertyName == nameof(MainWindowViewModel.CurrentPageLabel)) RenderPage(_viewModel.CurrentPageLabel);
    }

    private void RenderPage(string target)
    {
        FrameworkElement page = target switch
        {
            "Crosshair" => new CrosshairView(),
            "Macros" => new MacrosView(),
            "System Tweaks" => new SystemTweaksView(),
            _ => new DashboardView(),
        };
        page.DataContext = _viewModel.CurrentPage;
        PageHost.Content = page;
    }
}
