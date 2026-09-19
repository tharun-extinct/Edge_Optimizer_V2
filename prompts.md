Go through the codebase and update the README.md file, also include the tagline of 'Built by Gamers, For Gamers'
----------
create the README file, in such a way that it understandable for every users and gamers, not entirely in the developer perspective
------------------------------------
Check out the main branch and update the changes to that branch and return back to the master branch

I'll help you check out the main branch, update changes there, and return to master. Let me start by checking the current git status.

git status
git checkout main
git merge master
git checkout master

### 

---

I want you to create a Design file `/.github/design.md`, detailly explain the design and architecture of this project.
Things to remember,
- Use mermaid code for Architecture diagram
- Highlight / differentiate the important component in the mermaid code (Don't make a cookie cutter diagram)
- Follow the DRY priciple
- It should be tuned for GitHub copilot 


- Don't repeat the same thing again and again.
- Don't sacrifice the content

---

-The above given structure is only for reference purpose, don't implement as it is (use it as a reference) and stick to our core principles

---


create @mai from design.md


I want to come up with list of tech stack for this project and suggest the optimal one (for e.g. Database - Sqlite or json storage)







developer_instructions = """

Start by establishing the user's outcome, success criteria, target users, constraints, performance and reliability goals, platform and deployment needs, integrations, budget, delivery horizon, and non-goals. Derive answers from the request and available evidence first. If a material requirement remains unknown or competing requirements require a choice, use the available questioning tool to ask concise, decision-oriented questions before selecting a stack or proposing an architecture. Do not silently assume requirements or declare an optimal choice without a stated use case.

Inspect the relevant codebase, documentation, configuration, dependencies, and tests before offering recommendations. Clearly distinguish observed implementation from planned or inferred design. Trace ownership, data flow, public interfaces, deployment topology, and trust boundaries as needed by the request.

Inventory the stacks, frameworks, protocols, persistence technologies, cloud services, and worker/service technologies that are relevant to the problem. For each viable option, explain practical pros and cons for this requirement. Recommend one optimal option based on the gathered requirements, lifecycle cost, security, latency, reliability, scalability, maintainability, team fit, ecosystem maturity, and migration risk. Do not recommend technology replacement merely for novelty.

For every recommendation, discuss future scope: expected feature growth, compatibility and versioning, observability, operations, testability, and safe migration sequencing. Include only relevant micro-optimization opportunities, such as hot-path allocations, transport framing and batching, scheduling, startup cost, rendering or input latency, database access, caching, concurrency, or process boundaries. Quantify or propose measurement before pursuing a micro-optimization, and reject optimizations that weaken safety, correctness, observability, or maintainability.

Treat security-sensitive proposals involving identity, authorization, untrusted input, data access, privileged operations, payments, or critical infrastructure as high risk. Identify trust boundaries, validation points, failure modes, compatibility concerns, and the appropriate automated and integration tests. Do not recommend bypasses around security controls.

Respond with a concise architecture decision record: gathered requirements and unresolved questions, observed evidence, stack comparison with pros and cons, recommended option and rationale, future scope and measured optimization opportunities, affected interfaces or components, migration or compatibility risks, and verification required. Explicitly label any unresolved assumption or user decision. Do not claim validation that was not actually performed.
"""










build a ci verfication of compilation for main branch

?



WPF means Windows Presentation Foundation. It is Microsoft’s Windows desktop UI framework for C#/.NET. The interface is normally defined using XAML, while C# handles UI behavior. In this project, WPF would power only the Settings window—not the Rust optimizer engine.



Runner starts.
Open SQLite once and keep the connection available.
Query app_state.active_profile_id.
Load that profile, its macros and related settings in one transaction/query.
Keep the resulting profile in memory.
Use the in-memory copy during optimization.





# Agent Routing

Use progressive disclosure for PDF reader feature and architecture work:

1. Read [`blueprints/README.md`](blueprints/README.md) and classify the task by intent.
2. Load only the primary blueprint identified by the routing table.
3. Follow that blueprint's links to the applicable sections of [`.github/architecture.md`](.github/architecture.md).
4. Inspect the listed implementation and tests before changing behavior or status documentation.
5. Load an impact-check blueprint only when the change touches the shared behavior named in the manifest.

If no manifest row matches (for example, isolated CI, dependency, or general
documentation work), do not force a blueprint match. Inspect the directly
relevant files and load architecture context only if a shared contract changes.



Current code and verified tests override stale status prose. Never describe
planned behavior as implemented. 


README.md should contains -- necessary things for setting up the project including the commands 
for copilot-instructions.md and design.md -- follow the structure from this codebase





create skill for updating the Readme file, copilot-instructions.md and design and how it's structure -- get these details from this codebase 

---

README.md should contains -- necessary things for setting up the project including the commands 

for copilot-instructions.md and design.md -- c4 follow the structure from this codebase 

create the skill in this location .github/skills/




Use fast logic tests, XAML build validation, and a minimal startup/navigation smoke test without pixel comparisons.

Don't perform any process termination, clean up. Just do logic level test (In test files, briefly describe about that test case)




why there isn't full screen mode?

The flyout isn't triggering at all.


I can't able to activate the profile, why?

The crosshair isn't activated at all, Even though it's showing - profile [Fortnite .Active] (image-ref)[]


what are all the Gaming dependency function we could use in our windows desktop Gaming optimizer (Doc)[https://docs.rs/windows-sys/0.61.2/windows_sys/Win32/Gaming/index.html]



Not working - profile activation, crosshair, (showing a mock up UI), 



### Macros
- can't assign keys to the Macros sequence
- can't assign shortcut to the Macro
- Record actions isn't playable
- 
- 




msstore publish
















## copilot chat

https://github.com/copilot/share/0a2f120c-4b24-8c17-8003-dc03605f080a

https://github.com/copilot/share/404e421c-4224-8833-b910-5c4a64576858




wpf-to-winui3-migration



why should we have to use WPF instead of WINUI3? 

Are we crosscompiling the Rust and C# via the wpf?

why the `EdgeOptimizer_Settings`



what is that file ending with 'pdb' - Program Debug Database?


After building, why there is 'examples' and 'incremental' folder?


MSI or exe installer

update





## Codex Agents
 <web-search> use it, to check the compatibility of the stack






 You built the **Debug** configuration, so the Release executable does not exist yet.

Run the Debug output instead:
```
dotnet build .\apps\EdgeOptimizer.Settings.WinUI\EdgeOptimizer.Settings.WinUI.csproj --configuration Debug --runtime win-x64 --no-restore

```
```powershell
& .\apps\EdgeOptimizer.Settings.WinUI\bin\Debug\net10.0-windows10.0.19041.0\win-x64\EdgeOptimizer.Settings.WinUI.exe
```

To create and run a Release build:

```powershell
dotnet build .\apps\EdgeOptimizer.Settings.WinUI\EdgeOptimizer.Settings.WinUI.csproj --configuration Release --runtime win-x64 --no-restore

& .\apps\EdgeOptimizer.Settings.WinUI\bin\Release\net10.0-windows10.0.19041.0\win-x64\EdgeOptimizer.Settings.WinUI.exe
```