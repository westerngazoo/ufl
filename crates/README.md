# Crates

UFL Rust workspace crates. Each crate is created only when its governing spec
is accepted — nothing here is pre-scaffolded.

## Planned decomposition

| Crate | Responsibility | Atoms / Pillar | Spec |
|-------|----------------|----------------|------|
| `ufl-core` | Log-domain arithmetic + GA multivectors | ℒ ⊕ 𝒢ₖ ∗ — Pillars 1–2 | 0002, 0003, 0004 |
| `ufl-predicate` | Hehner pre/post-state predicates | ⟦P⟧ — Pillar 3 | 0005 |
| `ufl-syntax` | Lexer, parser, AST for UFL surface notation | — | 0006 |
| `ufl-eval` | parse → predicate-check → evaluate | — | 0007 |
| `ufl-substrate` | Substrate contract trait, cost model, CPU substrate | ⊗ — Pillar 4 | 0008 |
| `ufl-cli` | Entry-point binary | — | TBD |

When a crate is added, register it in the workspace `members` list in the
top-level `Cargo.toml`.
