using WinUIEx;
using Microsoft.UI.Xaml.Media;

namespace EdgeOptimizer.Settings.WinUI;

public sealed partial class MainWindow : WindowEx
{
    public MainWindow(ShellPage shell)
    {
        InitializeComponent();
        // SystemBackdrop requires a MicaBackdrop instance; the XAML string "Mica" is not type-convertible.
        SystemBackdrop = new MicaBackdrop();
        Root.Children.Add(shell);
        ExtendsContentIntoTitleBar = true;
        SetTitleBar(shell.DragRegion);
        WindowExtensions.CenterOnScreen(this, null, null);
    }
}
