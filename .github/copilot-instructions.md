---
applyTo: "**/*"
name: EdgeOptimizer
description: High-performance RUST-based gaming optimization application
---

**Project name:** `EdgeOptimizer`

**Project description:** EdgeOptimizer is a high-performance RUST-based application designed to 
  - kill unwanted background processes (that are necessary for gaming)
  - optimize system performance (Tweaking power plans, disabling Ethernet Energy Efficient mode for lower input delay, etc.)
  - Add Crosshair overlays for better aiming in FPS games
  - Boost network performance for reduced latency during online gaming sessions.


**This is how you should always act**:
- Always Plan first before writing any code. 
    - Break down the problem into smaller parts (Create Todo list and add 'Build and Test' as the final step)
    - Focus on performance and low latency optimizations
    - Ask when anything's unclear (might be tech stack or feature related but never assume)
    - Don't implement any feature, until it's explicitly mentioned


- Refining Ideas:
    - Suggest multiple approaches to implement/ solve a feature (approaches can be tech stack, architecture, algorithms, libraries, etc)


- When providing code suggestions, prioritize:
    - Performance optimization and Low latency execution
    - Always think about edge cases and how to handle them
    - Consider user experience and usability
    - Efficient error handling and logging
    - Clean, modular architecture
    - Think about scalability and maintainability of the codebase
    - Write code that are production ready


- For referring latest document and resolving package error use Context7 MCP.

---
# Edge Optimizer Architecture Overview
The Main GUI and FlyoutWindow are same `PROCESS` but are `DIFFERENT WINDOWS` within the Settings UI application.

**Here's the architecture:**
EdgeOptimizer.Settings -> Main GUI
EdgeOptimizer.Runner   -> separate process that manages the system tray icon
EdgeOptimizer.Crosshair -> Crosshair overlay



## EdgeOptimizer.Settings Process 
- Settings has NO tray management, it only has UI windows (MainWindow, FlyoutWindow)
- The Settings UI (including both the main GUI and flyout) runs as a separate process
- Within the Settings UI process, there are two distinct windows:
        MainWindow - the full settings application
        FlyoutWindow - the quick access flyout (It closes, when clicking outside)

    



### DispatcherQueues
DispatcherQueue is a modern Windows API for managing thread-safe UI operations. Think of it as a task queue for a specific UI thread.
Safely routes operations to the correct UI thread
UI elements can only be modified on the thread that created them. DispatcherQueue solves this safely.



## EdgeOptimizer.Runner Process
- Only the Runner process manages the system tray icon and handles tray icon clicks
- When user click the tray icon, the Runner sends an IPC message to the Settings UI process
- Runner process actively listens— using Windows Message Loop pattern
- Runner uses Win32 message loop to listen for tray icon clicks
- Runner manages tray, sends IPC to Settings (via Named Pipes), Settings uses DispatcherQueue to marshal to UI thread → Show flyout (UI thread)

    ### Tray Icon Behavior
        - Right-click → Shows context menu (Settings, Documentation, Report Bug, Exit)
        - single-click → Opens flyout window
        - double-click → Opens full Settings window (If already open, brings to front —no new instance)



## Macro Feature Implementation Details
Gaming macro in the side bar of the Main Settings.UI, 
    - Only the UI should be in the Main Settings
    - And the EdgeOptimizer.Macro should listen for the shortcut (shortcut listener)

- The macro Execution and the Shortcut Listener Should be a separate process `EdgeOptimizer.Macro`
- And the Macro Settings and the User Interaction should happens with in the Settings (Main UI)

- Macros should be Profile-specific -> Tied to profiles (activate with profile) 
- Macro persistence format ->  Extend existing config.rs 

---
The Macro UI should like the following, 
- Two containers `Macro List` and `Keys in macro`
---
The `Macro List` as the first container (from the left) with `Record` section (with a clickable icon - Only the icon should be clickable for recording event with milliseconds precision) on the bottom of the `Macro List` container.
- For 'Record'[color: green] and 'Stop' [color: red] with respective icons
---
The `Keys in macro` as the second container (next to the `Macro List`) with drop down section on the bottom called `Insert Event` with drop down icon on the right corner of that section (Drop down should be down right-side of the `Keys in macro`). 
---
`Keys in macro` containers drop down should have the following options in it, 
- Insert previous (dropdown options -> [Left ⬆, Left ⬇, Right ⬆, Right ⬇, Middle ⬆, Middle ⬇]),
- Insert after (dropdown options -> [Left ⬆, Left ⬇, Right ⬆, Right ⬇, Middle ⬆, Middle ⬇]), 
- Insert XY (Gamer / User should enter X & Y position of the mouse ),
- Insert Delay (Should always be in milliseconds)
---

**For Refactoring code:**
- Only delete if the files or Functions aren't called /used anywhere
- Delete Only if it's in the build error logs 

Before answering:
1. Query the memory MCP for related project decisions
2. If missing, ask me before proceeding
3. Store new decisions in memory after the response

