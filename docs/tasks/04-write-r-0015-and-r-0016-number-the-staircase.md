# T4 · Write R-0015 and R-0016: number the staircase, pre-register gates

- **Priority:** P0
- **Depends on:** none
- **Tags:** requirement, spec, rung-1, rung-4, docs

## Context
The self-eval staircase exists only as prose — no rung has a requirement number, and CLAUDE.md §1 blocks all code until one exists. Two requirements are owed: (a) the reframed R-0015 lives only in theory/two-language-substrate.md ("a discuss-with-Gustavo requirement, not yet written"); (b) the reflection rung (quote/eval/raise) is deferred in R-0014 §4 (requirements/0014-discovery-framework.md:95-96, "no quote/reify/Value→AST") with no R-number. There is also a terminology collision: "Rung-1" means rules-as-Rust-data in R-0014 §4 but operator-forms-enum in two-language-substrate.md:101 — future docs will talk past each other until ONE numbered ladder exists.

## Work (discuss-first with Gustavo, per constitution §4 step 1; three-lens review per step 2)
1. **requirements/0015** — the evolve-operator-SEMANTICS loop (not knobs; the hyperparameter direction is dead per the headroom verdict). Transcribe VERBATIM from theory/two-language-substrate.md: the pre-registered gate (evolved operator-set beats the hand-written baseline by ≥2 SE on a DISJOINT confirmation set, replicated on a second split, at a budget where the baseline is demonstrably not saturating), the C1-C7 non-negotiables, and the kill-criterion (a documented negative kills Rung-5 and redirects to object-level scaling, R-0013 AC3's T₃).
2. **requirements/0016** (or next free number) — reflection rung 1: `(quote e)`, `(eval q)`, structural `=` on syntax, and the `raise: &Eml -> Sexpr` inverse of lower. Load-bearing scoping decisions to state in the requirement: quote is a NAMED form, never the apostrophe (`'` is a legal symbol char, read.rs:62, and load-bearing for Hehner priming `x'` — SPEC-0004 §2.5); scope is code-as-value ONLY — Value→Sexpr reification is explicitly out because it can never be total (only `1` is a literal, lower.rs:35-36; `inf`/`nan` are legitimate eval results outside the reader's image, R-0001 AC3 vs read.rs:79).
3. **One rung ladder in one place**: put the numbered ladder (Rung-0 bank … Rung-5 substrate) in theory/two-language-substrate.md and point R-0014 §4 and all future docs at it, resolving the collision.

## Acceptance gate (falsifiable)
Both requirement files exist with Status: Accepted (three-lens findings addressed or deferred with decision-log entries) BEFORE any T5/T11 code is committed; the 2-SE gate and kill-criterion appear in requirements/0015 verbatim; the rung ladder appears in exactly one file with both R-0014 §4 and the substrate doc referencing it.

## Must NOT claim
Nothing is implemented here. R-0015's requirement must carry the non-sequitur caveat: the human-designed flip-graph beating a GA is NOT evidence a meta-search can evolve such a move.

## Files/crates
requirements/0015-*.md (new), requirements/0016-*.md (new), theory/two-language-substrate.md, requirements/0014-discovery-framework.md (§4 pointer), decision logs.
