//! The EML expression tree.
//!
//! See SPEC-0001 §2.2 and §3.

/// A UFL EML expression: a binary tree over the grammar
/// `S → 1 | <var> | eml(S, S)`.
///
/// The enum admits exactly the grammar — R-0001 AC1 holds structurally.
#[derive(Debug, Clone, PartialEq)]
pub enum Eml {
    /// The literal `1` — the sole numeric constant terminal.
    One,
    /// A free variable terminal, identified by name.
    Var(String),
    /// The binary `eml` node — `exp(exp_arg) − ln(log_arg)`.
    Node {
        exp_arg: Box<Eml>,
        log_arg: Box<Eml>,
    },
}

impl Eml {
    /// The literal `1`.
    pub fn one() -> Self {
        Eml::One
    }

    /// A free variable terminal.
    pub fn var(name: impl Into<String>) -> Self {
        Eml::Var(name.into())
    }

    /// The binary `eml(exp_arg, log_arg)` node.
    pub fn node(exp_arg: Eml, log_arg: Eml) -> Self {
        Eml::Node {
            exp_arg: Box::new(exp_arg),
            log_arg: Box::new(log_arg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R-0001 AC1 — `Eml::one()` is the literal `1` leaf and only the literal
    /// `1` leaf. Together with `ac1_var_constructor` and `ac1_node_constructor`
    /// and the closed enum, this enforces the grammar
    /// `S → 1 | <var> | eml(S, S)` structurally at the type boundary.
    #[test]
    fn ac1_one_constructor() {
        assert_eq!(Eml::one(), Eml::One);
    }

    /// R-0001 AC1 — `Eml::var(name)` constructs a named-variable leaf, no
    /// other variant.
    #[test]
    fn ac1_var_constructor() {
        assert_eq!(Eml::var("z"), Eml::Var("z".into()));
    }

    /// R-0001 AC1 — `Eml::node(a, b)` constructs the binary `eml` node with
    /// the `exp_arg` / `log_arg` asymmetry preserved.
    #[test]
    fn ac1_node_constructor() {
        let n = Eml::node(Eml::one(), Eml::var("x"));
        match n {
            Eml::Node { exp_arg, log_arg } => {
                assert_eq!(*exp_arg, Eml::One);
                assert_eq!(*log_arg, Eml::Var("x".into()));
            }
            other => panic!("expected Eml::Node, got {other:?}"),
        }
    }
}
