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
