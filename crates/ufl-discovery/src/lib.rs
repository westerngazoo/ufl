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
//!
//! R-0013 adds the second proposer family ([SPEC-0013](../../../specs/0013-matmul-moonshot.md)):
//! the Kauers–Moosbauer **flip-graph** ([`reduce_matmul`]) — a walk over exact
//! schemes whose moves preserve the tensor by construction, certified only by
//! the same [`RankDecomposition`] discharge.

#![forbid(unsafe_code)]

mod engine;
mod flipgraph;
mod generic;
mod genome;
mod predicate;
mod prng;
mod proposer;

pub use engine::{run, Config, EngineError, Outcome};
pub use flipgraph::{
    flip_at, naive, perturb, reconstruct_int, reduce, reduce_matmul, reduce_matmul_with,
    shared_factor_pairs, target_int, FlipConfig, FlipError, IntScheme, IntTriple, Variant,
};
pub use generic::{run_generic, run_matmul_generic, Fitness, GenericOutcome, Proposer};
pub use genome::Genome;
pub use predicate::RankDecomposition;
pub use prng::SplitMix64;
pub use proposer::{GaConfig, GaProposer};
