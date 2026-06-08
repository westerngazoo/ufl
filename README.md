# UFL — A Unified Formal Language for the Hardware-Software Continuum

**Author:** Gustavo Delgadillo (Goose) · **Org:** westerngazoo · **Status:** Active Research

> The boundary between hardware and software is an artifact of notation, not physics.

Every computation — etched in silicon, run by an OS scheduler, or inferred by a
neural network — is a constraint over a geometric object. UFL is a language
expressive enough to state that constraint directly, so the substrate becomes a
compilation target rather than a design decision.

**New to UFL?** [`docs/why-ufl.md`](docs/why-ufl.md) explains what UFL is and
why it can exist — in detail, and with a plain-language appendix for newcomers.
**Want the language tour?** [`docs/the-shape-of-ufl.md`](docs/the-shape-of-ufl.md)
shows what's built today, what writing UFL will look like, and how the same EML
tree compiles to each substrate (CPU, stack machine, FPGA, analog, neural).
**Want to *run* it?** Open [`docs/playground.html`](docs/playground.html) in a
browser — an interactive architecture diagram plus a live playground (a faithful
JS port of the evaluator, predicate checker, and tensor verifier): evaluate
`(eml 1 1)`, check `⟦x' = (eml x 1)⟧`, and verify the Strassen scheme.

## The four pillars

1. **Logarithmic Arithmetic Core** — all arithmetic reduces to log-domain
   operations: `×` → log-add, `÷` → log-sub, `xⁿ` → `n·log(x)`.
2. **Geometric Algebra Spatial Layer** — multivectors encode state; the
   geometric product is the universal composition operator.
3. **Hehner Predicative Logic Layer** — programs are predicates over pre/post
   state, making a gate, a function, and a neural layer formally equivalent.
4. **Substrate Orchestrator** — picks the lowest-cost substrate (silicon, CPU,
   GPU, analog) that satisfies a predicate.

## The six atoms

| Symbol | Name | Role |
|--------|------|------|
| ℒ | Log-Embed | Map a scalar into the log domain. |
| ⊕ | Log-Add | Addition in log-domain (= linear-domain multiplication). |
| 𝒢ₖ | Grade-Lift | Lift a scalar into a grade-k multivector. |
| ∗ | Geo-Product | Geometric product of two multivectors. |
| ⟦P⟧ | Predicate | Wrap an expression as a Hehner pre/post-state constraint. |
| ⊗ | Substrate-Bind | Annotate an expression with a substrate hint. |

## How we build it — requirement- and spec-driven

UFL is engineered to a world-class standard. Every feature passes an eight-step
loop: a requirement is agreed and recorded in [`requirements/`](requirements/),
a spec realizing it is written and reviewed in [`specs/`](specs/), tests come
first (TDD + e2e), code is described and reviewed before it is written, and
every change lands via a reviewed GitHub PR.

The full process — code philosophy, the SDLC, and the agent fleet (orchestrator,
architect, qa) — is the [engineering constitution in `CLAUDE.md`](CLAUDE.md).
There is no phased timeline; build order lives in [`ROADMAP.md`](ROADMAP.md).

## Repository layout

```
ufl/
├── CLAUDE.md         engineering constitution — how UFL is built
├── ROADMAP.md        milestones + requirement backlog + status
├── requirements/     what UFL must do (R-NNNN)
├── specs/            how each feature is built (SPEC-NNNN)
├── .claude/agents/   the SDLC agent fleet (orchestrator, architect, qa)
├── docs/             original research proposal + design notes
├── theory/           formal definitions (atoms, predicates, log-GA bridge)
├── crates/           Rust workspace crates (added per-spec)
├── gapu-connection/  GAPU architecture ↔ UFL pillar mapping
└── papers/           paper drafts
```

## Implementation

Rust workspace. Crates are added to `Cargo.toml` as their specs are accepted —
see [`crates/README.md`](crates/README.md) for the planned crate decomposition.

---

*Built on: garust · goose-os · wari · GAPU*
