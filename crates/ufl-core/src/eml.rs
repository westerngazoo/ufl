//! The EML expression tree.
//!
//! See SPEC-0001 §2.2 and §3.

/// A UFL EML expression: a binary tree over the grammar
/// `S → 1 | <var> | eml(S, S)`.
///
/// The enum admits exactly the grammar — R-0001 AC1 holds structurally.
#[derive(Debug)]
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

/// **Iterative** deep clone (R-0017): a two-stack post-order rebuild, so a tree
/// of any depth clones without recursing. `Eml` is heap-recursive, and `eq?` /
/// `eval_syntax` clone quoted subtrees, so a derived (recursive) `Clone` would
/// abort on the deep machine-generated ASTs the reflection loop makes normal.
impl Clone for Eml {
    fn clone(&self) -> Self {
        enum Task<'a> {
            Visit(&'a Eml),
            Rebuild,
        }
        let mut tasks = vec![Task::Visit(self)];
        let mut out: Vec<Eml> = Vec::new();
        while let Some(t) = tasks.pop() {
            match t {
                Task::Visit(e) => match e {
                    Eml::One => out.push(Eml::One),
                    Eml::Var(n) => out.push(Eml::Var(n.clone())),
                    Eml::Node { exp_arg, log_arg } => {
                        tasks.push(Task::Rebuild);
                        tasks.push(Task::Visit(log_arg));
                        tasks.push(Task::Visit(exp_arg));
                    }
                },
                Task::Rebuild => {
                    // Same net-one-value argument as `eval`: a Rebuild always
                    // follows its two children's Visits.
                    let (Some(log_arg), Some(exp_arg)) = (out.pop(), out.pop()) else {
                        unreachable!("clone stack underflow: each Rebuild follows two Visits")
                    };
                    out.push(Eml::node(exp_arg, log_arg));
                }
            }
        }
        let Some(root) = out.pop() else {
            unreachable!("clone produced no value: the root Visit always pushes one")
        };
        root
    }
}

/// **Iterative** structural equality (R-0017): a lockstep walk over an explicit
/// pair-stack, short-circuiting on the first difference. Same verdict as the
/// derived `PartialEq`, without the recursion (`eq?` compares `Sexpr`/`Eml`
/// with `==` on quoted — hence potentially deep — trees).
impl PartialEq for Eml {
    fn eq(&self, other: &Self) -> bool {
        let mut pairs = vec![(self, other)];
        while let Some((a, b)) = pairs.pop() {
            match (a, b) {
                (Eml::One, Eml::One) => {}
                (Eml::Var(x), Eml::Var(y)) if x == y => {}
                (
                    Eml::Node {
                        exp_arg: ae,
                        log_arg: al,
                    },
                    Eml::Node {
                        exp_arg: be,
                        log_arg: bl,
                    },
                ) => {
                    pairs.push((al, bl));
                    pairs.push((ae, be));
                }
                _ => return false,
            }
        }
        true
    }
}

impl Drop for Eml {
    fn drop(&mut self) {
        // Prevent stack overflow on deep trees by dropping iteratively.
        let mut stack = Vec::new();
        if let Eml::Node { exp_arg, log_arg } = self {
            let exp = std::mem::replace(&mut **exp_arg, Eml::One);
            let log = std::mem::replace(&mut **log_arg, Eml::One);
            stack.push(exp);
            stack.push(log);
        }

        while let Some(mut node) = stack.pop() {
            if let Eml::Node { exp_arg, log_arg } = &mut node {
                let exp = std::mem::replace(&mut **exp_arg, Eml::One);
                let log = std::mem::replace(&mut **log_arg, Eml::One);
                stack.push(exp);
                stack.push(log);
            }
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
            Eml::Node {
                ref exp_arg,
                ref log_arg,
            } => {
                assert_eq!(**exp_arg, Eml::One);
                assert_eq!(**log_arg, Eml::Var("x".into()));
            }
            ref other => panic!("expected Eml::Node, got {other:?}"),
        }
    }
}
