# UFL — A Unified Formal Language for the Hardware-Software Continuum

**Author:** Gustavo Delgadillo (Goose) · **Org:** westerngazoo · **Status:** Active Research

> The boundary between hardware and software is an artifact of notation, not physics.

Every computation — etched in silicon, run by an OS scheduler, or inferred by a
neural network — is a constraint over a geometric object. UFL is a language
expressive enough to state that constraint directly, so the substrate becomes a
compilation target rather than a design decision.

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

## How we build it — spec-driven

UFL is built **spec-first**. Every feature is described as a numbered spec in
[`specs/`](specs/) — testable requirements, non-goals, acceptance criteria —
*before* any Rust is written. We work through specs one at a time; each
implementation references the spec it satisfies.

There is no phased timeline. The build order is the dependency-ordered spec
sequence in [`specs/ROADMAP.md`](specs/ROADMAP.md).

## Repository layout

```
ufl/
├── docs/             original research proposal + design notes
├── specs/            numbered, spec-driven feature requirements
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
