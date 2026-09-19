using System.Collections.ObjectModel;
using System.Windows.Input;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using EdgeOptimizer.Settings.Core.Models;

namespace EdgeOptimizer.Settings.Core.ViewModels;

public sealed class MacrosViewModel : ObservableObject
{
    private readonly Func<Task> _saveAsync;
    private ProfileWorkspace? _profile;
    private MacroDefinition? _selectedMacro;
    private string _macroSearch = string.Empty;
    private string _feedbackText = "Macro edits are stored in memory only.";

    public MacrosViewModel(Func<Task>? saveAsync = null)
    {
        _saveAsync = saveAsync ?? (() => Task.CompletedTask);
        NewMacroCommand = new RelayCommand(NewMacro);
        AddActionCommand = new RelayCommand(AddAction, () => SelectedMacro is not null);
        DeleteStepCommand = new RelayCommand<MacroStep>(DeleteStep, step => SelectedMacro is not null && step is not null);
        DeleteMacroCommand = new RelayCommand(DeleteMacro, () => SelectedMacro is not null);
        DuplicateMacroCommand = new RelayCommand(DuplicateMacro, () => SelectedMacro is not null);
        SaveMacroCommand = new AsyncRelayCommand(SaveAsync);
        ChangeShortcutCommand = new RelayCommand(() => FeedbackText = "Shortcut capture will be connected through the macro worker contract.");
    }

    public ICommand NewMacroCommand { get; }
    public ICommand AddActionCommand { get; }
    public ICommand DeleteStepCommand { get; }
    public ICommand DeleteMacroCommand { get; }
    public ICommand DuplicateMacroCommand { get; }
    public ICommand SaveMacroCommand { get; }
    public ICommand ChangeShortcutCommand { get; }

    public ObservableCollection<MacroDefinition> Macros => _profile?.Macros ?? EmptyMacros;
    private static ObservableCollection<MacroDefinition> EmptyMacros { get; } = new();
    public IEnumerable<MacroDefinition> FilteredMacros => Macros.Where(FilterMacro);

    public MacroDefinition? SelectedMacro
    {
        get => _selectedMacro;
        set
        {
            if (SetProperty(ref _selectedMacro, value)) NotifyCommandState();
        }
    }

    public string MacroSearch
    {
        get => _macroSearch;
        set
        {
            if (SetProperty(ref _macroSearch, value)) OnPropertyChanged(nameof(FilteredMacros));
        }
    }

    public string FeedbackText { get => _feedbackText; private set => SetProperty(ref _feedbackText, value); }

    public void LoadProfile(ProfileWorkspace profile)
    {
        _profile = profile;
        SelectedMacro = profile.Macros.FirstOrDefault();
        OnPropertyChanged(nameof(Macros));
        OnPropertyChanged(nameof(FilteredMacros));
    }

    private bool FilterMacro(MacroDefinition macro) =>
        string.IsNullOrWhiteSpace(MacroSearch) || macro.Name.Contains(MacroSearch, StringComparison.OrdinalIgnoreCase);

    private void NewMacro()
    {
        if (_profile is null) return;
        var macro = new MacroDefinition($"New macro {Macros.Count + 1}", "Unassigned", Array.Empty<MacroStep>());
        Macros.Add(macro);
        SelectedMacro = macro;
        OnPropertyChanged(nameof(FilteredMacros));
        FeedbackText = "Created a temporary macro.";
    }

    private void AddAction()
    {
        SelectedMacro?.Steps.Add(new MacroStep("Key press", "Unassigned"));
        FeedbackText = "Added an action placeholder.";
    }

    private void DeleteStep(MacroStep? step)
    {
        if (SelectedMacro is null || step is null) return;
        SelectedMacro.Steps.Remove(step);
        FeedbackText = "Removed the action from preview state.";
    }

    private void DeleteMacro()
    {
        if (SelectedMacro is null) return;
        var index = Macros.IndexOf(SelectedMacro);
        Macros.Remove(SelectedMacro);
        SelectedMacro = Macros.Count == 0 ? null : Macros[Math.Clamp(index, 0, Macros.Count - 1)];
        OnPropertyChanged(nameof(FilteredMacros));
        FeedbackText = "Deleted the macro from preview state.";
    }

    private void DuplicateMacro()
    {
        if (SelectedMacro is null) return;
        var duplicate = new MacroDefinition($"{SelectedMacro.Name} copy", "Unassigned", SelectedMacro.Steps.ToArray());
        Macros.Add(duplicate);
        SelectedMacro = duplicate;
        OnPropertyChanged(nameof(FilteredMacros));
        FeedbackText = "Duplicated the macro in preview state.";
    }

    private void NotifyCommandState()
    {
        ((RelayCommand)AddActionCommand).NotifyCanExecuteChanged();
        ((RelayCommand<MacroStep>)DeleteStepCommand).NotifyCanExecuteChanged();
        ((RelayCommand)DeleteMacroCommand).NotifyCanExecuteChanged();
        ((RelayCommand)DuplicateMacroCommand).NotifyCanExecuteChanged();
    }

    private async Task SaveAsync()
    {
        await _saveAsync();
        FeedbackText = "Macro changes saved to Runner.";
    }
}
