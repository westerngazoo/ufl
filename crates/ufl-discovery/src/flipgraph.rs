//! The Kauers–Moosbauer flip-graph over **exact** schemes (SPEC-0013).
//!
//! Realizes [R-0013](../../../requirements/0013-matmul-moonshot.md) Gate-0 per
//! [SPEC-0013](../../../specs/0013-matmul-moonshot.md): a walk over an
//! unrestricted-integer workspace whose moves are **tensor-preserving by
//! construction** — every state reconstructs to `T_n` exactly — so the walk's
//! only objective is *rank*. The `{−1,0,+1}` `Scheme` type cannot hold the
//! intermediate coefficients a flip produces (§2.1), hence the [`IntScheme`]
//! workspace; only a final ternary state converts back.
//!
//! **Verifier-held (AC2):** nothing here certifies a scheme. The caller
//! discharges the returned [`Scheme`] through the exact
//! [`RankDecomposition`](crate::RankDecomposition) — a tensor-breaking bug in a
//! move can only *fail* to certify, never produce a false positive.
//!
//! Moves are **public, pure primitives** ([`shared_factor_pairs`], [`flip_at`],
//! [`reduce`], [`perturb`]); [`reduce_matmul`] is a thin driver over them, so a
//! future move-form interpreter composes the same primitives without rewriting
//! this module (SPEC-0013 §2.3).

use crate::prng::SplitMix64;
use ufl_tensor::{target, Scheme, SchemeError, Triple};

/// Workspace envelope: flips are skipped when a coefficient would leave
/// `|c| ≤ 2¹⁶`. This exists solely to keep `i64` reconstruction arithmetic
/// overflow-free — it does not constrain the walk (an overly tight cap starves
/// exploration; SPEC-0013 §2.3).
const ENVELOPE: i64 = 1 << 16;

/// A search failure — never a panic (CLAUDE.md §6).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FlipError {
    /// The budget ran out; the best exact rank reached is reported honestly.
    #[error("budget exhausted: best exact rank reached is {best_rank}")]
    NotFound {
        /// The lowest rank of any exact state visited.
        best_rank: usize,
    },
    /// The coefficient-conversion guard (SPEC-0013 §2.4): a state was asked to
    /// become a `Scheme` while a coefficient was outside `{−1,0,+1}`.
    #[error("workspace state is not ternary: max |coefficient| is {max_abs}")]
    NonTernary {
        /// The largest coefficient magnitude in the offending state.
        max_abs: i64,
    },
    /// A structural failure surfacing from `ufl-tensor`'s validating
    /// constructors (impossible for a ternary state; kept total, not unwrapped).
    #[error(transparent)]
    Scheme(#[from] SchemeError),
}

/// One workspace multiplication: equal-length `u`/`v`/`w` over unrestricted
/// `i64` (the flip intermediates the `{−1,0,+1}` [`Triple`] cannot hold).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntTriple {
    u: Vec<i64>,
    v: Vec<i64>,
    w: Vec<i64>,
}

impl IntTriple {
    /// The left factor vector.
    pub fn u(&self) -> &[i64] {
        &self.u
    }
    /// The right factor vector.
    pub fn v(&self) -> &[i64] {
        &self.v
    }
    /// The output vector.
    pub fn w(&self) -> &[i64] {
        &self.w
    }
}

/// The proposer workspace: an ordered list of [`IntTriple`]s of one shared
/// length. Opaque — states are built by [`naive`] and transformed by the move
/// primitives, so every reachable state is tensor-exact by construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntScheme {
    triples: Vec<IntTriple>,
    dim: usize,
}

impl IntScheme {
    /// The number of multiplications (the rank being minimized).
    pub fn rank(&self) -> usize {
        self.triples.len()
    }

    /// The shared vector length `d = n²`.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// The workspace triples, read-only.
    pub fn triples(&self) -> &[IntTriple] {
        &self.triples
    }

    /// Is every coefficient in `{−1, 0, +1}` (convertible to a [`Scheme`])?
    pub fn is_ternary(&self) -> bool {
        self.coeffs().all(|c| (-1..=1).contains(&c))
    }

    /// Convert to a verifiable [`Scheme`]. The typed conversion guard of
    /// SPEC-0013 §2.4: a non-ternary state is a [`FlipError::NonTernary`],
    /// never a truncation.
    pub fn to_scheme(&self) -> Result<Scheme, FlipError> {
        if !self.is_ternary() {
            let max_abs = self.coeffs().map(i64::abs).max().unwrap_or(0);
            return Err(FlipError::NonTernary { max_abs });
        }
        let mut scheme = Scheme::new();
        for t in &self.triples {
            let tern = |xs: &[i64]| xs.iter().map(|&x| x as i8).collect::<Vec<i8>>();
            scheme.push(Triple::new(tern(&t.u), tern(&t.v), tern(&t.w))?)?;
        }
        Ok(scheme)
    }

    fn coeffs(&self) -> impl Iterator<Item = i64> + '_ {
        self.triples
            .iter()
            .flat_map(|t| t.u.iter().chain(&t.v).chain(&t.w).copied())
    }
}

/// Which factor-vector two triples share — the flip's rewrite axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    /// Shared left factor `u`.
    U,
    /// Shared right factor `v`.
    V,
    /// Shared output vector `w`.
    W,
}

/// The naive rank-`n³` scheme: one `0/1` triple per `(i,j,k)` with
/// `u = e_{i·n+j}`, `v = e_{j·n+k}`, `w = e_{i·n+k}` — exactly the terms of
/// [`ufl_tensor::target`], so it reconstructs to `T_n` by definition.
pub fn naive(n: usize) -> IntScheme {
    let d = n * n;
    let unit = |idx: usize| {
        let mut e = vec![0i64; d];
        e[idx] = 1;
        e
    };
    let mut triples = Vec::with_capacity(n * n * n);
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                triples.push(IntTriple {
                    u: unit(i * n + j),
                    v: unit(j * n + k),
                    w: unit(i * n + k),
                });
            }
        }
    }
    IntScheme { triples, dim: d }
}

/// The flat row-major `d³` image of the real matmul target — derived from
/// [`ufl_tensor::target`] through its public accessor (one source of truth,
/// never recomputed; SPEC-0013 §2.2).
pub fn target_int(n: usize) -> Vec<i64> {
    let t = target(n);
    let d = t.dim();
    let mut flat = Vec::with_capacity(d * d * d);
    for p in 0..d {
        for q in 0..d {
            for r in 0..d {
                // In range by the loop bounds; `None` is unreachable, mapped
                // to the additive identity rather than a panic path.
                flat.push(t.get(p, q, r).unwrap_or(0));
            }
        }
    }
    flat
}

/// `Σ_t u_t[p]·v_t[q]·w_t[r]` as a flat row-major `d³` vector — the workspace
/// mirror of `ufl_tensor::reconstruct`. **Search guidance and invariant
/// checking only — never certification** (SPEC-0013 §2.2).
pub fn reconstruct_int(s: &IntScheme) -> Vec<i64> {
    let d = s.dim;
    let mut flat = vec![0i64; d * d * d];
    for t in &s.triples {
        for (p, &up) in t.u.iter().enumerate() {
            if up == 0 {
                continue;
            }
            for (q, &vq) in t.v.iter().enumerate() {
                if vq == 0 {
                    continue;
                }
                let uv = up * vq;
                for (r, &wr) in t.w.iter().enumerate() {
                    if wr != 0 {
                        flat[(p * d + q) * d + r] += uv * wr;
                    }
                }
            }
        }
    }
    flat
}

/// The flip frontier: every **ordered** pair of triples sharing **exactly
/// one** factor-vector, tagged with the shared slot. Ordered, because the
/// rewrite is asymmetric — `(i, j)` and `(j, i)` are distinct moves
/// (SPEC-0013 §2.3). Pairs sharing two or more slots belong to [`reduce`].
pub fn shared_factor_pairs(s: &IntScheme) -> Vec<((usize, usize), Variant)> {
    let mut pairs = Vec::new();
    for i in 0..s.triples.len() {
        for j in 0..s.triples.len() {
            if i == j {
                continue;
            }
            let (a, b) = (&s.triples[i], &s.triples[j]);
            let shared = [
                (a.u == b.u, Variant::U),
                (a.v == b.v, Variant::V),
                (a.w == b.w, Variant::W),
            ];
            if shared.iter().filter(|(eq, _)| *eq).count() == 1 {
                // Exactly one hit exists; take it.
                if let Some(&(_, variant)) = shared.iter().find(|(eq, _)| *eq) {
                    pairs.push(((i, j), variant));
                }
            }
        }
    }
    pairs
}

/// Apply the sum-preserving Kauers–Moosbauer flip to the ordered pair
/// `(i, j)` on the shared `variant` slot. `None` when the move is
/// inapplicable (bad indices, slot not the unique shared one) or when a
/// coefficient would leave the workspace envelope.
///
/// The three variants, each with its invariance proof:
///
/// - **U** (shared `u`): `(u,b,c), (u,b′,c′) → (u, b, c−c′), (u, b+b′, c′)`
///   because `u⊗b⊗(c−c′) + u⊗(b+b′)⊗c′ = u⊗b⊗c + u⊗b′⊗c′`.
/// - **V** (shared `v`): `(a,v,c), (a′,v,c′) → (a, v, c−c′), (a+a′, v, c′)`
///   because `a⊗v⊗(c−c′) + (a+a′)⊗v⊗c′ = a⊗v⊗c + a′⊗v⊗c′`.
/// - **W** (shared `w`): `(a,b,w), (a′,b′,w) → (a−a′, b, w), (a′, b+b′, w)`
///   because `(a−a′)⊗b⊗w + a′⊗(b+b′)⊗w = a⊗b⊗w + a′⊗b′⊗w`.
pub fn flip_at(s: &IntScheme, pair: (usize, usize), variant: Variant) -> Option<IntScheme> {
    let (i, j) = pair;
    if i == j || i >= s.triples.len() || j >= s.triples.len() {
        return None;
    }
    let (a, b) = (&s.triples[i], &s.triples[j]);
    let shared = (a.u == b.u, a.v == b.v, a.w == b.w);
    let unique = match shared {
        (true, false, false) => Variant::U,
        (false, true, false) => Variant::V,
        (false, false, true) => Variant::W,
        _ => return None, // zero or 2+ shared slots: not a flip
    };
    if unique != variant {
        return None;
    }
    let sub = |x: &[i64], y: &[i64]| x.iter().zip(y).map(|(p, q)| p - q).collect::<Vec<i64>>();
    let add = |x: &[i64], y: &[i64]| x.iter().zip(y).map(|(p, q)| p + q).collect::<Vec<i64>>();
    let (new_i, new_j) = match variant {
        Variant::U => (
            IntTriple {
                u: a.u.clone(),
                v: a.v.clone(),
                w: sub(&a.w, &b.w),
            },
            IntTriple {
                u: b.u.clone(),
                v: add(&a.v, &b.v),
                w: b.w.clone(),
            },
        ),
        Variant::V => (
            IntTriple {
                u: a.u.clone(),
                v: a.v.clone(),
                w: sub(&a.w, &b.w),
            },
            IntTriple {
                u: add(&a.u, &b.u),
                v: b.v.clone(),
                w: b.w.clone(),
            },
        ),
        Variant::W => (
            IntTriple {
                u: sub(&a.u, &b.u),
                v: a.v.clone(),
                w: a.w.clone(),
            },
            IntTriple {
                u: b.u.clone(),
                v: add(&a.v, &b.v),
                w: b.w.clone(),
            },
        ),
    };
    let within = |t: &IntTriple| {
        t.u.iter()
            .chain(&t.v)
            .chain(&t.w)
            .all(|c| c.abs() <= ENVELOPE)
    };
    if !within(&new_i) || !within(&new_j) {
        return None;
    }
    let mut next = s.clone();
    next.triples[i] = new_i;
    next.triples[j] = new_j;
    Some(next)
}

/// Reduce to fixpoint: merge any pair sharing **two** factor-vectors (the
/// third coefficients add — rank −1), and drop any triple containing an
/// all-zero vector (it contributes `0`). Never raises rank; preserves the
/// tensor (each merge is the distributive law read backwards).
pub fn reduce(s: &IntScheme) -> IntScheme {
    let mut triples = s.triples.clone();
    loop {
        triples.retain(|t| {
            let zero = |xs: &[i64]| xs.iter().all(|&x| x == 0);
            !(zero(&t.u) || zero(&t.v) || zero(&t.w))
        });
        let mut merged = None;
        'search: for i in 0..triples.len() {
            for j in (i + 1)..triples.len() {
                let (a, b) = (&triples[i], &triples[j]);
                let add = |x: &[i64], y: &[i64]| {
                    x.iter().zip(y).map(|(p, q)| p + q).collect::<Vec<i64>>()
                };
                let m = match (a.u == b.u, a.v == b.v, a.w == b.w) {
                    (true, true, _) => IntTriple {
                        u: a.u.clone(),
                        v: a.v.clone(),
                        w: add(&a.w, &b.w),
                    },
                    (true, false, true) => IntTriple {
                        u: a.u.clone(),
                        v: add(&a.v, &b.v),
                        w: a.w.clone(),
                    },
                    (false, true, true) => IntTriple {
                        u: add(&a.u, &b.u),
                        v: a.v.clone(),
                        w: a.w.clone(),
                    },
                    _ => continue,
                };
                merged = Some((i, j, m));
                break 'search;
            }
        }
        match merged {
            Some((i, j, m)) => {
                triples[i] = m;
                triples.swap_remove(j);
            }
            None => break,
        }
    }
    IntScheme {
        triples,
        dim: s.dim,
    }
}

/// Up to `k` random flips with **no reduction** — the plateau-escape kick
/// applied to the best-so-far checkpoint (SPEC-0013 §2.3/§2.4). Draws exactly
/// one `below` per attempted flip.
pub fn perturb(s: &IntScheme, k: usize, rng: &mut SplitMix64) -> IntScheme {
    let mut out = s.clone();
    for _ in 0..k {
        let pairs = shared_factor_pairs(&out);
        if pairs.is_empty() {
            break;
        }
        let (pair, variant) = pairs[rng.below(pairs.len() as u64) as usize];
        if let Some(next) = flip_at(&out, pair, variant) {
            out = next;
        }
    }
    out
}

/// The plateau policy as a named, testable object (SPEC-0013 §2.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlipConfig {
    /// Steps without a strict best-rank improvement before a perturbation.
    pub stall_window: usize,
    /// Flips applied to the checkpoint on each perturbation.
    pub perturb_flips: usize,
}

impl FlipConfig {
    /// The pinned Gate-0 policy (mirrors `GaConfig::pinned`): perturbation
    /// from the best-so-far checkpoint — never a restart from naive, which
    /// discards discovered structure (the pilot's load-bearing finding).
    pub fn pinned() -> Self {
        Self {
            stall_window: 400,
            perturb_flips: 6,
        }
    }
}

/// Walk the flip-graph from the naive scheme down to `target_rank` for
/// `T_n`, under the pinned plateau policy. Deterministic in `seed`; the
/// result is a *candidate* — certification belongs to the caller's verifier
/// (SPEC-0013 §2.5).
///
/// ```
/// use ufl_discovery::{reduce_matmul, RankDecomposition};
/// use ufl_predicate::Predicate;
///
/// // n = 1: the naive scheme is already the rank-1 answer.
/// let scheme = reduce_matmul(1, 1, 0, 10).expect("trivial");
/// assert_eq!(RankDecomposition::new(1, 1).discharge(&scheme), Ok(true));
/// ```
pub fn reduce_matmul(
    n: usize,
    target_rank: usize,
    seed: u64,
    budget: usize,
) -> Result<Scheme, FlipError> {
    reduce_matmul_with(n, target_rank, seed, budget, FlipConfig::pinned())
}

/// [`reduce_matmul`] with an explicit plateau policy.
///
/// Determinism contract (SPEC-0013 §2.4): per loop step the driver draws
/// exactly one `below` over the flip frontier; each perturbation draws one
/// `below` per attempted flip; nothing else touches the generator — which is
/// what makes the trajectory replayable through the public primitives.
pub fn reduce_matmul_with(
    n: usize,
    target_rank: usize,
    seed: u64,
    budget: usize,
    config: FlipConfig,
) -> Result<Scheme, FlipError> {
    let tgt = target_int(n);
    let mut rng = SplitMix64::new(seed);
    let mut s = reduce(&naive(n));
    debug_assert_eq!(reconstruct_int(&s), tgt, "naive scheme must be exact");
    if s.rank() <= target_rank && s.is_ternary() {
        return s.to_scheme();
    }
    let mut best = s.clone();
    let mut stall = 0usize;
    for _ in 0..budget {
        let pairs = shared_factor_pairs(&s);
        if !pairs.is_empty() {
            let (pair, variant) = pairs[rng.below(pairs.len() as u64) as usize];
            if let Some(next) = flip_at(&s, pair, variant) {
                s = reduce(&next);
                debug_assert_eq!(
                    reconstruct_int(&s),
                    tgt,
                    "a move broke the tensor: flip/reduce bug, not a candidate"
                );
            }
        }
        if s.rank() <= target_rank && s.is_ternary() {
            return s.to_scheme();
        }
        if s.rank() < best.rank() {
            best = s.clone();
            stall = 0;
        } else {
            stall += 1;
        }
        if stall >= config.stall_window {
            s = perturb(&best, config.perturb_flips, &mut rng);
            debug_assert_eq!(reconstruct_int(&s), tgt, "perturb must preserve the tensor");
            stall = 0;
        }
    }
    Err(FlipError::NotFound {
        best_rank: best.rank(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The naive scheme *is* the target, term by term.
    #[test]
    fn naive_reconstructs_the_target_exactly() {
        for n in 1..=3 {
            assert_eq!(reconstruct_int(&naive(n)), target_int(n), "n = {n}");
        }
    }

    /// The frontier excludes pairs sharing two slots (those belong to reduce)
    /// and `flip_at` refuses a variant that is not the unique shared slot.
    #[test]
    fn flip_rejects_non_flip_pairs() {
        let s = naive(2);
        // Triples 0 and 1 of naive(2) share u (i=0,j=0) but differ in v and w.
        assert!(flip_at(&s, (0, 1), Variant::U).is_some());
        assert!(flip_at(&s, (0, 1), Variant::V).is_none());
        assert!(flip_at(&s, (0, 0), Variant::U).is_none(), "i == j");
        assert!(flip_at(&s, (0, 99), Variant::U).is_none(), "out of range");
    }

    /// A non-ternary state is a typed conversion error, never a truncation.
    #[test]
    fn to_scheme_guards_the_ternary_envelope() {
        let s = naive(2);
        // Two U-flips in a row can push a coefficient to ±2.
        let flipped = flip_at(&s, (0, 1), Variant::U).expect("applicable");
        let again = flip_at(&flipped, (0, 1), Variant::U).expect("applicable");
        if !again.is_ternary() {
            match again.to_scheme() {
                Err(FlipError::NonTernary { max_abs }) => assert!(max_abs > 1),
                other => panic!("expected NonTernary, got {other:?}"),
            }
        }
    }
}
