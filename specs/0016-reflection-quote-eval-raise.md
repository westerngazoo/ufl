# SPEC-0016 — Reflection rung 1: quote, eval, raise

- **Status:** Draft (pending R-0016 acceptance + the three-lens review; no code
  before Accepted — constitution §4.2)
- **Realizes:** [R-0016](../requirements/0016-reflection-quote-eval-raise.md)
- **Author:** main session
- **Created:** 2026-07-02
- **Depends on:** SPEC-0003 (Sexpr core), SPEC-0004 (predicate layer), the
  unified depth contract (task T6 — `quote` must not land on an asymmetric codec)
- **Crate(s):** `ufl-syntax` (`raise`), `ufl-predicate` (`quote`/`eval`/`=`)

## 1. Motivation

Every UFL evaluator today is one-way and terminal: `text → Sexpr → Eml → Value`
with `Value = Complex<f64>` and `eval_pred → bool`. No value holds *syntax*, so
UFL code cannot be UFL data — and nothing above Rung 1 of the staircase can
exist (`theory/two-language-substrate.md`). The carrier is already in place:
`Sexpr` is `Clone + PartialEq` data with a reader **and** a printer whose
round-trip is documented and tested; `eval_pred.rs` already has the `Mode` +
`classify` seam, and `lower.rs`/`eval_form` are documented extension points.
What is missing is the landing type and two named forms — realizes R-0016.

## 2. Design

### 2.1 The third mode

`Mode` gains a `Syntax` variant beside `Numeric` and `Boolean`
([eval_pred.rs:27](../crates/ufl-predicate/src/eval_pred.rs)); `classify` routes:

| Form | Mode | Why |
|------|------|-----|
| `(quote e)` | **Syntax** | produces the child `Sexpr`, unevaluated |
| `(eval q)`  | **Numeric** | runs `q`'s syntax through the existing pipeline → `Value` |
| `(= a b)`   | **Boolean** | structural iff both operands are Syntax; else numeric (§2.4) |

`is_pred_head` gains `"eval"` on the numeric side is **not** correct — `eval` is
numeric, so it is handled in `lower`/`eval_num`, not `is_pred_head`. `quote` is
added to a new `is_syntax_head`.

### 2.2 `(quote e)` — code as value

A third evaluator, small by construction (quote is the only syntax-*producing*
form in Rung 1):

```rust
/// Evaluate an `Sexpr` in syntax position to the `Sexpr` it denotes.
fn eval_syntax(s: &Sexpr, _env: &Env) -> Result<Sexpr, PredError> {
    match s {
        Sexpr::List(items) => match items.split_first() {
            Some((Sexpr::Sym(h), [e])) if h == "quote" => Ok(e.clone()), // unevaluated
            Some((Sexpr::Sym(h), args)) if h == "quote" =>
                Err(arity("quote", 1, args.len())),
            _ => Err(PredError::ExpectedSyntax { found: describe(s) }),
        },
        _ => Err(PredError::ExpectedSyntax { found: describe(s) }),
    }
}
```

`(quote e)` does **not** evaluate `e`: `(quote (eml y 1))` with `y` unbound
yields the `Sexpr`, never `UnboundVariable` (R-0016 AC2). The child is not
lowered, so a non-numeric child (e.g. a bare symbol, or a future form) is legal
*as data*.

### 2.3 `(eval q)` — value from code, through the ONE pipeline

`(eval q)` is numeric; it is handled in the numeric evaluator (which today lowers
+ calls `ufl_core::eval`). It evaluates `q` to a syntax value, then **reuses**
`lower` + `ufl_core::eval` — never a second evaluator (R-0003 AC4 discipline):

```rust
// in eval_num's form dispatch:
"eval" => match args {
    [q] => {
        let quoted = eval_syntax(q, env)?;      // q must be syntax (a quote)
        let eml = ufl_syntax::lower(&quoted)?;   // the SAME lowering
        Ok(ufl_core::eval(&eml, env)?)           // the SAME verified evaluator
    }
    _ => Err(arity("eval", 1, args.len())),
},
```

Consequence (R-0016 AC1): `⟦(= (eval (quote E)) E)⟧` holds for every bound
numeric `E` in the reader's image — `eval ∘ quote` is identity on the numeric
image, because both sides reach `E` through the same lower+eval.

### 2.4 Structural `=` on syntax

The `=` arm classifies its two operands. Both Syntax → `Sexpr::PartialEq`
(exact, decidable); both Numeric → the existing numeric equality; **mixed → a
typed `PredError`** (no silent coercion):

```rust
"=" => match args {
    [a, b] => match (classify(a), classify(b)) {
        (Mode::Syntax, Mode::Syntax) => Ok(eval_syntax(a, env)? == eval_syntax(b, env)?),
        (Mode::Numeric, Mode::Numeric) => Ok(eval_num(a, env)? == eval_num(b, env)?),
        (ma, mb) => Err(PredError::MixedEquality { left: ma, right: mb }),
    },
    _ => Err(arity("=", 2, args.len())),
},
```

Numeric `=` stays numeric-only for numeric operands — R-0016 AC3.

### 2.5 `raise : &Eml → Sexpr` — closing the square

The missing inverse of `lower`, in `ufl-syntax`:

```rust
/// The inverse of [`lower`] on the reader's image: `raise ∘ lower = id`.
pub fn raise(e: &Eml) -> Sexpr {
    match e {
        Eml::One => Sexpr::num(1.0),                    // lower maps Num(1.0) → One
        Eml::Var(name) => Sexpr::sym(name),
        Eml::Node { exp_arg, log_arg } =>
            Sexpr::list([Sexpr::sym("eml"), raise(exp_arg), raise(log_arg)]),
    }
}
```

`raise` emits only reader-image `Sexpr`s for reader-image inputs (no `Num(inf)`,
no `Sym` with spaces), so `print(raise(e))` re-reads. **Depends on T6**: `raise`
and the printer must share the one depth contract, or a machine-deep `Eml`
raises to a `Sexpr` that `Display` cannot print symmetrically.

## 3. Code outline

Files touched: `crates/ufl-syntax/src/lower.rs` (+`raise`, re-exported),
`crates/ufl-predicate/src/eval_pred.rs` (`Mode::Syntax`, `eval_syntax`, the
`quote`/`eval`/`=` arms, `PredError::{ExpectedSyntax, MixedEquality}`). No new
crate; no new control forms.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode { Numeric, Boolean, Syntax }   // + Syntax
```

## 4. Non-goals

- **No `Value → Sexpr` reification** — code-as-value only; reification can never
  be total (`inf`/`nan` are legitimate eval results outside the reader's image;
  R-0016 scoping decision 2).
- **No in-language eval** (an `eval` *written in UFL*) — that is Rung 2, a
  separate future requirement; `(eval q)` here is a host form over the host
  pipeline.
- **No apostrophe reader-macro** — `quote` is a named form; `'` is Hehner
  priming (SPEC-0004 §2.5).
- **No head/arg accessors on syntax** beyond `=`, unless AC1–AC4 force them
  (deferred; a spec decision if the property tests need destructuring).

## 5. Open questions

1. The `=` mixed-mode case: hard error (§2.4) vs. deferring `=`-on-syntax to a
   distinct form (e.g. `(eq? a b)`) so numeric `=` stays syntactically pure. The
   three-lens (hater) should stress the operand-classification ambiguity —
   `classify` currently can't see through a variable bound to a quoted value
   (there are no such bindings in Rung 1, but Rung 2 will introduce them).
2. Whether `eval_syntax` should accept a variable that *names* a quoted form
   (needs a syntax-typed binding in `Env`) — out of scope for Rung 1 (no
   `Value` slot holds an `Sexpr`), but the boundary should be named so Rung 2
   inherits it cleanly.
3. `raise` placement: `ufl-syntax` (beside `lower`) vs. a new `reflect` module —
   provisionally beside `lower`, since it *is* `lower`'s inverse.
