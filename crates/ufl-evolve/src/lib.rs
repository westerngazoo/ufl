//! UFL — the evolver over the `GeoExpr` genotype (R-0011 / SPEC-0011).
//!
//! This crate will hold the memetic search (the generic `Fitness`/`Proposer`/
//! `run` seam + the structure-GA-plus-local-`Param`-refinement proposer). Its
//! first landed module is the **fair-MLP Gate-2 baseline** ([`baseline`]) — the
//! anti-strawman comparison the equivariant-OOD-generalization headline is scored
//! against (SPEC-0011 §2.5).

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod baseline;
pub mod memetic;
pub mod witness;

pub use baseline::{smallest_at, sweep, train_report, ArmFk, MlpReport};
pub use memetic::{gate1_fitness, magnitude, GeoFitness, GeoProposer, RotErr};
pub use witness::{
    fk_motor, fk_witness, measure_band, read_xy, witness_ctx, witness_env, BandReport,
};
