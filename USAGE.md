# Gaming Optimizer - Usage Guide

## Quick Start

1. **Build the application**
   ```bash
   cargo build --release
   ```

2. **Run the application**
   ```bash
   ./target/release/gaming_optimizer.exe
   ```
   The application will start minimized in your system tray (look for the icon in the bottom-right corner of your screen).

3. **Create your first profile**
   - Right-click the tray icon
   - Click "Settings" to open the configuration folder
   - Create a `profiles.json` file (see example below)

4. **Activate a profile**
   - Right-click the tray icon
   - Hover over "Profiles"
   - Click on your profile name

## Creating Gaming Profiles

### Step-by-Step Profile Creation

1. **Open Settings Folder**
   - Right-click the Gaming Optimizer tray icon
   - Click "⚙ Settings"
   - Windows Explorer will open to `%APPDATA%\GamingOptimizer\`

2. **Create profiles.json**
   - In the opened folder, create a new file named `profiles.json`
   - Copy the example below or use `profiles.example.json` as a template

3. **Edit Your Profile**
   ```json
   [
     {
       "name": "My Gaming Profile",
       "processes_to_kill": ["discord.exe", "chrome.exe"],
       "crosshair_image_path": null,
       "crosshair_x_offset": 0,
       "crosshair_y_offset": 0,
       "overlay_enabled": false
     }
   ]
   ```

4. **Save and Reload**
   - Save the file
   - The application automatically detects changes
   - Right-click the tray icon to see your new profile

### Profile Configuration Options

#### name (required)
The display name for your profile.
- Must be 1-50 characters
- Must be unique
- Example: `"CS:GO Performance"`

#### processes_to_kill (required)
Array of process names to terminate when profile is activated.
- Use full process names with `.exe` extension
- Case-insensitive matching
- Protected system processes are automatically blocked
- Example: `["discord.exe", "spotify.exe", "chrome.exe"]`

**Finding Process Names:**
1. Open Task Manager (Ctrl + Shift + Esc)
2. Go to "Details" tab
3. Look in the "Name" column
4. Copy the exact name (e.g., "Discord.exe" or "discord.exe")

#### crosshair_image_path (optional)
Path to your custom crosshair PNG image.
- Set to `null` if you don't want a crosshair
- Must be exactly 100x100 pixels
- Must be a PNG file with transparency
- Use forward slashes `/` or escaped backslashes `\\` in paths
- Example: `"C:/Users/YourName/AppData/Roaming/GamingOptimizer/crosshairs/csgo.png"`
- Example (no crosshair): `null`

#### crosshair_x_offset (required)
Horizontal offset from screen center in pixels.
- Range: -500 to +500
- Positive values move right
- Negative values move left
- Example: `10` (moves crosshair 10 pixels right)

#### crosshair_y_offset (required)
Vertical offset from screen center in pixels.
- Range: -500 to +500
- Positive values move down
- Negative values move up
- Example: `-5` (moves crosshair 5 pixels up)

#### overlay_enabled (required)
Whether to show the overlay when this profile is active.
- `true` = show overlay (if crosshair_image_path is set)
- `false` = don't show overlay
- Example: `true`

## Creating Custom Crosshairs

### Requirements
- Image format: PNG
- Dimensions: Exactly 100x100 pixels
- Transparency supported
- Maximum file size: No limit, but smaller is better

### Recommended Tools
- **GIMP** (Free): Export as PNG with transparency
- **Paint.NET** (Free): Save as PNG-24
- **Photoshop**: Export as PNG with transparency
- **Online Tools**: Various crosshair generators available

### Creating a Simple Crosshair

1. **Using GIMP (Free)**
   - Create new image: 100x100 pixels
   - Set background to transparent
   - Draw your crosshair using line/shape tools
   - Export as PNG

2. **Using Online Generators**
   - Search for "crosshair generator"
   - Design your crosshair
   - Download as PNG
   - Resize to 100x100 if needed

3. **Save Location**
   - Recommended: `%APPDATA%\GamingOptimizer\crosshairs\`
   - You can create this folder manually
   - Or store anywhere and use full path in profile

### Testing Your Crosshair

1. Save crosshair PNG to your chosen location
2. Update profile's `crosshair_image_path` with full path
3. Set `overlay_enabled` to `true`
4. Activate the profile from tray menu
5. Crosshair should appear at screen center

**Troubleshooting:**
- Not showing? Check image is exactly 100x100 pixels
- Wrong position? Adjust `x_offset` and `y_offset`
- Blurry? Ensure PNG is saved at 100% quality

## Common Workflows

### Workflow 1: Maximize FPS for Competitive Gaming

**Goal:** Kill background apps to free up CPU/RAM

1. Create profile with aggressive process list:
   ```json
   {
     "name": "Maximum Performance",
     "processes_to_kill": [
       "discord.exe",
       "spotify.exe",
       "chrome.exe",
       "msedge.exe",
       "slack.exe",
       "teams.exe",
       "steam.exe",
       "epicgameslauncher.exe"
     ],
     "crosshair_image_path": null,
     "crosshair_x_offset": 0,
     "crosshair_y_offset": 0,
     "overlay_enabled": false
   }
   ```

2. Before gaming session:
   - Activate "Maximum Performance" profile
   - Launch your game
   - Enjoy higher FPS

3. After gaming:
   - Deactivate profile (select "None")
   - Restart apps if needed

### Workflow 2: Custom Crosshair for Multiple Games

**Goal:** Use different crosshairs for different games

1. Create separate profile for each game:
   ```json
   [
     {
       "name": "CS:GO",
       "processes_to_kill": ["discord.exe"],
       "crosshair_image_path": "C:/Users/You/AppData/Roaming/GamingOptimizer/crosshairs/csgo.png",
       "crosshair_x_offset": 0,
       "crosshair_y_offset": 2,
       "overlay_enabled": true
     },
     {
       "name": "Valorant",
       "processes_to_kill": ["discord.exe"],
       "crosshair_image_path": "C:/Users/You/AppData/Roaming/GamingOptimizer/crosshairs/valorant.png",
       "crosshair_x_offset": 0,
       "crosshair_y_offset": 0,
       "overlay_enabled": true
     }
   ]
   ```

2. Before each game:
   - Activate the corresponding profile
   - Crosshair appears automatically

3. Toggle overlay anytime:
   - Right-click tray → "Overlay Visible"

### Workflow 3: Streaming Setup

**Goal:** Keep streaming software but kill other apps

1. Create streaming profile:
   ```json
   {
     "name": "Streaming",
     "processes_to_kill": [
       "chrome.exe",
       "msedge.exe",
       "spotify.exe",
       "discord.exe"
     ],
     "crosshair_image_path": null,
     "crosshair_x_offset": 0,
     "crosshair_y_offset": 0,
     "overlay_enabled": false
   }
   ```
   Note: OBS is NOT in the kill list

2. Before streaming:
   - Start OBS
   - Activate "Streaming" profile
   - Background apps killed, OBS still running

## Tips and Best Practices

### Process Management Tips

1. **Test Carefully**: Don't add processes you're not sure about
2. **Start Small**: Begin with obvious apps (Discord, Chrome)
3. **Check Protected List**: See README for processes that can't be killed
4. **Use Task Manager**: To find exact process names
5. **Monitor Impact**: Check if killing processes actually improves performance

### Crosshair Tips

1. **Keep It Simple**: Simple designs render better
2. **High Contrast**: Make sure crosshair is visible on all backgrounds
3. **Test In-Game**: Check visibility in actual gameplay
4. **Fine-Tune Position**: Use offsets to perfectly center
5. **Multiple Versions**: Create variations for different games/situations

### Profile Organization Tips

1. **Descriptive Names**: Use clear names like "CS:GO Max FPS" instead of "Profile 1"
2. **Game-Specific**: Create separate profiles for each game
3. **Purpose-Based**: Create profiles by purpose (streaming, recording, competitive)
4. **Backup**: Keep a copy of your profiles.json file

### Performance Tips

1. **Profile on SSD**: Keep app data on SSD for faster loading
2. **Regular Updates**: Update profiles as you install/uninstall apps
3. **Monitor Impact**: Use FPS counter to see actual improvements
4. **Clean Boot Test**: Compare with Windows clean boot for baseline

## Advanced Usage

### Multiple Monitor Setup

**Current Limitation:** Overlay only shows on primary monitor

**Workaround:**
- Set your gaming monitor as primary display in Windows
- Right-click desktop → Display settings → Make this my main display

### Auto-Start on Windows Boot

1. Press `Win + R`
2. Type `shell:startup` and press Enter
3. Create shortcut to `gaming_optimizer.exe` in this folder
4. Application will start with Windows

### Command Line Arguments

**Current Version:** No command line arguments supported

Future versions may support:
- `--profile "Profile Name"` - Auto-activate profile
- `--no-tray` - Run without system tray
- `--config path` - Use custom config location

### Integration with Game Launchers

**Steam:**
1. Add Gaming Optimizer as non-Steam game
2. Set launch options to activate specific profile (future feature)

**Currently:** Must manually activate profile before launching game

## Troubleshooting Common Issues

### Issue: Profile doesn't show in menu

**Solutions:**
- Check profiles.json syntax (use JSONLint.com)
- Ensure file is in correct location (%APPDATA%\GamingOptimizer\)
- Check for duplicate profile names
- Restart application

### Issue: Processes not being killed

**Solutions:**
- Verify exact process name in Task Manager
- Check if process is in protected list (see README)
- Try running application as Administrator
- Check if process requires elevated privileges

### Issue: Crosshair appears in wrong position

**Solutions:**
- Adjust `x_offset` and `y_offset` values
- Check monitor resolution hasn't changed
- Ensure game is in fullscreen mode
- Test with borderless window mode

### Issue: Crosshair not visible

**Solutions:**
- Verify PNG is exactly 100x100 pixels
- Check file path is correct (use forward slashes)
- Ensure `overlay_enabled` is `true`
- Try absolute path instead of relative
- Check image has visible content (not all transparent)

### Issue: Application won't start

**Solutions:**
- Ensure you're on Windows 10/11
- Check Windows Event Viewer for errors
- Verify all DLL dependencies are present
- Try running as Administrator
- Rebuild with `cargo build --release`

## Getting Help

If you encounter issues not covered here:

1. **Check README.md** - Contains additional troubleshooting
2. **Review planning.md** - Detailed technical specifications
3. **Check Data Directory** - Look for error logs
4. **Rebuild Application** - Try clean rebuild

## Feedback and Suggestions

This is an MVP release. Future versions may include:
- GUI settings editor
- Global hotkey support
- Game auto-detection
- Performance monitoring
- And more!
