//! R-0016 AC4 — `raise` closes the code↔data square: acceptance tests.
//!
//! `raise : &Eml → Sexpr` is the inverse of `lower` (SPEC-0016 §2.5). It is
//! **total** on `Eml`, but `raise ∘ lower = id` holds only on the **reader's
//! canonical image**: `lower` accepts any `Sexpr::Sym(name)` → `Eml::Var(name)`,
//! including names the reader never produces (e.g. `Sym("1")`, which the reader
//! re-reads as `Num(1.0)`). So the property is stated on that domain, and the
//! generator draws `Sym` payloads only from the reader-canonical token set (via
//! the predicate `read` itself exposes, so generator and reader cannot drift).
//!
//! Authored at loop step 3 (test plan), **before** `raise` exists — RED until
//! `ufl-syntax` gains `raise` and `is_reader_canonical_sym`.
//!
//! See `specs/0016-reflection-quote-eval-raise.md` §2.5, §2.6 test 5.

use ufl_prng::SplitMix64;
use ufl_syntax::{is_reader_canonical_sym, lower, raise, read, Sexpr};

/// A short pool of reader-canonical symbol tokens: non-empty, not a finite-float
/// spelling, and free of `(`/`)`/`;`/whitespace. Each is asserted canonical (so
/// the pool cannot silently rot), keeping the generator honest.
fn canonical_sym_pool() -> Vec<&'static str> {
    let pool = vec![
        "x", "y", "eml", "foo", "a-b", "π", "x'", "inf", "nan", "+", "v1",
    ];
    for tok in &pool {
        assert!(
            is_reader_canonical_sym(tok),
            "pool token {tok:?} must be reader-canonical (generator/reader drift guard)"
        );
    }
    pool
}

/// Generate a `Sexpr` in the reader's canonical image at bounded depth: leaves
/// are `1` (the sole lowerable literal) or a reader-canonical symbol; nodes are
/// `(eml a b)`. This is exactly the lowerable class, so `lower` never errors.
fn gen_reader_image(rng: &mut SplitMix64, depth: u32, pool: &[&'static str]) -> Sexpr {
    let leaf = depth == 0 || rng.below(2) == 0;
    if leaf {
        if rng.below(2) == 0 {
            Sexpr::num(1.0)
        } else {
            let idx = rng.below(pool.len() as u64) as usize;
            Sexpr::sym(pool[idx])
        }
    } else {
        Sexpr::list([
            Sexpr::sym("eml"),
            gen_reader_image(rng, depth - 1, pool),
            gen_reader_image(rng, depth - 1, pool),
        ])
    }
}

/// AC4 — `read(Display(raise(lower(s)))) == read(Display(s))` for every
/// reader-canonical `s` at bounded depth. Equivalently `raise ∘ lower = id` on
/// that domain, observed through the pinned read/print round-trip (SPEC-0016
/// §2.5). Depth is a fixed constant (independent of the global depth contract,
/// task T6) — the round-trip is proven up to that bound.
#[test]
fn raise_lower_round_trips_on_reader_image() {
    let mut rng = SplitMix64::new(0x0016_A15E);
    let pool = canonical_sym_pool();
    for _ in 0..5_000 {
        let s = gen_reader_image(&mut rng, 5, &pool);
        let round_tripped = raise(&lower(&s).expect("reader-image Sexpr must lower"));
        assert_eq!(
            read(&round_tripped.to_string()),
            read(&s.to_string()),
            "raise ∘ lower must be the identity on the reader image; s = {s}"
        );
        // On this domain the round trip is in fact the *literal* identity:
        // `raise(lower(s)) == s` structurally (the reader-image restriction
        // makes the codec exact, not merely read-equal).
        assert_eq!(
            round_tripped, s,
            "on the reader image, raise ∘ lower is the structural identity; s = {s}"
        );
    }
}

/// AC4 (negative) — the explicit `Sym("1")` case that motivates the domain
/// restriction. `lower(Sym("1")) = Var("1")`, `raise(Var("1")) = Sym("1")`, and
/// `Display → "1"`, but `read("1") = Num(1.0)` — so `raise ∘ lower` is NOT the
/// identity here: `Sym("1")` is outside the reader's canonical image (a
/// finite-float spelling reads back as `Num`). This is why AC4 restricts the
/// domain (SPEC-0016 §2.5).
#[test]
fn raise_lower_is_not_identity_off_the_reader_image() {
    let off_image = Sexpr::sym("1");
    // It is indeed off the reader image (the guard the generator relies on).
    assert!(
        !is_reader_canonical_sym("1"),
        "`1` must be classified NON-canonical (it re-reads as Num)"
    );

    let lowered = lower(&off_image).expect("a bare symbol lowers to a Var");
    let raised = raise(&lowered);
    // `raise` faithfully transposes to `Sym("1")` …
    assert_eq!(raised, Sexpr::sym("1"));
    // … but `read(Display(raise(lower(Sym("1")))))` is `Num(1.0)`, not `Sym("1")`
    // — the round trip fails precisely because the input was off the image.
    assert_eq!(read(&raised.to_string()), Ok(Sexpr::num(1.0)));
    assert_ne!(read(&raised.to_string()), Ok(off_image));
}

/// AC4 — `raise` is total on `Eml` (no `Result`) and inverts each of `lower`'s
/// three table rows: `One → 1`, `Var → sym`, `Node → (eml …)`. Spelled out on
/// the canonical `(eml 1 x)` form so a table-row regression is caught directly.
#[test]
fn raise_inverts_each_lower_row() {
    use ufl_core::Eml;
    assert_eq!(raise(&Eml::one()), Sexpr::num(1.0));
    assert_eq!(raise(&Eml::var("x")), Sexpr::sym("x"));
    assert_eq!(
        raise(&Eml::node(Eml::one(), Eml::var("x"))),
        Sexpr::list([Sexpr::sym("eml"), Sexpr::num(1.0), Sexpr::sym("x")])
    );
}
