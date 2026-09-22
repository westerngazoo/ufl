#!/usr/bin/env python3
"""
SPEC-0022 §4.1 — the boundary table's 80-digit reference.

Question
--------
SPEC-0022 §4.1 tabulates how the Gate-2 witness's error behaves as the joint
angles grow. Rev 3 of the spec attributed the growth to floating-point
argument-reduction loss shared by the witness and by `ArmFk::forward`, and
concluded that "the two walk away from the true value together."

That conclusion was never measured. It compared the witness *against the f64
reference* and against nothing else, so it could not tell which of the two was
drifting. This script supplies the missing column: the **true** forward
kinematics, computed to 80 significant digits, so each side can be scored
against the map itself rather than against the other.

Method
------
For each sampled `(t1, t2)` — the exact f64 values the Rust side used, parsed
losslessly — compute

    x = l1*cos(t1) + l2*cos(t1 + t2)
    y = l1*sin(t1) + l2*sin(t1 + t2)

in 80-digit `Decimal`, with `t1 + t2` summed **exactly** (no f64 rounding), and
`sin`/`cos` by Taylor series after exact reduction modulo an 80-digit 2*tau/2.

Then report, per band, the per-component RMSE of:

  * the witness vs exact        -- is the *witness* drifting?
  * `ArmFk::forward` vs exact   -- is the *reference* drifting?
  * the witness vs `forward`    -- the only column rev 3 had

A fourth column isolates the suspected mechanism: exact FK recomputed from the
**f64-rounded** sum `fl(t1 + t2)`, which is what `baseline.rs:27-28` forms and
what the witness never forms. If that column tracks the reference's error, the
drift is that one rounding.

Usage
-----
    cargo run -p ufl-evolve --release --example boundary_samples \
        | python3 experiments/0022-exact-fk-reference.py
"""
from decimal import Decimal as D, getcontext
import sys

getcontext().prec = 80

# tau to 80+ digits (docs/conventions.md: tau is the circle constant).
TAU = D(
    "6.283185307179586476925286766559005768394338798750211641949889184615632812"
    "57241799725606965068423413596429617302656461329418768921910116446345071881"
)


def _sin(x: D) -> D:
    """sin(x) to working precision, after exact reduction mod tau."""
    x = x - TAU * (x / TAU).to_integral_value(rounding="ROUND_FLOOR")
    term = total = x
    n = 1
    while abs(term) > D(10) ** -75 and n < 400:
        term = -term * x * x / D((2 * n) * (2 * n + 1))
        total += term
        n += 1
    return total


def _cos(x: D) -> D:
    return _sin(x + TAU / 4)


def exact_fk(t1: D, t2: D, l1: D, l2: D):
    s = t1 + t2
    return (l1 * _cos(t1) + l2 * _cos(s), l1 * _sin(t1) + l2 * _sin(s))


def main() -> None:
    l1 = l2 = None
    bands = {}
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        if line.startswith("#"):
            for tok in line.lstrip("# ").split():
                key, _, val = tok.partition("=")
                if key == "l1":
                    l1 = D(val)
                elif key == "l2":
                    l2 = D(val)
            continue
        if line.startswith("band_lo"):
            continue
        lo, hi, t1, t2, xw, yw, xr, yr = line.split(",")
        bands.setdefault((lo, hi), []).append(
            (float(t1), float(t2), float(xw), float(yw), float(xr), float(yr))
        )

    if l1 is None or l2 is None:
        sys.exit("input carried no `# l1=.. l2=..` header")

    print(f"arm: l1 = {l1}, l2 = {l2}   reference: {getcontext().prec} digits")
    print("per-component RMSE = sqrt(sum[(dx)^2 + (dy)^2] / 2n)\n")
    head = f"{'band':<18} {'witness vs exact':>17} {'f64 ref vs exact':>17} {'witness vs ref':>15} {'exact(fl(t1+t2))':>18}"
    print(head)
    print("-" * len(head))
    for (lo, hi), rows in bands.items():
        acc = [D(0), D(0), D(0), D(0)]
        for t1, t2, xw, yw, xr, yr in rows:
            ex, ey = exact_fk(D(t1), D(t2), l1, l2)
            # the same exact FK, but from the f64-rounded sum the harness forms
            fx, fy = exact_fk(D(t1), D(t1 + t2) - D(t1), l1, l2)
            for k, (px, py) in enumerate(
                ((D(xw), D(yw)), (D(xr), D(yr)), (D(xw), D(yw)), (D(xw), D(yw)))
            ):
                tx, ty = (ex, ey) if k < 2 else ((ex, ey) if k == 2 else (fx, fy))
                if k == 2:
                    tx, ty = D(xr), D(yr)
                acc[k] += (px - tx) ** 2 + (py - ty) ** 2
        n = D(len(rows) * 2)
        vals = [float((a / n).sqrt()) for a in acc]
        label = f"[{float(lo):g},{float(hi):g}]"
        print(
            f"{label:<18} {vals[0]:>17.3e} {vals[1]:>17.3e} {vals[2]:>15.3e} {vals[3]:>18.3e}"
        )


if __name__ == "__main__":
    main()
