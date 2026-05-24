
  * # UFL — Engineering Constitution

This file governs how UFL is built. It applies to every session, every agent,
and every change. UFL is a research language, but it is engineered to a
world-class standard — the rigor *is* the point.

Claude must read and honour this file in every session in this repository.

## 1. Prime directives

1. **Nothing is implemented without an accepted requirement and an accepted
   spec.** Code with no `requirements/` + `specs/` entry behind it does not get
   written.
2. **Decisions are made together.** Gustavo (Goose) is the final authority.
   Claude and the agents propose, analyse, and recommend — they never decide
   unilaterally. Every non-trivial choice is discussed and recorded in the
   relevant decision log.
3. **Describe before you build.** Before writing implementation code, Claude
   describes the approach in chat and provides a code snippet. Gustavo reviews
   it; we refine it together locally if needed. Only after sign-off does it
   become a committed file.
4. **Test-first, always.** A failing test exists before the code that satisfies
   it. No exceptions.
5. **Every change lands via a GitHub PR** reviewed by the architect agent and
   by Gustavo. `main` is never committed to directly.

## 2. Code philosophy — non-negotiable

- **Clean** — no dead code, no commented-out blocks, no TODO graveyards. Each
  unit does one thing.
- **Composable** — small, orthogonal pieces that combine. Prefer pure functions
  and explicit data flow over hidden state.
- **Clear** — the obvious reading is the correct reading. No cleverness that
  needs a comment to defend it.
- **Readable** — code is read far more than written. Names carry intent; control
  flow is shallow; functions fit on a screen.
- **SOLID** — single responsibility, open/closed, Liskov-safe, interface-
  segregated, dependency-inverted. Applied with judgement, not dogma.
- **Well-structured** — clear module and crate boundaries; dependencies point
  inward toward the core; no circular deps.
- **Best practices** — idiomatic Rust, no warnings, no premature abstraction,
  no premature optimization. Three similar lines beat the wrong abstraction.

## 3. The SDLC and the agent fleet

| Role | Who | Responsibility |
|------|-----|----------------|
| Product owner & final authority | **Gustavo (Goose)** | Approves requirements, specs, code outlines, and PRs. |
| Engineer | **Claude — main session** | Drives the loop; writes code after sign-off; opens PRs. |
| Scrum master / PM | **orchestrator agent** | Plans, tracks state, sequences the backlog, reports status. Writes no product code. |
| Architect | **architect agent** | Reviews every spec design and every PR for architecture quality and spec adherence. |
| Adversarial critic | **hater agent** | Runs on every spec design (loop step 2). Models the harshest competent reviewer the work will meet — finds hidden assumptions, hand-waved claims, edge cases, footguns, scenarios where it dies in the wild. Every finding is specific, citeable, technical. |
| Constructive critic | **nice-guy agent** | Runs on every spec design (loop step 2). Models the best mentor the work will meet — finds genuinely strong design moves, transferable patterns, surprising upsides, opportunities to amplify. Every finding is specific, citeable, technical. |
| QA | **qa agent** (one run per requirement) | Derives tests from acceptance criteria, owns e2e tests, signs off that a requirement is met. |

Note: in Claude Code a subagent cannot spawn another subagent. The
**orchestrator** therefore produces a plan/status report; the **main session**
executes it by invoking the architect, hater, nice-guy, and qa agents. The
orchestrator decides *what is next*; the main session and Gustavo carry it out.

The architect, hater, and nice-guy form a **three-lens review** at loop step 2:
architect asks *is it correct?*, hater asks *how does it die in the wild?*,
nice-guy asks *what does it unlock?* Different lenses, all three required for
every spec design. Routine PRs use the architect alone.

## 4. The requirement loop

Every requirement `R-NNNN` passes through these eight steps. None is skipped.

1. **Discuss** — Gustavo + Claude agree the requirement. Write
   `requirements/NNNN-*.md`. Acceptance criteria decided together.
2. **Spec** — write `specs/NNNN-*.md` realizing the requirement. The
   **three-lens review** runs: the architect agent reviews the design, the
   hater agent stress-tests it adversarially, and the nice-guy agent
   identifies strengths and opportunities. The spec moves `Draft → Accepted`
   when the architect approves *and* the hater/nice-guy findings have been
   addressed or explicitly deferred (with a decision-log entry recording
   why).
3. **Test plan** — the qa agent, scoped to `R-NNNN`, derives unit + e2e test
   cases from the acceptance criteria. Tests are written first and fail (red).
4. **Code outline** — Claude describes the implementation in chat with a
   snippet. Gustavo reviews; we modify it together locally if needed.
5. **Implement** — write code to make the tests pass (green), honouring §2.
6. **PR** — open on GitHub. The architect agent and Gustavo review.
7. **QA sign-off** — the qa agent verifies every acceptance criterion and runs
   unit + e2e suites.
8. **Merge & track** — the orchestrator updates `ROADMAP.md` and the registers.

## 5. Testing standard

- **TDD** — red → green → refactor. The failing test comes first.
- **Unit tests** per module; **e2e tests** per requirement.
- The qa agent owns each requirement's acceptance tests.
- `cargo test`, `cargo clippy`, and `cargo fmt --check` all green is a hard
  merge gate.

## 6. Rust conventions

- Edition 2021. One crate per bounded responsibility (see `crates/README.md`).
- No `unwrap`/`expect`/`panic!` in library code — return `Result`. Panics are
  for genuinely unreachable states only, with a justifying message.
- Every public item is documented. Doc examples compile.
- No `unsafe` without a spec section justifying it and an architect review.
- Errors are typed (`thiserror` or explicit enums), never stringly-typed.

## 7. Git & PR

- `main` is protected. All work happens on `R-NNNN-short-name` branches.
- One PR per requirement (or per coherent spec); small, reviewable diffs.
- PR description links the requirement and spec ids and shows the test results.
- Conventional, why-focused commit messages. Never skip hooks.

## 8. Source-of-truth files

| File / dir | Holds |
|------------|-------|
| `CLAUDE.md` | this constitution |
| `ROADMAP.md` | milestones + requirement backlog + status |
| `requirements/` | what UFL must do (`R-NNNN`) |
| `specs/` | how each feature is built (`SPEC-NNNN`) |
| `.claude/agents/` | the agent fleet |
| `docs/conventions.md` | notation & writing conventions (e.g. `τ` as the circle constant) |
| `theory/` | formal definitions (atoms, predicates, log-GA bridge) |
| `docs/ufl-first-draft.md` | the original research proposal |

When in doubt, the roadmap says what is next; the requirement says what; the
spec says how; this file says to what standard.
