# R-0006 — Exact Integer-Tensor Core (the discovery substrate)

- **Status:** Draft
- **Milestone:** M5 — Discovery
- **Owner:** Gustavo Delgadillo (Goose)
- **Created:** 2026-06-04
- **Pillar / atom:** Discovery substrate — the exact integer-tensor layer a
  matmul-decomposition scheme is verified against. Step 1 of **Path B** (see
  [`ufl-discovery/FINDINGS.md`](../ufl-discovery/FINDINGS.md)).
- **Depends on:** none (deliberately — exact integer arithmetic, *not* the
  `Complex<f64>` EML core; see FINDINGS C3).
- **Realized by:** SPEC-0006 (pending)
- **QA:** `qa` agent run scoped to R-0006

## 1. Statement

UFL gains an **exact integer-tensor core** — the substrate a matrix-
multiplication *decomposition scheme* is reconstructed and verified against.
This is the foundation of the discovery engine (the
[`ufl-discovery` PRD](https://docs.google.com/document/d/170cdfV8ZvglRa_9jz7Gr8MBV1WFyxNfxXFZ6G2Dxabo),
Phase 0): build the matmul target tensor, represent a scheme, reconstruct it,
and decide exact equality — all in integer arithmetic.

- **Target tensor `T_n`.** The matrix-multiplication tensor for `n×n` matrices,
  shape `(n², n², n²)`, integer (0/1) entries.
- **Scheme genotype.** A scheme is `Vec<Triple>`, where
  `Triple = (u, v, w)` and each of `u, v, w` is a length-`n²` vector of
  coefficients in `{-1, 0, +1}`. Each triple is one scalar multiplication; the
  scheme's length is its multiplication count `R`.
- **Reconstruction.** `reconstruct(scheme) = Σ_r u_r ⊗ v_r ⊗ w_r`, computed with
  exact integer (`i64`) accumulation.
- **Exact verification.** `error(scheme) = ‖reconstruct(scheme) − T_n‖²` as an
  integer; a scheme is *valid at rank R* iff `error == 0` and `|scheme| == R`.

## 2. Rationale

This is the cheapest, most foundational, and least-contested piece of the
discovery program. The PRD's Phase-0 gate — "the hardcoded Strassen 7-term
scheme reconstructs `T_2` with error 0; if this fails the verifier is wrong and
nothing downstream is trustworthy" — lives entirely here. It needs **none** of
the contested substrate: per FINDINGS, the EML core is `Complex<f64>` (wrong
field for exact integer work, C3), the predicate layer can't yet express
tensors (C1), and garust isn't integrated and isn't needed for plain integer
tensors (C2/C5). A dedicated integer path is exact, simple, and correct.

The tensor-equality *predicate* (so the verifier IS the Hehner discharge of
`P_n,R`, Path B's whole point) is a **separate, later requirement** that bridges
this core into `ufl-predicate`. R-0006 builds the exact arithmetic that
requirement will wrap.

## 3. Acceptance criteria

- **AC1 — Target tensor.** `tensor(n)` builds `T_n` for any `n ≥ 1`: shape
  `(n², n², n²)`, the matmul tensor with the correct 0/1 entries (entry
  `(i·n+j, j·n+k, i·n+k) = 1`, else 0). Verified for `n = 2` against the known
  `T_2`.
- **AC2 — Scheme genotype.** `Triple` and `Scheme` represent length-`n²`
  `{-1,0,+1}` vectors and a `Vec<Triple>`; construction rejects (typed error)
  any entry outside `{-1,0,+1}` or any wrong-length vector.
- **AC3 — Reconstruction.** `reconstruct(scheme, n)` computes
  `Σ_r u_r ⊗ v_r ⊗ w_r` with exact `i64` accumulation, returning a tensor of
  shape `(n², n², n²)`.
- **AC4 — Exact error.** `error(scheme, n) = Σ (reconstruct − T_n)²` as an
  `i64`; it is `0` iff the scheme reconstructs `T_n` exactly. No floating point
  anywhere in the path.
- **AC5 — Strassen fixture (the Phase-0 gate).** The canonical 7-term Strassen
  2×2 scheme, encoded as a fixture, reconstructs `T_2` with `error == 0`. This
  is the keystone correctness test — if it fails, the core is wrong.
- **AC6 — Naive baseline.** The naive `R = n³` scheme (one triple per
  `(i,j,k)`) reconstructs `T_n` with `error == 0`, for `n = 2` (R=8) and `n = 3`
  (R=27).

## 4. Constraints & non-goals

**Constraints**

- **Exact integer arithmetic only** — `i8` coefficients, `i64` accumulation. No
  `Complex<f64>`, no EML reuse (FINDINGS C3).
- New crate **`ufl-tensor`**, no dependency on `ufl-core`/`-syntax`/`-predicate`
  (it is the pure arithmetic layer they will later wrap).

**Non-goals** (later requirements)

- The **GA search** / discovery loop (a later requirement; PRD Phases 1–3).
- The **tensor-equality predicate** bridging this into `ufl-predicate` (the next
  Path-B requirement — what makes the verifier the Hehner discharge).
- `egg`, neural guidance, sizes beyond those tested, performance tuning.
- Any claim that this *is* the discovery engine — it is only the verifier's
  arithmetic.

## 5. Open questions

- **Tensor storage.** Dense `Vec<i64>` of length `n⁶` (flat, index by
  `(a·n² + b)·n² + c`) vs a sparse map. Dense is simplest and fine for the small
  `n` in scope (n=2: 64 entries; n=3: 729). SPEC-0006 fixes it; dense
  recommended.
- **Strassen fixture source.** The 7 triples must be transcribed from a known
  reference and themselves checked (AC5 is exactly that check). SPEC-0006 cites
  the source.
- **Crate naming / future bridge.** `ufl-tensor` now; the later predicate bridge
  and GA engine (`ufl-discovery`) depend on it. Confirm the split.

## 6. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-04 | Pursue the discovery program **Path B** (extend the substrate so the verifier is the Hehner discharge), as the new **headline** direction; pause the language-build thread (R-0005 value conditional shelved on its branch, recoverable). | Owner decision after the FINDINGS review. Path B earns the "more general than AlphaTensor *because it rides UFL's verifier*" thesis rather than asserting it. |
| 2026-06-04 | R-0006 is the **exact integer-tensor core**, a new `ufl-tensor` crate with **no dependency on the EML/Complex core**. | The discovery verifier is exact integer arithmetic over `{-1,0,+1}`; reusing the `Complex<f64>` EML core buys nothing and risks float drift (FINDINGS C3). Foundational, uncontested, and the home of the Phase-0 Strassen gate. |
| 2026-06-04 | The tensor-equality **predicate** (the Hehner bridge) is a **separate later requirement**, not R-0006. | Keeps R-0006 small and shippable; the bridge depends on this core existing first. |

## Changelog

- 2026-06-04 — created (Draft).
