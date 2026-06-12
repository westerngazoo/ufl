//! UFL — the discovery bridge.
//!
//! Realizes [R-0007](../../../requirements/0007-tensor-predicate.md) per
//! [SPEC-0007](../../../specs/0007-tensor-predicate.md): the matmul-
//! decomposition predicate `P_{n,R}` as a first-class **Hehner predicate** —
//! [`RankDecomposition`] implements `ufl_predicate::Predicate`, so the
//! discovery verifier *is* the same discharge contract the scalar checker
//! routes through. This is the bridge where the predicate and tensor domains
//! meet (`ufl-tensor` stays a pure leaf); the GA engine (R-0008) grows here.

#![forbid(unsafe_code)]

mod predicate;

pub use predicate::RankDecomposition;
