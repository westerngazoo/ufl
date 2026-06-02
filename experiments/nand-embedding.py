#!/usr/bin/env python3
"""
The continuous NAND — UFL's `eml` builds Boolean NAND, the discrete universality
bridge.

Claim (theory/universal-computability.md, Route A): UFL's numeric substrate
`eml` is not merely *analogous* to NAND ("the continuous Sheffer stroke"); it
*constructs* NAND on bit-encoded values, so it inherits all of digital logic's
universality. Encode false = 0.0, true = 1.0. Then

    AND(a,b)  = exp(ln a + ln b)         # the log-domain product
    NAND(a,b) = 1 - AND(a,b)

Every operation here (+, -, ×, exp, ln) is `eml`-expressible (AllEle), so the
whole of NAND is one `eml` tree. The one delicate point is the 0 input:
ln(0) = -inf. UFL's R-0001 AC3 (extended reals) guarantees that ln 0 = -inf
propagates and exp(-inf) = 0 with no trap or panic — so the 0-input case is
handled by exactly the machinery R-0001 already ships.

This script verifies the truth table and the 0-input path at the semantic level
(exp / ln / + / -). The follow-on (a Rust experiment) is to materialize the
*literal* `eml` tree — composing AllEle's +, -, × trees — and evaluate it
through `ufl-core`'s complex evaluator, confirming the same table end to end.

Run:
    python3 experiments/nand-embedding.py
"""

from __future__ import annotations

import math


def ln(x: float) -> float:
    """ln with the extended-reals convention ln 0 = -inf (R-0001 AC3)."""
    return -math.inf if x == 0.0 else math.log(x)


def eml_and(a: float, b: float) -> float:
    """AND(a,b) = exp(ln a + ln b). exp(-inf) = 0 absorbs any 0 input."""
    return math.exp(ln(a) + ln(b))


def eml_nand(a: float, b: float) -> float:
    """NAND(a,b) = 1 - AND(a,b)."""
    return 1.0 - eml_and(a, b)


# Boolean NAND is the Sheffer stroke — every Boolean function is a composition
# of NANDs (NOT a = NAND(a,a); AND = NAND(NAND(a,b),NAND(a,b)); etc.).
DERIVED = {
    "NOT a   = NAND(a,a)":            lambda a, b: eml_nand(a, a),
    "a AND b = NAND(NAND(a,b),..)":   lambda a, b: eml_nand(eml_nand(a, b), eml_nand(a, b)),
    "a OR b  = NAND(NOT a, NOT b)":   lambda a, b: eml_nand(eml_nand(a, a), eml_nand(b, b)),
}

EXPECTED_NAND = {(0, 0): 1, (0, 1): 1, (1, 0): 1, (1, 1): 0}


def main() -> None:
    print("eml-built NAND (false=0.0, true=1.0):\n")
    print("  a b | AND | NAND")
    print("  ----+-----+-----")
    ok = True
    for a in (0.0, 1.0):
        for b in (0.0, 1.0):
            A, N = eml_and(a, b), eml_nand(a, b)
            print(f"  {int(a)} {int(b)} |  {int(A)}  |  {int(N)}")
            ok &= int(N) == EXPECTED_NAND[(int(a), int(b))]
    print(f"\n  NAND truth table correct: {ok}")

    print("\n  the 0-input path (handled by R-0001 AC3 — no trap):")
    print(f"    AND(0,1) = exp(-inf + 0)   = {eml_and(0.0, 1.0)}")
    print(f"    AND(0,0) = exp(-inf + -inf) = {eml_and(0.0, 0.0)}")

    print("\n  derived gates (NAND is the Sheffer stroke — functionally complete):")
    for name, fn in DERIVED.items():
        row = {(a, b): int(round(fn(float(a), float(b)))) for a in (0, 1) for b in (0, 1)}
        print(f"    {name:<34} {row}")

    print(
        "\n  => eml builds NAND => all combinational logic. With predicate-level\n"
        "     state + sequencing + recursion (Route B), this lifts to all digital\n"
        "     computation. The continuous NAND is literal, not metaphor."
    )


if __name__ == "__main__":
    main()
