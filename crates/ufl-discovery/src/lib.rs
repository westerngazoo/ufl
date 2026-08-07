//! UFL — the discovery bridge.
//!
//! Realizes [R-0007](../../../requirements/0007-tensor-predicate.md) per
//! [SPEC-0007](../../../specs/0007-tensor-predicate.md): the matmul-
//! decomposition predicate `P_{n,R}` as a first-class **Hehner predicate** —
//! [`RankDecomposition`] implements `ufl_predicate::Predicate`, so the
//! discovery verifier *is* the same discharge contract the scalar checker
//! routes through. This is the bridge where the predicate and tensor domains
//! meet (`ufl-tensor` stays a pure leaf).
//!
//! R-0008 grows the **discovery engine** here ([SPEC-0008](../../../specs/0008-discovery-engine.md)):
//! a seeded genetic search ([`run`]) whose candidate source is the blind
//! [`GaProposer`] and whose accept step is the [`RankDecomposition`] discharge —
//! proposer-agnostic, verifier-exact.

#![forbid(unsafe_code)]

mod engine;
mod genome;
mod predicate;
mod prng;
mod proposer;

pub use engine::{run, Config, EngineError, Outcome};
pub use genome::Genome;
pub use predicate::RankDecomposition;
pub use prng::SplitMix64;
pub use proposer::{GaConfig, GaProposer};
