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

> **2026-05-28 — architectural pivot (the continuous LISP).** UFL adopts a
> **homoiconic S-expression** as its single surface syntax and IR; atoms are
> *forms* in one uniform tree. After a three-lens review the **synthesis** was
> chosen: the S-expression is a front-end that **lowers into the retained typed
> core** (`Eml` now, `Multivector` later) — homoiconicity and tree-rewriting
> *with* the typed enums' structural safety, not instead of it. This reshapes
> the roadmap: R-0003 is the s-expression core; the geometric / predicate /
> substrate atoms become *forms that lower into their typed cores*. See
> [R-0003](requirements/0003-sexpr-core.md) §6 for the decision and the review
> outcome.

### M1 — Numeric core & the typed spatial layer

The typed cores the s-expression forms lower into. Target crate: `ufl-core`.

| Req | Capability | Atoms / Pillar | Spec | Status |
|-----|------------|----------------|------|--------|
| [R-0001](requirements/0001-eml-operator-core.md) | EML operator core (the `eml` lowering target) | `eml` · numeric core | [SPEC-0001](specs/0001-eml-operator-core.md) | Done |
| [R-0002](requirements/0002-geometric-algebra-core.md) | Geometric algebra over G(3,0,0) — *via garust* (the GA lowering target) | 𝒢ₖ ∗ · Pillar 2 | SPEC-0002 (GA flow) | Accepted — building |

R-0002 note: realization **pivoted to garust** (2026-06-02). UFL's GA core is a
thin layer over `garust::Multivector<Complex<f64>, 3,0,0,8>`, contingent on a
garust `Scalar` split so `Complex<f64>` is an admissible coefficient. Built by a
separate **GA agent flow**, not the main session. The hand-rolled attempt
(frozen branch `c92a38a`) is superseded prior art (rotor oracle + conventions
carry over).

### M2 — The s-expression core  ·  *current*

UFL's single homoiconic surface/IR: the tree all atoms are *forms* within,
lowering into the typed cores. Target crate: `ufl-syntax` (new, → `ufl-core`).

| Req | Capability | Atoms / Pillar | Spec | Status |
|-----|------------|----------------|------|--------|
| [R-0003](requirements/0003-sexpr-core.md) | Homoiconic s-expression core — `Sexpr` AST + reader + lowering (`eml` form → `Eml`) | the AST itself | [SPEC-0003](specs/0003-sexpr-core.md) | **Done** (merged, PR #11) |

R-0003 *builds on* R-0001 (its `Eml` is the lowering target) and absorbs the
originally-planned surface syntax and evaluator.

### M3 — Forms on the core

The remaining atoms, each an s-expression *form* lowering into its typed core.

| Req | Capability | Atoms / Pillar | Status |
|-----|------------|----------------|--------|
| [R-0004](requirements/0004-predicate-layer.md) | **Predicate form (`⟦P⟧`) — the checker** (boolean substrate for control) | Pillar 3 | Spec'd ([SPEC-0004](specs/0004-predicate-layer.md), three-lens passed) |
| R-0005 | Geometric forms (`𝒢ₖ`, `∗`) — lower into R-0002's garust multivector | Pillar 2 | Backlog (after R-0002 GA core) |
| R-0006 | Substrate form + CPU substrate (`⊗`) | Pillar 4 | Backlog |

Predicate (R-0004) is built next by the **main session**; geometric forms
(R-0005) wait on the R-0002 GA core (built by the GA flow). The two are
independent — predicates need booleans + the s-expr core (done); geometric
forms need the garust multivector.

### M4 — Later

| Req | Capability | Source | Status |
|-----|------------|--------|--------|
| R-0007 | Macros / quasiquote (exploit homoiconicity) | LISP metaprogramming | Backlog |
| R-0008 | Grade-filtered neural layer | proposal §4 | Backlog |
| R-0009 | GAPU mapping + reservoir experiment | proposal §5 | Backlog |
| R-0010 | Log–GA compatibility (reconsidered — Q1 partly dissolves under EML-as-representation) | bridge · Q1 | Backlog |

## Sequencing rules

- A requirement enters `Discussing` only when every requirement it depends on is
  `Done`.
- M3's form requirements (R-0004..R-0006) depend on the R-0003 s-expr core.
- The proposal's §8 open research questions are tracked inside the requirement
  that must resolve them (Q1 → R-0010, Q2 → R-0004, Q3 → R-0006).
- This file is updated by the orchestrator whenever a requirement changes state.
  A full re-numbering/reflow after the pivot is the orchestrator's to finalize.

## Current focus

**R-0004 (predicate form `⟦P⟧`)** — the next build, in the **main session**.
This is where control universality (the "all computable" discharge — see
[`theory/universal-computability.md`](theory/universal-computability.md)) begins:
predicates over pre/post state, as s-expression forms lowering over the
EML-valued core. Discussion/scoping next, then the eight-step loop.

**In parallel (separate GA agent flow):** R-0002 — the garust-based GA core
([`requirements/0002-geometric-algebra-core.md`](requirements/0002-geometric-algebra-core.md)),
contingent on the garust `Scalar` split for `Complex<f64>`.

**Done:** R-0001 (EML core) and R-0003 (s-expr core) — `(eml 1 1)` evaluates
from text, reusing the verified evaluator.
