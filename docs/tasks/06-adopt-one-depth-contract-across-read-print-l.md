# T6 · Adopt one depth contract across read/print/lower/eval/Drop

- **Priority:** P0
- **Depends on:** none
- **Tags:** rung-1, spine, soundness, crate:ufl-syntax, crate:ufl-core

## Context
Recursion-depth policy is currently three uncoordinated accidents on exactly the surface the reflection loop owns: PR #40 caps only `read()` at a hard-coded MAX_DEPTH=128 while Sexpr construction, `Display` printing (sexpr.rs:43-51), `lower` (lower.rs:52), `eval`, `eval_pred`, and the Box-recursive `Drop` of Eml/Sexpr are all unbounded-recursive — so `read(print(x))` would break for any x deeper than 128 that the rest of the system happily builds (#35's own test constructs 1000-deep Sexprs). PR #38 makes eval heap-bounded but leaves recursive Drop, relocating the DoS rather than removing it — and its head tree is a stale revert bomb (closed in T2). Once quote lands (T5) and programs are machine-generated (the SPEC-0011 pilot already hit an 11 MB runaway render — ufl-geo/src/render.rs:6-8), deep ASTs re-read as input are the NORMAL case; an asymmetric cap silently breaks the round-trip invariant (sexpr.rs:36-39) the reflection loop is built on, and a stack overflow anywhere is a panic in lib code (§6).

## Work (requirement + spec first — 'bounded code↔data depth')
1. Write the small requirement/spec deciding ONE policy: either iterative implementations everywhere (no magic constant) or ONE shared, spec-recorded budget enforced with typed errors consistently across `read` (read.rs:86), `Display` (sexpr.rs:43), `lower` (lower.rs:52), `eval`, and `eval_pred`. Record the decision (and the constant, if any) in the decision log.
2. Reimplement the iterative eval on current main — the correct-in-spirit part of #38 — WITHOUT the three bare `values.pop().unwrap()`s (§6), keeping the log.rs SPEC-0001 §2.4 branch-convention comment, and keeping doc comments truthful (eval.rs still says "Recursive post-order walk").
3. Add iterative `Drop` for Eml and Sexpr (the Box-recursive Drop is the leak both PRs missed).
4. Add a typed error (e.g. `ReadError::RecursionDepthExceeded`) if the budget route is chosen; clean tests, no meta-babble (§2 Clean).

## Acceptance gate (falsifiable)
- Deep-tree e2e: a depth-1e5 ufl-prng-generated Sexpr survives read→lower→eval→print→read with identical result and no stack overflow or panic.
- If a cap is chosen: a boundary fuzz test proves read-accepts ⟺ print-emits at exactly the same depth (the codec is symmetric).
- A 100k-deep Eml drops without stack overflow.
- `grep -r 'unwrap\|expect' crates/ufl-core/src` shows zero hits in lib code.
- PRs #38/#40 closed with supersession notes pointing here (via T2).

## Must NOT claim
That this delivers reflection — it is the substrate contract quote depends on. Do not merge #40's 128 cap as-is anywhere.

## Files/crates
crates/ufl-syntax/src/{read.rs,sexpr.rs,lower.rs}, crates/ufl-core/src/{eval.rs,log.rs}, crates/ufl-predicate/src/eval_pred.rs, requirements/ + specs/ (new small pair), decision log.
