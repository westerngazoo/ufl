# UFL Roadmap

The single source of truth for what is being built and in what order.
Milestones group requirements; each requirement is realized by one or more
specs. Nothing moves without passing the requirement loop in
[`CLAUDE.md`](CLAUDE.md) §4.

## Status legend

`Backlog` → `Discussing` → `Spec'd` → `In progress` → `In review` → `Done`

## Milestones

### M0 — Foundation  ·  *in progress*

SDLC, repo scaffold, agent fleet, engineering constitution.

| Item | Status |
|------|--------|
| Repo scaffold + spec-driven structure | Done |
| `CLAUDE.md` engineering constitution | Done |
| `requirements/` + `specs/` registers | Done |
| Agent fleet (orchestrator, architect, qa) | Done |
| Reusable SDLC template extracted | In progress |

### M1 — Numeric & spatial core

The irreducible substrate: the EML operator and geometric algebra.
Target crate: `ufl-core`.

| Req | Capability | Atoms / Pillar | Spec | Status |
|-----|------------|----------------|------|--------|
| [R-0001](requirements/0001-eml-operator-core.md) | EML operator core | `eml` · numeric core | [SPEC-0001](specs/0001-eml-operator-core.md) | Done |
| [R-0002](requirements/0002-geometric-algebra-core.md) | Geometric algebra over G(3,0,0) | 𝒢ₖ ∗ · Pillar 2 | SPEC-0002 | Discussing |
| R-0003 | Log–GA compatibility (no precision blowup) | bridge · Q1 | SPEC-0003 | Backlog |

### M2 — Predicative logic layer

Programs as predicates over pre/post state. Target crate: `ufl-predicate`.

| Req | Capability | Atoms / Pillar | Spec | Status |
|-----|------------|----------------|------|--------|
| R-0004 | Hehner predicate layer | ⟦P⟧ · Pillar 3 | SPEC-0004 | Backlog |

### M3 — The UFL language

Surface notation and execution. Target crates: `ufl-syntax`, `ufl-eval`.

| Req | Capability | Atoms / Pillar | Spec | Status |
|-----|------------|----------------|------|--------|
| R-0005 | UFL surface syntax & AST | — | SPEC-0005 | Backlog |
| R-0006 | Evaluator (parse → predicate-check → evaluate) | — | SPEC-0006 | Backlog |

### M4 — Substrate orchestration

Substrate-agnostic compilation. Target crates: `ufl-substrate`, `ufl-cli`.

| Req | Capability | Atoms / Pillar | Spec | Status |
|-----|------------|----------------|------|--------|
| R-0007 | Substrate contract + CPU substrate | ⊗ · Pillar 4 | SPEC-0007 | Backlog |

### M5 — Neural & GAPU

| Req | Capability | Source | Spec | Status |
|-----|------------|--------|------|--------|
| R-0008 | Grade-filtered neural layer | proposal §4 | SPEC-0008 | Backlog |
| R-0009 | GAPU mapping + reservoir experiment | proposal §5 | SPEC-0009 | Backlog |

## Sequencing rules

- A requirement enters `Discussing` only when every requirement it depends on is
  `Done`.
- Requirements within M1 (R-0001..R-0003) depend only on M0; R-0001 and R-0002
  may be specced in parallel, R-0003 depends on both.
- The proposal's §8 open research questions are tracked inside the requirement
  that must resolve them (Q1 → R-0003, Q2 → R-0004, Q3 → R-0007).
- This file is updated by the orchestrator whenever a requirement changes state.

## Current focus

**R-0001 (EML operator core)** — **Done.** Merged (PR #7), architect-approved,
qa sign-off PASS, 20/20 tests green. `crates/ufl-core` ships the `Eml` tree and
reference evaluator; the `hello_eml` example and `docs/the-shape-of-ufl.md`
demonstrate it.

**R-0002 (geometric algebra over G(3,0,0))** — **Discussing.** Requirement
drafted ([`requirements/0002-geometric-algebra-core.md`](requirements/0002-geometric-algebra-core.md)):
multivectors over G(3,0,0) with atoms `𝒢ₖ` and `∗`, dense 8-coefficient
representation, complex coefficients reusing R-0001's `Value`. On acceptance it
becomes the first spec to go through the **three-lens review** (architect +
hater + nice-guy). R-0003 (log–GA compatibility) will be reconciled against the
EML primitive when its turn comes — with EML as IR rather than a domain, its
original Q1 framing is expected to partly dissolve.
