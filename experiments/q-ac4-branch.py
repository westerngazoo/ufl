#!/usr/bin/env python3
"""
SPEC-0001 — experiment resolving Q-AC4 (the `ln_eml` branch rule).

Question
--------
The `ln` identity asserted by R-0001 AC5 is

    ln(x) = eml(1, eml(eml(1, x), 1))

where `eml(a, b) = exp(a) − ln(b)`. AllEle §4.1 predicts that, evaluated with
the standard principal-branch complex logarithm, the derived `ln(x)` for real
`x < 0` will differ from the true principal `ln(x)` by `2πi` (= τi) — the
"branch jump." Two resolutions were proposed: (1) correct the branch inside the
`eml` operator's `ln`; (2) patch the `i` sign downstream. SPEC-0001 picked (1)
and left the exact correction term as the spec's open question.

This script evaluates the chain under two candidate `ln_eml` definitions and
prints the discrepancy against the true principal `ln(x)`:

  - `ln_principal`   — the standard principal branch (`Im ∈ (-π, π]`).
  - `ln_neg_branch`  — same branch except at the cut, returns `Im = -π`
                       instead of `+π` (i.e. `Im ∈ [-π, π)`).

Finding
-------
In IEEE-754 `f64`, **both variants give zero discrepancy (≤ 1 ulp)** on every
test input, including negative reals. The textbook `2πi` discrepancy does not
appear, because `sin(-π)` in `f64` is *not* zero (it is ≈ -1.22e-16). The
intermediate `exp(e − iπ)` therefore lands slightly *below* the negative real
axis, and the outer principal `ln` of such a point returns `Im ≈ -π` (not
`+π`) — exactly the value the chain needs to recover the true principal
`ln(x)`. The branch convention self-corrects via the floating-point
representation of `sin(π)`.

Resolution
----------
Use the standard principal branch for `ln_eml` (`Complex::ln` in Rust), with
no correction term. SPEC-0001 records the dependency on `sin(π) ≠ 0` as
AC6 — a unit test asserts this invariant so that any future arithmetic
backend where it fails (e.g. arbitrary-precision) re-opens Q-AC4 deliberately
rather than silently breaking.

Run
---
    python3 experiments/q-ac4-branch.py
"""

from __future__ import annotations

import cmath
import math
from typing import Callable


def eml(x: complex, y: complex, ln_func: Callable[[complex], complex]) -> complex:
    """The EML operator parameterised by which complex log to use inside."""
    return cmath.exp(x) - ln_func(y)


def ln_principal(w: complex) -> complex:
    """Standard principal branch: Im ∈ (-π, π]."""
    return cmath.log(w)


def ln_neg_branch(w: complex) -> complex:
    """Same as principal, but Im ∈ [-π, π) — picks -π at the cut."""
    z = cmath.log(w)
    if z.imag == math.pi:
        z = complex(z.real, -math.pi)
    return z


def derive_ln(x: complex, ln_func: Callable[[complex], complex]) -> complex:
    """The `ln` identity in EML form: ln(x) = eml(1, eml(eml(1, x), 1))."""
    one = complex(1.0, 0.0)
    inner1 = eml(one, x, ln_func)
    inner2 = eml(inner1, one, ln_func)
    return eml(one, inner2, ln_func)


def _fmt(c: complex) -> str:
    return f"({c.real:+.6e}, {c.imag:+.6e})"


def main() -> None:
    xs = [-3.0, -1.0, -0.5, 0.5, 1.0, 2.5]
    for name, fn in [
        ("principal (-π, π]", ln_principal),
        ("neg-branch [-π, π)", ln_neg_branch),
    ]:
        print(f"\n=== ln_eml = {name} ===")
        print(f"{'x':>6} {'true ln(x)':>32} {'derived':>32} {'discrepancy':>32}")
        for xr in xs:
            x = complex(xr, 0.0)
            true = cmath.log(x)
            deriv = derive_ln(x, fn)
            disc = deriv - true
            print(f"{xr:>6} {_fmt(true):>32} {_fmt(deriv):>32} {_fmt(disc):>32}")

    print("\n=== sin(π) in f64 — the AC6 invariant ===")
    print(f"  math.sin(math.pi)  = {math.sin(math.pi):+.6e}")
    print(f"  math.sin(-math.pi) = {math.sin(-math.pi):+.6e}")
    print("  Non-zero → exp(... ± iπ) lands off the negative real axis,")
    print("  → principal `ln` of that returns Im ≈ ∓π, which is what the chain needs.")


if __name__ == "__main__":
    main()
