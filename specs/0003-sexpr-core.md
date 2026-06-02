# SPEC-0003 — Homoiconic S-Expression Core

- **Status:** Accepted (2026-05-28 — three-lens review applied, architect APPROVE, hater/nice-guy findings addressed)
- **Realizes:** R-0003
- **Author:** Gustavo Delgadillo (Goose) — drafted with Claude
- **Created:** 2026-05-28
- **Depends on:** SPEC-0001 (the `Eml` typed core + `eval` the `eml` form lowers into)
- **Crate(s):** `ufl-syntax` (new) — depends on `ufl-core`

## 1. Motivation

SPEC-0003 realizes [R-0003](../requirements/0003-sexpr-core.md): UFL's
homoiconic S-expression surface and IR, as the **synthesis** — a front-end that
*lowers into* the retained typed core. It gives UFL its first real syntax: you
write `(eml 1 1)` as text and get `e`, through the pipeline

```
text ──read──▶ Sexpr ──lower──▶ Eml ──eval──▶ Value
```

The numerics are R-0001's, reused unchanged; SPEC-0003 adds the reader and the
lowering pass, and nothing about the verified `eml` evaluator changes.

## 2. Design

### 2.1 Crate

A new crate **`ufl-syntax`** holds the S-expression surface. It depends on
`ufl-core` (for `Eml`, `eval`, `Env`, `Value`, `EvalError`). Dependencies point
inward — syntax → core, never the reverse. Its `lib.rs` carries
`#![forbid(unsafe_code)]` (matching `ufl-core`), and the crate is registered in
the workspace `members` list. This is the crate the whole surface language
(more forms, later macros) grows in.

```
crates/ufl-syntax/
├── Cargo.toml          # depends on ufl-core
└── src/
    ├── lib.rs          # #![forbid(unsafe_code)]; re-exports; eval_str + UflError
    ├── sexpr.rs        # Sexpr type, constructors, Display (round-trippable)
    ├── read.rs         # reader: text → Sexpr; ReadError
    └── lower.rs        # lowering: Sexpr → Eml; LowerError
└── examples/
    └── hello_sexpr.rs  # runs the literal strings from the docs (see §2.6)
```

### 2.2 The `Sexpr` type — general homoiconic data

`Sexpr` is the one syntax tree. It is *general data* — it holds any number and
any symbol, because homoiconicity means code and data share one representation
(AC1). It is **not** restricted to R-0001's grammar; that restriction is the
lowering pass's job (§2.4), not the data structure's.

```rust
/// A UFL S-expression — the single homoiconic syntax tree (R-0003 AC1).
/// Code and data share this one representation.
#[derive(Debug, Clone, PartialEq)]
pub enum Sexpr {
    /// A numeric literal (real, finite; complex values are *derived*, never
    /// literal — see §2.3 for why the reader only produces finite `Num`s).
    Num(f64),
    /// A symbol — an operator/form name or a variable.
    Sym(String),
    /// An application / list: `(head arg ...)`.
    List(Vec<Sexpr>),
}
```

Constructors (`Sexpr::num`, `Sexpr::sym`, `Sexpr::list`) build trees as data,
and a `Display` impl renders them as text.

**The reader's image and the round-trip invariant.** `Sexpr` is general data, so
its public constructors can build values that have *no textual form* — e.g.
`Sexpr::sym("a b")` (a symbol with a space) or `Sexpr::num(f64::NAN)`. This is
intentional and unremarkable: not every datum is textually expressible, just as
not every Rust value implements `FromStr`. The round-trip invariant is therefore
scoped to the **reader's image** — the set of `Sexpr`s `read` can produce:

> For every `s` that `read` produces, `read(s.to_string()) == Ok(s)`.

This holds by construction (§2.3): the reader produces only finite `Num`s
(whose `Display` re-parses to the same finite value) and separator-free,
non-numeric `Sym`s (whose `Display` re-reads as the same symbol). It is the
property the AC1 round-trip test asserts, and it is well-defined precisely
because the tokenizer rule (§2.3) closes the reader's image.

**Equality.** `Sexpr` derives `PartialEq`; `Num` equality is IEEE `f64`
equality, so a *constructed* `Sexpr::num(f64::NAN)` is not equal to itself —
standard Rust `f64` behaviour. Reader-produced `Sexpr`s never contain `NaN`
(non-finite tokens are symbols, §2.3), so this affects neither the round-trip
invariant nor any reader-path test.

### 2.3 The reader — text → `Sexpr`

A standard S-expression reader: a tokenizer then a recursive parser. The
tokenizer rule is **normative** (it closes the reader's image, §2.2):

- **Delimiters:** `(`, `)`, whitespace (insignificant), and `;` (line comment
  to end of line — LISP convention).
- **Token:** a maximal run of characters containing none of the delimiters
  above.
- **Atom classification:** a token that parses as a **finite** `f64` becomes
  `Num`; *every other token* — including non-finite numeric spellings like
  `inf`, `nan`, `infinity` (which `f64::from_str` accepts) — becomes `Sym`.
  Excluding non-finite parses keeps `Num` always finite, which is what makes the
  round-trip invariant and `PartialEq` total on the reader's image.

So `1`, `2.5`, `-1`, `1e0` are numbers; `eml`, `x`, `+`, `inf`, `nan` are
symbols. A symbol therefore never contains a delimiter and never parses as a
finite number — the reader cannot produce `Sym("a b")` or `Sym("1")`.

- **Errors** — a typed `ReadError`, never a panic (AC2):

```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ReadError {
    #[error("unbalanced parentheses: unexpected ')'")]
    UnexpectedClose,
    #[error("unbalanced parentheses: unclosed '(' at end of input")]
    UnclosedList,
    #[error("empty input — no s-expression to read")]
    EmptyInput,
    #[error("unexpected trailing tokens after the first s-expression")]
    TrailingTokens,
}

pub fn read(src: &str) -> Result<Sexpr, ReadError>;
```

There is **no `InvalidToken` variant by construction**: every non-delimiter run
is a valid `Num` or `Sym`, so "stray tokens" (R-0003 AC2) can only manifest as
`TrailingTokens` (a second top-level form) or `UnexpectedClose` (a `)` with no
open list) — never a lexically invalid atom. `read` returns exactly one
top-level `Sexpr`; a `read_all` returning `Vec<Sexpr>` is a trivial follow-on.

### 2.4 The lowering pass — `Sexpr → Eml`

Lowering enforces **R-0001's grammar** (`S → 1 | var | eml(S, S)`), recovering
the typed core's structural guarantees at the lowering boundary (AC3). It is
total and side-effect-free, producing either an `Eml` or a typed `LowerError`.

| `Sexpr` | lowers to | notes |
|---------|-----------|-------|
| `Num(n)` where `n == 1.0` | `Eml::One` | the primitive is the *value* `1`, not the token (see below) |
| `Num(n)`, `n != 1.0` | `LowerError::UnsupportedLiteral(n)` | includes `-0.0` and every non-`1` finite real; only `1` is primitive in R-0001 |
| `Sym(s)` | `Eml::var(s)` | a variable (resolved at eval time) |
| `List([Sym("eml"), a, b])` | `Eml::node(lower a?, lower b?)` | the `eml` form |
| `List([Sym("eml"), ..])` wrong count | `LowerError::Arity { form: "eml", expected: 2, got }` | e.g. `(eml)` → `got: 0` |
| `List([Sym(other), ..])` | `LowerError::UnknownForm(other)` | only `eml` is known in R-0003 |
| `List` with non-symbol head, or empty `()` | `LowerError::NotAForm` | `()` is valid *data* but not a *form* |

**The `1` literal is a value, not a token.** `1`, `1.0`, `1.00`, `1e0`, `+1`
all parse to *exactly* the `f64` `1.0` (which is exactly representable), so the
`n == 1.0` test is total and trap-free, and any literal that rounds to exactly
`1.0` lowers to `Eml::One`. (Implementation note: the `f64` equality against the
`1.0` literal is the intended exact comparison; the `clippy::float_cmp` lint is
resolved in code by comparing against the named constant with a justifying
comment, not by weakening the check.)

```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum LowerError {
    #[error("unsupported numeric literal {0}: only `1` is primitive in this core")]
    UnsupportedLiteral(f64),
    #[error("unknown form: `{0}`")]
    UnknownForm(String),
    #[error("form `{form}` expects {expected} arguments, got {got}")]
    Arity { form: String, expected: usize, got: usize },
    #[error("not a form: a list must be a non-empty application with a symbol head")]
    NotAForm,
}

pub fn lower(s: &Sexpr) -> Result<Eml, LowerError>;
```

**Dispatch seam.** Lowering a list dispatches on the head symbol. With one form
(`eml`) this is a small `match`; a full *form table* (head → lowerer) is
deliberately deferred until there is more than one form (no premature
abstraction, CLAUDE.md §2). The dispatch function is structured so adding a form
is a localized edit — the documented seam where the future orchestrator/macro
layer will register rewrites (R-0003 §5).

### 2.5 The pipeline and the unified error

A convenience that runs the whole path, reusing `ufl_core::eval` verbatim:

```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum UflError {
    #[error(transparent)] Read(#[from] ReadError),
    #[error(transparent)] Lower(#[from] LowerError),
    #[error(transparent)] Eval(#[from] ufl_core::EvalError),
}

/// text → Sexpr → Eml → Value. The `env` supplies any free variables, so a
/// program with variables (e.g. `(eml x 1)`) requires the caller to bind them
/// — exactly as R-0001's `eval` does.
pub fn eval_str(src: &str, env: &Env) -> Result<Value, UflError> {
    let sexpr = read(src)?;
    let eml = lower(&sexpr)?;
    Ok(ufl_core::eval(&eml, env)?)
}
```

This is the seam that proves AC4 parity *through the typed core*: the `eml` the
lowering produces is fed to R-0001's existing evaluator, so the branch
convention and the `sin(τ/2)` self-correction are inherited, not re-derived.

### 2.6 The `hello_sexpr` example

`ufl-syntax` ships `examples/hello_sexpr.rs` that calls `eval_str` on the *exact
literal strings* the docs and the requirement already use — e.g.
`eval_str("(eml 1 (eml (eml 1 x) 1))", &env)` for `ln(x)`. The same string then
appears in `docs/why-ufl.md`, R-0003, this spec, the example, and the qa tests
— a four-way identity that *is* homoiconicity made visible, and the moment the
docs' notation becomes runnable syntax. It demonstrates AC1 + AC4 end to end.

## 3. Code outline

Representative — refined with the owner before implementation.

```rust
// sexpr.rs
pub enum Sexpr { Num(f64), Sym(String), List(Vec<Sexpr>) }

impl std::fmt::Display for Sexpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Sexpr::Num(n)  => write!(f, "{n}"),
            Sexpr::Sym(s)  => write!(f, "{s}"),
            Sexpr::List(xs) => {
                write!(f, "(")?;
                for (i, x) in xs.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "{x}")?;
                }
                write!(f, ")")
            }
        }
    }
}

// lower.rs
const ONE: f64 = 1.0;

pub fn lower(s: &Sexpr) -> Result<Eml, LowerError> {
    match s {
        // exact value-equality against the primitive `1` (1.0 is exactly
        // representable, so this is total and trap-free; see §2.4).
        Sexpr::Num(n) if *n == ONE => Ok(Eml::One),
        Sexpr::Num(n)              => Err(LowerError::UnsupportedLiteral(*n)),
        Sexpr::Sym(name)           => Ok(Eml::var(name)),
        Sexpr::List(items)         => lower_form(items),
    }
}

fn lower_form(items: &[Sexpr]) -> Result<Eml, LowerError> {
    // empty list and non-symbol head both fall through to NotAForm.
    let Some((Sexpr::Sym(head), args)) = items.split_first() else {
        return Err(LowerError::NotAForm);
    };
    match head.as_str() {
        // the dispatch seam — one arm per form
        "eml" => match args {
            [a, b] => Ok(Eml::node(lower(a)?, lower(b)?)),
            _ => Err(LowerError::Arity { form: "eml".into(), expected: 2, got: args.len() }),
        },
        other => Err(LowerError::UnknownForm(other.into())),
    }
}
```

## 4. Non-goals

- **Other forms** (`𝒢ₖ`, `∗`, `⟦P⟧`, `⊗`) and their lowering — later
  requirements; each lowers into its own typed core.
- **Arbitrary numeric literals** (`Num(n)`, `n != 1`) — rejected here; a later
  requirement may add a core that supports them. (`Sexpr` already represents
  them, so that addition changes only the `Num` arm of `lower`.)
- **Macros / quasiquote / a form-table registry** — deferred until form count
  warrants it.
- **`read_all` / a REPL / file loading** — trivial follow-ons, not in scope.
- **A general cons primitive** (`car`/`cdr`/`cons`, dotted/improper pairs) —
  deferred to the metaprogramming layer. Lists here are `Vec`-backed
  (ergonomic; no `nil`/improper-list edge cases), and the lowered `Eml::Node`
  already provides core-level *binary* pairing, so R-0003 needs no separate
  cons. See the decision log.
- Optimization; a heterogeneous runtime `Value` (complex-only here).

## 5. Open questions

*None blocking.* The symbol-charset rule (formerly open) is now the normative
tokenizer rule in §2.3. The `Display` of large `Num`s (`1e10` etc.) re-renders
in a normalized decimal form — the round-trip preserves the **value**, never
guarantees **text stability**; since R-0003 only lowers `1`, this has no
effect here, and is noted only so a future pretty-printer/REPL author does not
assume textual stability.

## 6. Acceptance criteria

Each maps a SPEC-0003 deliverable to an R-0003 acceptance criterion; the qa
agent's tests verify them.

- [ ] **AC1** — `Sexpr` is one type (`Num`/`Sym`/`List`), `Clone + PartialEq`,
  with constructors and a `Display`. The round-trip `read(s.to_string()) ==
  Ok(s)` holds **for every `s` in the reader's image** (§2.2) — the property
  the test asserts. Code is data.
- [ ] **AC2** — `read` parses `(eml 1 1)`, `(eml x 1)`, nesting, whitespace, and
  `;` comments to the right `Sexpr`; unbalanced parens / empty input / trailing
  tokens yield the matching `ReadError` — never a panic. (Per §2.3 there is no
  invalid-token class; `()` is *readable* as `List([])` and is rejected later,
  at lowering — see AC3.)
- [ ] **AC3** — `lower` enforces R-0001's grammar per the §2.4 table; unknown
  form, wrong `eml` arity, non-`1` literal, and non-form lists (including `()`)
  yield the matching `LowerError` at lowering time (before eval). `1`/var/
  `(eml a b)` lower correctly.
- [ ] **AC4** — through `eval_str`, `(eml 1 1) = e`, `(eml x 1) = exp(x)`, and
  `(eml 1 (eml (eml 1 x) 1)) = ln(x)` match R-0001's references within `1e-14`,
  over inputs including negative real `x` (the caller binds `x` in the `Env`) —
  using R-0001's *reused* evaluator.
- [ ] **AC5** — `eval_str` inherits R-0001's extended-reals behaviour (`ln 0`,
  `exp(-∞)` propagate, no panic) via the reused evaluator.
- [ ] **AC6** — every failure is a typed enum (`ReadError` / `LowerError` /
  `EvalError`, unified by `UflError`), never a panic, surfaced at the earliest
  layer that can detect it.

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-28 | New crate `ufl-syntax` depending on `ufl-core` (with `#![forbid(unsafe_code)]`, registered in the workspace). | Dependencies point inward (syntax → core); syntax is a bounded responsibility distinct from numerics (CLAUDE.md §6). |
| 2026-05-28 | `Sexpr` is general data (`Num(f64)`/`Sym`/`List`), not restricted to R-0001's grammar. | Homoiconicity means one general data structure (AC1); the grammar restriction belongs to the *lowering*, not the data. |
| 2026-05-28 | **Normative tokenizer rule** (§2.3): tokens are delimiter-bounded; only *finite* `f64` parses become `Num`; everything else (incl. `inf`/`nan` spellings) is a `Sym`. Promotes the former "symbol charset" open question to a fixed rule. | Closes the reader's image so the round-trip invariant and `PartialEq` are total on it; eliminates an `InvalidToken` error class; keeps `Num` always finite (so reader-produced `Sexpr`s never carry `NaN`). |
| 2026-05-28 | The round-trip invariant is scoped to the **reader's image**; public constructors may build textless data (`Sym("a b")`, `Num(NaN)`), which is intentional and out of the invariant. | Not every datum is textually expressible; the invariant is a property of `read`, enforced by the §2.3 rule, not a type-level guarantee over all constructible values. |
| 2026-05-28 | The grammar restriction lives in `lower`: `Num(n) where n == 1.0` → `One` (the *value* `1`, exactly representable, so the check is total/trap-free), non-`1` finite literal → typed `LowerError::UnsupportedLiteral`. | Faithful to R-0001's "only `1` is primitive" while keeping `Sexpr` general; arbitrary-literal support is a clean later addition with no `Sexpr` change. |
| 2026-05-28 | R-0003 AC2's "empty application" (`()`) is realized as `LowerError::NotAForm` at the **lowering** boundary, not a `ReadError`. | `()` is valid *data* (homoiconicity — `List([])` reads fine) but not a valid *form*; rejecting it is a grammar judgement, which is lowering's job. Refines the requirement's wording; recorded so qa expects a `LowerError`, not a `ReadError`. |
| 2026-05-28 | Lowering reuses `ufl_core::eval` verbatim; SPEC-0003 adds no numerics. | AC4 parity is through the *verified* core, so the branch convention and the 1-ulp `sin(τ/2)` self-correction are inherited, not re-implemented (the hater's migration risk, designed out). |
| 2026-05-28 | Dispatch is a `match` on the head symbol; the form-table registry is deferred. | One form needs no registry (no premature abstraction, CLAUDE.md §2); the seam is documented for when forms multiply. |
| 2026-05-28 | Ship a round-trippable `Display` and a `hello_sexpr` example now. | `Display` gives AC1 its round-trip oracle and structural rewrite-diffing; `hello_sexpr` runs the docs' literal strings, demonstrating AC1+AC4 and making the docs' notation runnable (the four-way string identity). |
| 2026-05-28 | Lists are `Vec`-backed; **no general cons-cell primitive** in R-0003. | `Vec` gives O(1) access, clean slice-pattern lowering, and no `nil`/improper-pair edge cases; the lowered `Eml::Node` already provides core-level binary pairing (`eml` *is* a cons). A general `cons`/`car`/`cdr` becomes valuable only at the metaprogramming layer (manipulating code as data) — there is a clean symmetry there (cons : structure :: `eml` : number :: NAND : Boolean) worth building when macros arrive, evaluated on its merits then. (Owner raised this; recorded for the macro requirement.) |

## Changelog

- 2026-05-28 — created (Draft).
- 2026-05-28 — three-lens review applied: normative tokenizer rule fixed in
  §2.3 (finite-only `Num`; `inf`/`nan` → `Sym`); round-trip invariant scoped to
  the reader's image with `PartialEq`/`NaN` and constructor caveats (§2.2);
  `()` → `LowerError::NotAForm` recorded (refines R-0003 AC2); `1` exact-value
  primitive + clippy `float_cmp` note (§2.4); `#![forbid(unsafe_code)]` +
  workspace registration (§2.1); AC4 env-binding note; `hello_sexpr` example
  added (§2.6); §5 open questions resolved; decision log extended.
