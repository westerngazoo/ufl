# SPEC-0016 — Reflection rung 1: quote, eval, raise

- **Status:** Draft (revised after the three-lens review, 2026-07-03; pending
  R-0016 acceptance — see the decision log in §6)
- **Realizes:** [R-0016](../requirements/0016-reflection-quote-eval-raise.md)
- **Author:** main session
- **Created:** 2026-07-02 · **Revised:** 2026-07-03
- **Depends on:** SPEC-0003 (Sexpr core), SPEC-0004 (predicate layer). *Coordinates
  with* the depth-contract work (docs/tasks T6) — but does **not** block on it; the
  round-trip invariant `raise`/`Display` must jointly satisfy is inlined in §2.5.
- **Crate(s):** `ufl-syntax` (`raise`), `ufl-predicate` (`quote`/`eval`/`eq?`)

## 1. Motivation

Every UFL evaluator today is one-way and terminal: `text → Sexpr → Eml → Value`
with `Value = Complex<f64>` and `eval_pred → bool`. No value holds *syntax*, so
UFL code cannot be UFL data — and nothing above Rung 1 of the staircase
(`theory/two-language-substrate.md`) can exist. The carrier is in place: `Sexpr`
is `Clone + PartialEq` data with a reader **and** a printer whose round-trip is
tested; `eval_pred.rs` has the `Mode`/`classify` seam. What is missing is the
landing type and the forms — realizes R-0016.

## 2. Design

### 2.1 The third mode, on the existing classifier

`Mode` gains a `Syntax` variant beside `Numeric`/`Boolean`
([eval_pred.rs:27](../crates/ufl-predicate/src/eval_pred.rs)). `classify` today
routes *every* non-pred-head list to `Numeric` via a catch-all
([eval_pred.rs:43-46](../crates/ufl-predicate/src/eval_pred.rs)) — so `(quote e)`
would silently be `Numeric` and die in `lower` as `UnknownForm`. We add an
`is_syntax_head` arm **before** the catch-all:

```rust
fn is_syntax_head(h: &str) -> bool { h == "quote" }

fn classify(s: &Sexpr) -> Mode {
    match s {
        Sexpr::Sym(t) if t == "true" || t == "false" => Mode::Boolean,
        Sexpr::Sym(_) | Sexpr::Num(_) => Mode::Numeric,
        Sexpr::List(items) => match items.split_first() {
            Some((Sexpr::Sym(h), _)) if is_pred_head(h)   => Mode::Boolean,
            Some((Sexpr::Sym(h), _)) if is_syntax_head(h) => Mode::Syntax,  // NEW, ordered here
            _ => Mode::Numeric,
        },
    }
}
```

Positive rule: **`quote` is the only Syntax head; `eval` is a Numeric head**
(handled by the numeric form-dispatch of §2.3, not by `classify`, since `(eval q)`
denotes a `Value`). `is_pred_head` is untouched.

### 2.2 `(quote e)` — code as value

A third evaluator, small by construction (quote is the only syntax-*producing*
form in Rung 1):

```rust
/// Evaluate an `Sexpr` in syntax position to the `Sexpr` it denotes.
fn eval_syntax(s: &Sexpr, _env: &Env) -> Result<Sexpr, PredError> {
    match s {
        Sexpr::List(items) => match items.split_first() {
            Some((Sexpr::Sym(h), [e])) if h == "quote" => Ok(e.clone()), // unevaluated
            Some((Sexpr::Sym(h), args)) if h == "quote" => Err(arity("quote", 1, args.len())),
            _ => Err(PredError::ExpectedSyntax { found: describe(s) }),
        },
        _ => Err(PredError::ExpectedSyntax { found: describe(s) }),
    }
}
```

`(quote e)` does **not** evaluate `e`: `(quote (eml y 1))` with `y` unbound yields
the `Sexpr`, never `UnboundVariable` (R-0016 AC2). The child is not lowered, so a
non-numeric child (a bare symbol, a nested `(quote …)`, a future form) is legal
**as data** — `eval_syntax` returns it verbatim.

### 2.3 `(eval q)` — value from code, through the ONE pipeline (the seam, made concrete)

`(eval q)` is Numeric. Today `eval_num` is three lines with **no** form dispatch
([eval_pred.rs:127-133](../crates/ufl-predicate/src/eval_pred.rs)): a boolean
guard, then `lower` + `ufl_core::eval`. This spec **introduces a numeric
form-dispatch** in front of that fallthrough — the seam is specified here, not
assumed:

```rust
fn eval_num(s: &Sexpr, env: &Env) -> Result<Value, PredError> {
    // (unchanged) boolean-shaped operand is a type error
    if classify(s) == Mode::Boolean {
        return Err(PredError::ExpectedNumber { found: describe(s) });
    }
    // NEW — numeric form-dispatch on head, BEFORE lower. One arm today: `eval`.
    if let Sexpr::List(items) = s {
        if let Some((Sexpr::Sym(head), args)) = items.split_first() {
            if head == "eval" {
                return match args {
                    [q] => {
                        let quoted = eval_syntax(q, env)?;      // q must be a quote
                        let eml = ufl_syntax::lower(&quoted)?;  // the SAME lowering
                        Ok(ufl_core::eval(&eml, env)?)          // the SAME evaluator
                    }
                    _ => Err(arity("eval", 1, args.len())),
                };
            }
        }
    }
    // (unchanged) fallthrough — BYTE-IDENTICAL to today's eval_num
    let eml = ufl_syntax::lower(s)?;
    Ok(ufl_core::eval(&eml, env)?)
}
```

No second evaluator: `(eval q)` reuses `lower` + `ufl_core::eval` (R-0003 AC4
discipline). The non-`eval` fallthrough is unchanged, so every existing numeric
result is preserved.

**Consequence (R-0016 AC1), stated precisely:** `⟦(= (eval (quote E)) E)⟧` holds
**conditionally on `E ∈ dom(eval)`** — i.e. `E` is lowerable (`lower` accepts only
`1`, bare vars, and `eml`-headed forms; [lower.rs:50-60](../crates/ufl-syntax/src/lower.rs))
**and** evaluates without error under `env` (a `var` unbound on both sides errors
on both sides; `=` then never compares). `eval ∘ quote` is the identity **on the
numeric image of the reader, restricted to `dom(eval)`** — not an unconditional
total identity (the reflection layer inherits the core's partiality honestly).

### 2.4 `(eq? a b)` — structural equality on syntax (numeric `=` untouched)

**Decision (three-lens resolution, §6):** structural equality on quoted syntax is
a **distinct form `(eq? a b)`**, *not* an overload of `=`. Numeric `=`
([eval_pred.rs:90-93](../crates/ufl-predicate/src/eval_pred.rs)) is left **exactly
as SPEC-0004 defines it** — it keeps delegating to `eval_num`, keeps its
`ExpectedNumber` guard, and no existing SPEC-0004 `=` test changes verdict. This
removes the private-`Mode`-in-public-error problem, the silent change to numeric
`=`, and the classifier ambiguity that would become unsound once Rung 2 adds
syntax-typed bindings.

`eq?` is a new **Boolean** head (`is_pred_head` gains `"eq?"`), evaluated in
`eval_form`:

```rust
"eq?" => match args {
    [a, b] => Ok(eval_syntax(a, env)? == eval_syntax(b, env)?),
    _ => Err(arity("eq?", 2, args.len())),
},
```

`eq?` compares the **denoted `Sexpr`s** (the quote children), via the exact,
decidable `Sexpr::PartialEq`: `(eq? (quote a) (quote a))` is `true`. Both operands
must be syntax (each is an `eval_syntax`, i.e. a `quote`); a non-syntax operand is
`ExpectedSyntax`. No mixed-mode case exists — the type is closed by the form.

### 2.5 `raise : &Eml → Sexpr` — closing the square, with the domain stated

The inverse of `lower`, in `ufl-syntax`:

```rust
/// The transpose of `lower`'s table. Total on `Eml`.
pub fn raise(e: &Eml) -> Sexpr {
    match e {
        Eml::One => Sexpr::num(1.0),
        Eml::Var(name) => Sexpr::sym(name),
        Eml::Node { exp_arg, log_arg } =>
            Sexpr::list([Sexpr::sym("eml"), raise(exp_arg), raise(log_arg)]),
    }
}
```

`raise` is **total** on `Eml` (no `Result`) — every `Eml` has a structural image.
But **`raise ∘ lower = id` is NOT unconditional** (three-lens finding, §6):
`lower` accepts *any* `Sexpr::Sym(name)` → `Eml::Var(name)`, including names the
reader does not produce. Counterexample: `lower(Sym("1")) = Var("1")`,
`raise(Var("1")) = Sym("1")`, `Display → "1"`, but `read("1") = Num(1.0)` — the
reader classifies any finite-float spelling as `Num`, and delimiters/whitespace as
structure. So the honest, testable invariant is:

> **AC4 (precise):** for every `Sexpr s` in the **reader's canonical image** —
> every `Sym` payload is a non-empty token that is **not** a finite-float spelling
> and contains **no delimiter or whitespace** ([read.rs:62,77-82](../crates/ufl-syntax/src/read.rs)) —
> `read(Display(raise(lower(s)))) == read(Display(s))`. Equivalently
> `raise ∘ lower = id` on that domain.

The AC1/AC4 property generators draw `Sym` payloads from the reader-canonical
token set only, and **§2.6 says so**. The depth question: `raise` and `Display`
recurse; on a machine-built `Eml` deeper than a printable bound they diverge. The
AC4 generator is therefore **bounded to a fixed depth** (a constant in the test,
independent of the still-unwritten global depth contract) — the round-trip is
proven up to that bound, and the depth-contract task (T6) later lifts the bound
process-wide. AC4 does **not** block on T6; it scopes its generator.

### 2.6 Tests (TDD — written first, red)

`crates/ufl-syntax/tests/` (`raise`) and `crates/ufl-predicate/tests/`:

1. **`eval_quote_is_identity_on_dom_eval`** (AC1): for generated `E` drawn from
   `{1, var(bound), eml(E,E)}` with `var`s bound in `env`, `eval_pred((= (eval
   (quote E)) E), env) == Ok(true)`. (Generator restricted to lowerable,
   error-free `E` per §2.3.)
2. **`quote_does_not_evaluate`** (AC2): `(eval_syntax (quote (eml y 1)))` with `y`
   unbound returns the `Sexpr` — no `UnboundVariable`; and `(quote e)` in numeric
   position fails typed (`UnknownForm("quote")` via `lower`, never a coercion).
3. **`eq_on_syntax_is_structural`** (AC3): `(eq? (quote (eml 1 1)) (quote (eml 1
   1)))` is `Ok(true)`; differing forms `Ok(false)`; a non-syntax operand is
   `ExpectedSyntax`. Numeric `=` unaffected: a pinned SPEC-0004 `=` suite still
   passes verbatim.
4. **`nested_eval_through_one_pipeline`**: `(eval (quote (eml 1 1)))` discharges;
   `(eval (quote (eval (quote 1))))` — nested eval — discharges to the same value
   the direct form gives (pins the single-pipeline reuse).
5. **`raise_lower_round_trips_on_reader_image`** (AC4): for `Sexpr`s generated with
   reader-canonical `Sym` tokens and bounded depth,
   `read(Display(raise(lower(s)))) == read(Display(s))`; plus the explicit
   negative `Sym("1")` case showing why the domain restriction exists.

## 3. Code outline

Files: `crates/ufl-syntax/src/lower.rs` (+`raise`, re-exported), `.../read.rs`
(the reader-canonical predicate reused by the AC4 generator),
`crates/ufl-predicate/src/eval_pred.rs` (`Mode::Syntax`, `is_syntax_head`, the
`classify` arm, `eval_syntax`, the numeric form-dispatch with the `eval` arm, the
`eq?` arm, `PredError::ExpectedSyntax`). **No `MixedEquality`, no private type in a
public error, no new crate, no new control forms.**

## 4. Non-goals

- **No `Value → Sexpr` reification** — code-as-value only (never total; `inf`/`nan`
  are legitimate results outside the reader's image; R-0016 scoping decision 2).
- **No in-language eval** (an `eval` *written in UFL*) — Rung 2, future requirement.
- **No apostrophe reader-macro** — `quote` is a named form (`'` is Hehner priming).
- **No overload of numeric `=`** — structural equality is `eq?` (§2.4).
- **No head/arg accessors on syntax** beyond `eq?` unless a later AC forces them.

## 5. Open questions

1. Whether `eval_num` should reject `Mode::Syntax` with an explicit
   `ExpectedNumber` (clearer) or let `lower` reject `quote`/`eval`-in-quote as
   `UnknownForm` (already typed). Provisionally the latter; confirm in review.
2. The exact reader-canonical `Sym` predicate the AC4 generator shares with the
   reader — factor it out of `read.rs` so generator and reader cannot drift.

## 6. Decision log — three-lens resolutions (2026-07-03)

| Finding (lens) | Resolution |
|---|---|
| `(eval q)` seam does not exist in `eval_num` (architect [blocking]) | §2.3 now specifies the numeric form-dispatch concretely; non-`eval` fallthrough byte-identical. |
| `MixedEquality { Mode }` won't compile — private-in-public (architect [blocking]) | Removed entirely — `eq?` (§2.4) makes mixed-mode impossible; no `Mode` in any public error. |
| `raise ∘ lower = id` false on `Sym("1")` etc. (hater [blocking]) | §2.5 restricts AC4's domain to reader-canonical `Sym` tokens; the `Sym("1")` negative is a required test. |
| The "T6 depth contract" dependency has no id (hater [blocking]) | Downgraded to a *coordination* note; the round-trip invariant is inlined (§2.5) and AC4's generator is depth-bounded, so SPEC-0016 does not block on T6. |
| numeric `=` silently changed / classifier Rung-2-unsound (architect [major] A4, hater [major] 4) | Resolved by the `eq?` decision (§2.4) — numeric `=` untouched; **Gustavo-confirmed, 2026-07-03**. |
| `eval∘quote=id` over-claimed as total (hater [major] 3) | §2.3 states it conditionally on `E ∈ dom(eval)`; generator restricted. |
| `classify` diff + meta-sentence (architect [major] 3) | §2.1 shows the diff; the "is not correct" aside replaced by the positive rule. |
| structural-`=` semantics unstated; nested-eval/quote-of-quote untested (architect [minor] 5, hater [minor] 5) | §2.4 states `eq?` compares denoted `Sexpr`s; §2.6 tests 3–4 added. |
| `raise` domain imprecision (architect [minor] 6) | Folded into the §2.5 precise AC4. |

R-0016 AC3 is amended in lockstep (`=` on syntax → `eq?`). Gustavo holds final
approval before Draft→Accepted (§1.2).
