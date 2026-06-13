//! Genotype and the total `express` map (SPEC-0008 §2.3).
//!
//! The proposer owns the mutable [`Genome`]; the phenotype is a `Scheme` built
//! through `ufl-tensor`'s validating constructors (*Guard Inside the
//! Candidate*). `express` is **total** — a malformed genome is a typed error,
//! never a silently shorter scheme.

use ufl_tensor::{Scheme, SchemeError, Triple};

/// A candidate genotype: `rank` triples, each `(u, v, w)` a length-`d` ternary
/// vector. Mutable and cheap to flip/cross; expressed to a `Scheme` for scoring.
#[derive(Clone, Debug, PartialEq)]
pub struct Genome {
    pub triples: Vec<[Vec<i8>; 3]>,
}

/// Express a genome to its phenotype. Total: validity is enforced here by
/// `Triple::new` / `Scheme::push`; a malformed genome surfaces as a typed
/// `SchemeError` rather than degrading fitness (R-0008 AC6).
// Consumed by `engine::run` in R-0008 step 5; unused during the red scaffold.
#[allow(dead_code)]
pub(crate) fn express(g: &Genome) -> Result<Scheme, SchemeError> {
    let mut scheme = Scheme::new();
    for [u, v, w] in &g.triples {
        scheme.push(Triple::new(u.clone(), v.clone(), w.clone())?)?;
    }
    Ok(scheme)
}
