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
| [R-0001](requirements/0001-eml-operator-core.md) | EML operator core (the `eml` lowering target) | `eml` · numeric core | [SPEC-0001](specs/0001-eml-operator-core.md) | **Done** (merged) |
| [R-0002](requirements/0002-geometric-algebra-core.md) | Geometric algebra over G(3,0,0) — *via garust* | 𝒢ₖ ∗ · Pillar 2 | — | **Superseded by [R-0009](requirements/0009-pga-kernel-binding.md)** |

R-0002 note: **superseded by R-0009** (2026-06-12). The geometric-neuroevolution
direction needs `Cl(3,0,1)` **PGA** — its ideal/null generator `e₀² = 0` gives
translations and rigid-body motors natively, which G(3,0,0) VGA cannot express.
R-0009 binds garust's `Cl(3,0,1)` kernel over **real `f64`** (the `ufl-ga` crate,
shipped); the earlier `Complex<f64>`/G(3,0,0) plan is retired. R-0002's prior art
(the hand-rolled rotor oracle + conventions, frozen branch `c92a38a`) carries
over as reference.

### M2 — The s-expression core  ·  *current*

UFL's single homoiconic surface/IR: the tree all atoms are *forms* within,
lowering into the typed cores. Target crate: `ufl-syntax` (new, → `ufl-core`).

| Req | Capability | Atoms / Pillar | Spec | Status |
|-----|------------|----------------|------|--------|
| [R-0003](requirements/0003-sexpr-core.md) | Homoiconic s-expression core — `Sexpr` AST + reader + lowering (`eml` form → `Eml`) | the AST itself | [SPEC-0003](specs/0003-sexpr-core.md) | **Done** (merged, PR #11) |

R-0003 *builds on* R-0001 (its `Eml` is the lowering target) and absorbs the
originally-planned surface syntax and evaluator.

### M3 — Forms on the core

| Req | Capability | Atoms / Pillar | Status |
|-----|------------|----------------|--------|
| [R-0004](requirements/0004-predicate-layer.md) | **Predicate form (`⟦P⟧`) — the checker** (boolean substrate for control) | Pillar 3 | **Done** (merged, PR #14; `ufl-predicate`, 34 tests) |

R-0004 (the predicate checker) is the one M3 form that shipped. The other
originally-planned M3 forms were **overtaken by the discovery pivot** (2026-06-04)
and reframed:

- **Geometric forms** (`𝒢ₖ`, `∗`) → now **R-0010** (lowering onto the R-0009
  `ufl-ga` PGA kernel), in M5.
- **R-0005** was reframed as the **value conditional** (`if`) and is **shelved**
  on its branch (M4, paused).
- The **substrate form** (`⊗`) remains paused (M4).

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
> **engine-validation step** of that program. *Confirmed at R-0009 (2026-06-18):
> real `f64`; R-0002 superseded; the `Cl(3,0,1)` PGA kernel (`ufl-ga`) is shipped,
> with the rotor-sandwich `R x R̃` validated as its keystone — the same sandwich
> R-0011's neuroevolution will rediscover by search.*

| Req | Capability | Status |
|-----|------------|--------|
| [R-0006](requirements/0006-integer-tensor-core.md) | **Exact integer-tensor core** (`ufl-tensor`) — `T_n`, scheme genotype, exact reconstruction + error. | **Done** (merged, PR #16) |
| [R-0007](requirements/0007-tensor-predicate.md) | **Tensor-equality predicate** — `P_n,R` as a Hehner discharge (`RankDecomposition`). Closes FINDINGS C1's discharge half. | **Done** (merged, PR #18) |
| [R-0008](requirements/0008-discovery-engine.md) | **Discovery engine** — seeded GA, accept step = the R-0007 discharge. **Re-scoped:** validate the loop + proposer-agnostic seam on a planted solvable target; **document the matmul falsification**. | **Done** (merged, PR #19; planted recovery 7/10, AC4 working-engine guard every seed) |
| [R-0009](requirements/0009-pga-kernel-binding.md) | **`Cl(3,0,1)` PGA kernel binding** (`ufl-ga` → garust **v0.1.0**, contract green) — multivectors, geo/outer/inner products, grade projection, rotor sandwich, motors. Supersedes R-0002. | **Done** (merged, PR #21; `ufl-ga`, 15 tests — keystone + bit-exact PGA null + convention-equivalence) |
| R-0010 | **Geometric s-expr forms + grade inference** — GA ops as forms; the dimensional type system (decidable per Haynes). | Planned |
| R-0011 | **Neuroevolution + the stronger proposer** — R-0008's seam, genotype = AST, **memetic/agentic proposer** (the GA-VisAgent pattern); fitness = accuracy − parsimony − grade-entropy. **Inherits the relocated Strassen prize** + the geometric gate (rediscover `R x R̃`). | Planned |

### M4 / language-build — *paused for the discovery pivot*

Resumable when the discovery thread reaches a milestone. The value conditional
exploration is shelved on branch `R-0005-value-conditional` (recoverable).

| Req | Capability | Source | Status |
|-----|------------|--------|--------|
| R-0005 | Value conditional (`if b a c`) | control | Shelved (branch) |
| (lang) | GA s-expr forms (`𝒢ₖ`, `∗`) — now **R-0010**, lowering onto the R-0009 `ufl-ga` kernel | Pillar 2 | Active thread (M5) |
| (lang) | Substrate form + CPU substrate (`⊗`) | Pillar 4 | Paused |
| (lang) | Macros / quasiquote; grade-filtered neural layer; GAPU mapping; Log–GA compat | proposal §4/§5, Q1 | Paused |

## Sequencing rules

- A requirement enters `Discussing` only when every requirement it depends on is
  `Done`.
- Geometric forms (R-0010) depend on the R-0003 s-expr core (done) **and** the
  R-0009 PGA kernel (done); the grade-type system tracks the proposal's §8
  geometric-typing question.
- This file is updated by the orchestrator whenever a requirement changes state.

## Current focus

**R-0010 — geometric s-expr forms + the grade-type system, is next.** With the
substrate complete (R-0009 `ufl-ga`, the `Cl(3,0,1)` PGA kernel), R-0010 builds
the geometric s-expression *forms* that lower onto it, plus a **decidable
grade-type system** (the dimensional types Haynes proved decidable). That gives
R-0011 a typed geometric AST to evolve.

**The Phase-1 arc (decided 2026-06-12 — [[project-neuroevolution-direction]]):**
R-0008 (engine, **Done**) → R-0009 (`Cl(3,0,1)` PGA kernel, **Done**) → R-0010
(geometric forms + grade inference, **next**) → R-0011 (neuroevolution over
geometric ASTs, inheriting the relocated Strassen prize). The differentiator is
**evolution**, which the validating literature (CliffordNet, GATr, Haynes)
doesn't do; the engine rides UFL's exact verifier (the predicate discharge), so
*transparency lives in the verifier, not the proposer* — which let the blind-GA
proposer's failure on Strassen become a documented result rather than a dead end.

**Paused:** the language-build thread (R-0005 value conditional shelved on its
branch; substrate `⊗` / macros / GAPU) — resumable later. **R-0002** (G(3,0,0))
is **superseded by R-0009**.

**Done (on `main`):** R-0001 (EML core), R-0003 (s-expr core), R-0004 (predicate
checker), R-0006 (integer-tensor verifier), R-0007 (the verifier *is* the Hehner
discharge), R-0008 (discovery engine + the honest matmul falsification), R-0009
(`Cl(3,0,1)` PGA kernel). **Six crates, 168 tests green.**
