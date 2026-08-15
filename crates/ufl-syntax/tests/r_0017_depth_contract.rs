//! R-0017 / SPEC-0017 — the one iterative depth contract.
//!
//! Every tree-walk on the `Sexpr`/`Eml` code↔data surface uses an explicit heap
//! work-stack: **no cap, no magic constant, no library-code abort at any depth**.
//!
//! Deep fixtures are built **iteratively** (a recursive builder would overflow
//! before the code under test ran) and run at the acceptance depth of 10⁵ inside
//! the **subprocess arena** (see [`arena`]).

use ufl_core::{eval, Eml, Env};
use ufl_syntax::{lower, raise, read, Sexpr};

/// The acceptance depth (AC1/AC3/AC4b): 10⁵, two orders past the old 128 cap.
const DEPTH: usize = 100_000;

/// Run the deep body of `case`, or return `false` in the parent — which has
/// already spawned the child and asserted its exit status.
///
/// A stack overflow is an `abort()`, **not** a catchable panic: a child *thread*
/// cannot `join()` it, and on the main thread it takes the whole test binary
/// down with it. So each deep case runs in a re-exec of this very test binary
/// and the pass signal is the child's **exit status** — a recursive regression
/// exits non-zero (SIGABRT) and fails *this* assertion, by name, without
/// deleting the evidence from its sibling tests.
///
/// The arena is sound only in a **debug** build: `--release` TCOs a
/// tail-recursive walk, so such a regression exits 0 at any depth. (Measured on
/// the pre-R-0017 recursive `eval_pred`: a `(pred …)` spine overflows at 10⁵ in
/// debug but returns `Ok` at 3·10⁶ in release — SPEC-0017 §2.9.) A release run
/// therefore *declines* rather than reporting a green it cannot justify.
fn arena(case: &str) -> bool {
    // Child mode: this process **is** the arena — run the body.
    if std::env::var("UFL_DEPTH_ARENA").as_deref() == Ok(case) {
        return true;
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
    // libtest exits 0 when its filter matches *nothing*, so a clean exit alone
    // would silently false-pass if the fn name and the case string ever drifted.
    assert!(
        stdout.contains("1 passed"),
        "arena[{case}] ran no test — `arena_{case}` and the case name have drifted apart:\n{stdout}"
    );
    false
}

/// Build a `(a (a (a … a)))` deep `Sexpr` iteratively. Every level is a
/// **two-element** list, so `Display`'s separator frame is exercised at depth
/// (a single-element nesting would never emit one).
fn deep_sexpr(depth: usize) -> Sexpr {
    let mut s = Sexpr::sym("a");
    for _ in 0..depth {
        s = Sexpr::list([Sexpr::sym("a"), s]);
    }
    s
}

/// Build `(eml (eml … 1) 1)`-style left-nested depth iteratively. Distinct from
/// [`deep_sexpr`]: only an `eml` spine lowers, so the `lower → eval` leg needs
/// its own fixture.
fn deep_eml(depth: usize) -> Eml {
    let mut e = Eml::one();
    for _ in 0..depth {
        e = Eml::node(e, Eml::one());
    }
    e
}

/// **AC2 — the codec is symmetric at every depth.** The old cap made `read`
/// reject at 129 what `Display` happily emitted; sweeping across and far past it
/// proves the asymmetry is gone.
#[test]
fn the_codec_is_symmetric_across_and_past_the_old_cap() {
    for depth in [1usize, 100, 128, 129, 200, 1_000, 10_000] {
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

/// **AC1 — the round trip at 10⁵**, compared as `String`: tree `==` is itself
/// one of the walks under test, so the codec check must not lean on it.
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "the depth arena is sound only in a debug build: --release TCOs tail recursion and would false-pass"
)]
fn arena_deep_round_trip_read_display() {
    if !arena("deep_round_trip_read_display") {
        return;
    }
    let printed = deep_sexpr(DEPTH).to_string();
    let again = read(&printed).expect("no cap, no overflow");
    assert!(
        again.to_string() == printed,
        "display∘read∘display must be stable at depth {DEPTH}"
    );
}

/// **AC1 — `raise → read → lower → eval` on a deep `(eml …)` spine** returns a
/// value with no overflow and no panic.
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "the depth arena is sound only in a debug build: --release TCOs tail recursion and would false-pass"
)]
fn arena_deep_eml_spine_lowers_and_evaluates() {
    if !arena("deep_eml_spine_lowers_and_evaluates") {
        return;
    }
    let text = raise(&deep_eml(DEPTH)).to_string();
    let parsed = read(&text).expect("deep read");
    let lowered = lower(&parsed).expect("deep lower");
    let v = eval(&lowered, &Env::new()).expect("deep eval");
    // The spine iterates `exp`, so it saturates to +∞ — a *value*, not a crash.
    assert!(!v.re.is_nan(), "a value, not a crash (got re = {})", v.re);
}

/// **AC4b — `Clone` and `PartialEq` at depth.** Both were *derived*, hence
/// recursive, and both sit on the reflection path.
///
/// Deep trees are compared with `assert!(a == b, …)`, never `assert_eq!`:
/// `Debug` is still derived and recursive (SPEC-0017 §2.7), so formatting a
/// failure message would itself overflow — reporting the exact abort the test
/// exists to distinguish.
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "the depth arena is sound only in a debug build: --release TCOs tail recursion and would false-pass"
)]
fn arena_deep_clone_and_structural_eq_do_not_abort() {
    if !arena("deep_clone_and_structural_eq_do_not_abort") {
        return;
    }
    let s = deep_sexpr(DEPTH);
    let copy = s.clone();
    assert!(
        copy == s,
        "iterative Clone + PartialEq agree at depth {DEPTH}"
    );

    let e = deep_eml(DEPTH);
    let e2 = e.clone();
    assert!(e2 == e, "same for Eml at depth {DEPTH}");

    // Inequality must also decide without recursing.
    let other = deep_sexpr(DEPTH - 1);
    assert!(other != s, "differing depth compares unequal, iteratively");
}

/// **`raise` is on the same codec** — the `Eml → Sexpr` leg. A deep
/// `Eml → raise → Display → read → lower` cycle used to overflow *at `raise`*.
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "the depth arena is sound only in a debug build: --release TCOs tail recursion and would false-pass"
)]
fn arena_deep_raise_closes_the_code_data_square() {
    if !arena("deep_raise_closes_the_code_data_square") {
        return;
    }
    let e = deep_eml(DEPTH);
    let s = raise(&e);
    let back = lower(&s).expect("lower ∘ raise is the identity on Eml's image");
    assert!(back == e, "raise ∘ lower = id, at depth {DEPTH}");
}

/// **AC3 (regression guard)** — the `Drop`s were already iterative (PR #40);
/// this pins them so a future refactor cannot silently re-recurse.
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "the depth arena is sound only in a debug build: --release TCOs tail recursion and would false-pass"
)]
fn arena_deep_trees_drop_without_overflow() {
    if !arena("deep_trees_drop_without_overflow") {
        return;
    }
    drop(deep_sexpr(DEPTH));
    drop(deep_eml(DEPTH));
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
/// *bigger-constant*. Walks every crate's `src/` **and `tests/`**: the stale
/// R-0003 cap test lived in `tests/`, invisible to a src-only check.
#[test]
fn no_depth_cap_machinery_remains() {
    let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/");
    let mut offenders = Vec::new();
    let mut crate_count = 0usize;
    for entry in std::fs::read_dir(crates)
        .expect("crates/ is readable")
        .flatten()
    {
        if !entry.path().is_dir() {
            continue;
        }
        crate_count += 1;
        for sub in ["src", "tests"] {
            collect(&entry.path().join(sub), &mut offenders);
        }
    }
    assert!(
        crate_count >= 10,
        "the sweep must cover every crate, saw only {crate_count}"
    );
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
                for needle in [
                    "get_max_depth",
                    "set_max_depth",
                    "MAX_DEPTH",
                    "RecursionDepthExceeded",
                ] {
                    // This file names them to forbid them; skip itself.
                    if text.contains(needle) && !path.ends_with("r_0017_depth_contract.rs") {
                        out.push(format!("{}: {needle}", path.display()));
                    }
                }
            }
        }
    }
}
