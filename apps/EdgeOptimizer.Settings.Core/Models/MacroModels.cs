using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;

namespace EdgeOptimizer.Settings.Core.Models;

public sealed class MacroDefinition : ObservableObject
{
    private string _name;
    private string _shortcut;
    private bool _isEnabled = true;
    private MacroRepeatMode _repeatMode = MacroRepeatMode.Once;
    private uint _repeatCount = 1;
    private string _stopKey = string.Empty;

    public MacroDefinition(string name, string shortcut, IEnumerable<MacroStep> steps)
    {
        _name = name;
        _shortcut = shortcut;
        Steps = new ObservableCollection<MacroStep>(steps);
    }

    public string Name { get => _name; set => SetProperty(ref _name, value); }
    public string Shortcut { get => _shortcut; set => SetProperty(ref _shortcut, value); }
    public bool IsEnabled { get => _isEnabled; set => SetProperty(ref _isEnabled, value); }
    public MacroRepeatMode RepeatMode { get => _repeatMode; set { if (SetProperty(ref _repeatMode, value)) OnPropertyChanged(nameof(RepeatModeIndex)); } }
    public int RepeatModeIndex { get => (int)RepeatMode; set => RepeatMode = (MacroRepeatMode)Math.Clamp(value, 0, 2); }
    public uint RepeatCount { get => _repeatCount; set { if (SetProperty(ref _repeatCount, Math.Clamp(value, 1u, 100u))) OnPropertyChanged(nameof(RepeatCountValue)); } }
    public double RepeatCountValue { get => RepeatCount; set => RepeatCount = checked((uint)Math.Clamp(value, 1, 100)); }
    public string StopKey { get => _stopKey; set => SetProperty(ref _stopKey, value); }
    public ObservableCollection<MacroStep> Steps { get; }
}

public sealed class MacroStep : ObservableObject
{
    private string _action;
    private string _value;

    public MacroStep(string action, string value) { _action = action; _value = value; }
    public string Action { get => _action; set => SetProperty(ref _action, value); }
    public string Value { get => _value; set => SetProperty(ref _value, value); }
}

public enum MacroRepeatMode { Once, Count, UntilKeyPressed }
