use crate::GeoExpr;
use std::fmt;

/// Maps a blade index `0..16` to conventional GA notation.
/// 0 -> 1
/// bits: 1 -> e1, 2 -> e2, 4 -> e3, 8 -> e0
/// e.g. 15 -> e0123
fn render_blade(idx: u8) -> String {
    if idx == 0 {
        return "1".to_string();
    }

    let mut out = String::from("e");
    // To match notation order we generally want e1, e2, e3, e0 or similar.
    // Based on issue 27:
    // 3->e12, 7->e123, 9->e01, 15->e0123.
    // 9 is 8+1 -> 8 (e0) and 1 (e1). So order is 0, 1, 2, 3.
    // 8 -> e0
    if idx & 8 != 0 {
        out.push('₀');
    }
    if idx & 1 != 0 {
        out.push('₁');
    }
    if idx & 2 != 0 {
        out.push('₂');
    }
    if idx & 4 != 0 {
        out.push('₃');
    }

    out
}

#[derive(Default)]
struct RenderCtx {
    bindings: Vec<(String, String)>,
    counter: usize,
}

impl RenderCtx {
    fn fresh_name(&mut self) -> String {
        self.counter += 1;
        if self.counter == 1 {
            "R".to_string()
        } else {
            format!("R_{}", self.counter - 1)
        }
    }

    fn render_node(&mut self, e: &GeoExpr) -> String {
        match e {
            GeoExpr::Param(v) => format!("{}", v),
            GeoExpr::Basis(idx) => render_blade(*idx),
            GeoExpr::Var(name) => name.clone(),
            GeoExpr::GradeLift(k, a) => format!("𝒢_{}({})", k, self.render_node(a)),
            GeoExpr::GeoProduct(a, b) => {
                let lhs = self.render_node(a);
                let rhs = self.render_node(b);
                format!("({} {})", lhs, rhs)
            }
            GeoExpr::Wedge(a, b) => {
                let lhs = self.render_node(a);
                let rhs = self.render_node(b);
                format!("({} ∧ {})", lhs, rhs)
            }
            GeoExpr::Inner(a, b) => {
                let lhs = self.render_node(a);
                let rhs = self.render_node(b);
                format!("({} · {})", lhs, rhs)
            }
            GeoExpr::Reverse(a) => format!("~{}", self.render_node(a)),
            GeoExpr::GradeProject(k, a) => format!("⟨{}⟩_{}", self.render_node(a), k),
            GeoExpr::Sandwich(a, b) => {
                let a_str = self.render_node(a);
                let b_str = self.render_node(b);

                // Introduce let-binding for `a`
                let name = self.fresh_name();
                self.bindings.push((name.clone(), a_str));

                format!("{} {} ~{}", name, b_str, name)
            }
            GeoExpr::Exp(a) => format!("exp({})", self.render_node(a)),
        }
    }
}

/// Renders a `GeoExpr` to GA notation, extracting repeated versors in Sandwiches
/// to `let R = ...;` statements to prevent exponential blow-up.
pub fn render(e: &GeoExpr) -> String {
    let mut ctx = RenderCtx::default();
    let root = ctx.render_node(e);

    if ctx.bindings.is_empty() {
        return root;
    }

    let mut out = String::new();
    for (name, val) in ctx.bindings {
        out.push_str(&format!("let {} = {}; ", name, val));
    }
    out.push_str(&root);
    out
}

impl fmt::Display for GeoExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", render(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blade_names() {
        assert_eq!(render_blade(0), "1");
        assert_eq!(render_blade(1), "e₁");
        assert_eq!(render_blade(2), "e₂");
        assert_eq!(render_blade(4), "e₃");
        assert_eq!(render_blade(8), "e₀");
        assert_eq!(render_blade(3), "e₁₂");
        assert_eq!(render_blade(7), "e₁₂₃");
        assert_eq!(render_blade(9), "e₀₁");
        assert_eq!(render_blade(15), "e₀₁₂₃");
    }

    #[test]
    fn test_keystone_sandwich() {
        use std::f64::consts::TAU;
        let e = GeoExpr::Sandwich(
            Box::new(GeoExpr::Exp(Box::new(GeoExpr::GeoProduct(
                Box::new(GeoExpr::Param(-TAU / 8.0)),
                Box::new(GeoExpr::Basis(3)),
            )))),
            Box::new(GeoExpr::Var("v".to_string())),
        );
        let out = render(&e);
        let param_str = format!("{}", -TAU / 8.0);
        let expected = format!("let R = exp(({} e₁₂)); R v ~R", param_str);
        assert_eq!(out, expected);
    }

    #[test]
    fn test_nested_sandwich_blowup() {
        let mut e = GeoExpr::Var("v".to_string());
        for _ in 0..10 {
            e = GeoExpr::Sandwich(
                Box::new(GeoExpr::Exp(Box::new(GeoExpr::Basis(3)))),
                Box::new(e),
            );
        }

        let out = render(&e);
        assert!(out.len() < 1000, "String length should be bounded");

        assert!(out.contains("let R = exp(e₁₂);"));
        assert!(out.contains("let R_1 = exp(e₁₂);"));
        assert!(out.contains("let R_9 = exp(e₁₂);"));
        // print it instead of strict checking the exact string, the sandwich renders without parenthesis according to logic:
        // format!("{} {} ~{}", name, b_str, name)
        assert!(out.ends_with("R_9 R_8 R_7 R_6 R_5 R_4 R_3 R_2 R_1 R v ~R ~R_1 ~R_2 ~R_3 ~R_4 ~R_5 ~R_6 ~R_7 ~R_8 ~R_9"));
    }

    #[test]
    fn test_precedence_parentheses() {
        let e = GeoExpr::GeoProduct(
            Box::new(GeoExpr::Wedge(
                Box::new(GeoExpr::Basis(1)),
                Box::new(GeoExpr::Basis(2)),
            )),
            Box::new(GeoExpr::Inner(
                Box::new(GeoExpr::Basis(3)),
                Box::new(GeoExpr::Basis(4)),
            )),
        );
        let out = render(&e);
        assert_eq!(out, "((e₁ ∧ e₂) (e₁₂ · e₃))");
    }
}
