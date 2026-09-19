# 🎮 Edge Optimizer

**Built by Gamers, For Gamers**

Edge Optimizer is your ultimate gaming companion - a powerful Windows application that boosts gaming performance by eliminating resource-hungry background processes, adding custom crosshair overlays, and optimizing your system for peak gaming performance.

> **Why Edge Optimizer?**  
> Every millisecond matters in competitive gaming. Edge Optimizer gives you the edge by freeing up system resources, reducing input lag, and enhancing your aiming precision with customizable crosshairs.

---

### 🎯 **Custom Crosshair Overlays**
Gain aiming precision with fully customizable PNG crosshairs that work seamlessly over fullscreen games (Fortnite, Valorant, CS2, and more).
- Pixel-perfect positioning with live adjustments
- Works like Xbox Game Bar - no game interference
- Click-through design - never blocks your shots
- Survives game crashes and app restarts

### ⚡ **Kill Resource Hogs**
Free up RAM, CPU, and network bandwidth by automatically terminating unwanted background apps before gaming.
- One-click process termination (Discord, Chrome, Spotify, etc.)
- Built-in safety - won't kill critical Windows processes
- Live process monitor with CPU/memory usage
- Create custom process kill lists per game

### 🎮 **Gaming Profiles**
Create unlimited profiles for different games with unique settings for each.
- Save crosshair positions per game
- Custom process kill lists per profile
- One-click profile switching from system tray
- Instant activation before launching your game

### 🖥️ **System Tray Integration**
Quick access without cluttering your screen - perfect for in-game adjustments.
- Left-click: Quick flyout menu
- Right-click: Full context menu
- Double-click: Open full settings
- Always accessible, never intrusive

---

## ⚡ Quick Start Guide

### 📥 **Installation**

1. **Download the latest release** from the [Releases page](../../releases)
2. **Extract the ZIP file** to a folder of your choice
3. **Run** `EdgeOptimizer.Settings.WinUI.exe`

That's it! No installation required - just extract and run.

### 🛠️ **Building from Source**

**Requirements:**
- [Rust](https://rustup.rs/) 1.70+ (with cargo)
- Windows 10/11
- Visual Studio Build Tools (for Windows API)

**Build Commands:**
```powershell
# Clone the repository
git clone https://github.com/yourusername/EdgeOptimizer.git
cd EdgeOptimizer

# Build in release mode (optimized for gaming)
# Build the Rust Runner and workers
cargo build --release

# Build all components
cargo build --release -p edge_optimizer_runner
cargo build --release -p edge_optimizer_crosshair

# Publish the WinUI 3 settings client beside the Runner executable
.\scripts\publish-winui-settings.ps1
```

**Executables will be in:**
- `target\release\EdgeOptimizer.Settings.WinUI.exe` - Main GUI application
- `target\release\EdgeOptimizer_Runner.exe` - System tray manager
- `target\release\EdgeOptimizer_Crosshair.exe` - Crosshair overlay
- `target\release\EdgeOptimizer_Macro.exe` - Unprivileged macro worker

Pull-request CI also publishes `edge-optimizer-windows-x64`, which places the self-contained WinUI client beside Runner and all workers. Use that combined artifact for functional testing; the WinUI-only artifact is intended for build diagnostics.

### 🎮 **Running the Application**

```powershell
# Run the release build (recommended for gaming)
.\target\release\EdgeOptimizer.Settings.WinUI.exe

# Or build and run directly
dotnet run --project .\apps\EdgeOptimizer.Settings.WinUI\EdgeOptimizer.Settings.WinUI.csproj --configuration Release

# Development mode (debug build)
dotnet run --project .\apps\EdgeOptimizer.Settings.WinUI\EdgeOptimizer.Settings.WinUI.csproj
```

---

## 📖 How to Use Edge Optimizer

### **Step 1: Create Your First Gaming Profile**

1. **Launch Edge Optimizer** by running `EdgeOptimizer.Settings.WinUI.exe`
2. **Click "New Profile"** button
3. **Name your profile** (e.g., "Fortnite", "Valorant", "CS2")
4. **Select processes to kill:**
   - Check the boxes next to apps you want closed (Discord, Chrome, etc.)
   - Use the search box to filter processes quickly
   - Click "Refresh" to update the process list
5. **Add a crosshair (optional):**
   - Click "Select Image" to choose your PNG crosshair
   - Use arrow buttons ▲▼◀▶ to position it perfectly
   - Click ⊙ to center on screen
6. **Save your profile**

### **Step 2: Activate Before Gaming**

1. **Select your profile** from the list
2. **Click "Activate Profile"**
3. Edge Optimizer will:
   - ✅ Close all selected background processes
   - ✅ Launch your crosshair overlay (if configured)
   - ✅ Show confirmation messages
4. **Launch your game** and dominate!

### **Step 3: Quick Access from System Tray**

Once running, Edge Optimizer lives in your system tray:
- **Left-click**: Open quick flyout menu
- **Right-click**: Context menu (Settings, Documentation, Exit)
- **Double-click**: Open full settings window

**Pro Tip:** Minimize to tray and use the quick flyout for instant profile switching between games!

---

## 🎯 Crosshair Setup Guide

### **Requirements:**
- **PNG image** with transparency
- Any size (automatically centered)
- Transparency/alpha channel support

### **Works With:**
✅ Fortnite (Fullscreen mode)  
✅ Valorant  
✅ CS2 (Counter-Strike 2)  
✅ Apex Legends  
✅ Call of Duty  
✅ Rainbow Six Siege  
✅ Most modern games using "Fullscreen" mode

### **How It Works:**
Edge Optimizer uses **Desktop Window Manager (DWM)** technology - the same tech that powers:
- Xbox Game Bar
- Discord overlays
- NVIDIA GeForce Experience
- Steam overlay

### **Fortnite Fullscreen Fix:**
If your crosshair doesn't appear in Fortnite:
1. Navigate to Fortnite installation folder
2. Right-click `FortniteClient-Win64-Shipping.exe`
3. Go to **Properties → Compatibility**
4. **UNCHECK** "Disable fullscreen optimizations"
5. Click OK and restart Fortnite

---

## 🛡️ Safety & Protected Processes

Edge Optimizer **prevents you from killing critical Windows processes** that could crash your system:

🔒 **Protected processes include:**
- `csrss.exe` - Client Server Runtime
- `dwm.exe` - Desktop Window Manager
- `explorer.exe` - Windows Explorer
- `lsass.exe` - Local Security Authority
- `svchost.exe` - Service Host
- `system` - System process
- `winlogon.exe` - Windows Logon

These processes are **automatically blocked** to ensure system stability.

---

## 📂 Configuration & Files

Edge Optimizer stores all settings in:
```
%APPDATA%\EdgeOptimizer\
├── profiles.json        # Your gaming profiles
└── crosshairs/          # (Optional) Store crosshair images here
```

All configuration is **automatically saved** - no manual file editing required!

---

## 🐛 Troubleshooting

### **Crosshair not appearing over game**
- ✅ Ensure game is in **"Fullscreen"** mode (not "Windowed Fullscreen")
- ✅ Check game exe properties: **uncheck** "Disable fullscreen optimizations"
- ✅ Verify your PNG image has transparency
- ✅ Try activating profile **after** game is running

### **Processes not closing**
- ✅ Verify process names include `.exe` extension
- ✅ Check if process is in the protected list
- ✅ Try running Edge Optimizer as **Administrator**
- ✅ Some processes require special permissions

### **Application won't start**
- ✅ Ensure you have **Visual Studio C++ Redistributables** installed
- ✅ Check Windows Event Viewer for error details
- ✅ Try running as Administrator
- ✅ Verify Windows 10/11 (older versions not supported)

### **Crosshair position is off**
- Use **arrow buttons** (▲▼◀▶) to adjust position
- Click **⊙ button** to reset to screen center
- Changes apply **instantly** - no restart needed

---

## 🔧 For Developers

### **Project Structure**
```
EdgeOptimizer/
├── crates/
│   ├── core/              # Shared library
│   │   └── src/
│   │       ├── common_apps.rs      # Common app database
│   │       ├── config.rs           # Config management
│   │       ├── crosshair_overlay.rs # Crosshair launcher
│   │       ├── flyout.rs           # Flyout window
│   │       ├── ipc.rs              # Inter-process communication
│   │       ├── process.rs          # Process management
│   │       └── profile.rs          # Profile data structures
│   ├── settings/          # Main GUI application
│   ├── runner/            # System tray manager
│   └── crosshair/         # Crosshair overlay process
├── Cargo.toml             # Workspace configuration
└── README.md
```

### **Tech Stack**
- **Language:** Rust 2021 Edition
- **GUI Framework:** WinUI 3 on .NET 10 - desktop presentation client
- **System Tray:** [tray-icon](https://crates.io/crates/tray-icon)
- **Process Management:** [sysinfo](https://crates.io/crates/sysinfo)
- **Image Handling:** [image](https://crates.io/crates/image)
- **Windows APIs:** [windows-rs](https://github.com/microsoft/windows-rs)

### **Development Commands**
```powershell
# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy

# Build debug version
cargo build

# Run debug version
dotnet run --project .\apps\EdgeOptimizer.Settings.WinUI\EdgeOptimizer.Settings.WinUI.csproj
```

---

## 🚀 Roadmap & Future Features

- [ ] Multi-monitor support with monitor selection
- [ ] Auto-detect games and activate profiles automatically
- [ ] Global hotkeys for overlay toggle
- [ ] Built-in crosshair library
- [ ] Performance metrics overlay (FPS, CPU, RAM)
- [ ] Network optimization tools (reduce latency)
- [ ] Power plan optimization
- [ ] Profile import/export and sharing
- [ ] Advanced process rules (CPU/memory thresholds)
- [ ] Custom crosshair designer
- [ ] Overlay opacity controls

---

## 📜 License

This project is provided as-is for gaming optimization purposes.

---

## 💬 Support & Community

**Need help?**
1. Check the [Troubleshooting](#-troubleshooting) section
2. Review game compatibility settings
3. Open an [Issue](../../issues) on GitHub

**Have feedback or suggestions?** We'd love to hear from you!

---

<div align="center">

**Built by Gamers, For Gamers** 🎮

Made with ❤️ and Rust

</div>

