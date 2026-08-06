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
///
/// **Iterative** (R-0017): a post-order walk over an explicit work-stack, so a
/// tree of any depth lowers without recursion and **without a depth cap**. The
/// error behaviour is unchanged — head/arity are validated *eagerly*, before
/// descending, so an `UnknownForm`/`Arity`/`NotAForm` still surfaces before any
/// child error, exactly as the recursive version did.
pub fn lower(s: &Sexpr) -> Result<Eml, LowerError> {
    enum Task<'a> {
        Visit(&'a Sexpr),
        Combine,
    }

    let mut tasks = vec![Task::Visit(s)];
    let mut out: Vec<Eml> = Vec::new();

    while let Some(task) = tasks.pop() {
        match task {
            Task::Visit(node) => match node {
                // The primitive is the *value* `1` (1.0 is exactly
                // representable), so a bit-pattern comparison is exact, total,
                // and free of the float_cmp lint (SPEC-0003 §2.4).
                Sexpr::Num(n) if n.to_bits() == 1.0_f64.to_bits() => out.push(Eml::one()),
                Sexpr::Num(n) => return Err(LowerError::UnsupportedLiteral(*n)),
                Sexpr::Sym(name) => out.push(Eml::var(name.as_str())),
                Sexpr::List(items) => {
                    // An empty list or a non-symbol head is not a form.
                    let Some((Sexpr::Sym(head), args)) = items.split_first() else {
                        return Err(LowerError::NotAForm);
                    };
                    match head.as_str() {
                        "eml" => match args {
                            [a, b] => {
                                // LIFO: Combine, then log, then exp — so `exp`
                                // is lowered first (SPEC-0001 §2.4's order).
                                tasks.push(Task::Combine);
                                tasks.push(Task::Visit(b));
                                tasks.push(Task::Visit(a));
                            }
                            _ => {
                                return Err(LowerError::Arity {
                                    form: "eml".to_string(),
                                    expected: 2,
                                    got: args.len(),
                                })
                            }
                        },
                        other => return Err(LowerError::UnknownForm(other.to_string())),
                    }
                }
            },
            Task::Combine => {
                // Each visited subtree nets exactly one `Eml`, and LIFO puts
                // both children before their Combine.
                let (Some(log_arg), Some(exp_arg)) = (out.pop(), out.pop()) else {
                    unreachable!("lower stack underflow: each Combine follows two Visits")
                };
                out.push(Eml::node(exp_arg, log_arg));
            }
        }
    }

    let Some(root) = out.pop() else {
        unreachable!("lower produced no value: the root Visit always pushes one")
    };
    Ok(root)
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
    // **Iterative** (R-0017): the same post-order machine as `lower`, so the
    // `Eml → Sexpr` leg of the codec is depth-safe too — a deep
    // `Eml → raise → Display → read` cycle would otherwise overflow here.
    enum Task<'a> {
        Visit(&'a Eml),
        Combine,
    }
    let mut tasks = vec![Task::Visit(e)];
    let mut out: Vec<Sexpr> = Vec::new();
    while let Some(task) = tasks.pop() {
        match task {
            Task::Visit(node) => match node {
                Eml::One => out.push(Sexpr::num(1.0)),
                Eml::Var(name) => out.push(Sexpr::sym(name.as_str())),
                Eml::Node { exp_arg, log_arg } => {
                    tasks.push(Task::Combine);
                    tasks.push(Task::Visit(log_arg));
                    tasks.push(Task::Visit(exp_arg));
                }
            },
            Task::Combine => {
                let (Some(log_s), Some(exp_s)) = (out.pop(), out.pop()) else {
                    unreachable!("raise stack underflow: each Combine follows two Visits")
                };
                out.push(Sexpr::list([Sexpr::sym("eml"), exp_s, log_s]));
            }
        }
    }
    let Some(root) = out.pop() else {
        unreachable!("raise produced no value: the root Visit always pushes one")
    };
    root
}
