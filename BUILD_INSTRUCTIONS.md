# Build Instructions - Gaming Optimizer

This guide walks you through building the Gaming Optimizer application on Windows.

## Prerequisites

### 1. Install Rust

Download and install Rust from the official website:
- Visit: https://rustup.rs/
- Download: `rustup-init.exe`
- Run the installer and follow prompts
- Choose default installation options

Verify installation:
```bash
rustc --version
cargo --version
```

### 2. Install Visual Studio Build Tools

Rust on Windows requires the Microsoft C++ build tools:

**Option A: Visual Studio 2022 (Recommended)**
1. Download Visual Studio 2022 Community (free)
2. During installation, select "Desktop development with C++"
3. Install

**Option B: Build Tools Only**
1. Download "Build Tools for Visual Studio 2022"
2. Install "C++ build tools" workload
3. Include Windows 10/11 SDK

Verify installation:
```bash
cl
# Should show Microsoft C/C++ Compiler
```

### 3. Clone or Download Repository

If using Git:
```bash
git clone <repository-url>
cd Gaming_optimizer
```

Or download and extract the source code.

## Building the Application

### Debug Build (For Development)

```bash
cargo build
```

**Output:** `target/debug/gaming_optimizer.exe` (~15-25 MB)

**Use when:**
- Testing changes
- Debugging issues
- Development work

### Release Build (For Production)

```bash
cargo build --release
```

**Output:** `target/release/gaming_optimizer.exe` (~2-5 MB)

**Use when:**
- Actual gaming use
- Distribution to others
- Maximum performance needed

**Build time:** 2-10 minutes depending on your system

### What Happens During Build

1. **Dependency Download** (first time only)
   - Downloads ~200 MB of Rust crates
   - Caches in `~/.cargo/registry/`
   - Subsequent builds reuse cache

2. **Compilation**
   - Compiles all dependencies
   - Compiles Gaming Optimizer source code
   - Links everything into final executable

3. **Optimization** (release build only)
   - Link-time optimization (LTO)
   - Dead code elimination
   - Symbol stripping
   - Results in smaller, faster binary

## Build Troubleshooting

### Error: "cargo: command not found"

**Solution:**
1. Close and reopen terminal/command prompt
2. Verify Rust installation: `rustup show`
3. Add to PATH if needed:
   - `%USERPROFILE%\.cargo\bin`

### Error: "link.exe not found" or "MSVC linker error"

**Solution:**
1. Install Visual Studio Build Tools (see Prerequisites)
2. Restart terminal after installation
3. Run `cargo clean` then rebuild

### Error: "failed to fetch registry"

**Solution:**
1. Check internet connection
2. Try again (may be temporary)
3. Use VPN if behind firewall
4. Check cargo registry: `cargo search serde`

### Error: "out of memory" during compilation

**Solution:**
1. Close other applications
2. Build with fewer parallel jobs: `cargo build --release -j 2`
3. Increase swap/page file size
4. Use 64-bit system (32-bit may struggle)

### Warnings during build

**Common warnings are normal:**
- "unused variable" - Safe to ignore
- "unused import" - Safe to ignore
- "function is never used" - Test/utility functions

**Warnings don't prevent successful build.**

## Running the Application

### Run Without Building Executable

```bash
# Debug mode
cargo run

# Release mode
cargo run --release
```

Application starts in system tray (look bottom-right corner).

### Run the Built Executable

```bash
# Debug
.\target\debug\gaming_optimizer.exe

# Release
.\target\release\gaming_optimizer.exe
```

Or double-click the `.exe` file in Windows Explorer.

## Distribution

To share with others:

1. **Build release version:**
   ```bash
   cargo build --release
   ```

2. **Copy executable:**
   - Source: `target\release\gaming_optimizer.exe`
   - Destination: Anywhere you want

3. **The executable is standalone** - no dependencies needed
   - Users don't need Rust installed
   - Users don't need Visual Studio
   - Just the single `.exe` file

4. **Optional: Include documentation**
   - README.md
   - USAGE.md
   - profiles.example.json

## Development Workflow

### Making Changes

1. Edit source files in `src/`
2. Build: `cargo build`
3. Test: `cargo run`
4. Repeat

### Running Tests

```bash
cargo test
```

Runs all unit tests in the codebase.

### Formatting Code

```bash
cargo fmt
```

Automatically formats code to Rust standards.

### Linting Code

```bash
cargo clippy
```

Checks for common mistakes and improvements.

### Cleaning Build Artifacts

```bash
cargo clean
```

Removes `target/` directory. Useful if:
- Build errors persist
- Want to free disk space
- Switching between debug/release

## Build Configuration Details

### Optimization Levels

**Debug (`cargo build`):**
- opt-level = 0 (no optimization)
- Includes debug symbols
- Fast compilation
- Slower execution
- Larger binary

**Release (`cargo build --release`):**
- opt-level = 3 (maximum optimization)
- LTO enabled
- Strips debug symbols
- Slow compilation
- Fast execution
- Small binary

### Customizing Build

Edit `Cargo.toml` `[profile.release]` section:

```toml
[profile.release]
opt-level = 3      # 0-3, s (size), z (min size)
lto = true        # Link-time optimization
codegen-units = 1  # Parallel codegen (1 = better opt)
strip = true      # Strip debug symbols
panic = "abort"   # Abort on panic (smaller binary)
```

## Platform Notes

### Windows 10/11 (Primary Target)

- ✅ Fully supported
- ✅ All features work
- ✅ Tested platform

### Windows 7/8

- ⚠️ May work but untested
- ⚠️ Some APIs might not be available
- 🔧 Recommend Windows 10+ for best experience

### Linux/macOS

- ❌ Not supported
- ❌ Uses Windows-specific APIs
- ❌ Won't compile

## Performance Notes

### Binary Size

- Debug: ~15-25 MB (with debug info)
- Release: ~2-5 MB (optimized, stripped)
- Further reduction possible with compression

### Memory Usage

- Idle: ~5-10 MB
- With overlay: ~15-20 MB
- Very lightweight for background app

### CPU Usage

- Idle: ~0% CPU
- Profile activation: Brief spike during process killing
- Overlay rendering: Minimal (software rendering)

## Advanced: Cross-Compilation

To build for different Windows architectures:

### 64-bit Windows (x86_64)
```bash
cargo build --release --target x86_64-pc-windows-msvc
```

### 32-bit Windows (i686)
```bash
rustup target add i686-pc-windows-msvc
cargo build --release --target i686-pc-windows-msvc
```

Output in `target/<architecture>/release/`

## Support

### Build Issues

1. Check this guide's Troubleshooting section
2. Verify prerequisites installed correctly
3. Try `cargo clean` then rebuild
4. Check Rust version: `rustc --version` (should be 1.70+)

### Runtime Issues

See README.md and USAGE.md for application-specific help.

## Next Steps

After successful build:

1. ✅ Run the application
2. ✅ Check system tray for icon
3. ✅ Create gaming profiles (see USAGE.md)
4. ✅ Test process killing
5. ✅ Create crosshair overlay (optional)
6. ✅ Enjoy optimized gaming!

---

**Build successfully completed?** 🎉

You now have `gaming_optimizer.exe` ready to optimize your gaming experience!

See USAGE.md for detailed instructions on using the application.
