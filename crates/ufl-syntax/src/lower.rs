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
pub fn lower(s: &Sexpr) -> Result<Eml, LowerError> {
    match s {
        // The primitive is the *value* `1` (1.0 is exactly representable), so a
        // bit-pattern comparison is exact, total, and free of the float_cmp
        // lint — not a weakened check (SPEC-0003 §2.4).
        Sexpr::Num(n) if n.to_bits() == 1.0_f64.to_bits() => Ok(Eml::one()),
        Sexpr::Num(n) => Err(LowerError::UnsupportedLiteral(*n)),
        Sexpr::Sym(name) => Ok(Eml::var(name.as_str())),
        Sexpr::List(items) => lower_form(items),
    }
}

/// Lower a list as a form: dispatch on the head symbol. This `match` is the
/// documented seam where future forms (and the orchestrator/macro layer) plug
/// in; the form-table registry is deferred until form count warrants it.
fn lower_form(items: &[Sexpr]) -> Result<Eml, LowerError> {
    // An empty list or a non-symbol head is not a form.
    let Some((Sexpr::Sym(head), args)) = items.split_first() else {
        return Err(LowerError::NotAForm);
    };
    match head.as_str() {
        "eml" => match args {
            [a, b] => Ok(Eml::node(lower(a)?, lower(b)?)),
            _ => Err(LowerError::Arity {
                form: "eml".to_string(),
                expected: 2,
                got: args.len(),
            }),
        },
        other => Err(LowerError::UnknownForm(other.to_string())),
    }
}

/// Raise an [`Eml`](ufl_core::Eml) back into its [`Sexpr`] — the transpose of
/// [`lower`]'s table, and the Rust-side inverse that closes the code↔data square
/// (SPEC-0016 §2.5). Each row inverts one `lower` case: `One → 1`, `Var → sym`,
/// `Node → (eml …)`.
///
/// `raise` is **total** on `Eml` (no `Result`) — every `Eml` has a structural
/// image. `raise ∘ lower = id` holds on the **reader's canonical image** (see
/// [`is_reader_canonical_sym`](crate::is_reader_canonical_sym)); it is *not*
/// unconditional, because `lower` accepts `Sym` payloads the reader never emits
/// (e.g. `lower(Sym("1")) = Var("1")`, but `read("1") = Num(1.0)`).
pub fn raise(e: &Eml) -> Sexpr {
    match e {
        Eml::One => Sexpr::num(1.0),
        Eml::Var(name) => Sexpr::sym(name.as_str()),
        Eml::Node { exp_arg, log_arg } => {
            Sexpr::list([Sexpr::sym("eml"), raise(exp_arg), raise(log_arg)])
        }
    }
}
