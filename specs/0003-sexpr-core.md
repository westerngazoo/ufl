# SPEC-0003 — Homoiconic S-Expression Core

- **Status:** Draft
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
inward — syntax → core, never the reverse. This is the crate the whole surface
language (more forms, a pretty-printer, later macros) grows in.

```
crates/ufl-syntax/
├── Cargo.toml          # depends on ufl-core
└── src/
    ├── lib.rs          # re-exports; the eval_str pipeline + UflError
    ├── sexpr.rs        # Sexpr type, constructors, Display (round-trippable)
    ├── read.rs         # reader: text → Sexpr; ReadError
    └── lower.rs        # lowering: Sexpr → Eml; LowerError
```

### 2.2 The `Sexpr` type — general homoiconic data

`Sexpr` is the one syntax tree. It is *general data* — it holds any number and
any symbol, because homoiconicity means code and data share one representation
(AC1). It is **not** artificially restricted to R-0001's grammar; that
restriction is the lowering pass's job (§2.4), not the data structure's.

```rust
/// A UFL S-expression — the single homoiconic syntax tree (R-0003 AC1).
/// Code and data share this one representation.
#[derive(Debug, Clone, PartialEq)]
pub enum Sexpr {
    /// A numeric literal (real; complex values are *derived*, never literal).
    Num(f64),
    /// A symbol — an operator/form name or a variable.
    Sym(String),
    /// An application / list: `(head arg ...)`.
    List(Vec<Sexpr>),
}
```

Constructors (`Sexpr::num`, `Sexpr::sym`, `Sexpr::list`) and a `Display` impl
that round-trips (`read(s.to_string()) == Ok(s)`) make the tree constructible,
traversable, comparable, and printable — the "code is data" surface of AC1. The
`Display` is cheap and earns its keep as the round-trip oracle; it is the first
place the docs' notation *is* the runnable syntax.

### 2.3 The reader — text → `Sexpr`

A standard S-expression reader: a tokenizer then a recursive parser.

- **Tokens:** `(`, `)`, atoms, whitespace (insignificant), line comments
  (`;` to end of line — LISP convention).
- **Atom classification:** a token that parses as `f64` becomes `Num`;
  everything else becomes `Sym`. (So `1`, `2.5`, `-1` are numbers; `eml`, `x`,
  `+foo` are symbols.)
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

`read` returns exactly one top-level `Sexpr` (trailing forms are an error;
a `read_all` returning `Vec<Sexpr>` is a trivial follow-on if needed).

### 2.4 The lowering pass — `Sexpr → Eml`

Lowering enforces **R-0001's grammar** (`S → 1 | var | eml(S, S)`), recovering
the typed core's structural guarantees at the lowering boundary (AC3). It is
total and side-effect-free, producing either an `Eml` or a typed `LowerError`.

The rules:

| `Sexpr` | lowers to | notes |
|---------|-----------|-------|
| `Num(1.0)` | `Eml::One` | the only primitive literal |
| `Num(n)`, `n ≠ 1` | `LowerError::UnsupportedLiteral(n)` | only `1` is primitive in R-0001; arbitrary literals are a later requirement |
| `Sym(s)` | `Eml::var(s)` | a variable (resolved at eval time) |
| `List([Sym("eml"), a, b])` | `Eml::node(lower a?, lower b?)` | the `eml` form |
| `List([Sym("eml"), ..])` wrong count | `LowerError::Arity { form: "eml", expected: 2, got }` | |
| `List([Sym(other), ..])` | `LowerError::UnknownForm(other)` | only `eml` is known in R-0003 |
| `List` with non-symbol head, or empty | `LowerError::NotAForm` | |

```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum LowerError {
    #[error("unsupported numeric literal {0}: only `1` is primitive in this core")]
    UnsupportedLiteral(f64),
    #[error("unknown form: `{0}`")]
    UnknownForm(String),
    #[error("form `{form}` expects {expected} arguments, got {got}")]
    Arity { form: String, expected: usize, got: usize },
    #[error("not a form: a list must have a symbol head")]
    NotAForm,
}

pub fn lower(s: &Sexpr) -> Result<Eml, LowerError>;
```

**Dispatch seam.** Lowering a list dispatches on the head symbol. With one form
(`eml`) this is a small `match`; a full *form table* (head → lowerer) is
deliberately deferred until there is more than one form (no premature
abstraction, CLAUDE.md §2). The dispatch function is structured so adding a
form is a localized edit — the documented seam where the future
orchestrator/macro layer will register rewrites (R-0003 §5).

### 2.5 The pipeline and the unified error

A convenience that runs the whole path, reusing `ufl_core::eval` verbatim:

```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum UflError {
    #[error(transparent)] Read(#[from] ReadError),
    #[error(transparent)] Lower(#[from] LowerError),
    #[error(transparent)] Eval(#[from] ufl_core::EvalError),
}

/// text → Sexpr → Eml → Value, with the env supplying any free variables.
pub fn eval_str(src: &str, env: &Env) -> Result<Value, UflError> {
    let sexpr = read(src)?;
    let eml = lower(&sexpr)?;
    Ok(ufl_core::eval(&eml, env)?)
}
```

This is the seam that proves AC4 parity *through the typed core*: the `eml` the
lowering produces is fed to R-0001's existing evaluator, so the branch
convention and the `sin(τ/2)` self-correction are inherited, not re-derived.

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
pub fn lower(s: &Sexpr) -> Result<Eml, LowerError> {
    match s {
        Sexpr::Num(n) if *n == 1.0 => Ok(Eml::One),
        Sexpr::Num(n)              => Err(LowerError::UnsupportedLiteral(*n)),
        Sexpr::Sym(name)           => Ok(Eml::var(name)),
        Sexpr::List(items)         => lower_form(items),
    }
}

fn lower_form(items: &[Sexpr]) -> Result<Eml, LowerError> {
    let Some((Sexpr::Sym(head), args)) = items.split_first().map(|(h, a)| (h, a))
        else { return Err(LowerError::NotAForm); };
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

(The `let-else` head match also covers the non-symbol-head and empty-list cases
as `NotAForm`.)

## 4. Non-goals

- **Other forms** (`𝒢ₖ`, `∗`, `⟦P⟧`, `⊗`) and their lowering — later
  requirements; each lowers into its own typed core.
- **Arbitrary numeric literals** (`Num(n)`, `n ≠ 1`) — rejected here; a later
  requirement may add a core that supports them.
- **Macros / quasiquote / a form-table registry** — deferred until form count
  warrants it.
- **`read_all` / a REPL / file loading** — trivial follow-ons, not in scope.
- Optimization; a heterogeneous runtime `Value` (complex-only here).

## 5. Open questions

- **`Display` of `Num`.** `write!("{n}")` renders `1.0` as `1` and `2.5` as
  `2.5` (Rust `f64 Display`), so round-trip holds — but `1e10`-style inputs
  re-render in a normalized form. Acceptable (round-trip preserves *value*, and
  R-0003 only lowers `1`); to be confirmed with qa. No semantic impact.
- **Symbol charset.** Which characters may a symbol contain? Proposed: any
  non-whitespace, non-paren, non-`;` run that does not parse as a number.
  SPEC fixes the exact rule; the architect reviews.

## 6. Acceptance criteria

Each maps a SPEC-0003 deliverable to an R-0003 acceptance criterion; the qa
agent's tests verify them.

- [ ] **AC1** — `Sexpr` is one type (`Num`/`Sym`/`List`), `Clone + PartialEq`,
  with constructors and a `Display` that round-trips (`read(s.to_string()) ==
  Ok(s)` for read-produced `s`). Code is data.
- [ ] **AC2** — `read` parses `(eml 1 1)`, `(eml x 1)`, nesting, whitespace, and
  `;` comments to the right `Sexpr`; unbalanced parens / empty / trailing tokens
  yield the matching `ReadError` — never a panic.
- [ ] **AC3** — `lower` enforces R-0001's grammar per the §2.4 table; unknown
  form, wrong `eml` arity, non-`1` literal, and non-form lists yield the
  matching `LowerError` at lowering time (before eval). `1`/var/`(eml a b)`
  lower correctly.
- [ ] **AC4** — through `eval_str`, `(eml 1 1) = e`, `(eml x 1) = exp(x)`, and
  `(eml 1 (eml (eml 1 x) 1)) = ln(x)` match R-0001's references within R-0001's
  tolerance (`1e-14`), over inputs including negative real `x` — using R-0001's
  *reused* evaluator.
- [ ] **AC5** — `eval_str` inherits R-0001's extended-reals behaviour (`ln 0`,
  `exp(-∞)` propagate, no panic) via the reused evaluator.
- [ ] **AC6** — every failure is a typed enum (`ReadError` / `LowerError` /
  `EvalError`, unified by `UflError`), never a panic, surfaced at the earliest
  layer that can detect it.

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-28 | New crate `ufl-syntax` depending on `ufl-core`. | Dependencies point inward (syntax → core); this is where the whole surface language grows. A submodule would entangle syntax with the numeric core. |
| 2026-05-28 | `Sexpr` is general data (`Num(f64)`/`Sym`/`List`), not restricted to R-0001's grammar. | Homoiconicity means one general data structure (AC1); the grammar restriction belongs to the *lowering*, not the data. |
| 2026-05-28 | The grammar restriction lives in `lower`: `Num(1.0)`→`One`, non-`1` literal → typed `LowerError`. Resolves R-0003 §5 "numeric literals." | Faithful to R-0001's "only `1` is primitive" while keeping `Sexpr` general; arbitrary-literal support is a clean later addition with no `Sexpr` change. |
| 2026-05-28 | Lowering reuses `ufl_core::eval` verbatim; SPEC-0003 adds no numerics. | AC4 parity is through the *verified* core, so the branch convention and the 1-ulp `sin(τ/2)` self-correction are inherited, not re-implemented (the hater's migration risk, designed out). |
| 2026-05-28 | Dispatch is a `match` on the head symbol; the form-table registry is deferred. | One form needs no registry (no premature abstraction, CLAUDE.md §2); the seam is documented for when forms multiply. |
| 2026-05-28 | A round-trippable `Display` is included now. | Cheap; gives AC1 its round-trip oracle and makes "the docs' notation is the runnable syntax" demonstrable. |

## Changelog

- 2026-05-28 — created (Draft).
