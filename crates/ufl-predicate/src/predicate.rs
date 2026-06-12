//! The `Predicate` discharge trait and the guarded scalar [`State`]
//! (SPEC-0007 §2.1–§2.2).
//!
//! A predicate is a *decidable property of a candidate state* — Hehner's
//! notion, generalized over the domain. The scalar instance is the
//! [`Sexpr`] itself (homoiconic: the s-expression *is* the predicate); the
//! tensor instance (`RankDecomposition`) lives in `ufl-discovery`.

use ufl_core::{Env, Value};
use ufl_syntax::Sexpr;

use crate::{combined_env, eval_pred, CheckError};

/// A predicate is a *decidable property of a candidate state*. Discharging it
/// is the check (decidable); searching for a satisfying candidate (R-0008) and
/// selecting a substrate (the future orchestrator) are separate concerns.
pub trait Predicate {
    /// The kind of state this predicate ranges over.
    type Candidate;
    /// The typed failure channel.
    type Error: std::error::Error;
    /// Decide whether `candidate` satisfies this predicate. Total within the
    /// domain's stated envelope: a malformed candidate *or an undischargeable
    /// predicate* (e.g. a non-boolean `Sexpr`, an unbound variable) is a typed
    /// `Error`, never a panic.
    fn discharge(&self, candidate: &Self::Candidate) -> Result<bool, Self::Error>;
}

/// The guarded pre/post state a scalar predicate is checked against. The
/// **only** constructor applies SPEC-0004 §2.5's rules — pre vars bind by
/// name, post vars bind primed, and a binding name containing `'` is rejected
/// (`ReservedName`) — so the guard lives inside the candidate and the trait
/// path cannot bypass it.
pub struct State {
    env: Env,
}

impl State {
    /// Build the combined state, enforcing the priming/reserved-name rules.
    /// Delegates to the same logic `check` has always used.
    pub fn new(pre: &[(&str, Value)], post: &[(&str, Value)]) -> Result<State, CheckError> {
        Ok(State {
            env: combined_env(pre, post)?,
        })
    }

    pub(crate) fn env(&self) -> &Env {
        &self.env
    }
}

/// The homoiconic scalar instance: the s-expression *is* the Hehner predicate
/// (SPEC-0007 §2.2). Discharging evaluates it as a boolean over the state.
impl Predicate for Sexpr {
    type Candidate = State;
    type Error = CheckError;

    fn discharge(&self, state: &State) -> Result<bool, CheckError> {
        Ok(eval_pred(self, state.env())?)
    }
}
