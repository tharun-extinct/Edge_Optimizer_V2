using EdgeOptimizer.Settings.Core.Services;
using EdgeOptimizer.Settings.Core.ViewModels;
using EdgeOptimizer.Settings.WinUI.Services;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;

namespace EdgeOptimizer.Settings.WinUI;

public partial class App : Application
{
    private readonly ServiceProvider _services;
    public static MainWindow? Window { get; private set; }

    public App()
    {
        InitializeComponent();
        var services = new ServiceCollection();
        services.AddSingleton<IFilePicker, WinUIFilePicker>();
        services.AddSingleton<IRunnerClient>(_ => new TransitionalBincodeRunnerClient(DispatcherQueue.GetForCurrentThread()));
        services.AddSingleton<MainWindowViewModel>();
        services.AddSingleton<ShellPage>();
        services.AddSingleton<MainWindow>();
        _services = services.BuildServiceProvider();
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        Window = _services.GetRequiredService<MainWindow>();
        var runner = _services.GetRequiredService<IRunnerClient>();
        runner.WindowCommandReceived += (_, command) =>
        {
            if (Window is null) return;
            switch (command)
            {
                case RunnerWindowCommand.Show:
                case RunnerWindowCommand.BringToFront: Window.Activate(); break;
                case RunnerWindowCommand.Hide: Window.AppWindow.Hide(); break;
                case RunnerWindowCommand.Exit: Window.Close(); break;
            }
        };
        Window.Activate();
        _ = _services.GetRequiredService<MainWindowViewModel>().InitializeAsync();
    }
}
