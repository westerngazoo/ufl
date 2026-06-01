#!/usr/bin/env python3
"""
SPEC-0002 — rotor-orientation oracle for R-0002 AC5/AC6.

The three-lens review found that the rotor `cos(τ/8) + e₁₂·sin(τ/8)` sends
`e₁ → −e₂`, contradicting the spec's stated `e₁ → e₂`. The fix: use the
standard rotor convention `R = exp(−B̂ θ/2) = cos(θ/2) − B̂ sin(θ/2)`, i.e. the
bivector component is `−sin(τ/8)`. This script independently derives the
G(3,0,0) geometric product from the 3-bit-mask rule and verifies the corrected
rotor, so the convention in `docs/conventions.md` and SPEC-0002 §6 has a
runnable oracle.

Run:
    python3 experiments/r0002-rotor.py
"""

from __future__ import annotations

import math

# Storage order (grade-then-lexicographic) → basis-vector tuple. See
# docs/conventions.md and SPEC-0002 §2.1.
BLADES = {
    0: (), 1: (1,), 2: (2,), 3: (3,),
    4: (1, 2), 5: (1, 3), 6: (2, 3), 7: (1, 2, 3),
}
INDEX = {v: k for k, v in BLADES.items()}

NAMES = ["1", "e1", "e2", "e3", "e12", "e13", "e23", "e123"]
# Reverse negates grades 2 and 3 (k(k-1)/2 odd).
NEGATE = [False, False, False, False, True, True, True, True]


def blade_product(a: tuple[int, ...], b: tuple[int, ...]) -> tuple[int, int]:
    """(sign, result storage index) for two basis blades in G(3,0,0).

    Rule (SPEC-0002 §2.4.1): concatenate the basis-vector lists, sort by
    adjacent transpositions counting swaps for the sign, then cancel adjacent
    equal indices (each eₖ² = +1, no sign — the (+,+,+) signature).
    """
    arr = list(a) + list(b)
    sign = 1
    for i in range(len(arr)):
        for j in range(len(arr) - 1 - i):
            if arr[j] > arr[j + 1]:
                arr[j], arr[j + 1] = arr[j + 1], arr[j]
                sign = -sign
    out: list[int] = []
    k = 0
    while k < len(arr):
        if k + 1 < len(arr) and arr[k] == arr[k + 1]:
            k += 2
        else:
            out.append(arr[k])
            k += 1
    return sign, INDEX[tuple(out)]


def zero() -> list[float]:
    return [0.0] * 8


def mul(x: list[float], y: list[float]) -> list[float]:
    out = zero()
    for i in range(8):
        for j in range(8):
            sign, r = blade_product(BLADES[i], BLADES[j])
            out[r] += sign * x[i] * y[j]
    return out


def reverse(m: list[float]) -> list[float]:
    return [(-c if NEGATE[i] else c) for i, c in enumerate(m)]


def add(a: list[float], b: list[float]) -> list[float]:
    return [x + y for x, y in zip(a, b)]


def vec(c: list[float]) -> list[float]:
    m = zero(); m[1], m[2], m[3] = c; return m


def scalar(s: float) -> list[float]:
    m = zero(); m[0] = s; return m


def bivector(b: list[float]) -> list[float]:
    m = zero(); m[4], m[5], m[6] = b; return m


def show(m: list[float]) -> str:
    return " + ".join(f"{c:+.4f}{NAMES[i]}" for i, c in enumerate(m) if abs(c) > 1e-12) or "0"


def main() -> None:
    c = math.cos(math.tau / 8)  # math.tau = 2π = UFL's τ
    s = math.sin(math.tau / 8)
    print(f"cos(τ/8) = {c:.6f}   sin(τ/8) = {s:.6f}")

    # Corrected rotor: bivector component −sin(τ/8) ⇒ +τ/4 rotation, e₁ → e₂.
    R = add(scalar(c), bivector([-s, 0.0, 0.0]))
    Rr = reverse(R)

    print("\n=== AC5: R = 𝒢₀(cos τ/8) + 𝒢₂([−sin τ/8, 0, 0]) ===")
    for name, v in [("e1", vec([1, 0, 0])), ("e2", vec([0, 1, 0])), ("e3", vec([0, 0, 1]))]:
        out = mul(mul(R, v), Rr)
        print(f"  R ∗ {name} ∗ ~R = {show(out):<12}  |out| = {math.sqrt(sum(x*x for x in out)):.6f}")
    print("  expected: e1 → +e2, e2 → −e1, e3 → e3 (fixed); norm preserved")

    print("\n=== AC6: v = e·e1  (e from eml(1,1)) ===")
    v = vec([math.e, 0.0, 0.0])
    out = mul(mul(R, v), Rr)
    print(f"  R ∗ v ∗ ~R = {show(out)}   expected +{math.e:.5f}e2")


if __name__ == "__main__":
    main()
