//! R-0017 / SPEC-0017 — the one iterative depth contract.
//!
//! Every tree-walk on the `Sexpr`/`Eml` code↔data surface uses an explicit heap
//! work-stack: **no cap, no magic constant, no library-code abort at any depth**.
//!
//! Deep fixtures are built **iteratively** (a recursive builder would overflow
//! before the code under test ran). The depths here are chosen to sit far above
//! the old 128 cap while staying inside the measured stack ceiling for a debug
//! test binary (~1–5k frames for a *recursive* walk) — so a regression that
//! reintroduces recursion fails loudly rather than passing quietly.

use ufl_core::{eval, Eml, Env};
use ufl_syntax::{lower, raise, read};

/// Build `(eml (eml … 1) 1)`-style left-nested depth iteratively.
fn deep_eml(depth: usize) -> Eml {
    let mut e = Eml::one();
    for _ in 0..depth {
        e = Eml::node(e, Eml::one());
    }
    e
}

/// Build a `(a (a (a …)))` deep `Sexpr` iteratively.
fn deep_sexpr(depth: usize) -> ufl_syntax::Sexpr {
    use ufl_syntax::Sexpr;
    let mut s = Sexpr::sym("a");
    for _ in 0..depth {
        s = Sexpr::list([s]);
    }
    s
}

/// **AC2 — the codec is symmetric at every depth.** The old cap made `read`
/// reject at 129 what `Display` happily emitted; sweeping across and far past it
/// proves the asymmetry is gone.
#[test]
fn the_codec_is_symmetric_across_and_past_the_old_cap() {
    for depth in [1usize, 100, 128, 129, 200, 1_000, 5_000] {
        let s = deep_sexpr(depth);
        let printed = s.to_string(); // would have PANICKED past the cap
        let reparsed = read(&printed).expect("read accepts exactly what Display emits");
        assert_eq!(
            reparsed.to_string(),
            printed,
            "round-trip must be byte-identical at depth {depth}"
        );
    }
}

/// **AC1 — the round trip at depth**, compared as `String` (tree `==` is itself
/// one of the walks under test, so the codec check must not lean on it).
#[test]
fn deep_round_trip_read_display() {
    let s = deep_sexpr(20_000);
    let printed = s.to_string();
    let again = read(&printed).expect("no cap, no overflow");
    assert_eq!(again.to_string(), printed, "display∘read∘display is stable");
}

/// **AC1 — `read → lower → eval` on a deep `(eml …)` spine** returns a value
/// with no overflow and no panic.
#[test]
fn deep_eml_spine_lowers_and_evaluates() {
    let spine = deep_eml(20_000);
    let text = raise(&spine).to_string();
    let parsed = read(&text).expect("deep read");
    let lowered = lower(&parsed).expect("deep lower");
    let v = eval(&lowered, &Env::new()).expect("deep eval");
    assert!(
        v.re.is_finite() || v.re.is_infinite(),
        "a value, not a crash"
    );
}

/// **AC4b — the reflection path.** `Clone` and `PartialEq` were *derived*, hence
/// recursive; `eq?` compares quoted `Sexpr`s with `==` and `eval_syntax` clones
/// them, so `(eq? (quote DEEP))` used to abort **in library code**. Both are now
/// iterative.
#[test]
fn deep_clone_and_structural_eq_do_not_abort() {
    let s = deep_sexpr(20_000);
    let copy = s.clone();
    assert_eq!(copy, s, "iterative Clone + PartialEq agree at depth");

    let e = deep_eml(20_000);
    let e2 = e.clone();
    assert_eq!(e2, e, "same for Eml");

    // Inequality must also short-circuit without recursing.
    let other = deep_sexpr(19_999);
    assert_ne!(other, s, "differing depth compares unequal, iteratively");
}

/// **`raise` is on the same codec** — the `Eml → Sexpr` leg. A deep
/// `Eml → raise → Display → read → lower` cycle used to overflow *at `raise`*.
#[test]
fn deep_raise_closes_the_code_data_square() {
    let e = deep_eml(20_000);
    let s = raise(&e);
    let back = lower(&s).expect("lower ∘ raise is the identity on Eml's image");
    assert_eq!(back, e, "raise ∘ lower = id, at depth");
}

/// **AC3 (regression guard)** — the `Drop`s were already iterative (PR #40);
/// this pins them so a future refactor cannot silently re-recurse.
#[test]
fn deep_trees_drop_without_overflow() {
    drop(deep_sexpr(50_000));
    drop(deep_eml(50_000));
}

/// The error behaviour `read` must preserve exactly — in particular the
/// single-top-level-datum rule (a complete datum followed by *anything* is
/// `TrailingTokens`, not `UnexpectedClose`).
#[test]
fn read_error_behaviour_is_unchanged() {
    use ufl_syntax::ReadError;
    assert!(matches!(read("(a))"), Err(ReadError::TrailingTokens)));
    assert!(matches!(read("a)"), Err(ReadError::TrailingTokens)));
    assert!(matches!(read("(a) b"), Err(ReadError::TrailingTokens)));
    assert!(matches!(read(")"), Err(ReadError::UnexpectedClose)));
    assert!(matches!(read("(a"), Err(ReadError::UnclosedList)));
}

/// **No cap survives anywhere** — the policy is *iterative*, not
/// *bigger-constant*. (The grep covers `tests/` too: the stale R-0003 cap test
/// lived there, invisible to a src-only check.)
#[test]
fn no_depth_cap_machinery_remains() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("repo root");
    let mut offenders = Vec::new();
    for crate_dir in ["ufl-core", "ufl-syntax", "ufl-predicate"] {
        for sub in ["src", "tests"] {
            let dir = root.join("crates").join(crate_dir).join(sub);
            collect(&dir, &mut offenders);
        }
    }
    assert!(
        offenders.is_empty(),
        "depth-cap machinery must be gone, found in: {offenders:?}"
    );

    fn collect(dir: &std::path::Path, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let Ok(text) = std::fs::read_to_string(&path) else {
                    continue;
                };
                for needle in ["get_max_depth", "set_max_depth", "RecursionDepthExceeded"] {
                    // This file names them to forbid them; skip itself.
                    if text.contains(needle) && !path.ends_with("r_0017_depth_contract.rs") {
                        out.push(format!("{}: {needle}", path.display()));
                    }
                }
            }
        }
    }
}
