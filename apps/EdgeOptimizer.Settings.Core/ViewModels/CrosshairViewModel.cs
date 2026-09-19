using System.IO;
using System.Windows.Input;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using EdgeOptimizer.Settings.Core.Models;
using EdgeOptimizer.Settings.Core.Services;

namespace EdgeOptimizer.Settings.Core.ViewModels;

public sealed class CrosshairViewModel : ObservableObject
{
    private readonly IFilePicker _filePicker;
    private readonly Func<Task> _saveAsync;
    private ProfileWorkspace? _profile;
    private string _feedbackText = "Preview values are stored in memory only.";

    public CrosshairViewModel(IFilePicker filePicker, Func<Task>? saveAsync = null)
    {
        _filePicker = filePicker;
        _saveAsync = saveAsync ?? (() => Task.CompletedTask);
        MoveCommand = new RelayCommand<string>(Move);
        CenterCommand = new RelayCommand(Center);
        ReplaceImageCommand = new AsyncRelayCommand(ReplaceImageAsync);
        RemoveImageCommand = new RelayCommand(RemoveImage);
        HidePreviewCommand = new RelayCommand(HidePreview);
        ResetCommand = new RelayCommand(Reset);
        SaveCommand = new AsyncRelayCommand(SaveAsync);
    }

    public ICommand MoveCommand { get; }
    public ICommand CenterCommand { get; }
    public IAsyncRelayCommand ReplaceImageCommand { get; }
    public ICommand RemoveImageCommand { get; }
    public ICommand HidePreviewCommand { get; }
    public ICommand ResetCommand { get; }
    public ICommand SaveCommand { get; }

    public int XOffset
    {
        get => _profile?.CrosshairXOffset ?? 0;
        set
        {
            if (_profile is null || _profile.CrosshairXOffset == Math.Clamp(value, -500, 500)) return;
            _profile.CrosshairXOffset = value;
            OnPropertyChanged();
            OnPropertyChanged(nameof(OffsetSummary));
        }
    }

    public int YOffset
    {
        get => _profile?.CrosshairYOffset ?? 0;
        set
        {
            if (_profile is null || _profile.CrosshairYOffset == Math.Clamp(value, -500, 500)) return;
            _profile.CrosshairYOffset = value;
            OnPropertyChanged();
            OnPropertyChanged(nameof(OffsetSummary));
        }
    }

    public bool OverlayEnabled
    {
        get => _profile?.OverlayEnabled ?? false;
        set
        {
            if (_profile is null || _profile.OverlayEnabled == value) return;
            _profile.OverlayEnabled = value;
            OnPropertyChanged();
        }
    }

    public string ImageName => _profile?.CrosshairImageName ?? "No image selected";
    public string? ImagePath => _profile?.CrosshairImagePath;
    public string OffsetSummary => $"Offset X  {XOffset}  •  Y  {YOffset}";
    public string FeedbackText { get => _feedbackText; private set => SetProperty(ref _feedbackText, value); }

    public void LoadProfile(ProfileWorkspace profile)
    {
        _profile = profile;
        OnPropertyChanged(nameof(XOffset));
        OnPropertyChanged(nameof(YOffset));
        OnPropertyChanged(nameof(OverlayEnabled));
        OnPropertyChanged(nameof(ImageName));
        OnPropertyChanged(nameof(ImagePath));
        OnPropertyChanged(nameof(OffsetSummary));
    }

    private void Move(string? direction)
    {
        switch (direction)
        {
            case "Up": YOffset -= 1; break;
            case "Down": YOffset += 1; break;
            case "Left": XOffset -= 1; break;
            case "Right": XOffset += 1; break;
            default: Center(); break;
        }
    }

    private void Center()
    {
        XOffset = 0;
        YOffset = 0;
        FeedbackText = "Crosshair centered in the preview.";
    }

    private async Task ReplaceImageAsync()
    {
        var selectedPath = await _filePicker.PickPngAsync();
        if (string.IsNullOrWhiteSpace(selectedPath) || _profile is null) return;
        _profile.CrosshairImageName = Path.GetFileName(selectedPath);
        _profile.CrosshairImagePath = selectedPath;
        OnPropertyChanged(nameof(ImageName));
        OnPropertyChanged(nameof(ImagePath));
        FeedbackText = "Image selected for preview. Managed asset storage is not connected yet.";
    }

    private void RemoveImage()
    {
        if (_profile is null) return;
        _profile.CrosshairImageName = "No image selected";
        _profile.CrosshairImagePath = null;
        OnPropertyChanged(nameof(ImageName));
        OnPropertyChanged(nameof(ImagePath));
        FeedbackText = "Crosshair image removed from preview state.";
    }

    private void HidePreview()
    {
        OverlayEnabled = false;
        FeedbackText = "Preview hidden.";
    }

    private void Reset()
    {
        if (_profile is null) return;
        OverlayEnabled = true;
        _profile.CrosshairImageName = "dot-crosshair.png";
        _profile.CrosshairImagePath = null;
        OnPropertyChanged(nameof(ImageName));
        OnPropertyChanged(nameof(ImagePath));
        Center();
        FeedbackText = "Crosshair preview reset.";
    }

    private async Task SaveAsync()
    {
        await _saveAsync();
        FeedbackText = "Crosshair settings saved to Runner.";
    }
}
