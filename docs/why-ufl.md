# Why UFL?

*A detailed, founded explanation of what UFL is, why it can exist, and how it
differs from every language that came before it. For a plain-language version
with no prerequisites, skip to [Appendix A](#appendix-a--ufl-for-newcomers).*

---

## 1. One example, to begin

Here is the most famous constant in mathematics:

```
e = eml(1, 1)
```

In every programming language you have used, `e` (Euler's number, ≈ 2.71828…)
is a *magic constant* — a number baked into the standard library
(`Math.E`, `std::f64::consts::E`) that you accept on faith. You cannot see
where it comes from; it is simply handed to you.

In UFL, `e` is not magic. It is **constructed** — from two copies of the digit
`1` and a single operator, `eml`. And the same is true of *everything else*:

```
eˣ      = eml(x, 1)
ln x    = eml(1, eml(eml(1, x), 1))
```

The exponential function is a one-node tree. The natural logarithm is a
three-node tree. Addition, multiplication, sine, the square root — every one is
a tree built from the *same single operator*. There is no `+` primitive, no `×`
primitive, no `sin` button. There is `eml`, and there is `1`.

That is the whole idea. The rest of this document explains why it is possible,
what UFL builds on top of it, and why it matters.

---

## 2. The problem: computation has no common vocabulary

Modern computing is split into worlds that share no formal language.

- **Systems languages** (C, Rust, Zig) describe *sequential transformations in
  time*.
- **Hardware description languages** (VHDL, Verilog, Chisel) describe
  *concurrent state machines in space*.
- **Neural frameworks** (PyTorch, JAX) describe *differentiable function
  composition over tensors*.

None can express the others without a foreign-function boundary. A gate, a
function, and a neural layer are the same kind of thing — a constraint over
some values — but we have no notation in which that sameness is visible. The
boundary between hardware and software is, as UFL's founding proposal puts it,
*an artifact of notation, not physics*
(see [`ufl-first-draft.md`](ufl-first-draft.md), §1).

There is a second, smaller redundancy hiding inside even a single language.
Look at a scientific calculator: dozens of buttons — `+ − × ÷ √ xⁿ logₓ
sin cos tan sinh …`. Each has its own rules. We *know* they are redundant —
`sin x = cos(x − π/2)`, `√x = x^(1/2)`, `xⁿ = eⁿ ˡⁿ ˣ` — and over centuries
mathematics reduced the list from "dozens" to "a handful." But it stalled
there. Every language, every calculator, every CPU still exposes an **operator
zoo**: a heterogeneous pile of primitives, each demanding its own
implementation on every substrate.

Two questions follow. *How small can the operator zoo get — one?* And: *if it
were one, would the hardware/software boundary still need to exist?* UFL's
answer to the first question comes from a recent result; its answer to the
second is the language itself.

---

## 3. The discovery: one operator is enough

In digital electronics, one fact underlies everything: a single two-input gate,
**NAND** (the Sheffer stroke), suffices to build *any* Boolean circuit. Every
laptop, every phone, every server is — at the bottom — NAND repeated billions
of times. Continuous mathematics had no such primitive. Until, it turns out, it
did.

The paper [`AllEle.html`](AllEle.html) — *"All elementary functions from a
single operator"* — establishes it. A single binary operator,

```
eml(x, y) = exp(x) − ln(y)
```

together with the constant `1`, generates the **entire scientific-calculator
repertoire**: the constants `e`, `π`, `i`; the arithmetic operations
`+ − × ÷` and exponentiation; and every elementary transcendental and algebraic
function. `eml` is the **continuous Sheffer stroke** — the NAND of real
mathematics.

### 3.1 How it was found

This was not anticipated, and it was not derived by elegant theory. The author
began with a 36-element starting list — a concrete scientific calculator's
worth of constants, unary functions, and binary operations — and ran systematic
*ablation*: remove a primitive, check whether the rest can still reconstruct
everything, repeat. That drove the list down through progressively smaller
configurations (the paper names them Calc 3, 2, 1, 0) until it stalled. The
final step — recognizing that the missing primitive was not among the *named*
functions, and enumerating candidate binary operators paired with candidate
constants — surfaced `eml`. Verification used a numeric "bootstrapping" sieve
(the `VerifyBaseSet` procedure), since direct symbolic proof was intractable.
The discovery is *constructive*: the paper exhibits the actual expression for
every one of the 36 primitives.

### 3.2 What it looks like

Every elementary expression, rewritten in EML form, becomes a **binary tree of
identical nodes**. The grammar is as small as a grammar can be:

```
S → 1 | eml(S, S)
```

(UFL adds variables as a second kind of leaf — see §4.) This language is
context-free and *isomorphic to the full binary trees* — the same combinatorial
objects counted by the Catalan numbers. A few worked values, with their tree
depths from the paper:

| Quantity            | EML form                            | Tree depth |
|---------------------|-------------------------------------|------------|
| `e`                 | `eml(1, 1)`                         | 1          |
| `eˣ`                | `eml(x, 1)`                         | 1          |
| `ln x`              | `eml(1, eml(eml(1, x), 1))`         | 3          |
| `x × y`             | *(a tree of identical `eml` nodes)* | 8          |

### 3.3 Honest caveats, up front

A founded explanation states the costs.

- **EML uses complex numbers internally.** Reaching `i`, `π`, and the
  trigonometric functions requires `ln` of negative values — Euler's formula
  routed through the complex logarithm. Real results, complex intermediates.
  The paper argues this is inevitable and, in practice, a minor inconvenience.
- **Trees grow.** Multiplication is depth 8; trigonometric functions are far
  deeper. The uniform representation trades node *variety* for node *count*.
  §5 and §6 explain why that trade is worth making.
- **EML is not unique.** The paper reports cousins — `edl(x,y) = exp(x)/ln(y)`
  with constant `e`, and `−eml(y,x) = ln(x) − exp(y)` with constant `−∞`. UFL
  selects EML deliberately; the reasoning is recorded in
  [`requirements/0001-eml-operator-core.md`](../requirements/0001-eml-operator-core.md).
- **Edge cases exist.** Branch cuts of the complex logarithm, and the
  extended-reals identities `ln 0 = −∞`, `e^(−∞) = 0`, must be handled with
  care — as they must in any serious floating-point or symbolic system.

None of this weakens the central fact. It sharpens it: *one operator is
sufficient*, and the price of using it is bookkeeping, not impossibility.

---

## 4. What UFL builds on it

UFL (Unified Formal Language) is a research language whose **numeric substrate
is the EML operator**. Where a conventional language has an operator zoo, UFL
has one operator and one literal. A UFL program is, at the bottom, *`eml` all
the way down* — exactly as a digital circuit is *NAND all the way down*.

This supersedes the original proposal's "Pillar 1 — Logarithmic Arithmetic
Core." That pillar had the right instinct (arithmetic reduces to log-domain
operations) but the loose form; EML is the precise, stronger fact. Adopting it
**collapses the proposal's six atoms to five**:

| Atom        | Role                                                        |
|-------------|-------------------------------------------------------------|
| `eml`       | the single numeric operator — all arithmetic and elementary functions |
| `𝒢ₖ`        | grade-lift — give a number geometric meaning (a *k*-vector) |
| `∗`         | geometric product — the universal spatial composition       |
| `⟦P⟧`       | predicate — wrap an expression as a constraint over pre/post state |
| `⊗`         | substrate-bind — annotate where an expression should run    |

The first decision is locked: see
[`requirements/0001-eml-operator-core.md`](../requirements/0001-eml-operator-core.md)
(R-0001, Accepted) and [`specs/0001-eml-operator-core.md`](../specs/0001-eml-operator-core.md)
(SPEC-0001). The numeric layer — the `eml` representation and a reference
evaluator — is UFL's first executable artifact. The geometric, predicative, and
substrate layers (atoms `𝒢ₖ`, `∗`, `⟦P⟧`, `⊗`) are designed in the founding
proposal and will be built, requirement by requirement, on top of it.

A crucial consequence of EML being a *representation* rather than a *domain*:
`eml(x, 1)` does not produce some special "EML-domain number" — it produces
`eˣ`, an ordinary value. There is no domain to enter and leave, no conversion
cost at the boundary. EML is a uniform way of *expressing how a value is
computed* — an intermediate representation — not a transform applied to the
value itself.

---

## 5. Why uniformity is the whole point

A reasonable objection: *if multiplication is a depth-8 tree, isn't this just a
worse way to compute?* For naïve evaluation — yes. But UFL never promises to
*evaluate* raw EML trees fast. EML is the **canonical intermediate
representation**; a separate substrate layer *compiles* a tree to whatever runs
fastest on the chosen hardware.

This is where uniformity pays for itself. Because every UFL expression is a tree
of one repeatable element, the *same* expression can be compiled to:

- a routine on a CPU,
- a stack-machine program (a single-instruction machine — the instruction is
  `eml`),
- an FPGA or analog circuit, where `eml` becomes one repeated physical element,
- a differentiable layer in a neural framework.

No operator zoo has to be ported to each substrate. There is one element to
implement, once, per substrate. This is the concrete mechanism behind UFL's
founding thesis — that the hardware/software boundary is notational. With a
uniform notation, the substrate becomes a *compilation target*, not a design
decision.

UFL layers three ideas on top, and they separate cleanly:

- **Predicates say *what*.** Following Hehner's predicative programming, a UFL
  program is a constraint that any valid execution must satisfy — a predicate
  over the pre-state and post-state. A gate, a function, and a neural layer
  become formally the same kind of object: a predicate.
- **EML trees are the universal *how*.** Any predicate that is satisfiable by an
  elementary computation has an EML-tree mechanism.
- **The orchestrator decides *where*.** Given a predicate and a cost model, it
  selects the lowest-cost substrate whose compiled EML tree satisfies the
  predicate.

EML does not compete with the predicate layer; it is the canonical mechanism
the orchestrator compiles. *What / how / where* — three separable concerns,
each with a home.

There is a forward-looking bonus. The AllEle paper shows that a *parameterized*
EML tree — coefficients on every leaf — is trainable by ordinary gradient
descent, and that trained weights can **snap to exact closed forms**. A
universal, uniform, differentiable representation of all elementary functions
is exactly what a learning system over geometric computation wants. UFL's neural
ambitions inherit this for free.

---

## 6. How UFL differs — a direct comparison

| Dimension                  | Systems languages (C, Rust) | HDLs (Verilog, Chisel) | Neural frameworks (PyTorch) | **UFL**                       |
|-----------------------------|-----------------------------|------------------------|-----------------------------|-------------------------------|
| Numeric primitives          | dozens (`+ − × ÷ ** exp ln sin …`) | gates + arithmetic blocks | tensor ops + autograd | **one — `eml`**               |
| Numeric literals            | many                        | `0`, `1`               | tensors / constants         | **one — `1`**                 |
| Core grammar                | large, with precedence      | modules and wires      | dynamic computation graph   | **`S → 1 \| x \| eml(S,S)`**  |
| Unit of structure           | statement / expression      | concurrent process     | layer                       | a node of one uniform tree    |
| Target substrate            | CPU (fixed)                 | silicon (fixed)        | GPU / CPU (fixed)           | chosen per expression         |
| Crosses the HW/SW boundary  | no                          | no                     | no                          | **yes, by construction**      |
| Program *is* a…             | sequence of instructions    | state machine          | function composition        | constraint (predicate) + a tree mechanism |

The pattern: every existing tool fixes a substrate and exposes a zoo of
primitives tuned to it. UFL fixes neither. It commits to *one* primitive — at
the cost of larger trees — and buys, in exchange, a notation in which hardware,
software, and learning are the same kind of statement. The depth-8
multiplication is not a bug to apologize for; it is the price of a property no
other language has.

---

## 7. Where UFL is today

A founded document is honest about status. UFL is an **active research
language**, early in construction, built under a strict requirement- and
spec-driven process (see [`CLAUDE.md`](../CLAUDE.md) and
[`ROADMAP.md`](../ROADMAP.md)).

- **Decided and specified:** EML as the foundational primitive — R-0001
  (Accepted) and SPEC-0001 (Draft). The numeric core `ufl-core` — the `Eml`
  representation and a reference evaluator — is the first thing being built.
- **Designed, not yet built:** the geometric-algebra layer, the Hehner
  predicate layer, and the substrate orchestrator. These are laid out in the
  founding proposal [`ufl-first-draft.md`](ufl-first-draft.md) and will each
  pass through the requirement loop.
- **Open research questions** remain — branch-cut conventions, predicate
  expressiveness, the right substrate cost model — tracked alongside the
  requirements that must resolve them.

UFL is, at this moment, a thesis with its first stone laid. This document
explains why the thesis is sound enough to keep laying stones.

---

## 8. Foundations and further reading

- [`AllEle.html`](AllEle.html) — *"All elementary functions from a single
  operator."* The result that makes UFL possible. Source of every claim in §3.
- [`ufl-first-draft.md`](ufl-first-draft.md) — the UFL founding proposal: the
  four pillars, the atoms, the substrate orchestrator, the hardware/software
  thesis.
- [`requirements/0001-eml-operator-core.md`](../requirements/0001-eml-operator-core.md)
  and [`specs/0001-eml-operator-core.md`](../specs/0001-eml-operator-core.md) —
  the first requirement and spec, where EML is formally adopted.
- [`README.md`](../README.md), [`CLAUDE.md`](../CLAUDE.md),
  [`ROADMAP.md`](../ROADMAP.md) — the project, its engineering standard, and its
  build order.

---

## Appendix A — UFL for newcomers

*No mathematics or programming background assumed. If §1–§8 felt dense, start
here.*

### What is an "operator"?

An operator is a tiny machine: values go in, one value comes out. `+` is an
operator — give it `2` and `3`, it returns `5`. A scientific calculator is
basically a box of these machines, one per button: an add machine, a multiply
machine, a sine machine, a square-root machine — dozens of them.

### The big question

All those machines feel different. But mathematicians noticed long ago that
they secretly overlap — you can build some of them out of others. So a natural
question: **how few machines do you really need?** Could you get away with…
ten? Five? Two?

The surprising answer, discovered recently: **one.** One machine, plus the
number `1`, can do everything the whole calculator does.

That machine is called `eml`. It takes two numbers and combines them in a
specific way (it exponentiates the first, takes the logarithm of the second,
and subtracts — but you do not need to know that to get the idea).

### The Lego analogy

Think of Lego. A Lego castle, a Lego spaceship, a Lego car — they all look
different, but they are all made from the *same little bricks*. The brick is
simple; the *arrangement* is what makes a castle a castle.

Digital computers already work this way. Deep down, a computer is made of one
kind of part — a "NAND gate" — repeated billions of times. Different
arrangements give you a calculator, a web browser, a video game. One brick,
endless buildings.

`eml` is that brick — but for *ordinary mathematics*, the math with decimals
and curves and `π`, not just the on/off math computers usually use. Before
this discovery, math had no single brick. Now it has one.

### What is a "tree"?

When you combine things step by step, you get a tree shape — like a sports
tournament bracket, where pairs of teams combine into winners, and winners
combine again, until one champion remains.

Computing `ln x` (the natural logarithm) in UFL looks like this — every `E`
is one `eml` brick:

```
            E
           / \
          1   E
             / \
            E   1
           / \
          1   x
```

Read it bottom-up: combine `1` and `x`, combine that result with `1`, combine
*that* with `1`. Three bricks, snapped together. That arrangement *is* the
logarithm. A different arrangement of the same brick is multiplication; another
is sine. Same brick, different buildings.

### Why "build `e` from two 1s" is a big deal

`e` is a famous number, `2.71828…`, that shows up everywhere in science. In a
normal calculator it is just *there* — a number printed on a key, no
explanation. In UFL you write `eml(1, 1)`: you **build** `e` out of two `1`s
and one brick. Nothing is handed to you on faith. Everything is constructed,
and you can see the construction.

### Why bother?

Because if *everything* is the same brick, then a piece of math no longer cares
*where* it runs. The same arrangement of `eml` bricks can become:

- a program on a normal computer chip,
- a custom electronic circuit,
- a part of an AI system that learns.

Today, those are three separate worlds with three separate toolkits. UFL's bet
is that with one brick, they become one world. That is the whole point of the
language — and `e = eml(1, 1)` is the smallest possible glimpse of it.
