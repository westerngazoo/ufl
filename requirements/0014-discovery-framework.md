# R-0014 — The Shared Discovery Framework (one search/rewrite substrate, three verifier instances)

- **Status:** Draft (2026-06-28 — design-panel pressure-tested; scoped honestly)
- **Milestone:** M5 — Discovery (the **unifying** requirement)
- **Owner:** Gustavo Delgadillo (Goose)
- **Created:** 2026-06-28
- **Pillar / atom:** the discovery *loop* itself — written once, instantiated thrice.
- **Depends on:** R-0003 (`ufl-syntax` — the homoiconic `Sexpr`), R-0008
  (`ufl-discovery` — the proposer-agnostic / verifier-exact seam), R-0010/R-0011
  (the grade/closure pruning harness), R-0011's `ufl-evolve` (the generic engine
  this generalizes). **Reuses, rewrites nothing.**
- **Realized by:** SPEC-0014 (pending)
- **QA:** `qa` agent run scoped to R-0014

## 1. Statement

Build the **shared discovery substrate**: *one* generic, deterministic
**search/rewrite loop** — the proposer-agnostic / verifier-exact seam generalized
to be **genome-generic**, plus the grade/closure-bound **harness**, plus the
**search-as-rewrite** framing (which *is* R-0012's equality saturation, viewed
twice) — **parameterized by per-lane atom-sets and verifier instances.** The three
discovery lanes become **instances**, not rewrites: R-0011 geometric (`Cl(3,0,1)`),
R-0012 𝔽₂ logic, R-0013 matmul. **Each keeps its own value type and irreducible
atoms.** The discovery loop is written and tested *once*; new domains are new
instances behind the same seam.

This is the natural generalization of a decision the project **already made** —
R-0011 §5 Q2 chose "a new `ufl-evolve` crate that reuses the *seam pattern*, not
the matmul types." R-0014 finishes that generalization across all three lanes.

## 2. Rationale & honest scope (the design-panel correction)

A 4-lens design panel (2026-06-28) and the repo's own
[`theory/universal-computability.md`](../theory/universal-computability.md)
**refuted the maximal claim** that one `{eml, +}` substrate *generates* all three
lanes. `eml` is **value-universal over a closed elementary class** ("no recursion,
no branching, no state — a function language, not a model of computation"). The
universality de-risk closed three *value-level primitives* (rotation, AND/NAND,
matmul arithmetic) — real and elegant (the exp/ln duality: `× = exp(ln+ln)`, so
one linear combiner breaks the symmetry) — but the lanes live in **three different
value types with irreducible structure**: `Cl(3,0,1)` multivectors (the geometric
product, `e₀²=0`, grades — garust atoms), exact `i64`/ternary tensors (the rank
structure), and the 𝔽₂ ring (XOR, `x²=x`, Nullstellensatz). Two lanes are *designed*
not to route through `eml` (R-0012: "logic has no metric"; `GeoExpr` lowers
straight onto the kernel). **Rewriting the merged, green `ufl-ga`/`ufl-geo`/
`ufl-tensor` onto `{eml,+}` would discard exactly-integer and exactly-null-metric
code for an unproven unification — a boil-the-ocean rewrite (CLAUDE.md §2).**

So R-0014 unifies the **AST + search harness**, *never the atoms*. What genuinely
unifies and is buildable now: the homoiconic AST handling (R-0003), the
proposer-agnostic seam (R-0008), the decidable grade/closure pruning (R-0010), and
search-as-rewrite (R-0012's egg is the first instance). The `{eml,+}` *prize* is
preserved as a **bounded, falsifiable** sub-result (§3 AC3), not a foundation.

## 3. Acceptance criteria

- **AC1 — A genome-generic, deterministic search loop.** Generalize
  `ufl-discovery::engine::run` to `Proposer<G>{seed, vary}` + a `Fitness` trait
  (boolean-discharge | real-valued | three-valued-saturation adaptors), in
  `ufl-evolve` (or `ufl-search`). Deterministic via the shared `ufl-prng`;
  Verifier-Held Transparency preserved (proposer answer-blind).
- **AC2 — Two lanes on one loop (the proof it pays, falsifiable in days).**
  **Re-host the already-green matmul GA (R-0008) on the generic loop with
  byte-identical results** — same seed ⇒ the same trajectory/outcome as today's
  `run()` (a regression gate against R-0008's planted-recovery 7/10). Then plug
  R-0011's geometric fitness in as a **second** instance behind the same loop. The
  proposer-agnostic seam becomes *real* across two lanes, not asserted.
- **AC3 — The eml prize, discharged (bounded + falsifiable).** Materialize the
  **literal `eml` tree for NAND** and evaluate it end-to-end through `ufl-core`,
  verifying functional completeness — the result *owed* in
  `theory/universal-computability.md §7`. **Plus one new probe** (the panel's): a
  single matmul entry as an `eml`-tree evaluated through `ufl-core` and checked
  against the **exact `i64`** verifier — testing whether `eml` carries the
  discrete/integer regime, **honestly** (a documented leak on branch/precision is
  a valid result). Touches no merged lane.
- **AC4 — Search-as-rewrite unifies with egg.** The search is framed as
  *rewrite/saturate under a cost*; R-0012's equality saturation is an instance of
  the same seam (egg-extraction-cost vs verifier-residual). **Gated like R-0012:**
  if rewrite-under-cost can't match the GA's planted-recovery, the "everything is
  rewrite" unification is real-but-inert — recorded honestly (the R-0012 AC4
  decision-rule discipline).
- **AC5 — The non-goals are enforced, not implied.** R-0014 **does not** rewrite
  `ufl-ga`/`ufl-geo`/`ufl-tensor`/`ufl-discovery` onto `{eml,+}`; the lanes retain
  their atoms and value types. This is an *acceptance criterion*, stated in the
  spec and checked at review.

## 4. Constraints & non-goals

**Constraints** — reuse the merged crates as instances; the generic loop must
reproduce R-0008's seeded outcomes byte-identically (AC2).

**Non-goals (deferred, labeled — the panel's discipline)**
- **One universal `{eml,+}` substrate that generates all lanes** — refuted (§2).
- **True metacircularity** (rules as `eml`-forms; the search written in the
  substrate it searches) — a real **reflection gap**: every `eval` is `&AST →
  Value`, terminal; there is no `quote`/`reify`/`Value→AST`. Rules-as-Rust-data
  (`AST→AST`) is the achievable layer here; metacircularity is a *future*
  requirement. (Rung numbering is owned by the canonical ladder in
  `theory/two-language-substrate.md` — the reflection rung is **R-0016**, the
  operator-semantics probe is **R-0015**.)
- **The temperature→0 continuous↔discrete / 𝔽₂ bridge** — unbuilt, explicitly
  deferred in R-0012 (only if R-0012 returns positive). **Not** presented as a
  current rung; its own future requirement, gated.
- **One concrete shared AST for all three lanes** — keep the per-lane typed ASTs
  (`GeoExpr`, `Genome`) behind a generic rewrite/seam; do not force one enum.
- **The "Closure Principle" as an unnamed harness** — define it concretely against
  the existing `GradeSet`/`realized ⊆ grade` bound, or drop the term (the panel
  found zero referent in-repo).

## 5. Open questions (SPEC-0014 decides)

- The `Fitness` trait shape spanning boolean discharge / real-valued / three-valued
  saturation without leaking a black-box cost egg can't propagate through e-classes.
- Per-lane typed ASTs behind a generic engine **vs** lowering them through `Sexpr`
  (the R-0003 seam — `Sexpr` currently lowers only the `eml` head; the rest are
  `UnknownForm`).
- The AC3 eml-carries-integer probe outcome (does `eml` over `Complex<f64>`
  round-trip the exact `{-1,0,1}` regime, or reintroduce floating error?).

## 6. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-28 | **R-0014 = a shared FRAMEWORK (one search/rewrite seam + harness, per-lane atoms), NOT one `{eml,+}` substrate that subsumes the lanes.** | A 4-lens design panel + the repo's own `theory/universal-computability.md` refuted the maximal claim: `eml` is *value*-universal only; the lanes have irreducible lane-specific atoms (`Cl(3,0,1)`, exact tensors, 𝔽₂ ring); two are designed not to route through `eml`. |
| 2026-06-28 | **No merged crate is rewritten onto `{eml,+}` (an acceptance criterion).** | Rewriting green, exactly-integer / null-metric code for an unproven unification violates CLAUDE.md §2 (no premature abstraction). |
| 2026-06-28 | **The `{eml,+}` prize is kept as a bounded, falsifiable sub-result** (discharge the owed NAND tree; probe the integer regime). | The exp/ln duality is real and elegant; it earns *one* small pure-eml requirement, not the substrate's foundation. |
| 2026-06-28 | **Metacircularity + the temperature/𝔽₂ bridge are deferred, labeled.** | A real reflection gap (`eval: &AST → Value`, no `quote`) and an unbuilt, R-0012-deferred bridge. Keep the broadening conscious, not drift (CLAUDE.md §1.1). |
| 2026-06-28 | **First build = generalize the seam + re-host the matmul GA byte-identically, then add the geometric fitness** (AC2). | Falsifiable in days, zero risk to merged crates, makes the proposer-agnostic seam real across two lanes — and is the generalization R-0011 §5 Q2 already chose. |

## Changelog

- 2026-06-28 — created (Draft) after a 4-lens design panel scoped the unified-
  substrate vision honestly: R-0014 unifies the AST + search harness (one loop,
  three verifier instances), keeps per-lane atoms, preserves the eml exp/ln-duality
  prize as a bounded sub-result, and defers metacircularity + the temperature
  bridge with the reflection gap cited.
