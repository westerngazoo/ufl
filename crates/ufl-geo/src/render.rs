//! Translate a `GeoExpr` back into human geometric-algebra notation (#27).
//!
//! The "translate-back" leg of UFL's thesis: a discovered `GeoExpr` *is* the math
//! (homoiconic), so this renders it in conventional GA notation — `R v ~R` for the
//! versor sandwich, `exp(…)`, `e₁₂`, `⟨·⟩_k`, `𝒢_k(·)`. **Sandwich versors that
//! are not atoms are bound to names** (`let R = …`) so a deeply-nested versor stays
//! bounded: a naive `R v ~R` expansion prints the versor twice per level and is
//! exponential (a pilot run produced an 11 MB string).

use crate::expr::GeoExpr;

/// Render a `GeoExpr` as conventional geometric-algebra notation.
///
/// A versor sandwich whose rotor is a non-atom is emitted as a `let`-bound name
/// (`let R = exp(…)`) followed by `R v ~R`, keeping nested versors bounded in
/// length; product chains are parenthesised to stay unambiguous.
pub fn render(e: &GeoExpr) -> String {
    let mut ctx = Ctx::default();
    let body = node(e, &mut ctx);
    let mut out = String::new();
    for (name, def) in &ctx.lets {
        out.push_str("let ");
        out.push_str(name);
        out.push_str(" = ");
        out.push_str(def);
        out.push('\n');
    }
    out.push_str(&body);
    out
}

/// Accumulates `let`-bound rotor definitions (in dependency order) and hands out
/// fresh names.
#[derive(Default)]
struct Ctx {
    lets: Vec<(String, String)>,
    next: usize,
}

impl Ctx {
    fn fresh(&mut self) -> String {
        const NAMES: [&str; 6] = ["R", "S", "T", "U", "V", "W"];
        let name = NAMES
            .get(self.next)
            .map(|s| (*s).to_string())
            .unwrap_or_else(|| format!("R{}", self.next + 1));
        self.next += 1;
        name
    }
}

/// A leaf — never needs parentheses as a factor.
fn is_atom(e: &GeoExpr) -> bool {
    matches!(e, GeoExpr::Param(_) | GeoExpr::Basis(_) | GeoExpr::Var(_))
}

/// Render `e` as a *factor* in a product / reverse context: parenthesise it
/// unless it is an atom or a self-delimiting functional form (`exp`, `⟨⟩`, `𝒢`).
fn factor(e: &GeoExpr, ctx: &mut Ctx) -> String {
    let s = node(e, ctx);
    let self_delimiting = matches!(
        e,
        GeoExpr::Exp(_) | GeoExpr::GradeProject(..) | GeoExpr::GradeLift(..)
    );
    if is_atom(e) || self_delimiting {
        s
    } else {
        format!("({s})")
    }
}

fn node(e: &GeoExpr, ctx: &mut Ctx) -> String {
    match e {
        GeoExpr::Param(x) => fmt_param(*x),
        GeoExpr::Var(name) => name.clone(),
        GeoExpr::Basis(i) => blade_name(*i),
        GeoExpr::GeoProduct(a, b) => format!("{} {}", factor(a, ctx), factor(b, ctx)),
        GeoExpr::Wedge(a, b) => format!("{}∧{}", factor(a, ctx), factor(b, ctx)),
        GeoExpr::Inner(a, b) => format!("{}·{}", factor(a, ctx), factor(b, ctx)),
        GeoExpr::Reverse(a) => format!("~{}", factor(a, ctx)),
        GeoExpr::Exp(a) => format!("exp({})", node(a, ctx)),
        GeoExpr::GradeProject(k, a) => format!("⟨{}⟩_{k}", node(a, ctx)),
        GeoExpr::GradeLift(k, a) => format!("𝒢_{k}({})", node(a, ctx)),
        GeoExpr::Sandwich(rotor, x) => {
            // Bind a non-atom rotor to a name so nesting stays bounded
            // (rendering it once into a `let`, not twice inline).
            let rotor_str = if is_atom(rotor) {
                node(rotor, ctx)
            } else {
                let def = node(rotor, ctx);
                let name = ctx.fresh();
                ctx.lets.push((name.clone(), def));
                name
            };
            format!("{rotor_str} {} ~{rotor_str}", factor(x, ctx))
        }
    }
}

/// The blade's name (garust `Cl(3,0,1)` convention: `e₀` is the null generator,
/// bit 3). Subscripts ascend `0 < 1 < 2 < 3`, so e.g. blade 9 (`e₀ ∧ e₁`) is `e₀₁`.
fn blade_name(i: u8) -> String {
    if i == 0 {
        return "1".to_string();
    }
    if i >= 16 {
        return format!("e?{i}");
    }
    const GENERATORS: [(u8, char); 4] = [(8, '₀'), (1, '₁'), (2, '₂'), (4, '₃')];
    let mut name = String::from("e");
    for (bit, sub) in GENERATORS {
        if i & bit != 0 {
            name.push(sub);
        }
    }
    name
}

/// A parameter to 3 significant figures (trailing zeros trimmed).
fn fmt_param(x: f64) -> String {
    if x == 0.0 {
        return "0".to_string();
    }
    let magnitude = x.abs().log10().floor() as i32;
    let decimals = (2 - magnitude).clamp(0, 12) as usize;
    let s = format!("{x:.decimals$}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}
