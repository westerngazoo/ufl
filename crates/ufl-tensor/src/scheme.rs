//! The scheme genotype — `Triple` and `Scheme`, validated and length-consistent
//! (SPEC-0006 §2.4). These are fully implemented (no deferral): a `Triple`
//! self-validates, and a `Scheme` enforces one shared length, so the `d`/`n`
//! desync that would panic `reconstruct` is impossible by construction.

/// A failure constructing a triple or extending a scheme (SPEC-0006 §2.4/§2.6).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SchemeError {
    #[error("u/v/w lengths differ: {u}, {v}, {w}")]
    Ragged { u: usize, v: usize, w: usize },
    #[error("empty vector: a triple's vectors must be non-empty")]
    Empty,
    #[error("coefficient {0} outside {{-1, 0, +1}}")]
    Coefficient(i8),
    #[error("triple length {got} ≠ scheme length {expected}")]
    Mismatch { expected: usize, got: usize },
    #[error("scheme dim {got} ≠ n² = {expected} for n = {n}")]
    DimMismatch {
        n: usize,
        expected: usize,
        got: usize,
    },
}

/// One scalar multiplication: `u`, `v`, `w` are equal-length, non-empty vectors
/// with entries in `{-1, 0, +1}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Triple {
    u: Vec<i8>,
    v: Vec<i8>,
    w: Vec<i8>,
}

fn validate_coeffs(v: &[i8]) -> Result<(), SchemeError> {
    for &c in v {
        if !(-1..=1).contains(&c) {
            return Err(SchemeError::Coefficient(c));
        }
    }
    Ok(())
}

impl Triple {
    /// Validated constructor: `u`/`v`/`w` must share one non-empty length and
    /// hold only `{-1, 0, +1}`. Rejection is a typed `SchemeError`, never a
    /// panic (R-0006 AC2).
    pub fn new(u: Vec<i8>, v: Vec<i8>, w: Vec<i8>) -> Result<Self, SchemeError> {
        if u.len() != v.len() || v.len() != w.len() {
            return Err(SchemeError::Ragged {
                u: u.len(),
                v: v.len(),
                w: w.len(),
            });
        }
        if u.is_empty() {
            return Err(SchemeError::Empty);
        }
        validate_coeffs(&u)?;
        validate_coeffs(&v)?;
        validate_coeffs(&w)?;
        Ok(Self { u, v, w })
    }

    /// The shared vector length (`= d` for a real scheme).
    pub fn len(&self) -> usize {
        self.u.len()
    }

    /// Always `false` — a triple's vectors are non-empty by construction.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// The left factor vector `u` — read-only (SPEC-0013 §2.1 accessor
    /// promotion: a `Scheme` the language can construct but never read is not
    /// first-class data; invariants stay constructor-enforced).
    pub fn u(&self) -> &[i8] {
        &self.u
    }

    /// The right factor vector `v` — read-only, same contract as [`Triple::u`].
    pub fn v(&self) -> &[i8] {
        &self.v
    }

    /// The output vector `w` — read-only, same contract as [`Triple::u`].
    pub fn w(&self) -> &[i8] {
        &self.w
    }
}

/// An ordered list of triples. Invariant: all triples share one length, so the
/// scheme has a single well-defined `dim`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Scheme {
    triples: Vec<Triple>,
}

impl Scheme {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a triple, rejecting one whose length differs from the scheme's
    /// existing length (`Mismatch`). Keeps the scheme length-consistent.
    pub fn push(&mut self, t: Triple) -> Result<&mut Self, SchemeError> {
        if let Some(d) = self.dim() {
            if t.len() != d {
                return Err(SchemeError::Mismatch {
                    expected: d,
                    got: t.len(),
                });
            }
        }
        self.triples.push(t);
        Ok(self)
    }

    /// The number of triples (the multiplication count `R`).
    pub fn rank(&self) -> usize {
        self.triples.len()
    }

    /// The shared triple length (`d`); `None` if the scheme is empty.
    pub fn dim(&self) -> Option<usize> {
        self.triples.first().map(Triple::len)
    }

    pub fn triples(&self) -> &[Triple] {
        &self.triples
    }
}
