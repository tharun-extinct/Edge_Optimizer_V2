using EdgeOptimizer.Settings.Core.Models;
using EdgeOptimizer.Settings.Core.ViewModels;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace EdgeOptimizer.Settings.WinUI.Views;

public partial class MacrosView : UserControl
{
    public MacrosView()
    {
        InitializeComponent();
    }

    private void OnDeleteStepClick(object sender, RoutedEventArgs args)
    {
        if (sender is Button { Tag: MacroStep step } && DataContext is MacrosViewModel viewModel)
            viewModel.DeleteStepCommand.Execute(step);
    }
}
