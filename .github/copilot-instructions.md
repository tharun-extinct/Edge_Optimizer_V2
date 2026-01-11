---
applyTo: "**/*"
name: Edge Optimizer
description: High-performance RUST-based gaming optimization application
---

**Project name:** `Edge Optimizer`


**Project description:** Edge Optimizer is a high-performance RUST-based application designed to 
  - kill unwanted background processes (that are necessary for gaming)
  - optimize system performance (Tweaking power plans, disabling Ethernet Energy Efficient mode for lower input delay, etc.)
  - Add Crosshair overlays for better aiming in FPS games
  - Boost network performance for reduced latency during online gaming sessions.


**This is how you should always act**:
- Always Plan first before writing any code. 
    - Break down the problem into smaller parts (Create Todo list)
    - Focus on performance and low latency optimizations
    - Ask when anything's unclear (might be tech stack or feature related but never assume)


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
- The Settings UI (including both the main GUI and flyout) runs as a separate process
- Within the Settings UI process, there are two distinct windows:
        MainWindow - the full settings application
        FlyoutWindow - the quick access flyout (It closes, when clicking outside)

    ### Flyout Behavior
    - Right-click → Shows traditional context menu (Settings [Double clicks opens Main GUI], Documentation, Report Bug, Close)
    - Left single-click → Opens flyout window
    - Left double-click → Opens full Settings window (If already open, brings to front)



### DispatcherQueues
DispatcherQueue is a modern Windows API for managing thread-safe UI operations. Think of it as a task queue for a specific UI thread.
Safely routes operations to the correct UI thread
UI elements can only be modified on the thread that created them. DispatcherQueue solves this safely.




## EdgeOptimizer.Runner Process
- The Runner process manages the system tray icon and handles tray icon clicks
- When user click the tray icon, the Runner sends an IPC message to the Settings UI process
- Runner process actively listens— using Windows Message Loop pattern
- Runner uses Win32 message loop to listen for tray icon clicks
- Tray click (Runner) → IPC message → DispatcherQueue → Show flyout (UI thread)



**For Refactoring code:**
- Only delete if the files or Functions aren't called /used anywhere
- Delete Only if it's in the build error logs 

Before answering:
1. Query the memory MCP for related project decisions
2. If missing, ask me before proceeding
3. Store new decisions in memory after the response


Documentation Links:
-[Windows-rs](https://github.com/microsoft/windows-rs)
-[Iced GUI](https://github.com/iced-rs/iced)