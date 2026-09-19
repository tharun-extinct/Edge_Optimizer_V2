using System.IO.Pipes;
using System.Text;
using EdgeOptimizer.Settings.Core.Models;
using EdgeOptimizer.Settings.Core.Services;
using Microsoft.UI.Dispatching;

namespace EdgeOptimizer.Settings.WinUI.Services;

/// <summary>
/// Compatibility client for Runner's existing Rust/Bincode pipe. This restores
/// behavior during migration; the shared Protobuf contract remains the target.
/// </summary>
public sealed class TransitionalBincodeRunnerClient : IRunnerClient, IAsyncDisposable
{
    private const string PipeName = "EdgeOptimizerIPC";
    private const int MaximumMessageBytes = 1024 * 1024;
    private readonly DispatcherQueue _dispatcher;
    private readonly SemaphoreSlim _writeLock = new(1, 1);
    private NamedPipeClientStream? _pipe;
    private CancellationTokenSource? _lifetime;
    private string? _pendingActivation;

    public TransitionalBincodeRunnerClient(DispatcherQueue dispatcher) => _dispatcher = dispatcher;

    public bool IsConnected => _pipe?.IsConnected == true;
    public event EventHandler<bool>? ConnectionChanged;
    public event EventHandler<RunnerSnapshot>? SnapshotReceived;
    public event EventHandler<string>? StatusReceived;
    public event EventHandler<IReadOnlyList<ProcessItem>>? ProcessSnapshotReceived;
    public event EventHandler<string?>? ActiveProfileChanged;
    public event EventHandler<RunnerWindowCommand>? WindowCommandReceived;

    public async Task StartAsync(CancellationToken cancellationToken = default)
    {
        if (IsConnected) return;
        _pipe = new NamedPipeClientStream(".", PipeName, PipeDirection.InOut, PipeOptions.Asynchronous);
        await _pipe.ConnectAsync(3000, cancellationToken);
        _pipe.ReadMode = PipeTransmissionMode.Message;
        _lifetime = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        Post(() => ConnectionChanged?.Invoke(this, true));
        _ = ReadLoopAsync(_lifetime.Token);
        await SendAsync(writer => writer.Write((uint)0), cancellationToken); // GuiToTray::RequestState
    }

    public Task SaveProfilesAsync(IReadOnlyList<ProfileWorkspace> profiles, CancellationToken cancellationToken = default) =>
        SendAsync(writer =>
        {
            writer.Write((uint)1); // GuiToTray::ProfilesUpdated
            writer.Write((ulong)profiles.Count);
            foreach (var profile in profiles) BincodeCodec.WriteProfile(writer, profile);
        }, cancellationToken);

    public Task SetActiveProfileAsync(string? profileName, CancellationToken cancellationToken = default) =>
        SendAsync(writer => { writer.Write((uint)2); BincodeCodec.WriteOptionString(writer, profileName); }, cancellationToken);

    public Task SetOverlayVisibilityAsync(bool visible, CancellationToken cancellationToken = default) =>
        SendAsync(writer => { writer.Write((uint)3); writer.Write(visible); }, cancellationToken);

    public Task ActivateProfileAsync(ProfileWorkspace profile, CancellationToken cancellationToken = default)
    {
        _pendingActivation = profile.Name;
        return SendOrchestrationAsync(writer =>
        {
            writer.Write((uint)0); // SettingsToRunnerCommand::ActivateProfile
            BincodeCodec.WriteProfile(writer, profile);
            writer.Write((byte)0); // game_session_id: None
        }, cancellationToken);
    }

    public Task RequestCleanupAsync(string cleanupKind, CancellationToken cancellationToken = default) =>
        SendOrchestrationAsync(writer =>
        {
            writer.Write((uint)3); // SettingsToRunnerCommand::RequestCleanup
            writer.Write(cleanupKind.Equals("browser-cache", StringComparison.OrdinalIgnoreCase) ? 1u : 0u);
        }, cancellationToken);

    public Task RequestProcessSnapshotAsync(CancellationToken cancellationToken = default) =>
        SendAsync(writer => writer.Write((uint)6), cancellationToken);

    private Task SendOrchestrationAsync(Action<BinaryWriter> writePayload, CancellationToken cancellationToken)
    {
        return SendAsync(writer =>
        {
            writer.Write((uint)5); // GuiToTray::Orchestration
            writer.Write((ushort)2);
            BincodeCodec.WriteString(writer, "edge-settings-winui");
            BincodeCodec.WriteString(writer, $"winui-{DateTimeOffset.UtcNow.ToUnixTimeMilliseconds()}-{Guid.NewGuid():N}");
            writer.Write((ulong)DateTimeOffset.UtcNow.ToUnixTimeMilliseconds());
            writer.Write((uint)0); // AuthContext::InteractiveUser (claim only)
            writePayload(writer);
        }, cancellationToken);
    }

    private async Task SendAsync(Action<BinaryWriter> writeMessage, CancellationToken cancellationToken)
    {
        var pipe = _pipe;
        if (pipe?.IsConnected != true) throw new InvalidOperationException("Runner is not connected.");
        using var buffer = new MemoryStream();
        using (var writer = new BinaryWriter(buffer, Encoding.UTF8, leaveOpen: true)) writeMessage(writer);
        if (buffer.Length > MaximumMessageBytes) throw new InvalidDataException("Runner message exceeds the 1 MiB limit.");
        await _writeLock.WaitAsync(cancellationToken);
        try
        {
            await pipe.WriteAsync(buffer.GetBuffer().AsMemory(0, checked((int)buffer.Length)), cancellationToken);
            await pipe.FlushAsync(cancellationToken);
        }
        finally { _writeLock.Release(); }
    }

    private async Task ReadLoopAsync(CancellationToken cancellationToken)
    {
        try
        {
            while (!cancellationToken.IsCancellationRequested && _pipe?.IsConnected == true)
            {
                using var message = new MemoryStream();
                var chunk = new byte[8192];
                do
                {
                    var count = await _pipe.ReadAsync(chunk, cancellationToken);
                    if (count == 0) return;
                    message.Write(chunk, 0, count);
                    if (message.Length > MaximumMessageBytes) throw new InvalidDataException("Runner response exceeds the 1 MiB limit.");
                } while (!_pipe.IsMessageComplete);
                message.Position = 0;
                using var reader = new BinaryReader(message, Encoding.UTF8, leaveOpen: false);
                HandleMessage(reader);
            }
        }
        catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested) { }
        catch (Exception error) { Post(() => StatusReceived?.Invoke(this, $"Runner connection failed: {error.Message}")); }
        finally { Post(() => ConnectionChanged?.Invoke(this, false)); }
    }

    private void HandleMessage(BinaryReader reader)
    {
        switch (reader.ReadUInt32())
        {
            case 0: // StateSnapshot
                var count = checked((int)reader.ReadUInt64());
                var profiles = new List<ProfileWorkspace>(count);
                for (var index = 0; index < count; index++) profiles.Add(BincodeCodec.ReadProfile(reader));
                var active = BincodeCodec.ReadOptionString(reader);
                var overlayVisible = reader.ReadBoolean();
                Post(() => SnapshotReceived?.Invoke(this, new RunnerSnapshot(profiles, active, overlayVisible)));
                break;
            case 1:
                var profileName = BincodeCodec.ReadString(reader);
                Post(() => ActiveProfileChanged?.Invoke(this, profileName));
                Post(() => StatusReceived?.Invoke(this, $"Runner activated {profileName}."));
                break;
            case 2:
                Post(() => ActiveProfileChanged?.Invoke(this, null));
                Post(() => StatusReceived?.Invoke(this, "Runner deactivated the current profile."));
                break;
            case 3: Post(() => StatusReceived?.Invoke(this, "Runner changed overlay visibility.")); break;
            case 4:
            case 7: Post(() => WindowCommandReceived?.Invoke(this, RunnerWindowCommand.BringToFront)); break;
            case 5: Post(() => WindowCommandReceived?.Invoke(this, RunnerWindowCommand.Show)); break;
            case 6: Post(() => WindowCommandReceived?.Invoke(this, RunnerWindowCommand.Hide)); break;
            case 8: Post(() => WindowCommandReceived?.Invoke(this, RunnerWindowCommand.Exit)); break;
            case 9: ReadOrchestrationEvent(reader); break;
            case 10:
                var processCount = checked((int)reader.ReadUInt64());
                var processes = new List<ProcessItem>(processCount);
                for (var index = 0; index < processCount; index++)
                {
                    var name = BincodeCodec.ReadString(reader);
                    var cpu = reader.ReadSingle();
                    var memoryKb = reader.ReadUInt64();
                    processes.Add(new ProcessItem(name, $"{cpu:F1}%", $"{memoryKb / 1024d:F1} MB", false));
                }
                Post(() => ProcessSnapshotReceived?.Invoke(this, processes));
                break;
            default: throw new InvalidDataException("Runner sent an unknown response.");
        }
    }

    private void ReadOrchestrationEvent(BinaryReader reader)
    {
        var version = reader.ReadUInt16();
        _ = BincodeCodec.ReadString(reader); // client id
        _ = BincodeCodec.ReadString(reader); // request id
        _ = reader.ReadUInt64();
        _ = reader.ReadUInt32(); // serialized identity claim
        if (version != 2) throw new InvalidDataException($"Unsupported Runner protocol version {version}.");
        switch (reader.ReadUInt32())
        {
            case 0:
                var state = (EngineState)reader.ReadUInt32();
                Post(() => StatusReceived?.Invoke(this, $"Engine state: {state}."));
                break;
            case 1: ReadOperationResult(reader, "Optimization"); break;
            case 2: ReadOperationResult(reader, "Cleanup"); break;
            case 3:
            case 4:
                var message = BincodeCodec.ReadString(reader);
                Post(() => StatusReceived?.Invoke(this, message));
                break;
            default: throw new InvalidDataException("Runner sent an unknown orchestration event.");
        }
    }

    private void ReadOperationResult(BinaryReader reader, string operation)
    {
        _ = BincodeCodec.ReadString(reader);
        var success = reader.ReadBoolean();
        var summary = BincodeCodec.ReadString(reader);
        BincodeCodec.SkipStringVector(reader);
        BincodeCodec.SkipStringVector(reader);
        BincodeCodec.SkipStringVector(reader);
        BincodeCodec.SkipStringVector(reader);
        Post(() => StatusReceived?.Invoke(this, $"{operation} {(success ? "completed" : "failed")}: {summary}"));
        if (operation == "Optimization")
        {
            var activated = success ? _pendingActivation : null;
            _pendingActivation = null;
            if (success) Post(() => ActiveProfileChanged?.Invoke(this, activated));
        }
    }

    private void Post(Action action)
    {
        if (!_dispatcher.TryEnqueue(() => action())) action();
    }

    public async ValueTask DisposeAsync()
    {
        _lifetime?.Cancel();
        if (_pipe is not null) await _pipe.DisposeAsync();
        _writeLock.Dispose();
        _lifetime?.Dispose();
    }

    private enum EngineState { Starting, Ready, Degraded, Disconnected }
}

internal static class BincodeCodec
{
    public static void WriteString(BinaryWriter writer, string value)
    {
        var bytes = Encoding.UTF8.GetBytes(value);
        writer.Write((ulong)bytes.Length);
        writer.Write(bytes);
    }

    public static string ReadString(BinaryReader reader)
    {
        var length = checked((int)reader.ReadUInt64());
        return Encoding.UTF8.GetString(reader.ReadBytes(length));
    }

    public static void WriteOptionString(BinaryWriter writer, string? value)
    {
        writer.Write((byte)(value is null ? 0 : 1));
        if (value is not null) WriteString(writer, value);
    }

    public static string? ReadOptionString(BinaryReader reader) => reader.ReadByte() == 0 ? null : ReadString(reader);

    public static void WriteProfile(BinaryWriter writer, ProfileWorkspace profile)
    {
        WriteString(writer, profile.Name);
        var selectedProcesses = profile.Processes.Where(process => process.IsSelected).Select(process => process.Name).ToArray();
        writer.Write((ulong)selectedProcesses.Length);
        foreach (var process in selectedProcesses) WriteString(writer, process);
        WriteOptionString(writer, profile.CrosshairImagePath);
        writer.Write(profile.CrosshairXOffset);
        writer.Write(profile.CrosshairYOffset);
        writer.Write(profile.OverlayEnabled);
        writer.Write(profile.FanBoostEnabled);
        writer.Write((ulong)profile.Macros.Count);
        foreach (var macro in profile.Macros) WriteMacro(writer, macro);
    }

    public static ProfileWorkspace ReadProfile(BinaryReader reader)
    {
        var profile = new ProfileWorkspace(ReadString(reader), false);
        profile.Processes.Clear();
        var processCount = checked((int)reader.ReadUInt64());
        for (var index = 0; index < processCount; index++) profile.Processes.Add(new ProcessItem(ReadString(reader), "—", "—", true));
        profile.CrosshairImagePath = ReadOptionString(reader);
        profile.CrosshairImageName = profile.CrosshairImagePath is null ? "No image selected" : Path.GetFileName(profile.CrosshairImagePath);
        profile.CrosshairXOffset = reader.ReadInt32();
        profile.CrosshairYOffset = reader.ReadInt32();
        profile.OverlayEnabled = reader.ReadBoolean();
        profile.FanBoostEnabled = reader.ReadBoolean();
        profile.Macros.Clear();
        var macroCount = checked((int)reader.ReadUInt64());
        for (var index = 0; index < macroCount; index++) profile.Macros.Add(ReadMacro(reader));
        return profile;
    }

    private static void WriteMacro(BinaryWriter writer, MacroDefinition macro)
    {
        WriteString(writer, macro.Name);
        writer.Write(macro.IsEnabled);
        writer.Write((ulong)macro.Steps.Count);
        foreach (var step in macro.Steps) WriteStep(writer, step);
        WriteShortcut(writer, macro.Shortcut);
        writer.Write((uint)macro.RepeatMode);
        if (macro.RepeatMode == MacroRepeatMode.Count) writer.Write(macro.RepeatCount);
        if (macro.RepeatMode == MacroRepeatMode.UntilKeyPressed) WriteString(writer, macro.StopKey);
    }

    private static MacroDefinition ReadMacro(BinaryReader reader)
    {
        var name = ReadString(reader);
        var enabled = reader.ReadBoolean();
        var actionCount = checked((int)reader.ReadUInt64());
        var steps = new List<MacroStep>(actionCount);
        for (var index = 0; index < actionCount; index++) steps.Add(ReadStep(reader));
        var shortcut = ReadShortcut(reader);
        var repeatMode = (MacroRepeatMode)reader.ReadUInt32();
        var repeatCount = repeatMode == MacroRepeatMode.Count ? reader.ReadUInt32() : 1u;
        var stopKey = repeatMode == MacroRepeatMode.UntilKeyPressed ? ReadString(reader) : string.Empty;
        return new MacroDefinition(name, shortcut, steps) { IsEnabled = enabled, RepeatMode = repeatMode, RepeatCount = repeatCount, StopKey = stopKey };
    }

    private static void WriteStep(BinaryWriter writer, MacroStep step)
    {
        if (step.Action.Equals("Key up", StringComparison.OrdinalIgnoreCase)) { writer.Write(1u); WriteString(writer, step.Value); writer.Write(0ul); }
        else if (step.Action.Equals("Wait", StringComparison.OrdinalIgnoreCase)) { writer.Write(4u); writer.Write(ParseUnsigned(step.Value)); }
        else { writer.Write(0u); WriteString(writer, step.Value); writer.Write(0ul); }
    }

    private static MacroStep ReadStep(BinaryReader reader) => reader.ReadUInt32() switch
    {
        0 => new MacroStep("Key down", ReadStringWithTrailingDelay(reader)),
        1 => new MacroStep("Key up", ReadStringWithTrailingDelay(reader)),
        2 => ReadMouseStep(reader),
        3 => new MacroStep("Mouse move", $"{reader.ReadInt32()}, {reader.ReadInt32()}"),
        4 => new MacroStep("Wait", $"{reader.ReadUInt64()} ms"),
        _ => throw new InvalidDataException("Unknown macro action."),
    };

    private static string ReadStringWithTrailingDelay(BinaryReader reader) { var value = ReadString(reader); _ = reader.ReadUInt64(); return value; }
    private static MacroStep ReadMouseStep(BinaryReader reader) { var button = reader.ReadUInt32(); var down = reader.ReadBoolean(); return new MacroStep(down ? "Mouse down" : "Mouse up", button switch { 0 => "Left", 1 => "Right", _ => "Middle" }); }

    private static void WriteShortcut(BinaryWriter writer, string display)
    {
        if (string.IsNullOrWhiteSpace(display) || display.Equals("Unassigned", StringComparison.OrdinalIgnoreCase) || display.Equals("Not set", StringComparison.OrdinalIgnoreCase)) { writer.Write((byte)0); return; }
        var parts = display.Split('+', StringSplitOptions.TrimEntries | StringSplitOptions.RemoveEmptyEntries);
        writer.Write((byte)1);
        writer.Write(parts.Any(part => part.Equals("Ctrl", StringComparison.OrdinalIgnoreCase)));
        writer.Write(parts.Any(part => part.Equals("Alt", StringComparison.OrdinalIgnoreCase)));
        writer.Write(parts.Any(part => part.Equals("Shift", StringComparison.OrdinalIgnoreCase)));
        writer.Write(parts.Any(part => part.Equals("Win", StringComparison.OrdinalIgnoreCase)));
        WriteString(writer, parts.LastOrDefault() ?? string.Empty);
    }

    private static string ReadShortcut(BinaryReader reader)
    {
        if (reader.ReadByte() == 0) return "Unassigned";
        var parts = new List<string>();
        if (reader.ReadBoolean()) parts.Add("Ctrl");
        if (reader.ReadBoolean()) parts.Add("Alt");
        if (reader.ReadBoolean()) parts.Add("Shift");
        if (reader.ReadBoolean()) parts.Add("Win");
        parts.Add(ReadString(reader));
        return string.Join(" + ", parts);
    }

    public static void SkipStringVector(BinaryReader reader)
    {
        var count = checked((int)reader.ReadUInt64());
        for (var index = 0; index < count; index++) _ = ReadString(reader);
    }

    private static ulong ParseUnsigned(string value) => ulong.TryParse(new string(value.TakeWhile(char.IsDigit).ToArray()), out var parsed) ? parsed : 0;
}
