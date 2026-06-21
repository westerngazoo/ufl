//! The geometric AST `GeoExpr` + the eval environment (SPEC-0010 §2.1).

use std::collections::HashMap;
use ufl_ga::Mv;

/// A geometric program: a tree of `Cl(3,0,1)` forms over the `ufl-ga` kernel.
/// `Clone` + inspectable — the genotype R-0011 mutates and recombines. Leaves
/// carry raw `u8` blade/grade indices; out-of-range values are rejected at the
/// `eval`/`typecheck` boundary (a plain data enum, simplest for R-0011).
#[derive(Clone, Debug, PartialEq)]
pub enum GeoExpr {
    /// An evolvable scalar parameter (R-0011 tunes these). Grade `{0}`.
    Param(f64),
    /// A basis blade by index `0..16` (`e1=1, e2=2, e3=4, e0=8, e12=3, …`).
    Basis(u8),
    /// An input multivector, bound at eval.
    Var(String),
    /// 𝒢ₖ — a scalar lifted to the lowest-index grade-`k` blade.
    GradeLift(u8, Box<GeoExpr>),
    /// The geometric product `a ∗ b`.
    GeoProduct(Box<GeoExpr>, Box<GeoExpr>),
    /// The outer (wedge) product `a ∧ b`.
    Wedge(Box<GeoExpr>, Box<GeoExpr>),
    /// The (Hestenes) inner product `a · b`.
    Inner(Box<GeoExpr>, Box<GeoExpr>),
    /// Reverse `~a`.
    Reverse(Box<GeoExpr>),
    /// Grade projection `⟨a⟩ₖ`.
    GradeProject(u8, Box<GeoExpr>),
    /// The versor sandwich `r x ~r`.
    Sandwich(Box<GeoExpr>, Box<GeoExpr>),
    /// `exp` — a rotor/motor from a bivector.
    Exp(Box<GeoExpr>),
}

/// The eval environment: input variables bound to multivectors
/// (mirrors `ufl_core::Env`).
#[derive(Clone, Debug, Default)]
pub struct Env {
    vars: HashMap<String, Mv>,
}

impl Env {
    /// An empty environment.
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind a variable to a multivector.
    pub fn bind(&mut self, name: impl Into<String>, value: Mv) {
        self.vars.insert(name.into(), value);
    }

    /// Look up a bound variable (`Mv` is `Copy`).
    pub fn get(&self, name: &str) -> Option<Mv> {
        self.vars.get(name).copied()
    }
}

/// The lowest-index grade-`k` blade index (pinned, SPEC-0010 §2.2): `0→0`
/// (scalar), `1→1` (e1), `2→3` (e12), `3→7` (e123), `4→15` (pseudoscalar).
/// `None` for `k > 4`. Lowest-index dodges the null `e₀`-bearing blades.
pub(crate) fn lowest_blade(k: u8) -> Option<u8> {
    match k {
        0 => Some(0),
        1 => Some(1),
        2 => Some(3),
        3 => Some(7),
        4 => Some(15),
        _ => None,
    }
}
