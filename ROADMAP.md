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
| [R-0004](requirements/0004-predicate-layer.md) | **Predicate form (`⟦P⟧`) — the checker** (boolean substrate for control) | Pillar 3 | **Done** (merged, PR #14; `ufl-predicate`, 34 tests) |
| R-0005 | Geometric forms (`𝒢ₖ`, `∗`) — lower into R-0002's garust multivector | Pillar 2 | Backlog (after R-0002 GA core) |
| R-0006 | Substrate form + CPU substrate (`⊗`) | Pillar 4 | Backlog |

Predicate (R-0004) is built next by the **main session**; geometric forms
(R-0005) wait on the R-0002 GA core (built by the GA flow). The two are
independent — predicates need booleans + the s-expr core (done); geometric
forms need the garust multivector.

### M5 — Discovery → Geometric Neuroevolution  ·  *current headline*

> **2026-06-04 — pivot.** After the [`ufl-discovery` PRD](https://docs.google.com/document/d/170cdfV8ZvglRa_9jz7Gr8MBV1WFyxNfxXFZ6G2Dxabo)
> review ([`ufl-discovery/FINDINGS.md`](ufl-discovery/FINDINGS.md)): an
> **AlphaTensor-style discovery engine** — rediscover Strassen by search + exact
> verification — built **Path B**, so the verifier *is* the Hehner predicate
> discharge of `P_n,R`.
>
> **2026-06-12 — direction set.** After reviewing the external Clifford-Lisp
> roadmap + five validating arXiv papers (CliffordNet 2601.06793; the Haynes
> Program-Hypergraph series 2603.25414 / 2603.18104; GATr/AB-GATr 2605.18816),
> UFL's Phase-1 centre of gravity is **neuroevolution of `Cl(3,0,1)` geometric
> ASTs** — the one thing that family of work *doesn't* do (all gradient-trained),
> and the thing UFL already half-built. The discovery engine is reframed as the
> **engine-validation step** of that program. *Coefficient field, R-0002
> supersession, and the minimal geometric gate are proposed (real `f64`;
> supersede; rediscover the sandwich `R x R̃`) — pending confirmation at R-0009.*

| Req | Capability | Status |
|-----|------------|--------|
| [R-0006](requirements/0006-integer-tensor-core.md) | **Exact integer-tensor core** (`ufl-tensor`) — `T_n`, scheme genotype, exact reconstruction + error. | **Done** (merged, PR #16) |
| [R-0007](requirements/0007-tensor-predicate.md) | **Tensor-equality predicate** — `P_n,R` as a Hehner discharge (`RankDecomposition`). Closes FINDINGS C1's discharge half. | **Done** (merged, PR #18) |
| [R-0008](requirements/0008-discovery-engine.md) | **Discovery engine** — seeded GA, accept step = the R-0007 discharge. **Re-scoped:** validate the loop + proposer-agnostic seam on a planted solvable target; **document the matmul falsification**. | In review (`ufl-discovery` green — planted recovery 7/10, AC4 guard every seed; engine implemented) |
| R-0009 | **`Cl(3,0,1)` PGA kernel** (real `f64`, via garust) — multivectors, geo/outer/inner products, rotors/motors. Supersedes R-0002 (G(3,0,0)). | Planned |
| R-0010 | **Geometric s-expr forms + grade inference** — GA ops as forms; the dimensional type system (decidable per Haynes). | Planned |
| R-0011 | **Neuroevolution + the stronger proposer** — R-0008's seam, genotype = AST, **memetic/agentic proposer** (the GA-VisAgent pattern); fitness = accuracy − parsimony − grade-entropy. **Inherits the relocated Strassen prize** + the geometric gate (rediscover `R x R̃`). | Planned |

### M4 / language-build — *paused for the discovery pivot*

Resumable when the discovery thread reaches a milestone. The value conditional
exploration is shelved on branch `R-0005-value-conditional` (recoverable).

| Req | Capability | Source | Status |
|-----|------------|--------|--------|
| R-0005 | Value conditional (`if b a c`) | control | Shelved (branch) |
| (lang) | GA s-expr forms (`𝒢ₖ`, `∗`) — lower into R-0002's garust multivector | Pillar 2 | Paused (after GA core) |
| (lang) | Substrate form + CPU substrate (`⊗`) | Pillar 4 | Paused |
| (lang) | Macros / quasiquote; grade-filtered neural layer; GAPU mapping; Log–GA compat | proposal §4/§5, Q1 | Paused |

## Sequencing rules

- A requirement enters `Discussing` only when every requirement it depends on is
  `Done`.
- M3's form requirements (R-0004..R-0006) depend on the R-0003 s-expr core.
- The proposal's §8 open research questions are tracked inside the requirement
  that must resolve them (Q1 → R-0010, Q2 → R-0004, Q3 → R-0006).
- This file is updated by the orchestrator whenever a requirement changes state.
  A full re-numbering/reflow after the pivot is the orchestrator's to finalize.

## Current focus

**R-0008 — the discovery engine (engine-validation step).** Steps 1–2 of Path B
are **Done**: R-0006 (the exact integer verifier, Strassen gate) and R-0007 (the
verifier *is* the Hehner discharge — `P_{2,7}(strassen) → Ok(true)`). The active
work is the seeded GA that finds an exact rank-7 scheme **without being given
Strassen**, accepting via the R-0007 discharge and emitting a re-verifiable
certificate — proving the genetic-search loop on a *known-answer* problem before
its genotype generalizes to geometric ASTs (R-0011). *Requirement accepted;
SPEC-0008 next.*

**The Phase-1 arc (decided 2026-06-12 — [[project-neuroevolution-direction]]):**
R-0008 (engine) → R-0009 (`Cl(3,0,1)` PGA kernel, real `f64`, supersedes R-0002)
→ R-0010 (geometric forms + grade inference) → R-0011 (neuroevolution over
geometric ASTs). The differentiator is **evolution**, which the validating
literature (CliffordNet, GATr, Haynes) doesn't do.

**Paused:** the language-build thread (R-0005 value conditional shelved on its
branch; substrate / macros / GAPU) — resumable later. **R-0002** (G(3,0,0) GA
core, separate GA flow) is **superseded by R-0009** (the signature moves to
`Cl(3,0,1)` PGA) — hand-off to be coordinated.

**Done:** R-0001 (EML core), R-0003 (s-expr core), R-0004 (predicate checker),
R-0006 (integer-tensor core), R-0007 (tensor predicate) — 153 tests green across
five crates.
