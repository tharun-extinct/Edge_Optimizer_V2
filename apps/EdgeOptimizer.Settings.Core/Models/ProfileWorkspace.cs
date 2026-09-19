using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;

namespace EdgeOptimizer.Settings.Core.Models;

public sealed class ProfileWorkspace : ObservableObject
{
    private bool _isActive;
    private bool _overlayEnabled = true;
    private string _crosshairImageName = "dot-crosshair.png";
    private string? _crosshairImagePath;
    private int _crosshairXOffset;
    private int _crosshairYOffset;
    private bool _fanBoostEnabled = true;
    private bool _recycleBinEnabled = true;
    private bool _browserCacheEnabled = true;

    public ProfileWorkspace(string name, bool isActive)
    {
        Name = name;
    
        _isActive = isActive;
        Macros.Add(new MacroDefinition("Build combo", "Ctrl + F8", new[]
        {
            new MacroStep("Key down", "W"),
            new MacroStep("Wait", "120 ms"),
            new MacroStep("Key press", "Space"),
            new MacroStep("Key up", "W")
        }));
        Macros.Add(new MacroDefinition("Quick heal", "Unassigned", new[]
        {
            new MacroStep("Key press", "H")
        }));

        Processes.Add(new ProcessItem("Discord.exe", "0.4%", "156.2 MB", true));
        Processes.Add(new ProcessItem("chrome.exe", "1.2%", "512.7 MB", true));
        Processes.Add(new ProcessItem("Spotify.exe", "0.3%", "123.4 MB", true));
        Processes.Add(new ProcessItem("Steam.exe", "0.6%", "287.9 MB", false));
    }

    public string Name { get; }
    public ObservableCollection<MacroDefinition> Macros { get; } = new();
    public ObservableCollection<ProcessItem> Processes { get; } = new();

    public bool IsActive { get => _isActive; set => SetProperty(ref _isActive, value); }
    public bool OverlayEnabled { get => _overlayEnabled; set => SetProperty(ref _overlayEnabled, value); }
    public string CrosshairImageName { get => _crosshairImageName; set => SetProperty(ref _crosshairImageName, value); }
    public string? CrosshairImagePath { get => _crosshairImagePath; set => SetProperty(ref _crosshairImagePath, value); }
    public int CrosshairXOffset { get => _crosshairXOffset; set => SetProperty(ref _crosshairXOffset, Math.Clamp(value, -500, 500)); }
    public int CrosshairYOffset { get => _crosshairYOffset; set => SetProperty(ref _crosshairYOffset, Math.Clamp(value, -500, 500)); }
    public bool FanBoostEnabled { get => _fanBoostEnabled; set => SetProperty(ref _fanBoostEnabled, value); }
    public bool RecycleBinEnabled { get => _recycleBinEnabled; set => SetProperty(ref _recycleBinEnabled, value); }
    public bool BrowserCacheEnabled { get => _browserCacheEnabled; set => SetProperty(ref _browserCacheEnabled, value); }
}
