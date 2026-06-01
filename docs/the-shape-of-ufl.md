# The Shape of UFL — today, tomorrow, and where it runs

*Three concrete snapshots: what UFL is **right now** (after R-0001), what
writing UFL programs **will look like** when the surface lands, and how the
**same EML tree compiles to every substrate** the proposal targets.*

For the founded explanation of the underlying ideas, read
[`why-ufl.md`](why-ufl.md). This document is the language tour.

---

## 1. Today — what's actually built

After [R-0001](../requirements/0001-eml-operator-core.md) /
[SPEC-0001](../specs/0001-eml-operator-core.md), the language is:

- **One atom.** `eml(x, y) = exp(x) − ln(y)`.
- **One literal.** `1`.
- **Variables.** Free names as terminals.
- **A grammar.** `S → 1 | <var> | eml(S, S)` — enforced structurally by the
  `Eml` enum.
- **A reference evaluator.** Recursive, post-order, over `Complex<f64>`.

That's the whole executable language. No surface syntax yet — trees are
constructed via the Rust API.

### 1.1 The runnable example

[`crates/ufl-core/examples/hello_eml.rs`](../crates/ufl-core/examples/hello_eml.rs)
builds the EML trees for `e`, `exp(x)`, and `ln(x)`, evaluates them, and
prints. Run it:

```sh
cargo run -p ufl-core --example hello_eml
```

Selected output:

```
e — Euler's number, built from two `1`s and one operator:
  eml(1, 1)                        =     2.718282

exp(x) — a one-node tree:
  eml(x, 1)  with x =  2.5         =    12.182494

ln(x) — a three-node tree (works on negative reals via the f64 self-correction):
  eml(1, eml(eml(1, x), 1))  x =   -1 =     0.000000 +     3.141593 i

Derived Euler's identity — ln(-1) = iτ/2, built from `eml` and `1`:
  eml(1, eml(eml(1,x), 1))  x = -1 =     0.000000 +     3.141593 i
```

Euler's identity (`ln(-1) = iτ/2`) emerges from *nothing but* `eml` and `1`.
That is the entire current language, executing.

### 1.2 The Rust surface, today

```rust
use ufl_core::{eval, Eml, Env, Value};

// e = eml(1, 1)
let e_tree = Eml::node(Eml::one(), Eml::one());
let e = eval(&e_tree, &Env::new()).unwrap();   // ≈ 2.71828

// ln(x) = eml(1, eml(eml(1, x), 1))
let ln_tree = Eml::node(
    Eml::one(),
    Eml::node(
        Eml::node(Eml::one(), Eml::var("x")),
        Eml::one(),
    ),
);
let mut env = Env::new();
env.bind("x", Value::new(-1.0, 0.0));
let v = eval(&ln_tree, &env).unwrap();         // ≈ 0 + iτ/2
```

Verbose, intentionally so — this is the *primitive* API; the surface syntax in
§2 is what humans will write.

---

## 2. Tomorrow — what writing UFL will look like

> Everything in this section is **illustrative only**. There is no parser yet
> (R-0005) and no GA / predicate / substrate atoms yet (R-0002 / R-0004 /
> R-0007). The shapes below are the design target — what the corresponding
> Rust tree-builder reduces to once each layer lands.

### 2.1 Numeric — the EML layer (already executable in spirit)

```ufl
-- a one-atom programming language: `eml` and `1`.
let e         = eml(1, 1)                          -- ≈ 2.71828
let exp x     = eml(x, 1)
let ln x      = eml(1, eml(eml(1, x), 1))

-- derived constants emerge — built, not assumed
let i         = exp(ln(-1) / 2)                    -- ≈ (0, +1)
let τ         = 2 · imag(ln(-1))                   -- the full turn
```

Each `let` is an EML tree under a name. `2 ·` and `/2` would be sugar for
deeper EML subtrees (multiplication is depth 8 per AllEle Table 4). The
surface is friendly; the core is one operator.

Today's Rust API is the *same* set of trees, written more verbosely. R-0005
(syntax + AST) and R-0006 (evaluator over the parsed AST) close the gap.

### 2.2 Geometric — multivectors over EML scalars (R-0002)

```ufl
-- promote scalars to multivectors by grade
let v   = 𝒢₁([eml(3, 1), 0, 0])                    -- e³·e₁   (eml(3,1) = e³)
let R   = 𝒢₀(cos(τ/8)) − 𝒢₂([sin(τ/8), 0, 0])      -- unit rotor: +τ/4 in e₁∧e₂
let v'  = R ∗ v ∗ ~R                                -- sandwich: rotates e₁ → e₂
```

Coefficients are `Complex<f64>` — the values EML evaluates to. The geometric
product `∗` is table-driven over G(3,0,0)'s 8 basis blades; `~R` is the
Clifford reverse (negates grade-2 and grade-3 components). The rotor sign and
blade order follow [`conventions.md`](conventions.md); the `−sin` gives a `+τ/4`
rotation sending `e₁ → e₂`. (A general bivector exponential `exp(B) → R` is a
later requirement; here the rotor is assembled from its `cos`/`sin` parts
directly.)

### 2.3 Predicates — programs as constraints (R-0004)

```ufl
-- a program is a predicate over pre/post state (Hehner)
⟦ grade(v') = 1 ∧ |v'| = |v| ⟧

-- composes via conjunction (sequential) and existential (parallel)
⟦ P(s, s') ⟧ ∧ ⟦ Q(s', s'') ⟧
```

A gate, a function, and a neural layer are the *same kind* of thing — a
predicate. The evaluator is one strategy; a substrate is another. Both must
*satisfy* the predicate.

### 2.4 Substrate hints — the orchestrator (R-0007)

```ufl
-- annotate an expression with a substrate hint
let v'_fast = (R ∗ v ∗ ~R) ⊗ analog

-- the orchestrator may override the hint if a cheaper substrate
-- satisfies the surrounding predicate
```

`⊗` is a *suggestion*, not a constraint. The orchestrator picks the lowest-cost
substrate whose compiled tree satisfies the active predicates.

---

## 3. Where it runs — substrates

UFL's central bet: *the same EML tree compiles to every numeric substrate.* No
operator zoo to re-port. Each section below is the **shape** the tree takes on
that substrate — not (except for CPU) a built implementation.

### 3.1 CPU / Rust — **built today (R-0001)**

The reference evaluator IS the CPU substrate. A tree compiles to a chain of
`Complex<f64>` operations against `num-complex`'s `.exp()` and `.ln()`:

```rust
// for eml(x, y):  exp(eval(x)) - ln_eml(eval(y))
//
// for eml(1, 1):  exp(Value::new(1, 0)) - (Value::new(1, 0)).ln()
//              =  e − 0
//              =  e
```

Status: ✅ shipped, 20/20 tests green, all six R-0001 ACs met. The principal
branch self-corrects via `sin(τ/2) ≠ 0` in `f64` (Q-AC4); AC6 is the tripwire.

### 3.2 Stack machine — *one* numeric opcode

Conventional CPUs have an instruction per ALU op (ADD, MUL, SIN, …). A UFL
stack machine has **one numeric opcode** — `EML` — plus a constant push and a
variable load. The whole instruction set:

```
PUSH 1        ; push the literal 1
LOAD <name>   ; push a bound variable
EML           ; pop y, pop x, push exp(x) − ln(y)
HALT
```

Compiled trees:

```
;; e = eml(1, 1)
PUSH 1
PUSH 1
EML
HALT

;; ln(x) = eml(1, eml(eml(1, x), 1))
PUSH 1
PUSH 1
LOAD x
EML
PUSH 1
EML
EML
HALT
```

Compilation is mechanical: post-order traversal of the EML tree → 3 opcodes
per node (`exp_arg-bytecode … log_arg-bytecode … EML`). A single-instruction
ALU is achievable in genuinely tiny silicon — see AllEle §4.2.

Status: ⬜ planned (lands with R-0007 substrate work).

### 3.3 FPGA / digital silicon — one cell, replicated

A digital `eml` cell is a small block: two unary CORDIC / LUT blocks feeding a
subtractor.

```
                     ┌──────┐
           x ──────▶ │ exp  │ ───▶ ●
                     └──────┘      │
                                ┌──┴──┐
                                │  −  │ ────▶ result
                                └──┬──┘
                     ┌──────┐      │
           y ──────▶ │  ln  │ ───▶ ●
                     └──────┘
```

`exp` and `ln` in fixed-point are well-known FPGA macros (CORDIC, Padé
approximants, segmented LUTs). An EML tree of depth N becomes a pipeline of N
identical cells — uniform routing, predictable timing, one verified block
replicated.

This is the structural analogue of *NAND all the way down* — exactly the move
that made digital logic uniform, lifted to continuous arithmetic.

Status: ⬜ planned. Worth noting: the orchestrator (R-0007) picks this
substrate when the predicate demands high throughput / low jitter and tolerates
the LUT's quantization noise.

### 3.4 Analog — log-converters + subtractor

The cleanest physical realization: voltage-to-log converters (exploiting the
transistor V_BE / collector-current logarithmic relation), then an op-amp
subtractor:

```
        ┌────────────────┐
        │ V → log V       │
   x ──▶│ (transistor +   │ ──▶ V₁
        │  op-amp)        │
        └────────────────┘
                              ┌────────────┐
                              │  V₁ − V₂   │ ──▶ V_out   ≈ eml(x, y)
                              └────────────┘
        ┌────────────────┐
        │ V → log V       │
   y ──▶│ (transistor +   │ ──▶ V₂
        │  op-amp)        │
        └────────────────┘
```

`exp` is the inverse circuit (log-to-voltage via the same junction). A
complete `eml` analog cell is ~3 active devices. Stochastic resonance can
amplify precision in noise-tolerant settings (per GAPU's reservoir layer).

This is what makes UFL's substrate story serious: continuous mathematics maps
onto continuous physics directly. There is no "ALU" to design — a depth-N tree
is N cells wired in pipeline.

Status: ⬜ planned. Realistic precision per node ≈ 8–10 bits; the orchestrator
picks analog when the predicate's precision budget admits it (e.g. neural
inner loops where stochasticity is benign).

### 3.5 Differentiable / neural — trainable trees

Parameterise the leaves of an EML tree with weights `w[i]` and the tree
becomes a **trainable function** of its inputs (AllEle §4.3 / proposal §4).
Adam (or any first-order optimizer) tunes the weights against a dataset; when
the generating law is elementary, the weights **snap to exact closed forms**:

```
;; depth-3 EML tree with weights:
;;
;;   eml( w[0]·1 + w[1]·x ,
;;        eml( w[2]·1 + w[3]·x ,
;;             w[4]·1 ) )
;;
;; Train against (x, f(x)) pairs. When f is elementary, the weights converge
;; to a configuration whose tree evaluates to f exactly — the network has
;; *discovered* the closed form.
```

This is the universal trainable architecture: any elementary law is some
configuration of weights on the same uniform tree. UFL's neural story (atom
`⟦P⟧` composed with parameterised EML trees) inherits this for free.

Status: ⬜ planned. Direct port of AllEle's symbolic-regression experiments.

### 3.6 The whole substrate table

| Substrate | Element | Compiled form of a tree | Status |
|---|---|---|---|
| **CPU / Rust** | `Complex<f64>` calling `.exp()` / `.ln()` | a call chain | ✅ shipped (R-0001) |
| **Stack machine** | one opcode `EML` + `PUSH 1` / `LOAD` | post-order bytecode | ⬜ R-0007 |
| **FPGA / silicon** | one digital `eml` cell | pipeline of N cells | ⬜ future |
| **Analog** | log-conv + V−V subtractor | wired pipeline of cells | ⬜ future |
| **Differentiable / neural** | parameterised tree + Adam | trainable function | ⬜ future |

---

## 4. The orchestrator (atom `⊗`) — the binding glue

The substrate orchestrator is what makes the table above operational. Given
(a) an EML tree, (b) the active predicates (`⟦P⟧`), and (c) a cost model, it
picks the lowest-cost substrate whose compiled tree satisfies the predicate.
The tree itself is invariant across substrates; only the compilation strategy
changes.

```
       EML tree  +  ⟦P⟧  +  cost model
                    │
                    ▼
            ┌───────────────┐
            │  orchestrator │      "satisfies P, lowest cost is analog"
            └───────────────┘
                    │
                    ▼
           [ analog cells wired ]
```

This is the concrete mechanism behind UFL's founding thesis — that the
hardware/software boundary is *notational*. With a uniform notation, the
substrate becomes a compilation target, not a design decision.

Status: ⬜ R-0007.

---

## 5. What this buys

- **One operator to port per substrate**, not dozens. Bring up `eml` once on
  each target and you have the whole language.
- **A predicate is a contract**, not an implementation. Multiple substrates
  can satisfy it; the orchestrator picks.
- **The same tree** is the object of CPU evaluation, FPGA synthesis, analog
  pipeline routing, and neural training. No translation, no semantic gap.

That is what motivates the choice of `eml` as the foundational primitive, and
why R-0001 — just one atom and one literal — is already the load-bearing piece.

---

## 6. Read more

- [`why-ufl.md`](why-ufl.md) — the founded explanation of *why* one operator
  is enough.
- [`AllEle.html`](AllEle.html) — the discovery paper.
- [`ufl-first-draft.md`](ufl-first-draft.md) — the founding proposal (pillars
  and original atoms; predates EML's selection — see decision log in R-0001).
- [`../requirements/0001-eml-operator-core.md`](../requirements/0001-eml-operator-core.md)
  and [`../specs/0001-eml-operator-core.md`](../specs/0001-eml-operator-core.md)
  — the formal contract for what's built.
- [`../crates/ufl-core/examples/hello_eml.rs`](../crates/ufl-core/examples/hello_eml.rs)
  — the runnable demo behind §1.1.
