//! The literal `eml` primitive-tree builders (SPEC-0014N §1).
//!
//! Everything is built from the three atoms `Eml::{one, var, node}`, where
//! `node(x, y) ≡ exp(x) − ln(y)`. These live in `tests/` deliberately: they are
//! *derivations under test*, not a public numeric API (SPEC-0014N §2.1).
//!
//! **Domain (§1, measured — these are not unconditional identities):**
//! `sub`/`neg`/`add` need `|second operand| ≤ 709.782712893`; `ln` holes below
//! `e^e/f64::MAX ≈ 8.4298e-308`; and `mul`/`nand` inherit the `ln` cliff on their
//! **first** operand, so `nand` is total on `{0,1}` but partial below that.

use ufl_core::Eml;

/// `exp(z) = eml(z, 1)`.
pub fn exp_t(z: Eml) -> Eml {
    Eml::node(z, Eml::one())
}

/// `e = eml(1, 1) = exp(1)`.
pub fn e_t() -> Eml {
    Eml::node(Eml::one(), Eml::one())
}

/// `0 = eml(1, exp(e)) = e − e`. Exhaustively the unique minimal zero over the
/// closed trees on `{1}` (SPEC-0014N §1).
pub fn zero_t() -> Eml {
    Eml::node(Eml::one(), exp_t(e_t()))
}

/// `ln(y) = eml(1, eml(eml(1, y), 1))` — **R-0001 AC5's shipped identity**, 7
/// nodes, the exhaustive minimum. Its inner intermediate is `e^e/y`, which is
/// where the `8.4298e-308` cliff comes from.
pub fn ln_t(y: Eml) -> Eml {
    Eml::node(Eml::one(), Eml::node(Eml::node(Eml::one(), y), Eml::one()))
}

/// `a − b = eml(ln a, exp b)`.
pub fn sub_t(a: Eml, b: Eml) -> Eml {
    Eml::node(ln_t(a), exp_t(b))
}

/// `−b = 0 − b` — routes through `ln 0 = −∞` (R-0001 AC3).
pub fn neg_t(b: Eml) -> Eml {
    sub_t(zero_t(), b)
}

/// `a + b = a − (−b)`.
pub fn add_t(a: Eml, b: Eml) -> Eml {
    sub_t(a, neg_t(b))
}

/// `a × b = exp(ln a + ln b)` — the log-domain product.
pub fn mul_t(a: Eml, b: Eml) -> Eml {
    exp_t(add_t(ln_t(a), ln_t(b)))
}

/// `NAND(a, b) = 1 − (a × b)` — the whole gate, as one `eml` tree.
pub fn nand_t(a: Eml, b: Eml) -> Eml {
    sub_t(Eml::one(), mul_t(a, b))
}

/// The bit encoding: `false ↦ 0`, `true ↦ 1`.
pub fn bit(b: bool) -> Eml {
    if b {
        Eml::one()
    } else {
        zero_t()
    }
}

/// `NOT a = NAND(a, a)` — note this **duplicates** its argument, which is the
/// source of the `50·2^g − 49` chained-NOT blow-up (§3): a property of the NAND
/// presentation, not of `Eml`.
pub fn not_t(a: Eml) -> Eml {
    nand_t(a.clone(), a)
}

/// `a AND b = NOT(NAND(a, b))`.
pub fn and_t(a: Eml, b: Eml) -> Eml {
    not_t(nand_t(a, b))
}

/// `a OR b = NAND(NOT a, NOT b)` — the chain `theory/universal-computability.md`
/// cites for functional completeness.
pub fn or_t(a: Eml, b: Eml) -> Eml {
    nand_t(not_t(a), not_t(b))
}

/// Node count (leaves included) — the unit §3's size laws are stated in.
pub fn nodes(e: &Eml) -> usize {
    match e {
        Eml::One | Eml::Var(_) => 1,
        Eml::Node { exp_arg, log_arg } => 1 + nodes(exp_arg) + nodes(log_arg),
    }
}

/// Tree depth — `10·g + 3` for a `g`-gate chained-NOT (§5 sequencing note).
pub fn depth(e: &Eml) -> usize {
    match e {
        Eml::One | Eml::Var(_) => 1,
        Eml::Node { exp_arg, log_arg } => 1 + depth(exp_arg).max(depth(log_arg)),
    }
}

/// Every subtree's value, for the §2.2 intermediate census (infinities vs NaN).
pub fn subtree_values(e: &Eml, env: &ufl_core::Env, out: &mut Vec<ufl_core::Value>) {
    if let Ok(v) = ufl_core::eval(e, env) {
        out.push(v);
    }
    if let Eml::Node { exp_arg, log_arg } = e {
        subtree_values(exp_arg, env, out);
        subtree_values(log_arg, env, out);
    }
}
