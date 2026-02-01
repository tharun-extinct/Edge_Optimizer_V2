# Rules for Developer:
- If the build is not as you expected -> Go through the `instruction` or `design` file
- 
---


IPC = Named Pipes (not DispatcherQueue)

 ← 

marshalling refers to the process of converting an object or data structure into a format that can be easily transmitted, stored, or shared between different components or systems. This process is essential in distributed systems, such as remote procedure calls (RPC), web services, and object-oriented programming, to facilitate the communication of data.


---


I want EdgeOptimizer.Runner as a startup process, which proactively listen System tray events But the Settings process has flyout and listening system tray behavior listed in the instruction file
---
Flyout positioning



---



--------------------------
./target/release/EdgeOptimizer_Settings.exe
-> After running this, 
    - The settings window appeared 
    - The runner process also appeared in the task manager
--------------------------
crates/
├── core/         # NOT a process - it's a LIBRARY (shared code)
├── settings/     # Binary crate → EdgeOptimizer_Settings.exe
├── runner/       # Binary crate → EdgeOptimizer_Runner.exe
└── crosshair/    # Binary crate → EdgeOptimizer_Crosshair.exe


Each binary crate has its own build.rs that embeds unique Windows metadata.


core/
--------------
GUI components (gui/)
Profile management (profile.rs)
Config handling (config.rs)
Process management (process.rs)
Crosshair overlay logic (crosshair_overlay.rs)
etc.


```
Runner Process 
    │
    ├─ Flyout (hwnd)
    │   └─ Window Procedure
    │       └─ switch(message) { ...  }
    │           ├─ Tray clicks
    │           ├─ Hotkeys
    │           └─ System notifications
    │
    └─ Main Message Loop
        while (GetMessage(&msg, ... ))
            DispatchMessage(&msg);
```


How does the Powertoys.Settings binary spawns the Runner process and calls the gui:run() from the core library

-------------------------------
Your application uses std::env::current_exe() to:

Find its own executable path
Derive the parent directory
Locate sibling executables like EdgeOptimizer_Runner.exe



When you run with forward slashes (/), Windows may canonicalize the path differently, resulting in:

A properly resolved absolute path like EdgeOptimizer_Settings.exe
This allows the Runner and other components to be found correctly
When you run with backslashes (\) directly in PowerShell, depending on how the shell passes the path, current_exe() might return:

A shorter or non-canonicalized form
Potentially causing the Runner process spawn to fail (notice your exit code was 1)
-------------------------------



## Problems
- **UI elements can only be modified on the thread that created them** → DispatcherQueue solves this safely
    - Priority Levels: You can prioritize urgent operations (Normal, High, Low)
    - Async-Safe: Returns immediately, action executes later on UI thread

✅ Runner handles ALL hotkeys for all modules
✅ Single centralized system using RegisterHotKey Win32 API


```
Runner Process (EdgeOptimizer.Runner.exe)
├── Main executable code
├── Module management (Optimizer, Crosshair, Macro)
├── IPC communication system
└── Hidden Window (for tray icon)                           <————— Triggers Flyout
    └── Window Procedure (message handler)
```



https://docs.rs/windows-sys/0.61.2/windows_sys/


https://www.builder.io/blog/agents-md



✔ ✘
✓ ✗

Windows API:

Windows Message Parameters:  
In the Windows API, when a window procedure (WndProc) receives a message, it comes with two parameters:

WPARAM → an integer-sized value (often used to pass additional information like key codes, mouse button states, or identifiers).

LPARAM → a pointer-sized value (often used for coordinates, handles, or references).

WM_HOTKEY is a predefined Windows message that gets sent to a window when a registered system-wide hotkey is pressed.


## In-Process Modules (DLLs)
When hotkey is pressed → Callback executes in Runner → Module DLL function called directly

## Separate Process Modules
When hotkey is pressed → Callback executes in Runner → Runner signals separate process via:
 - Named Events (EventWaitHandle)
 - IPC messages
 - Process launch

- Always use mermaid code blocks to represent flowcharts, diagrams, and other visual representations.


[✔]





cargo clean --release

taskkill /F /IM EdgeOptimizer_Runner.exe 2>$null; 
taskkill /F /IM edge_optimizer_runner.exe 2>$null;
taskkill /F /IM "gaming_optimizer.exe" /IM "edge_optimizer_settings.exe" /IM "edge_optimizer_runner.exe" 2>$null;
taskkill /F /IM "edge_optimizer_crosshair.exe" 2>&1

cargo clean; 
cargo build --release 2>&1
.\target\release\EdgeOptimizer_Runner.exe

cargo run --bin EdgeOptimizer_Settings




Get-Process | Where-Object { $_.Name -like "edge_optimizer*" } | Select-Object Name, Id






The .rc files are Windows Resource Script files. They define metadata that gets embedded directly into the .exe file during compilation.

What they do:
Metadata	            Where it appears
----------------------------------------------
FileDescription         Task Manager "App name" column
ProductName	            File Properties → Details tab
InternalName	        Internal identifier
OriginalFilename	    Original file name (even if renamed)
FileVersion / ProductVersion	Version info in Properties
CompanyName	            Publisher/Company in Properties
LegalCopyright	        Copyright notice


Example - What settings.rc does:
Visual representation:


Without these files:
Task Manager would just show the raw executable name (edge_optimizer_settings) with no friendly "App name".

How they work:
build.rs reads the .rc file
winres crate compiles it into a .res binary resource
The linker embeds it into the final .exe
Windows reads this metadata when displaying the app
---

make sure On Double click on tray icon,

If the Settings.Main window is not opened -> Open it
If its already opened -> bring to front

---


----
Build / Test as the final step in the todos list


---
- Flyout should be shown by Runner directly when Settings isn't running (not spawn Settings)
- Runner needs to check if Settings process exists before spawning
- Settings IPC connection isn't being tracked properly


 Runner handle flyout locally when Settings isn't connected, and add proper process detection:

No duplicate spawn protection - The spawn_settings_window function spawns even if window exists.

`crates\runner\src\main.rs:51` IPC connection not properly tracked - settings_connected is never set to true until a message arrives.

 ---

 Does the flyout and window are handled by the flag based invoke method?

---


---
The Shortcut Settings should looks like the following, 
- Selectable options are `CTRL`, `ALT`, `SHIFT`, `WIN` (Multi-select , If any of those option is already been selected for that profile -> The check box should be filled with green color)
- The shortcut options should be beside the `Keys in macro` container - 2 x 2 (2 by 2). Beside these, there should be an input feild for listening the shortcut confirmational key which is [A-Z a-z F1-F12 0-9]
- The position of the container should be lower from the mid-height of the `Macro List` and `Keys in macro` containers.
---
On top of the shortcut container, there should macro cycle times settings, the options are radio select (one at a time),
- Cycle until <inline input feild> pressed
- Specified cycle time <inline input feild default=1>
---

The drop down should be displayed, only hovering on the container of the sections and the container of the drop down list. The provided image is How the nested dropdown should look like. 
Don't show any implement code, right now — just confirm the new feature implementation.

Future Enhancement, 
Short cut conflict detector


Global Shortcut detector
 ---




Cycle until the key released 
Cycle until the key clicked again
Specified cycle times [_]


Add up -> 
Onclicking the record icon, the text should change to 'stop'

---
The right click for the `Macro List` container, 
- It should show two options `RENAME` and `DELETE`
- It should work only on the selected 'Macro' within the `Macro List`

---

The right click for the `Keys in macro` container, 
- Each parameter should be selected separately
- On Right-Click, It should have `Change parameter` and `Delete` options
- It should work only on the selected 'Key binds' within the macro keys list



---
The UI isn't what I expected, UI looks like shit.

make it looks like as we have discussed and `Macro List` shouldn't be in side bar - it should be a part of UI


I don't separated navigation bar [Profile section, Macro section] and I just want 'Macro' below the sidebar of the 'Profile'


---
macro 'Record' is not at all working and the 'Insert Events' dropdown options are showing by default with in the 'Keys in macro' container. I want the 'Insert Events' section to be aligned with the 'Record' button of the 'Macro List'


The mouse events shouldn't be recorded, I want only the keyboard events to be recorded
- The mouse events should Only be inserted from the 'Insert events' buttons from the 'keys in Macro' container
---







[] - format to display 
---
Hook?
A hook is a mechanism that allows you to intercept and monitor events in the Windows operating system before they reach their destination.

Think of it like a spy or wiretap on Windows' internal messaging system.
---


use Windows Low-Level Keyboard Hooks (WH_KEYBOARD_LL via SetWindowsHookEx), which:

✅ Captures ALL keyboard events system-wide before they reach applications (while recording key events for Macro)
✅ Can intercept AND block key events (though TGMacroR doesn't appear to block by default)
⚠️ Does NOT register with Windows shortcut manager (so no automatic conflict prevention)
⚠️ Runs alongside Windows shortcuts - meaning conflicts WILL occur


List out the shortcuts in Window from the shortcut manager (to avoid it)




------
- If user try switch to another Macro while Recording the input key in the Macro, It should show a mini pop up "Recording!"
- Slightly increase the height of the container of the 'Macro List' and 'Keys in Macro' container from the inside
- if user press the Recording in the Macro List Section
    - It should completely blocks the input from the keyboard to any other applications and the inputs should be read only by the `EdgeOptimizer`
    - If a Macro is selected from the `Macro List` and then if its recorded -> `Keys in Macro` should start from a clean slate 
    - 


Issues:
- Cross Macro writing (During recording)
- Not recording while in the application (Recording the keyboard events only outside the application)


.dat



Icon color: #00b5d6, #059aff

```
rust_inputs/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Main library entry
│   ├── read/
│   │   ├── mod.rs          # Read module
│   │   ├── input_listener.rs   # Global hook manager
│   │   ├── keyboard_hook.rs    # Keyboard hook implementation
│   │   └── mouse_hook.rs       # Mouse hook implementation
│   ├── send/
│   │   ├── mod.rs          # Send module
│   │   ├── keyboard.rs     # Send keyboard input
│   │   └── mouse.rs        # Send mouse input
│   ├── types/
│   │   ├── mod.rs          # Type definitions
│   │   ├── enums.rs        # Virtual key codes, flags
│   │   └── structs.rs      # Data structures

```



The mouse events shouldn't be recorded, I want only the keyboard events to be recorded
- The mouse events should Only be inserted from the 'Insert events' buttons from the 'keys in Macro' container
---
Currently, it not at all listening the keyboard events