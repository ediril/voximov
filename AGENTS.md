## Working Style
- Push back when what the user is saying is wrong or doesn't match your understanding.
- Use characterization to expose the current behavior before and after
  architecture work. Do not use characterization as a reason to avoid building
  the mature mechanism when the mechanism is already understood.
- Do not make changes just to make a test pass.
- When tests fail, report what the failure says.
- Avoid premature optimization, but flag designs that clearly will not scale
  past small column counts.
- Explain architecture and representation changes in enough detail for the user
  to assess direction without reading code. Summaries should include the
  mechanism being changed, assumptions being made, what is intentionally not changing, and any risks of drift such as storage handles or diagnostics becoming model logic.
- Do not move at "tiny step" pace by default. Small edits are useful when they
  reduce risk without changing the architecture, but architecture work should
  land as a complete mechanism.
- Prefer clean, single-purpose implementation shapes over keeping alternate
  paths, duplicate trace fields, or unused helpers "just in case." We use git;
  recreate removed code later if a real need returns.
- If an implementation is intentionally temporary, call it a scaffold explicitly,
  record what mature mechanism it is standing in for, and treat it as a debt
  item to remove or replace. Do not keep scaffolds merely because they are
  convenient, easy to test, or already present.
- Default to the most mature implementable mechanism. If the mature mechanism
  cannot be implemented yet, state the missing dependency explicitly and add it
  to `TODO.md` rather than quietly building a narrower substitute.
- If the mature mechanism is blocked by a structural dependency, make removing
  or fixing that blocker the next implementation task. Do not land a bounded,
  partial, or proxy version of the mechanism unless the user explicitly asks for
  that scaffold after the blocker is explained.
- Before calling UI or interaction work done, read the new code for bugs and
  run the browser tests, adding a spec that drives the new
  behavior. Passing rests and a screenshot only show the happy path renders.
- Speak precisely about verification level. Do not say a mechanism "works",
  "is fixed", or "is done" for an architecture goal when only an isolated unit
  test or controlled diagnostic passes. State the actual level of evidence:
  unit mechanism, subsystem diagnostic, full text pipeline, or full hierarchy
  pipeline. For architecture goals, completion means the relevant full pipeline
  works or the remaining full-pipeline failure is explicitly reported.
- Choose production-grade dependencies and designs instead of local-only hacks or proof-of-concept shortcuts.
- Production may move from shared hosting to a VM when OS-level dependencies are the right production solution.
- Do not use paid libraries unless the user explicitly approves them.
- Do not add fallback paths unless the user explicitly asks for them.
- Do not add database migrations unless the user explicitly asks for them.
- Do not start, stop, or otherwise manage the local application server; the user controls it.
- Do not tune thresholds just to make a test pass.
- Keep old comparison modes available when promoting a new default, unless there
  is a clear reason to remove them.
- Preserve deterministic behavior.
- Prefer extracting common patterns into reusable components to reduce duplication
- Favor simplicity over complexity - remove unused exports, parameters, and code paths
- Do not over-engineer or build in anticipation. Build what the current problem
  and the real data need. When a plan includes something only a possible future
  case would need (an extra source, layer, pass, or option), raise it and discuss
  it with the user first instead of building it.
- Eliminate code duplication through component extraction and parameterization
- Do not add code comments. Use clear names and focused functions.

## Working with the Owner
- The owner runs git. Use only read-only git commands (status, log, diff,
  show). Never commit, reset, rebase, push, pull, stash, or switch branches;
  if git is in a broken state, diagnose it read-only and give the exact
  commands. After every change, end with a ready-to-paste commit message for
  exactly what is uncommitted, with no Co-Authored-By or other attribution
  trailer.
- Work directly in this checkout so the owner can see changes as they happen.
  Do not use separate git worktrees or copies of the repository.
- Do implementation work yourself in the main session. Do not delegate
  building or editing to subagents; a separate read-only reviewer is fine for
  large or risky changes.
- Add tests only for behavior that could realistically break and that a user
  or caller would notice. Do not add tests that restate constants or
  implementation details.
- Do not test UI copy (tab or section names, headings, labels, wording, or
  section order). Test behavior: calculations, data rules, scoping, API
  results. When a change is only wording or layout, remove copy assertions it
  breaks and add none.
- When a display reads badly, fix how it reads, not what it computes, unless
  the owner says the numbers are wrong. Before changing a calculation, run it
  on a copy of the real data and check the numbers by hand.
- Before reporting a change done, read the new code for bugs, edge cases, and
  stale assumptions, and drive the real interactions in a browser while
  watching for errors. Say in the report what was exercised.
- Verification budget: tests for new behavior (one test per rule, a browser
  spec when there is a new interaction), one full run of the project's test
  suites and checks (listed in `README.md`) before reporting done, and a
  check against a copy of the real data when a calculation changes. Skip,
  unless there is a specific reason: mutation-checking every test, repeated
  full browser runs, screenshots after a spec already drove the interaction,
  and byte-for-byte equivalence harnesses outside large refactors.

## Versioning
- Follow the version scheme in the project's versioning note in `decisions/`:
  what the version number means, when it changes, and where it is set.

## Documentation Conventions
- Use `TODO.md` as the live ordered work queue. The first unchecked item is
  the next task. When an item is finished, remove it from `TODO.md` and add
  it to `CHANGELOG.md` in that file's format. Update it when a
  new next step is inserted or when the direction changes enough that the
  queue needs rethinking.
- Use `decisions/` for dated architectural decisions and current rationale.
  Reserve decision notes for bigger decisions; capture small clarifications
  and corrections in the commit message instead. Read the notes that cover
  the area you are about to change before changing it; they hold the
  project's own rules.
- Use `plans/` only for future work/roadmaps, not decisions already made.
- Use `social/` for public-facing post drafts or demo narration.
- Keep `README.md` current and avoid stale claims.
- If a user gives a durable directive, capture it here or in a dated decision
  note as appropriate.
- This file and `decisions/` hold every directive. The project memory file
  (`~/.claude/projects/<project path>/memory/MEMORY.md`) is a link to this
  file or an exact copy of it. When it is a copy and either changes, copy the
  change to the other.
