# T8 · Land ufl-evolve memetic engine: Refiner seam + typecheck pruning

- **Priority:** P1
- **Depends on:** T2, T7
- **Tags:** rung-0, spine, crate:ufl-evolve, crate:ufl-geo, R-0011

## Context
The memetic layer is the ONLY proposer that ever solved the geometric lane — 6/16 seeds rediscovered the τ/4 rotation, and the ablation WITHOUT elite Param-refinement scored 0/16 (theory/discovery-results.md) — and it is currently a deleted pilot. Meanwhile the entire "grade as the evolution constraint" premise has zero consumers: `git grep typecheck origin/main` hits only crates/ufl-geo/examples/hello_geo.rs. R-0014 AC2's second half (geometric fitness as a second run_generic instance) is unmet, and naive memetic designs break answer-blindness (local refinement must SCORE candidates; if the proposer scores, it sees the target).

## Work (after #33 merges via T2; per SPEC-0011 §2.2/§3 and SPEC-0014 via T7)
1. Implement the memetic engine in crates/ufl-evolve: tree-GA proposer over GeoExpr + elite Param-refinement, NaN-safe Fit, hosted on `run_generic`.
2. **Refiner seam**: a `Refiner<G>` trait next to Proposer/Fitness — `fn neighbors(&self, elite: &G, rng) -> Vec<G>`. The ENGINE holds the hill-climb: the refiner proposes neighbor genomes, only the engine scores them via Fitness — so the proposer/refiner pair stays answer-blind and Verifier-Held Transparency survives the memetic upgrade. This is also the generic answer to "where does local refinement plug in" for every future lane, including flip-graph perturbation policies.
3. **Wire `ufl_geo::typecheck` (grade.rs:151) as the Screen instance** (T7's hook) — the first real consumer of the grade harness; SPEC-0011 AC2 becomes an architectural fact, not a convention.
4. **Typed param-slots**: add a slot-enumeration API to crates/ufl-geo (e.g. `params_mut(&mut GeoExpr) -> Vec<&mut f64>` or a Hole view over Param leaves, grade {0} by construction) and make elite Param-refinement a first-class, unit-tested operator over slots instead of ad-hoc tree walking. The slot mechanism is the first concrete typed quotation site — the shape R-0015's operator DSL will need.
5. **Execute SPEC-0014's relocation**: traits/engine move to ufl-evolve (pure substrate, deps: ufl-prng only); ufl-discovery re-exports so tests/r_0014_generic_seam.rs is unchanged.

## Acceptance gate (falsifiable)
- Deterministic tests reproduce the pilot: rotor-sandwich rediscovery on ≥6/16 pinned seeds at gens=400/pop=400, AND the ablation harness with the Refiner disabled scores 0/16 — the refinement step's load-bearing status becomes a committed regression, not folklore.
- SPEC-0011 AC2 fuzz green: every proposer-emitted genome typechecks or is counted as filtered, never scored.
- Unit test: refinement never changes typecheck's verdict (slots are grade-{0}).
- `cargo tree -p ufl-discovery` shows no ufl-geo/ufl-ga edge; the r_0014 byte-identical sweep still green post-relocation.

## Must NOT claim
That the 6/16 result generalizes beyond the rotation task, or that grade pruning caused it (the ablation isolates refinement, not the screen).

## Files/crates
crates/ufl-evolve (engine, refiner, tasks per topology), crates/ufl-geo/src/{grade.rs,expr.rs} (slot API), crates/ufl-discovery/src/generic.rs (re-exports), specs/0011-*.md cross-refs.
