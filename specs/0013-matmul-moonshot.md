# SPEC-0013 — Flip-graph matmul reduction (Gate-0)

- **Realizes:** [R-0013](../requirements/0013-matmul-moonshot.md) AC1 + AC2
- **Status:** Draft (lightweight review — architect-on-PR, per the approved path;
  a known algorithm realizing an accepted requirement, not a novel design)
- **Crate:** `ufl-discovery` (new module `flipgraph`)

## 2.1 The problem the type system poses

`ufl_tensor::Triple::new` hard-validates coefficients to `{−1,0,+1}`
([scheme.rs:34](../crates/ufl-tensor/src/scheme.rs)). The Kauers–Moosbauer flip is
tensor-preserving but transiently leaves that set:

```
share first factor a:  (a,b,c),(a,b′,c′) → (a,b,c−c′),(a,b+b′,c′)
proof: a⊗b⊗(c−c′) + a⊗(b+b′)⊗c′ = a⊗b⊗c + a⊗b′⊗c′   ✓ (sum invariant)
```

`b+b′` can be ±2, so **intermediate states are not valid `Scheme`s**. Therefore the
flip-graph walks over an **internal unrestricted-integer workspace**, and only an
end state back in `{−1,0,+1}` (which Strassen's rank-7 is) is converted to a
`Scheme` and handed to the verifier. This keeps `ufl-tensor` unchanged (R-0013
non-goal) and keeps the verifier the sole acceptance authority (AC2).

## 2.2 Internal representation (proposer workspace)

- `IntTriple { u: Vec<i64>, v: Vec<i64>, w: Vec<i64> }`, `IntScheme(Vec<IntTriple>)`
  — equal-length, unrestricted-integer vectors.
- `naive(n) -> IntScheme` — the rank-`n³` scheme: one `0/1` triple per `(i,j,k)`,
  `u = e_{i·n+j}`, `v = e_{j·n+k}`, `w = e_{i·n+k}` (matches `target(n)`; for `n=2`,
  rank 8, all in `{0,1}`).
- `reconstruct_int(&IntScheme, n) -> Tensor` — mirrors `ufl_tensor::reconstruct`
  over `i64`. **Search guidance / invariant checking only — NOT certification.**

## 2.3 Moves (tensor-preserving by construction)

- `flip(&IntScheme, rng) -> IntScheme` — pick two triples sharing exactly one
  factor-vector (`u`, `v`, or `w` equal); apply the sum-preserving rewrite above
  (the three symmetric variants). Each variant carries a doc-comment proof of
  invariance. Guard `i64` against overflow (reject/skip a move that would).
- `reduce(&IntScheme) -> IntScheme` — to fixpoint: merge any pair sharing **two**
  factor-vectors (`(a,b,c),(a,b,c′) → (a,b,c+c′)`, rank −1); drop any triple
  containing a zero vector (contributes `0`). Never raises rank; preserves the
  tensor.

## 2.4 Search

```
reduce_matmul(n, target_rank, seed, budget) -> Result<Scheme, FlipError>
```

- `s = reduce(naive(n))`; `rng = SplitMix64::new(seed)`; track the best (min-rank
  exact) state.
- Loop ≤ `budget`: `s = reduce(flip(s, rng))`. If `rank(s) ≤ target_rank` **and**
  every coefficient is in `{−1,0,+1}`, convert to a `Scheme` (via `Triple::new`) and
  **return it**. On a stall of `k` steps, restart from the kept checkpoint with a
  fresh perturbation (flip-graph plateau escape).
- **Invariant** (debug-asserted after every move): `reconstruct_int(s,n) ==
  target(n)`. A move that violates it is a bug, not a candidate.
- `FlipError`: `NotFound { best_rank }` (budget exhausted), plus a typed guard for
  the coefficient-conversion edge. **No `unwrap`/`expect`/`panic!` in the module.**

## 2.5 Certification is the caller's, via the real verifier (AC2)

`reduce_matmul` returns a `Scheme`; the **caller** (the test, and any future driver)
discharges it through `RankDecomposition::new(n, target_rank)`. The module has no
path that asserts a scheme is correct — a tensor-breaking bug surfaces as the
verifier returning `Ok(false)`/`Err`, never a false `Ok(true)`.

## 2.6 Tests (TDD — written first, red)

`crates/ufl-discovery/tests/r_0013_flipgraph.rs`:

1. **`flip_graph_reaches_certified_rank7_t2`** (AC1): `reduce_matmul(2, 7, SEED,
   BUDGET)` is `Ok(scheme)`; `RankDecomposition::new(2,7).discharge(&scheme) ==
   Ok(true)`; re-certify through a **freshly built** `RankDecomposition`; a
   **bilinear check** — for 20,000 seeded random integer 2×2 `A,B`, the scheme's
   `m_t = ⟨u_t,ā⟩·⟨v_t,b̄⟩`, `c[r]=Σ_t w_t[r]·m_t` equals `A·B` flattened. Deterministic.
2. **`flip_preserves_the_tensor`**: after any single flip, `reconstruct_int ==
   target(2)`.
3. **`reduce_only_drops_rank_and_preserves_tensor`**.
4. **`verifier_is_the_sole_judge`** (AC2): a hand-corrupted rank-7 scheme (one `w`
   entry flipped) discharges `Ok(false)`, never `Ok(true)`.

## 2.7 Falsifiable gate

If no `(SEED, BUDGET)` within a laptop-minute reaches a certified `{−1,0,+1}`
rank-7, the move set is incomplete — recorded honestly, the module lands with the
best rank reached and the negative documented. (Reachability is established:
⟨2,2,2⟩ is trivial in the KM flip graph and the deleted pilot reached it 3/3.)
