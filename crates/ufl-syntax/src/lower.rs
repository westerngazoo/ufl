//! The lowering pass — [`Sexpr`] → [`Eml`](ufl_core::Eml) (SPEC-0003 §2.4).
//!
//! Lowering enforces R-0001's grammar (`S → 1 | var | eml(S, S)`), recovering
//! the typed core's structural guarantees at the lowering boundary. It is
//! total and side-effect-free.

use ufl_core::Eml;

use crate::sexpr::Sexpr;

/// A failure while lowering an [`Sexpr`] into [`Eml`](ufl_core::Eml)
/// (SPEC-0003 §2.4 / AC3).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum LowerError {
    #[error("unsupported numeric literal {0}: only `1` is primitive in this core")]
    UnsupportedLiteral(f64),
    #[error("unknown form: `{0}`")]
    UnknownForm(String),
    #[error("form `{form}` expects {expected} arguments, got {got}")]
    Arity {
        form: String,
        expected: usize,
        got: usize,
    },
    #[error("not a form: a list must be a non-empty application with a symbol head")]
    NotAForm,
}

/// Lower an [`Sexpr`] into R-0001's typed [`Eml`](ufl_core::Eml).
pub fn lower(_s: &Sexpr) -> Result<Eml, LowerError> {
    unimplemented!("R-0003 implementation — lowering, see SPEC-0003 §2.4")
}
