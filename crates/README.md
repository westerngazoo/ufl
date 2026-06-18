# Crates

UFL Rust workspace crates. Each crate is created only when its governing spec
is accepted — nothing here is pre-scaffolded.

## Current decomposition

| Crate | Responsibility | Atoms / Pillar | Spec | Status |
|-------|----------------|----------------|------|--------|
| `ufl-core` | The EML numeric core: `Eml` tree + reference evaluator over `Complex<f64>` | `eml` · `1` | SPEC-0001 | shipped |
| `ufl-syntax` | Homoiconic s-expression surface: `Sexpr`, reader, lowering into `Eml` | the AST itself | SPEC-0003 | shipped |
| `ufl-predicate` | Hehner predicate checker over pre/post state; the `Predicate` discharge trait | `⟦P⟧` — Pillar 3 | SPEC-0004, SPEC-0007 | shipped |
| `ufl-tensor` | Exact integer-tensor core for matmul decomposition (`T_n`, schemes, reconstruction) — pure leaf | discovery substrate | SPEC-0006 | shipped |
| `ufl-discovery` | Discovery bridge + engine: the `RankDecomposition` predicate (SPEC-0007) + the seeded GA search (SPEC-0008) | discovery | SPEC-0007, SPEC-0008 | shipped |
| `ufl-ga` | `Cl(3,0,1)` PGA geometric kernel — a thin facade over garust (real `f64`); `Mv`, named basis constructors, `Motor`/`Point` | `𝒢ₖ` `∗` — Pillar 2 | SPEC-0009 | shipped |

Note: the root-level `ufl-discovery/` directory is the discovery thread's
*research-artifact* home (FINDINGS.md, writeups); the crate lives at
`crates/ufl-discovery`. Deliberately distinct homes.

## Planned (paused language-build thread)

| Crate | Responsibility | Atoms / Pillar |
|-------|----------------|----------------|
| `ufl-substrate` | Substrate contract trait, cost model, CPU substrate | `⊗` — Pillar 4 |
| `ufl-cli` | Entry-point binary | — |

When a crate is added, register it in the workspace `members` list in the
top-level `Cargo.toml` and update this table in the same PR.
