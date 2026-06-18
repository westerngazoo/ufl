//! UFL — the geometric-algebra substrate (Pillar 2).
//!
//! Realizes [R-0009](../../../requirements/0009-pga-kernel-binding.md) per
//! [SPEC-0009](../../../specs/0009-pga-kernel-binding.md): a **thin facade** over
//! garust's `Cl(3,0,1)` Projective Geometric Algebra kernel (real `f64`). The
//! geometric forms (R-0010) and neuroevolution (R-0011) target this surface.
//!
//! The facade is deliberately thin — it does *not* re-implement geometric
//! algebra. Its earned value is (1) UFL-named basis constructors that hide
//! garust's blade-index convention, (2) a single curated import path, and (3)
//! UFL-owned geometric-correctness tests. **Insulation against garust churn is
//! held by the version pin** (a `v0.1.0` tag → a locked rev), not by a wrapping
//! type — see SPEC-0009 §2.4.

#![forbid(unsafe_code)]

/// UFL's geometric value: a `Cl(3,0,1)` PGA multivector over `f64`. A
/// transparent alias for garust's kernel — its inherent methods (`wedge`,
/// `inner`, `grade`, `reverse`, `sandwich`, `norm`, …) and the geometric product
/// `*` are available directly.
pub type Mv = garust::Pga3;

pub use garust::pga::Point;
pub use garust::Motor;

/// UFL-named basis constructors for `Cl(3,0,1)`. The blade indices are pinned to
/// garust's convention (the degenerate generator is last); each is total — every
/// index is `< DIM (16)`, so `Mv::basis` cannot panic (SPEC-0009 §2.2).
pub mod basis {
    use super::Mv;

    /// The grade-0 scalar `s`.
    pub fn scalar(_s: f64) -> Mv {
        unimplemented!("R-0009 step 5 — see SPEC-0009 §2.2")
    }

    /// `e1` (`e1² = +1`).
    pub fn e1() -> Mv {
        unimplemented!("R-0009 step 5 — see SPEC-0009 §2.2")
    }

    /// `e2` (`e2² = +1`).
    pub fn e2() -> Mv {
        unimplemented!("R-0009 step 5 — see SPEC-0009 §2.2")
    }

    /// `e3` (`e3² = +1`).
    pub fn e3() -> Mv {
        unimplemented!("R-0009 step 5 — see SPEC-0009 §2.2")
    }

    /// `e0` — the ideal/null generator (`e0² = 0`).
    pub fn e0() -> Mv {
        unimplemented!("R-0009 step 5 — see SPEC-0009 §2.2")
    }

    /// The grade-4 pseudoscalar.
    pub fn pseudoscalar() -> Mv {
        unimplemented!("R-0009 step 5 — see SPEC-0009 §2.2")
    }
}
