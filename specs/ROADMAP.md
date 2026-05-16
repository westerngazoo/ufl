# Spec Roadmap

This replaces the four-phase timeline from the original proposal (§6). UFL is
built as a dependency-ordered sequence of specs, worked one at a time. No dates,
no phases — a spec is started when its dependencies are `Implemented`.

## Sequence

| Spec | Title | Realizes | Depends on |
|------|-------|----------|------------|
| 0001 | Spec process & repo conventions | the build method itself | — |
| 0002 | Log-domain arithmetic | ℒ Log-Embed, ⊕ Log-Add (Pillar 1) | 0001 |
| 0003 | Geometric algebra multivectors over G(3,0,0) | 𝒢ₖ Grade-Lift, ∗ Geo-Product (Pillar 2) | 0001 |
| 0004 | Log–GA bridge | log-domain × grade structure (open Q1) | 0002, 0003 |
| 0005 | Hehner predicates | ⟦P⟧ Predicate (Pillar 3) | 0001 |
| 0006 | UFL surface syntax & AST | the notation in the proposal sketches | 0001 |
| 0007 | Evaluator | parse → predicate-check → evaluate | 0004, 0005, 0006 |
| 0008 | Substrate contract & CPU substrate | ⊗ Substrate-Bind (Pillar 4) | 0007 |
| later | Neural layer: grade-filtered geometric product | §4 of the proposal | 0007 |
| later | GAPU mapping & reservoir experiment | §5 of the proposal | 0008 |

## Notes

- Ordering is by dependency, not priority. 0002, 0003, 0005, and 0006 have no
  dependency on each other beyond 0001 and may be specced in any order.
- The proposal's §8 open research questions are tracked inside the specs that
  must resolve them (e.g. Q1 → spec 0004, Q2 → spec 0005, Q3 → spec 0008).
- This table is updated whenever a spec is added, accepted, or reordered.
