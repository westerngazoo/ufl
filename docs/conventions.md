# UFL Conventions

Notation and writing conventions UFL uses across its docs, requirements, specs,
code, and agent output. Append-only; new conventions are added with a short
rationale and a date.

This file is part of UFL's source-of-truth set (see [`CLAUDE.md`](../CLAUDE.md)
§8). Every session and every agent should honour what is recorded here.

## Notation

### Circle constant — τ (tau), not π (pi)

UFL uses **`τ = 2π`** as its circle constant. Wherever a UFL doc, requirement,
spec, test, or generated artifact talks about *the* circle constant — the angle
of a full turn, the Euler-formula constant, the imaginary-axis quarter-turn —
it writes `τ`. A full turn is `τ` radians; a half-turn is `τ/2`; a quarter-turn
(Euler's `i`) is `τ/4`.

`π` may still appear in two contexts only:

- inside *quoted* or *cited* material from external sources (e.g. the AllEle
  paper, the founding proposal);
- when explicitly contrasting with `τ`, in which case the bridge `π = τ/2` is
  the assumed identity.

**Rationale.** A full turn being one whole unit (`1 τ`) — rather than two
half-turns (`2π`) — makes angle algebra and most circle-related identities
read directly. UFL is foundationally re-deriving the elementary basis from a
single operator (`eml`); locking in the cleaner circle convention while we are
still the only callers costs nothing and is consistent with UFL's stance of
building from a clean base.

**Decided:** 2026-05-19.

### Geometric algebra G(3,0,0) — blade order, masks, and rotor orientation

UFL's geometric layer ([SPEC-0002](../specs/0002-geometric-algebra-core.md))
fixes the following, once, for every layer above it.

**Blade storage order — grade-then-lexicographic.** A G(3,0,0) multivector is 8
complex coefficients in this fixed index order:

| index | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|-------|---|---|---|---|---|---|---|---|
| blade | `1` | `e₁` | `e₂` | `e₃` | `e₁₂` | `e₁₃` | `e₂₃` | `e₁₂₃` |
| grade | 0 | 1 | 1 | 1 | 2 | 2 | 2 | 3 |

**Basis-vector mask.** Internally each blade is a 3-bit mask over
`{e₁=bit0, e₂=bit1, e₃=bit2}`; grade = `popcount(mask)`. The map from storage
index to mask is the fixed permutation

```
MASK = [0b000, 0b001, 0b010, 0b100, 0b011, 0b101, 0b110, 0b111]
       //  1     e₁     e₂     e₃     e₁₂    e₁₃    e₂₃    e₁₂₃
```

The mask encoding is **signature-agnostic** — the result-blade rule
`mask_i XOR mask_j` is dimension- and signature-independent; only the *sign*
rule specializes G(3,0,0)'s `(+,+,+)` signature. This is the seam where a
future G(p,q,r) requirement plugs in.

**Rotor orientation.** A rotor for a `+θ` rotation in the oriented plane of a
unit bivector `B̂` is

```
R = exp(−B̂ θ/2) = cos(θ/2) − B̂ sin(θ/2),   applied as  v' = R ∗ v ∗ ~R.
```

So a `+τ/4` rotation in the `e₁∧e₂` plane (`B̂ = e₁₂`) is
`R = cos(τ/8) − e₁₂ sin(τ/8)` — i.e. the **bivector component is `−sin(τ/8)`**
— and it sends `e₁ → e₂`, `e₂ → −e₁`, with `e₃` fixed.

**Decided:** 2026-05-28 (with SPEC-0002 acceptance).

## Engineering patterns

### The Oracle-Tripwire pattern

Any value that is *materialized* for speed or convenience (a lookup table, a
hand-chosen constant, a branch choice) but *could be derived* from a rule must
ship the rule as a test oracle. The derivation is the source of truth; the
materialization is the fast path; a generation test asserting the two agree is
the tripwire that fails loudly on any silent divergence (a hand edit, a
reorder, a changed assumption).

Instances so far:

- **SPEC-0001** — `ln_eml` uses the principal branch with no correction term;
  the correctness of that choice depends on `sin(τ/2) ≠ 0` in `f64`, asserted
  by the AC6 tripwire test.
- **SPEC-0002** — the 64-entry `CAYLEY` geometric-product table is the fast
  path; the 3-bit-mask rule (`mask_i XOR mask_j` + transposition-parity sign)
  is the oracle; a generation test asserts every entry agrees.

**Decided:** 2026-05-28.
