# The Two-Language Substrate — the architecture for a self-improving discoverer

**Status:** Architectural *direction* (hypothesis), gated on the de-risks below.
Pending the metacircular requirement (after the object-level discovery lands).

## The thesis

A system that *improves its own discovery process* needs the **strange loop**:
the search and the evaluator must be **data the system can rewrite and
re-evaluate** (reflection — Lisp's `quote`/`eval` closed into a cycle). UFL's
evaluators are today **one-way and terminal** (`&AST → Value`, in Rust): the
searcher lives in the host language, where the system can neither see nor modify
it. No reflection ⇒ no self-modification ⇒ a fixed engine over a fixed space.

A Lisp is the *native* home for that loop. **Rust is the wrong tool for the
self-modifying layer** (it forces you to hand-build and fight `eval`/`quote`/
reflection) — and the *right* tool for the part that must never lie.

## The split (and why the split itself is the point)

| Layer | Language | Why |
|---|---|---|
| **Self-modifying discovery** — forms, operators, and eventually `eval` as first-class data the system rewrites | **a Lisp** (Scheme/Racket/Chez) | homoiconic, reflective, metacircular — the strange loop is native |
| **Verifier + exact kernels** — the predicates, `Cl(3,0,1)` (garust), exact `i64` tensors, the soundness gate | **Rust** | rigid, exact, deterministic, fast — the ground truth the proposer **cannot reach** |

The two-language boundary makes **Verifier-Held Transparency an architectural
fact, not a discipline**: the proposer is in another language, across an FFI it
cannot cross, so it *cannot corrupt its own verifier*. That is the
**can't-fool-itself** property — the one thing the field's messy neural
self-improvement lacks, and the honest core of the "god" goal: **verified
recursive self-improvement.**

## The gates (de-risk before any commitment — the panel/constitution discipline)

1. **Mechanics** — does a Scheme metacircular search calling a Rust verifier over
   FFI actually close the loop, stay verified, and run fast enough (FFI call rate
   for millions of verifier calls)? (Toy target; a laptop-day.)
2. **Meta-evolution design** — concretely, how do the search operators become
   *forms the system evolves*, scored by whether the evolved searcher hits the
   verifier's targets faster? The minimal falsifiable "strange-loop" requirement.
3. **Precedent** — what works / fails in metacircular evaluators, learned-
   optimizer / hyper-heuristic self-modification, and verified self-improvement;
   the drift/reward-hacking failure modes the held-out verifier must anchor.

## Sequencing (why not now)

The object level — *does the search discover anything at all?* — is being proven
first (the geometric + matmul shots). You cannot close a loop on a process that
does not yet work; switching languages before the object level is the premature
rewrite the R-0014 design panel warned against. The Scheme question goes **live at
the metacircular requirement**, the step *after* the object loop stands. Keep
Rust as the incorruptible verifier; earn the Lisp layer with gate 1.
