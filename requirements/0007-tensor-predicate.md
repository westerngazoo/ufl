# R-0007 — Tensor-Equality Predicate (the Hehner-discharge bridge)

- **Status:** Draft
- **Milestone:** M5 — Discovery
- **Owner:** Gustavo Delgadillo (Goose)
- **Created:** 2026-06-04
- **Pillar / atom:** Predicative logic over the tensor domain — the bridge that
  makes the matmul-decomposition verifier **the Hehner discharge of `P_n,R`**,
  not a parallel ad-hoc check. Closes [FINDINGS](../ufl-discovery/FINDINGS.md)
  C1. Step 2 of **Path B**.
- **Depends on:** R-0006 (`ufl-tensor` — `target`/`reconstruct`/`scheme_error`)
  and R-0004 (the predicate concept: a decidable property of a state).
- **Realized by:** SPEC-0007 (pending)
- **QA:** `qa` agent run scoped to R-0007

## 1. Statement

UFL must be able to express and **discharge** the matmul-decomposition predicate

```
P_n,R(scheme) ≡ ( reconstruct(scheme) == T_n )  ∧  ( rank(scheme) == R )
```

as a **Hehner predicate** — a decidable property of a candidate state — through
UFL's predicate layer, so that the discovery engine's verifier *is* that
discharge rather than a separate hand-rolled check. This is the requirement that
earns the thesis "more general than AlphaTensor *because it rides UFL's
verifier*": the same predicate-discharge concept that checks `⟦ x' = (eml x 1) ⟧`
(R-0004) must also check `⟦ reconstruct(scheme) = T_n ⟧`.

Today (R-0004) the predicate layer is **scalar-only**: `check` evaluates a
boolean s-expression over a `Complex<f64>` state; it has no tensors and cannot
even write `0`/`2` (FINDINGS C1). R-0007 closes that gap for the tensor domain.

## 2. Rationale

R-0006 built the exact tensor arithmetic (`is_valid(scheme, n, R)` already
decides `P_n,R`). What R-0006 does **not** do is make that decision an instance
of UFL's *predicate* concept — it is a bespoke function in `ufl-tensor`. R-0007
unifies it: the verifier becomes a predicate discharge, so the discovery engine
(R-0008) "checks a predicate" exactly as the rest of UFL does, and the
universal-computability story (`theory/universal-computability.md`) gains a
second predicate domain alongside the scalar one.

## 3. The realization fork (decide before SPEC-0007)

There are two faithful ways to make `P_n,R` "a predicate the layer discharges".
This is a genuine design decision for the owner; SPEC-0007 commits to one.

**Option A — a domain-general predicate-discharge abstraction (recommended).**
Introduce in `ufl-predicate` a small abstraction: *a predicate is a decidable
property of a candidate state*, with a `discharge(candidate) -> bool`. R-0004's
scalar predicate (an `Sexpr` over a pre/post state) is one instance; the tensor
predicate `TensorEq { n, rank }` over a `Scheme` candidate is another. The
discovery engine discharges the tensor predicate directly (a typed Rust call,
fast enough for a search loop checking millions of candidates).

- *For:* unifies scalar + tensor under one discharge concept (the milestone's
  whole point); the two instances genuinely justify the abstraction; fast in the
  GA inner loop; no heterogeneous-value-enum expansion of the s-expr language.
- *Against:* a trait/abstraction to design carefully (no premature generality);
  the predicate isn't *written as s-expr text*.

**Option B — tensor values + forms in the s-expression language.**
Add a tensor value kind and forms (`(target n)`, `(reconstruct s)`, a
tensor-`=`) so `P_n,R` is literally an s-expr: `(and (= (reconstruct s) (target
n)) (= (rank s) R))`, checked by `eval_pred`.

- *For:* maximally homoiconic — the predicate is literally code-as-data in the
  language; most literally "P_n,R is a predicate in UFL".
- *Against:* a large value-model expansion (the heterogeneous runtime value the
  R-0003/R-0004 synthesis deliberately avoided); awkward and slow to build/check
  a big integer scheme as an s-expr per candidate in the GA loop; schemes are
  not naturally s-expressions.

**Recommendation: Option A.** The discovery engine checks millions of
candidates, so the discharge must be a typed call, not s-expr evaluation; and
two real instances (scalar Hehner + tensor) earn the abstraction without
speculation. Option B's homoiconic appeal is real but mismatched to a search
loop; tensor s-expr forms can come later if a human ever wants to *write* a
tensor predicate by hand.

## 4. Acceptance criteria

*(Phrased to hold under Option A; SPEC-0007 adjusts if Option B is chosen.)*

- **AC1 — A predicate is a dischargeable property.** `ufl-predicate` exposes an
  abstraction whose contract is `discharge(candidate) -> bool` (total; typed
  error, never panic, on a malformed candidate).
- **AC2 — Scalar predicate is an instance.** R-0004's scalar/Hehner check is
  expressible as (or adapted to) the abstraction without changing its behaviour
  — the existing R-0004 tests still pass.
- **AC3 — The tensor predicate.** `TensorEq { n, rank }` discharges to `true` on
  a scheme iff `reconstruct(scheme) == T_n` and `rank(scheme) == R` — i.e. it is
  exactly `is_valid` (R-0006), now framed as a predicate.
- **AC4 — Strassen through the predicate.** Discharging the tensor predicate
  `TensorEq { n: 2, rank: 7 }` on the Strassen scheme returns `true`; on a
  broken scheme, `false`. The keystone, now via the predicate layer.
- **AC5 — Honest discharge, not a wrapper that hides errors.** A dim/`n`
  mismatch surfaces as a typed error (reusing R-0006's `DimMismatch`), never a
  panic or a silent `false`.
- **AC6 — The discovery engine can discharge it.** The abstraction is callable
  in a tight loop (no per-call allocation surprises that would make a
  million-candidate search impractical) — demonstrated by a benchmark-style test
  discharging the predicate over many candidates.

## 5. Open questions

- **The fork (§3)** — A vs B. Owner decides before SPEC-0007.
- **Where the abstraction lives** — in `ufl-predicate` (depending on
  `ufl-tensor` for the tensor instance), or a thin bridge crate, to keep
  `ufl-predicate` from depending on `ufl-tensor`? The dependency direction needs
  the architect's eye (predicate → tensor is inward-ish; or invert via a trait
  the tensor crate implements).
- **Relation to the orchestrator** — discharge is the *check*; the search/solve
  (R-0008) and the eventual substrate orchestrator are separate. Confirm R-0007
  is checking-only (decidable), consistent with R-0004.

## 6. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-04 | R-0007 makes `P_n,R` a **Hehner predicate discharge**, closing FINDINGS C1 — the step that earns the "rides UFL's verifier" thesis. | R-0006 decides `P_n,R` but as a bespoke `ufl-tensor` function; R-0007 unifies it with UFL's predicate concept. |
| 2026-06-04 | Realization fork (§3) surfaced; **Option A (domain-general discharge abstraction) recommended** over Option B (tensor s-expr forms). | The GA engine checks millions of candidates → the discharge must be a typed call, not s-expr evaluation; two real instances justify the abstraction; Option B's heterogeneous-value expansion is deferred. Owner to confirm. |

## Changelog

- 2026-06-04 — created (Draft).
