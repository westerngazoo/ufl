//! The `Sexpr` type — UFL's single homoiconic syntax tree (SPEC-0003 §2.2).

/// A UFL S-expression — the one syntax tree. Code and data share this
/// representation (R-0003 AC1). It is *general data*: it can hold any finite
/// number and any symbol. The restriction to R-0001's grammar is the lowering
/// pass's job (`crate::lower`), not this type's.
#[derive(Debug, Clone, PartialEq)]
pub enum Sexpr {
    /// A numeric literal. The reader only ever produces *finite* values
    /// (SPEC-0003 §2.3); complex values are *derived*, never literal.
    Num(f64),
    /// A symbol — an operator/form name or a variable.
    Sym(String),
    /// An application / list: `(head arg ...)`.
    List(Vec<Sexpr>),
}

impl Sexpr {
    /// A numeric-literal atom.
    pub fn num(n: f64) -> Self {
        Sexpr::Num(n)
    }

    /// A symbol atom.
    pub fn sym(name: impl Into<String>) -> Self {
        Sexpr::Sym(name.into())
    }

    /// A list `(items...)`.
    pub fn list(items: impl Into<Vec<Sexpr>>) -> Self {
        Sexpr::List(items.into())
    }
}

impl std::fmt::Display for Sexpr {
    /// Render an `Sexpr` as text. For every `s` the reader produces,
    /// `read(s.to_string()) == Ok(s)` (the round-trip invariant, scoped to the
    /// reader's image — SPEC-0003 §2.2).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sexpr::Num(n) => write!(f, "{n}"),
            Sexpr::Sym(s) => write!(f, "{s}"),
            Sexpr::List(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, ")")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Structural unit tests for the `Sexpr` type itself (SPEC-0003 §2.2).
    //!
    //! These touch only the constructors, `Clone`/`PartialEq`, and `Display` —
    //! none of `read`/`lower`/`eval_str`. They are therefore **green now**
    //! (independent of the pending reader/lowering implementation) and pin the
    //! data structure + its textual rendering, which is the round-trip oracle
    //! the AC1 reader test (in `tests/r_0003_acceptance.rs`) relies on.

    use super::Sexpr;

    /// AC1 — there is one syntax tree type with three variants, built by three
    /// constructors. `num`/`sym`/`list` produce `Num`/`Sym`/`List` verbatim.
    #[test]
    fn ac1_constructors_produce_each_variant() {
        assert_eq!(Sexpr::num(1.0), Sexpr::Num(1.0));
        assert_eq!(Sexpr::sym("eml"), Sexpr::Sym("eml".to_string()));
        assert_eq!(
            Sexpr::list([Sexpr::num(1.0), Sexpr::sym("x")]),
            Sexpr::List(vec![Sexpr::Num(1.0), Sexpr::Sym("x".to_string())])
        );
    }

    /// AC1 — `Sexpr` is `Clone + PartialEq`: a program is ordinary, comparable,
    /// duplicable data (code is data).
    #[test]
    fn ac1_is_clone_and_partial_eq() {
        let s = Sexpr::list([Sexpr::sym("eml"), Sexpr::num(1.0), Sexpr::sym("x")]);
        assert_eq!(s.clone(), s);
    }

    /// AC1 — exhaustively matching `Sexpr` requires exactly the three
    /// homoiconic variants. If a fourth is ever added, this stops compiling and
    /// the one-tree guarantee must be re-examined.
    #[test]
    fn ac1_three_variants_are_exhaustive() {
        fn classify(s: &Sexpr) -> &'static str {
            match s {
                Sexpr::Num(_) => "num",
                Sexpr::Sym(_) => "sym",
                Sexpr::List(_) => "list",
            }
        }
        assert_eq!(classify(&Sexpr::num(1.0)), "num");
        assert_eq!(classify(&Sexpr::sym("x")), "sym");
        assert_eq!(classify(&Sexpr::list([])), "list");
    }

    /// AC1 — `Display` renders the atoms and the canonical `(eml 1 1)` form.
    /// This pins the round-trip oracle: `Display` must emit text the reader
    /// re-reads to the same value (asserted against `read` in the e2e suite).
    #[test]
    fn ac1_display_renders_atoms_and_lists() {
        assert_eq!(Sexpr::num(1.0).to_string(), "1");
        assert_eq!(Sexpr::sym("eml").to_string(), "eml");
        assert_eq!(
            Sexpr::list([Sexpr::sym("eml"), Sexpr::num(1.0), Sexpr::num(1.0)]).to_string(),
            "(eml 1 1)"
        );
    }

    /// AC1 — `Display` of a nested form matches the docs' `ln(x)` notation
    /// exactly: `(eml 1 (eml (eml 1 x) 1))`. One space between siblings, no
    /// trailing space, parens around every list.
    #[test]
    fn ac1_display_renders_nested_form() {
        let ln_x = Sexpr::list([
            Sexpr::sym("eml"),
            Sexpr::num(1.0),
            Sexpr::list([
                Sexpr::sym("eml"),
                Sexpr::list([Sexpr::sym("eml"), Sexpr::num(1.0), Sexpr::sym("x")]),
                Sexpr::num(1.0),
            ]),
        ]);
        assert_eq!(ln_x.to_string(), "(eml 1 (eml (eml 1 x) 1))");
    }

    /// AC1 — the empty list is valid data and renders as `()` (it is rejected
    /// only later, at lowering — see AC3). Pins that `()` is representable.
    #[test]
    fn ac1_display_renders_empty_list() {
        assert_eq!(Sexpr::list([]).to_string(), "()");
    }
}
