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

> **2026-05-28 — architectural pivot.** UFL adopts a **homoiconic
> S-expression** as its single AST (the "full LISP" direction); the per-layer
> *typed enums* are dropped in favour of one uniform tree whose atoms are
> *forms*. This reshapes the roadmap below: R-0003 is now the s-expression core
> (absorbing the originally-planned surface-syntax + evaluator work), and the
> geometric / predicate / substrate atoms are re-expressed as *forms on the
> core*. See [R-0003](requirements/0003-sexpr-core.md) for the decision and the
> recorded tradeoff.

### M1 — Numeric core & the (paused) typed spatial layer

Target crate: `ufl-core`.

| Req | Capability | Atoms / Pillar | Spec | Status |
|-----|------------|----------------|------|--------|
| [R-0001](requirements/0001-eml-operator-core.md) | EML operator core | `eml` · numeric core | [SPEC-0001](specs/0001-eml-operator-core.md) | Done |
| R-0002 | Geometric algebra over G(3,0,0) — *typed enums* | 𝒢ₖ ∗ · Pillar 2 | SPEC-0002 | **Paused** |

R-0002 note: the typed-enum implementation is **paused** after the pivot, frozen
on branch `R-0002-geometric-algebra` (tip `c92a38a`, TDD-red, recoverable). Its
GA behaviour and qa test plan will be re-expressed as s-expression forms in M3.

### M2 — The s-expression core  ·  *current*

UFL's single homoiconic AST: the tree all atoms are *forms* within.
Target crate: `ufl-core` (reshaped).

| Req | Capability | Atoms / Pillar | Spec | Status |
|-----|------------|----------------|------|--------|
| [R-0003](requirements/0003-sexpr-core.md) | Homoiconic s-expression core — `Sexpr` AST + reader + evaluator (`eml` form) | the AST itself | SPEC-0003 | Discussing |

R-0003 supersedes R-0001's `Eml` *representation* (behaviour re-verified through
the s-expr path) and absorbs the originally-planned surface syntax and evaluator.

### M3 — Forms on the core

The remaining atoms, each re-expressed as an s-expression *form* rather than a
bespoke typed AST.

| Req | Capability | Atoms / Pillar | Status |
|-----|------------|----------------|--------|
| R-0004 | Geometric forms (`𝒢ₖ`, `∗`) — re-expresses the paused R-0002 | Pillar 2 | Backlog |
| R-0005 | Predicate form (`⟦P⟧`) | Pillar 3 | Backlog |
| R-0006 | Substrate form + CPU substrate (`⊗`) | Pillar 4 | Backlog |

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
  that must resolve them (Q1 → R-0010, Q2 → R-0005, Q3 → R-0006).
- This file is updated by the orchestrator whenever a requirement changes state.
  A full re-numbering/reflow after the pivot is the orchestrator's to finalize.

## Current focus

**R-0003 (homoiconic s-expression core)** — the pivot. Requirement drafted
([`requirements/0003-sexpr-core.md`](requirements/0003-sexpr-core.md)), pending
owner acceptance. On acceptance it goes through the three-lens review and then
SPEC-0003. R-0001 is **Done** (merged, qa-signed); R-0002 is **Paused** (typed
GA, frozen on its branch) pending re-expression as s-expr forms.
