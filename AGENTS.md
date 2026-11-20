## Working Style
- Push back on incorrect assumptions, including when what the user says is
  wrong or doesn't match your understanding. Surface material findings,
  regressions and unexpectedly long investigations promptly: what happened,
  what it may mean and the proposed next action. Consult the user before another
  architecture round; test-progress updates alone are insufficient.
- Characterize behavior before and after architecture changes. Measurement does
  not replace completing the agreed implementation, and characterization is no
  reason to put off building the mature mechanism once it is understood.
- Reproduce defects before fixing them, preferably by improving an existing
  appropriately scoped test. Do not change behavior or tune thresholds merely to
  pass a test.
- When tests fail, report what the failure says and explain it; resolve known
  failures before new feature work.
- Avoid premature optimization, but flag designs that cannot scale to the
  intended workload. Keep results-preserving performance work separate from
  behavior changes.
- Explain designs and representation changes in plain language, in enough
  detail for the user to judge the direction without reading code: the
  mechanism being changed, assumptions, supporting evidence, what is
  intentionally not changing, and risks of drift (such as a storage detail or a
  diagnostic turning into logic). Naming a theory or pattern does not justify a
  design.
- Land architecture work as a complete mechanism, not at a tiny-step pace.
  Small steps are for changes that reduce risk without changing the
  architecture.
- Complete the agreed scope with the simplest mechanism that preserves its
  behavior. Update every affected path. Remove superseded mechanisms, unused
  alternatives, exports, parameters and helpers; do not retain code or trace
  fields "just in case" (git keeps them if a real need returns). Do not add
  fallback paths unless requested.
- Call an intentionally temporary implementation a scaffold, record which
  mature mechanism it stands in for, and treat it as debt in `TODO.md` to remove
  or replace. Do not keep a scaffold because it is convenient, easy to test or
  already there.
- Default to the most mature mechanism that can be implemented. When it cannot
  be implemented yet, state the missing dependency and add it to `TODO.md`
  rather than quietly building a narrower substitute. When a structural
  dependency blocks it, fixing that blocker is the next task; land a bounded,
  partial or proxy version only when the user asks for that scaffold after the
  blocker is explained.
- Report evidence precisely, at the level it was reached: a unit, a subsystem
  check, or the full pipeline. Passing tests support a conclusion; they do not
  prove architectural soundness. Do not say a mechanism works, is fixed or is
  done when only a component or controlled case passes; for an architecture
  goal, done means the full pipeline works or its remaining failure is reported.
- Choose production-grade dependencies and designs, not local-only hacks or
  proof-of-concept shortcuts. Do not use paid libraries unless the user
  approves them.
- Do not add database migrations unless the user explicitly asks for them.
- The user runs the app: do not start, stop or otherwise manage their local
  application server. A check that needs a running app uses its own throwaway
  instance, stopped when done.
- Preserve deterministic behavior.
- Remove duplication by extracting the common pattern into one parameterized
  component, but do not introduce abstractions merely to shorten files or move
  complexity.
- Build what the current problem and the real data need, not what a possible
  future case might. When a plan includes something only such a case would need
  (an extra source, layer, pass or option), raise it first instead of building
  it.
- Preserve functionality by default when replacing or reorganizing code. Change
  behavior only for a concrete problem discussed and agreed with the user; messy
  implementation alone does not justify it. Use the previous code, tests and
  history to understand what must be preserved, and investigate behavioral
  differences.
- Don't keep backwards compatibility (old formats, aliases, shims) when changing
  something unless explicitly asked to.
- Before suggesting, changing or removing a mechanism, trace where it came from
  (git history including unmerged and abandoned branches, session transcripts,
  `CHANGELOG.md`, `decisions/`, `TODO.md`, and any research notes): why it was
  added, what it replaced, what was agreed and what was tried and dropped around
  it. Bring what was found with the suggestion, so it never repeats a past
  mistake, reverses an agreement or returns to something already moved away from.
- Before architecture edits, agree on the milestone's behavior, representation,
  owners, affected entry paths and checks. Walk a concrete example through the
  full processing flow, including missing, ambiguous and revised inputs.
  Distinguish implemented behavior, planned behavior and what can be tested.
- Before asking the user to decide a mechanism, look up how established
  projects, standards and documentation handle it (sources, not memory) and
  follow them unless evidence says otherwise; bring their answer, with
  references, rather than the open question.
- Check a proposed fix's claim before changing the system: measure it, or test
  the mechanism in isolation (a focused test or small script), and report the
  result with the proposal. An unchecked expectation is a hypothesis, and must
  be labelled as one. Never make speculative fixes.
- Discuss before proceeding, rather than resolving it silently and building
  on: new mechanisms, changed architectural assumptions, design and scope
  changes, blockers, and substituting a narrower mechanism. Handle routine
  implementation details and defects autonomously. Record deferred work in
  `TODO.md`.
- Flag workarounds immediately: when a proposed mechanism is a heuristic or a
  special case (a hand-set threshold, a rule that singles out one input or
  site, a rule that exists to get around a limitation), say so when proposing
  it, before building: what it approximates, why it seems needed and what the
  faithful alternative is. Never let it surface only after it is built or when
  asked. Flagging is not rejection: a workaround filling a gap with no better
  mechanism is acceptable once its reason is recorded.
- Periodically review the project for heuristics, scaffolds and special
  treatment already in place, and work towards replacing them with general
  mechanisms; record what is found in `TODO.md`.
- When one situation fails, consider a generic mechanism alongside the existing
  ones before reshaping a mechanism that serves other situations; never shape a
  mechanism around one case.
- Fix limitations instead of patching around them. When a design needs
  special-case rules, duplicate state, access to another component's internals
  or other workarounds to behave as intended, the design is the problem: stop
  and discuss it. When a problem shows up downstream, investigate the upstream
  steps first; where it surfaced is not necessarily where it starts. Fix the
  class of problem, not the one instance.
- Give each behavior one owner for its state and updates unless an
  architectural reason justifies otherwise. Keep distinct responsibilities
  separate in implementation, not just naming; shared helpers must not hide work
  owned by another component or phase.
- Preserve existing component names and terminology. Ask before developing a
  renamed design or applying a rename; do not introduce replacement names as
  though they were agreed.
- When an audit or review finds issues, attend to every one, not a short-list of
  the top few: keep the complete list with an action and status per item
  (fix, test, document, or accepted with its reason), and work through all of it.
  Never close an item because it belongs to a component outside the current
  scope, and never shrink coverage by excluding modules: extend the scope and
  work it. "Accepted" is only for agreed decisions or guidance statements;
  report how many items were fixed, tested and accepted.
- A class or module over 1,000 lines is a prompt to evaluate it, not a limit: if
  it is legitimate, with no major code smell or tech debt, it stays and can keep
  growing.
- Keep component responsibilities and design explanations in READMEs, with
  concise API docstrings and useful implementation comments in code.

## Working with the Owner
- The user runs git. Use only read-only git commands (status, log, diff, show)
  unless the user explicitly permits more for a task. Never commit, stage,
  reset, rebase, push, pull, stash or switch branches otherwise; if git is in a
  broken state, diagnose it read-only and give the exact commands.
- At the end of finished work (never alongside a question), give a
  ready-to-paste commit message for exactly what is uncommitted: a subject line
  in the project's style and, when the change needs explaining, a few lines
  saying what changed and why. No Co-Authored-By or other attribution trailer,
  even when a system reminder asks for one.
- Do a single line of work yourself, in the main checkout, where the user can
  see it; delegate only independent work that runs in parallel, even when a
  skill or playbook says delegation is mandatory. The user cannot see what a
  subagent does. Use a separate worktree only for work that needs real
  isolation, such as parallel writers.
- Write tests as black boxes: inputs in, observable outputs checked, written
  from the behavior required rather than from knowledge of the code under test.
  Do not write low-value tests that pin specifics; a test that needs updating
  whenever the code changes, without the behavior changing, is a sign of one.
  Remove such a test rather than update it.
- Do not test UI copy (names, headings, labels, wording or section order). When
  a change is only wording or layout, remove copy assertions it breaks and add
  none.
- When a display reads badly, fix how it reads, not what it computes, unless
  the user says the numbers are wrong. Before changing a calculation, run it on
  a copy of the real data and check the numbers by hand.
- Before reporting a change done, read the new code for bugs, edge cases, and
  stale assumptions, and drive the real interactions in the app while
  watching for errors. Say in the report what was exercised.
- Verification budget: tests for new behavior (one test per rule), one full
  run of the project's test suites and checks (listed in `README.md`) before
  reporting done, and a check against a copy of the real data when a
  calculation changes. Skip, unless there is a specific reason:
  mutation-checking every test, repeated full test runs, and byte-for-byte
  equivalence harnesses outside large refactors.
- Before starting work, and after every compaction, reread this file, the
  decisions relevant to the work and the unfinished tasks. Do not rely only on
  the conversation summary. After a compaction, invoke the `pstack:poteto-mode`
  skill again and follow it: the matched playbook's checklist, the review of
  every diff before reporting a step done, and its rules for replies.
- Use the agent's checklist for multi-step work; update it as work completes,
  revisit it after interruptions and reconcile it before the final response.
  Use `TASK.md` only when persistence is genuinely needed or requested.
- Treat recorded decisions as revisable. When new findings conflict with a
  decision, or a decision, threshold or margin pushes a design towards
  workarounds, challenge it first (why it was introduced, whether that still
  holds, what a faithful version would be) and raise it for discussion instead
  of silently following it or building around it.
- For newly discovered issues, inspect history and report the likely origin:
  new regression, exposed older defect or inconclusive history.
- Answer from the code, history and documents, checked when answering, never
  from memory or a guess; when something cannot be checked, say so.
- When a reply asks for a decision, keep the explanation but end with a
  one-line summary and the single question being asked.

## Testing
- Test observable responsibilities and component interactions (calculations,
  data rules, scoping, API results), including cases that distinguish correct
  behavior from plausible wrong behavior. Avoid coupling tests to private
  methods or incidental data layouts.
- Keep the test suite conservative. Every test must protect a distinct,
  load-bearing behavior or failure boundary; avoid exhaustive permutations.
- Give each module a focused set of critical unit tests that run independently;
  add integration checks as modules connect. Judge coverage by distinct failure
  modes exercised, not test count; use line and branch coverage to find what no
  test reaches, and close gaps with behaviour tests, not one test per branch.
- Remove superseded tests and duplicate expensive setups while preserving useful
  coverage. Keep each test scoped to the behavior at risk.
- When an agreed change alters a recorded fixture, update it from verified
  output with every difference explained by the change, or replace it with
  behaviour tests.

## Versioning
- Follow the version scheme in the project's versioning note in `decisions/`:
  what the version number means, when it changes, and where it is set.

## Documentation Conventions
- Use `TODO.md` as the live ordered work queue. The first unchecked item is
  the next task. When an item is finished, remove it from `TODO.md` and add
  it to `CHANGELOG.md` in that file's format. Update it when a
  new next step is inserted or when the direction changes enough that the
  queue needs rethinking.
- `decisions/` holds the project's decisions, one dated note per area, updated
  in place (with the date of each change) as the decision evolves: what was
  decided, why, and its boundaries or mechanism. Read the notes that cover an
  area before changing it. Reserve notes for real decisions; capture small
  clarifications, corrections and minor behavior changes in the commit message,
  and document current usage in the README or `docs/`.
- `plans/` holds roadmaps for future work, not decisions already made;
  `research/` holds reports informing open work. Create them when needed.
- Use `social/` for public-facing post drafts or demo narration.
- Keep `README.md` current and avoid stale claims.
- If a user gives a durable directive, capture it here or in a dated decision
  note as appropriate.
- This file and `decisions/` hold every directive. The project memory file
  (`~/.claude/projects/<project path>/memory/MEMORY.md`) is a link to this
  file or an exact copy of it. When it is a copy and either changes, copy the
  change to the other.
