---
name: blueprints
description: Create, reorganize, and maintain architecture.md and blueprints/ as a progressively disclosed technical documentation system. Use when an agent must establish a feature-blueprint structure, add or update a feature blueprint, define shared architecture contracts, create manifest-based task routing, split duplicated design notes, align documentation with code and tests, or validate links and dependencies between architecture and feature documentation.
---

# Blueprints

Maintain a documentation system in which agents load only the context needed for a task while preserving one authoritative source for shared technical contracts.

## Workflow

### 1. Discover the repository contract

1. Read applicable repository instructions before editing.
2. Locate existing architecture, feature specifications, agent instructions, implementation notes, and test documentation.
3. Inspect relevant code and tests before asserting current behavior or completion status.
4. Reuse established documentation paths when they are coherent. When creating the system from scratch, use `architecture.md` and `blueprints/`.
5. Preserve unrelated user changes.

Treat verified code and tests as authoritative for current behavior. Treat explicit requirements as authoritative for intended behavior. Label conflicts, uncertainty, and unverified claims instead of guessing.

### 2. Separate shared and feature-specific knowledge

Put a rule in `architecture.md` when multiple features must obey it or when changing it could affect multiple features. Typical shared material includes:

- system boundaries and component responsibilities;
- data ownership and lifecycle;
- shared representations, invariants, and formulas;
- cross-cutting workflows;
- persistence, recovery, concurrency, security, or failure contracts;
- testing boundaries shared by several features.

Put material in a *BLUEPRINT* when it describes one user-facing capability or one cohesive feature domain. State how shared contracts affect that feature without copying their definitions.

Do not make both documents authoritative for the same invariant. Link to an exact architecture heading, then describe only the feature-specific consequence.

### 3. Create or update `architecture.md`

1. Give every reusable contract a stable, descriptive heading.
2. Define each invariant once, including units, coordinate spaces, ownership, ordering, and error semantics where applicable.
3. Describe cross-cutting flows from input through state changes, side effects, and verification.
4. Link to the blueprint manifest rather than embedding complete feature specifications.
5. Keep product wishes and unverified implementation claims distinguishable from enforced contracts.

Read [references/templates.md](references/templates.md) when creating a new architecture document or normalizing an inconsistent one.

### 4. Create or update the blueprint manifest

Use `blueprints/README.md` as the routing manifest.

For every feature domain, record:

- task concepts and synonyms;
- the primary blueprint;
- exact applicable `architecture.md` headings;
- dependent or affected blueprints to inspect;
- principal implementation and test areas.

Route by task intent, not only by filenames, because shared files often implement several features. Prefer feature-domain filenames in kebab case; avoid sequence-based names such as `feature-01.md`.

Define these loading rules in the manifest:

1. Load one primary blueprint for an isolated feature task.
2. Load only its linked architecture sections for ordinary feature work.
3. Load impact-check blueprints only when the requested change can affect them.
4. Load the complete architecture and all manifest-listed consumers for structural or cross-cutting contract changes.
5. Inspect code and tests before expanding context when routing is uncertain.
6. Do not load unrelated blueprints.

When the repository has agent instruction files and the task includes establishing progressive loading, add a concise equivalent routing protocol to those files. Point them to the manifest; do not duplicate the full routing table.

### 5. Create or update each feature blueprint

Keep each blueprint independently useful after its declared dependencies are loaded. Include:

1. `Outcome`
2. `Current verified status`
3. `Architecture dependencies`
4. `Feature-specific implications`
5. `Related blueprints`
   - required dependencies;
   - impact-check dependencies.
6. `Relevant implementation and tests`
7. `Acceptance criteria`
8. `Remaining gaps`

Link architecture dependencies to exact headings. For every dependency, explain its concrete consequence for this feature. Use evidence-based status labels such as `Implemented`, `Partial`, `Planned`, `Deprecated`, or `superseded by {blueprintName}`, and include verification evidence or a verification date when useful.

Read [references/templates.md](references/templates.md) for reusable document shapes and [references/examples.md](references/examples.md) for routing and dependency examples.

### 6. Propagate changes deliberately

For a feature-only change:

1. Update the primary blueprint.
2. Check only linked architecture contracts and impact-check blueprints.
3. Update the manifest if concepts, ownership, code areas, or dependencies changed.

For a shared-contract change:

1. Update `architecture.md` first.
2. Find every manifest entry linked to the changed heading.
3. Inspect and update each consumer's implications, status, criteria, and gaps.
4. Check implementation and tests for contract drift.

For a newly discovered feature:

1. Decide whether it is a distinct capability or part of an existing cohesive domain.
2. Add or extend a blueprint accordingly.
3. Add task concepts, architecture links, impact checks, and code/test areas to the manifest.

### 7. Validate before finishing

Verify:

- all local Markdown links and architecture anchors resolve;
- every manifest blueprint exists and every blueprint appears in the manifest;
- every blueprint contains the required contract sections;
- shared invariants are defined once in `architecture.md`;
- feature implications do not silently redefine shared contracts;
- cross-cutting headings list all known consumers;
- status claims agree with inspected code and tests;
- renamed or superseded documentation has no stale references;
- ordinary routing examples load the smallest sufficient document set.

Run repository-provided documentation checks when available. Otherwise use non-destructive link, heading, reference, and duplicate-content checks appropriate to the environment.

Report what changed, what was validated, and any claims that remain unverified.

## Guardrails

- Do not invent project-specific architecture, feature status, paths, or test coverage.
- Do not rename or delete existing documentation unless the request authorizes the migration and references are updated.
- Do not turn `architecture.md` into a collection of feature specifications.
- Do not make blueprints depend on hidden conversational context.
- Do not mark work complete based only on prose.
- Do not duplicate the manifest across agent instruction files.
