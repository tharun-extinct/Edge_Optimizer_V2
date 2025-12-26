# Gaming Optimizer

A lightweight Rust desktop application for Windows that optimizes gaming performance by managing system processes and providing a customizable crosshair overlay.

## Features

- **Process Management**: Automatically kill unwanted system processes to free up resources
- **Crosshair Overlay**: Display custom PNG crosshairs (100x100) with click-through functionality
- **Gaming Profiles**: Create multiple profiles with different optimization settings
- **System Tray Integration**: Quick access to all features from the system tray
- **Safety Protection**: Built-in blocklist prevents killing critical system processes
- **Low Memory Footprint**: Optimized for minimal resource usage

## Tech Stack

- **Language**: Rust (2021 edition)
- **Platform**: Windows only
- **Dependencies**:
  - `tray-icon` - System tray functionality
  - `winit` - Window creation for overlay
  - `image` - PNG image loading
  - `sysinfo` - Process management
  - `softbuffer` - Software rendering

## Build Requirements

- Rust 1.70+ (with cargo)
- Windows 10/11
- Visual Studio Build Tools (for Windows API)

## Building

### Debug Build
```bash
cargo build
```

### Release Build (Optimized)
```bash
cargo build --release
```

The release build is optimized for:
- Small binary size (LTO enabled, symbols stripped)
- Fast execution (opt-level 3)
- Low memory footprint

### Output Location
- Debug: `target/debug/gaming_optimizer.exe`
- Release: `target/release/gaming_optimizer.exe`

## Running

```bash
# Run debug build
cargo run

# Run release build
cargo run --release

# Or run the executable directly
./target/release/gaming_optimizer.exe
```

The application will start in the system tray. Look for the Gaming Optimizer icon in your Windows system tray.

## Configuration

### Data Directory

The application stores configuration and profiles in:
```
%APPDATA%\GamingOptimizer\
├── config.json          # Application state
├── profiles.json        # Gaming profiles
└── crosshairs/          # Optional: Store crosshair images here
```

### Creating Profiles

1. Right-click the tray icon and select **Settings**
2. This opens the data directory in File Explorer
3. Create or edit `profiles.json` with your gaming profiles

### Example profiles.json

```json
[
  {
    "name": "CS:GO",
    "processes_to_kill": ["discord.exe", "spotify.exe", "chrome.exe"],
    "crosshair_image_path": "C:/Users/YourName/AppData/Roaming/GamingOptimizer/crosshairs/csgo.png",
    "crosshair_x_offset": 0,
    "crosshair_y_offset": 2,
    "overlay_enabled": true
  },
  {
    "name": "Valorant",
    "processes_to_kill": ["discord.exe", "obs64.exe"],
    "crosshair_image_path": null,
    "crosshair_x_offset": 0,
    "crosshair_y_offset": 0,
    "overlay_enabled": false
  }
]
```

### Profile Fields

- **name**: Profile name (1-50 characters, must be unique)
- **processes_to_kill**: Array of process names to terminate (e.g., "discord.exe")
- **crosshair_image_path**: Path to 100x100 PNG crosshair image (or `null` for no crosshair)
- **crosshair_x_offset**: Horizontal offset in pixels from center (-500 to +500)
- **crosshair_y_offset**: Vertical offset in pixels from center (-500 to +500)
- **overlay_enabled**: Whether to show overlay when profile is active

## Usage

### Activating a Profile

1. Right-click the tray icon
2. Hover over **Profiles**
3. Select your desired profile
4. The application will:
   - Kill all specified processes
   - Show the crosshair overlay (if configured)
   - Update the tray tooltip

### Toggling Overlay

1. Right-click the tray icon
2. Click **Overlay Visible** to toggle on/off
3. This option is only enabled when a profile with a crosshair is active

### Deactivating Profile

1. Right-click the tray icon
2. Hover over **Profiles**
3. Select **(None)**

### Exiting

1. Right-click the tray icon
2. Click **Exit**

## Protected Processes

The following critical Windows processes cannot be killed for system stability:

- `csrss.exe` - Client Server Runtime
- `dwm.exe` - Desktop Window Manager
- `explorer.exe` - Windows Explorer
- `lsass.exe` - Local Security Authority
- `services.exe` - Services Control Manager
- `smss.exe` - Session Manager
- `system` - System process
- `wininit.exe` - Windows Init
- `winlogon.exe` - Windows Logon
- `svchost.exe` - Service Host

## Crosshair Requirements

- **Format**: PNG with transparency
- **Size**: Exactly 100x100 pixels
- **Location**: Can be anywhere accessible, but recommended to store in `%APPDATA%\GamingOptimizer\crosshairs\`

Invalid images will be rejected with an error message.

## Troubleshooting

### Application doesn't start
- Check that you're running on Windows
- Ensure all dependencies are built correctly
- Check Windows Event Viewer for errors

### Crosshair not showing
- Verify the PNG is exactly 100x100 pixels
- Check that the file path in profiles.json is correct
- Ensure overlay_enabled is set to true
- Check application logs for image loading errors

### Process won't kill
- Verify the process name is correct (case-insensitive, include .exe)
- Check if the process is in the protected list
- Ensure you have permission to terminate the process
- Try running as administrator

### Profiles not loading
- Check that profiles.json is valid JSON
- Verify file is in `%APPDATA%\GamingOptimizer\`
- Check for syntax errors in JSON

## Development

### Project Structure

```
Gaming_optimizer/
├── Cargo.toml           # Dependencies and build config
├── src/
│   ├── main.rs          # Application entry point and event loop
│   ├── tray.rs          # System tray management
│   ├── overlay.rs       # Crosshair overlay window
│   ├── process.rs       # Process killing logic
│   ├── profile.rs       # Gaming profile management
│   └── config.rs        # Configuration and storage
└── README.md
```

### Testing

```bash
# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

### Code Style

```bash
# Format code
cargo fmt

# Run linter
cargo clippy
```

## Known Limitations

- **Windows only** - Uses Windows-specific APIs
- **Single monitor** - Overlay centers on primary monitor only
- **Manual profile editing** - No GUI settings window (uses JSON files)
- **No hotkeys** - Overlay toggle only via tray menu
- **No auto-activation** - Profiles must be manually activated

## Future Enhancements

Potential features for future versions:

- GUI settings window with visual editor
- Global hotkey support for overlay toggle
- Multi-monitor support with monitor selection
- Customizable crosshair sizes
- Game detection and auto-profile activation
- Performance metrics display
- Multiple simultaneous overlays
- Crosshair opacity adjustment

## License

This project is provided as-is for gaming optimization purposes.

## Support

For issues or questions, please check:
1. This README for common solutions
2. The troubleshooting section
3. Application logs in the data directory