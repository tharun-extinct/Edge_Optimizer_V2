using System.Collections.ObjectModel;

namespace EdgeOptimizer.Settings.Core.Models;

public sealed class MacroDefinition
{
    public MacroDefinition(string name, string shortcut, IEnumerable<MacroStep> steps)
    {
        Name = name;
        Shortcut = shortcut;
        Steps = new ObservableCollection<MacroStep>(steps);
    }

    public string Name { get; set; }
    public string Shortcut { get; set; }
    public bool IsEnabled { get; set; } = true;
    public MacroRepeatMode RepeatMode { get; set; } = MacroRepeatMode.Once;
    public uint RepeatCount { get; set; } = 1;
    public string StopKey { get; set; } = string.Empty;
    public ObservableCollection<MacroStep> Steps { get; }
}

public sealed record MacroStep(string Action, string Value);

public enum MacroRepeatMode
{
    Once,
    Count,
    UntilKeyPressed,
}
