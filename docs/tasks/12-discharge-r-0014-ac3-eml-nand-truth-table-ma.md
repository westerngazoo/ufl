# T12 · Discharge R-0014 AC3: eml-NAND truth table + matmul-entry probe

- **Priority:** P1
- **Depends on:** T7
- **Tags:** rung-2, theory, R-0014, crate:ufl-core

## Context
The one "Owed" row in the honest universality ledger (theory/universal-computability.md §6 table, §7.1) standing between UFL and its discrete-universality claim, now formalized as R-0014 AC3 (Draft on main) but unbuilt: the literal eml-NAND tree has never been run through ufl-core. The honest closed class remains "elementary functions — total, terminating, no recursion, no branching, no state" (§1). This is also the first machine-generated-deep-tree consumer, stress-testing T6's depth contract and the future quote path on real payloads. It is goal-clause (a) — code↔data — made falsifiable at the only scale today's language honestly supports.

## Work (test-first; realized under SPEC-0014 as R-0014 AC3's discharge)
1. Materialize the literal Eml trees for +, −, × from AllEle.
2. Build NAND = 1 − exp(ln a + ln b) as one tree and run the 4-row truth table through `ufl_core::eval`, including the ln 0 = −∞ edge (R-0001 AC3) — porting experiments/nand-embedding.py's assertions to a Rust e2e test in crates/ufl-core (or a small ufl-universality test crate).
3. Run AC3's second probe: one matmul entry as an eml tree, checked against the exact i64 verifier.
4. Close the ledger row in theory/universal-computability.md either way.

## Acceptance gate (falsifiable)
- The 4-row truth table exact through the complex evaluator within the documented tolerance; AND(0,b)=0 arriving via exp(−∞) with no trap.
- The i64 matmul entry matched, OR a documented precision/branch leak — per R-0014 AC3 a documented leak is a valid negative result; either way the ledger row closes (no silent middle).
- cargo test/clippy/fmt green.

## Must NOT claim
Control universality ("standard theory, but unbuilt" — §6), self-hosting, or that eml is a programming language in the control sense — the class stays elementary until T13.

## Files/crates
crates/ufl-core/tests/ (new e2e) or a small test crate, experiments/nand-embedding.py (source of assertions), theory/universal-computability.md (§6/§7.1 row closure), requirements/0014-discovery-framework.md (AC3).
