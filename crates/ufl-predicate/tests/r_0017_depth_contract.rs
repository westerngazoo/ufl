//! R-0017 / SPEC-0017 §2.6, §2.9 — the boolean spine and the reflection path.
//!
//! Two things live here that the `ufl-syntax` half of this suite cannot reach:
//!
//! - **T-shortcircuit** — `and`/`or` are now a resumable continuation machine
//!   rather than a recursive `all`/`any`, so their **laziness** is a property of
//!   the frame protocol instead of the language's. It is pinned here, including
//!   the *nested* case, where the machine must resume a parent frame with no
//!   "last boolean" of its own yet.
//! - **T-reflection-deep (AC4b)** — the two forms the requirement names as
//!   aborting *in library code* today: `(eq? (quote DEEP) (quote DEEP))` (which
//!   clones and structurally compares a quoted tree) and `(eval (quote DEEP))`
//!   (which clones, lowers, then evaluates one). Exercising `Clone`/`PartialEq`
//!   directly is not the same test — the abort was reached *through* reflection.

use ufl_core::Env;
use ufl_predicate::{eval_pred, PredError};
use ufl_syntax::{raise, Sexpr};

/// The acceptance depth (AC4b): 10⁵.
const DEPTH: usize = 100_000;

/// Run the deep body of `case`, or return `false` in the parent — which has
/// already spawned the child and asserted its exit status.
///
/// A stack overflow is an `abort()`, **not** a catchable panic, so the deep
/// cases run in a re-exec of this test binary and the pass signal is the child's
/// **exit status**.
///
/// The arena is sound only in a **debug** build, and this crate is where that
/// matters most. Measured against the pre-R-0017 *recursive* `eval_pred`
/// (`git show main:…`), whose `(pred e)` arm is a genuine tail call, on a
/// `(pred (pred … true))` spine:
///
/// | profile | 10⁵ | 3·10⁶ |
/// |---------|-----|-------|
/// | debug   | overflows (caught) | — |
/// | release | `Ok(true)` | `Ok(true)` |
///
/// Release TCOs it to constant stack, so a release arena false-passes at **any**
/// depth. Hence the decline below rather than a reassuring green (SPEC-0017
/// §2.9).
///
/// (Deliberately duplicated from the `ufl-syntax` suite: two integration-test
/// crates cannot share code without a crate existing solely to hold twenty
/// lines of process spawning.)
fn arena(case: &str) -> bool {
    if std::env::var("UFL_DEPTH_ARENA").as_deref() == Ok(case) {
        return true; // child mode: this process **is** the arena
    }
    // Normally unreachable: the `cfg_attr(ignore)` on each arena test means a
    // release run reports them as **ignored, with the reason**, rather than a
    // green that hides a skip. This still guards `--release -- --ignored`.
    if !cfg!(debug_assertions) {
        eprintln!("arena[{case}]: declined — only a debug build can detect a recursive regression");
        return false;
    }
    let exe = std::env::current_exe().expect("the running test binary has a path");
    let out = std::process::Command::new(exe)
        .arg(format!("arena_{case}"))
        .args(["--exact", "--nocapture", "--test-threads=1"])
        .env("UFL_DEPTH_ARENA", case)
        .output()
        .expect("spawn the arena subprocess");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "arena[{case}] died ({}) — a SIGABRT here means a walk went recursive again\
         \n--- child stdout ---\n{stdout}\n--- child stderr ---\n{}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout.contains("1 passed"),
        "arena[{case}] ran no test — `arena_{case}` and the case name have drifted apart:\n{stdout}"
    );
    false
}

/// A predicate that **errors if it is ever evaluated**: `zzz` is unbound, so
/// reaching it yields `PredError::Eval(UnboundVariable)`. Every short-circuit
/// assertion below is only meaningful because [`bad_really_is_bad`] proves this.
fn bad() -> Sexpr {
    Sexpr::list([Sexpr::sym("="), Sexpr::num(1.0), Sexpr::sym("zzz")])
}

fn sym(t: &str) -> Sexpr {
    Sexpr::sym(t)
}

/// Nest `head` around `leaf` `depth` times, iteratively — a recursive builder
/// would overflow before the code under test ran.
fn spine(head: &str, depth: usize, leaf: Sexpr) -> Sexpr {
    let mut s = leaf;
    for _ in 0..depth {
        s = Sexpr::list([sym(head), s]);
    }
    s
}

/// Build a `(a (a … a))` deep `Sexpr` iteratively — the tree that gets quoted.
fn deep_sexpr(depth: usize) -> Sexpr {
    let mut s = sym("a");
    for _ in 0..depth {
        s = Sexpr::list([sym("a"), s]);
    }
    s
}

/// The load-bearing precondition for every short-circuit test: the operand that
/// must *not* be reached does in fact fail when it is.
#[test]
fn bad_really_is_bad() {
    assert!(matches!(
        eval_pred(&bad(), &Env::new()),
        Err(PredError::Eval(_))
    ));
}

/// **T-shortcircuit (§2.6)** — `and`/`or` never evaluate an operand past the
/// decision, so an unreached error is still never surfaced.
#[test]
fn and_or_short_circuit_including_the_nested_case() {
    let env = Env::new();
    let f = || sym("false");
    let t = || sym("true");

    assert_eq!(
        eval_pred(&Sexpr::list([sym("and"), f(), bad()]), &env),
        Ok(false)
    );
    assert_eq!(
        eval_pred(&Sexpr::list([sym("or"), t(), bad()]), &env),
        Ok(true)
    );

    // The nested case: the outer `and` resumes on a value produced by an inner
    // `or` frame — the machine must *pop* that value, not peek at a stale one,
    // and must not have consulted any "last boolean" when it first entered.
    let inner_or = Sexpr::list([sym("or"), f(), f()]);
    assert_eq!(
        eval_pred(&Sexpr::list([sym("and"), inner_or, bad()]), &env),
        Ok(false)
    );

    // Two frame kinds interleaved, both resuming from below.
    let inner_and = Sexpr::list([sym("and"), t(), f()]);
    let not_true = Sexpr::list([sym("not"), t()]);
    assert_eq!(
        eval_pred(&Sexpr::list([sym("or"), inner_and, not_true]), &env),
        Ok(false)
    );

    // The empty base cases the machine must produce without ever popping.
    assert_eq!(eval_pred(&Sexpr::list([sym("and")]), &env), Ok(true));
    assert_eq!(eval_pred(&Sexpr::list([sym("or")]), &env), Ok(false));
}

/// **AC4b — `(eq? (quote DEEP) (quote DEEP))`.** `eq?` clones each quoted child
/// (`eval_syntax`) and compares the two with `==`; both were derived, hence
/// recursive, so this form aborted **in library code**.
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "the depth arena is sound only in a debug build: --release TCOs tail recursion and would false-pass"
)]
fn arena_deep_eq_on_quoted_trees() {
    if !arena("deep_eq_on_quoted_trees") {
        return;
    }
    let quoted = || Sexpr::list([sym("quote"), deep_sexpr(DEPTH)]);
    let form = Sexpr::list([sym("eq?"), quoted(), quoted()]);
    assert_eq!(eval_pred(&form, &Env::new()), Ok(true));

    // Inequality must decide iteratively too.
    let other = Sexpr::list([sym("quote"), deep_sexpr(DEPTH - 1)]);
    let differ = Sexpr::list([sym("eq?"), quoted(), other]);
    assert_eq!(eval_pred(&differ, &Env::new()), Ok(false));
}

/// **AC4b — `(eval (quote DEEP))`.** Discharges a deep quoted tree through the
/// one pipeline: clone (`eval_syntax`) → `lower` → `ufl_core::eval`. Three
/// formerly-recursive walks in a single form.
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "the depth arena is sound only in a debug build: --release TCOs tail recursion and would false-pass"
)]
fn arena_deep_eval_of_a_quoted_spine() {
    if !arena("deep_eval_of_a_quoted_spine") {
        return;
    }
    // Only an `eml` spine lowers, so the quoted tree is built via `raise`.
    let mut e = ufl_core::Eml::one();
    for _ in 0..DEPTH {
        e = ufl_core::Eml::node(e, ufl_core::Eml::one());
    }
    let quoted = Sexpr::list([sym("quote"), raise(&e)]);
    let form = Sexpr::list([
        sym("="),
        Sexpr::list([sym("eval"), quoted]),
        Sexpr::num(1.0),
    ]);
    // The spine saturates to +∞, so the comparison is `false` — the claim under
    // test is that it *completes* rather than aborting in library code.
    assert_eq!(eval_pred(&form, &Env::new()), Ok(false));
}

/// **`(pred e)` at depth — the tail-call trap.** `(pred e)` is not a leaf, and a
/// recursive implementation of it is a *tail call*: measured, that survives
/// 3·10⁶ under `--release` and dies at 10⁵ in debug (see [`arena`]). It is a
/// tail **launch** here — push the operand, no resume frame — so the frame stack
/// does not grow at all.
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "the depth arena is sound only in a debug build: --release TCOs tail recursion and would false-pass"
)]
fn arena_deep_pred_spine() {
    if !arena("deep_pred_spine") {
        return;
    }
    let form = spine("pred", DEPTH, sym("true"));
    assert_eq!(eval_pred(&form, &Env::new()), Ok(true));
}

/// `not` at depth — an even count returns the leaf unchanged.
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "the depth arena is sound only in a debug build: --release TCOs tail recursion and would false-pass"
)]
fn arena_deep_not_spine() {
    if !arena("deep_not_spine") {
        return;
    }
    let form = spine("not", DEPTH, sym("true"));
    assert_eq!(eval_pred(&form, &Env::new()), Ok(true));
}

/// A left-nested `and` spine at depth: 10⁵ `And` frames outstanding at once.
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "the depth arena is sound only in a debug build: --release TCOs tail recursion and would false-pass"
)]
fn arena_deep_and_spine() {
    if !arena("deep_and_spine") {
        return;
    }
    let form = spine("and", DEPTH, sym("true"));
    assert_eq!(eval_pred(&form, &Env::new()), Ok(true));
}

/// The `or` counterpart — a distinct frame kind with its own resume arm.
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "the depth arena is sound only in a debug build: --release TCOs tail recursion and would false-pass"
)]
fn arena_deep_or_spine() {
    if !arena("deep_or_spine") {
        return;
    }
    let form = spine("or", DEPTH, sym("false"));
    assert_eq!(eval_pred(&form, &Env::new()), Ok(false));
}
