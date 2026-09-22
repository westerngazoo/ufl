# D-0002 — SPEC-0020: the form, and who verified it

- **Date:** 2026-09-05
- **Decided by:** Gustavo, on the main session's measurement
- **Realized by:** [SPEC-0020](../specs/0020-single-visit-grade.md) §2.2, §2.2b, §6, §8

## Two decisions

**1. The slice-pattern form over the visitor form.** Both are correct (0
mismatches vs the old `grade`/`typecheck` on 300,000 random trees each), both fix
the exponential (`visits == node_count` on every shape at every rung), both are
allocation-free and exhaustive on `GeoExpr`. They differ in exactly two measured
ways, pulling opposite directions:

| | slice (chosen) | visitor |
|---|---|---|
| `typecheck` throughput, release | 0.53× old | 0.50× old |
| `check` depth ceiling, **debug**, 1 MiB | 1,377 (old: 1,406) | **400** |

The visitor is 6% faster on a function that is 15–26% of lane wall-clock — ~1%
end to end — and in `cargo test`'s profile loses 72% of stack depth against
today's code on a `pub` function. The slice form is at parity with today in
debug and +75% in release. Gustavo chose depth on the CI profile over 1% of
production throughput. This **reverses the architect agent's round-1
preference**, which was formed on release numbers alone; the agent did not have
the debug measurement when it recommended.

**2. Acceptance on main-session verification.** CLAUDE.md §4 step 2 moves a
spec to Accepted when *the architect agent approves*. On SPEC-0020 the architect
agent returned REQUEST CHANGES in round 1, every finding was folded, and the
agent then **stalled twice** on round 2 (as did the hater; four stalls on this
spec, six in the session). The main session ran every check the architect was
asked to make — compiling the spec's code as printed under `#![deny(warnings)]`,
the 300K differential for both forms, the pinned rotor-nested series, the visit
ladder on all three shapes, the four precedence cases, the depth bisect in both
profiles, and the `is_versor` reference grep — and reported them as its own,
not the agent's. Gustavo accepted on that basis by choosing the form.

## Why record this

The substitution is a process deviation, and CLAUDE.md §4 says a deviation is
recorded, not smoothed over. The record must show that SPEC-0020's round-2
verification was done by the engineer and accepted by the owner — a weaker
guarantee than an independent agent's approval, chosen because the agent could
not deliver one, with the full verification listed in SPEC-0020 §8 so it can be
re-run by anyone.

## Consequences

- SPEC-0020 §2.2b keeps the visitor form's code and numbers as the documented
  alternative, so the trade can be revisited if debug depth stops mattering or
  the closure starts inlining in debug.
- Agent stalls are now a known session hazard. If the pattern continues, the
  fallback of main-session verification with an explicit decision-log entry is
  the recorded precedent; it is not a licence to skip the agents when they work.
