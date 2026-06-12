# Requirements

A **requirement** states *what* UFL must do — a capability or property, from the
problem perspective, independent of implementation. Requirements are the WHAT;
[`specs/`](../specs/) are the HOW.

Every requirement is decided **together** (Gustavo + Claude) before a spec is
written, and every requirement is owned by a `qa` agent run that verifies it.

## Process

1. **Discuss.** Gustavo and Claude agree the capability and its acceptance
   criteria. See [`CLAUDE.md`](../CLAUDE.md) §4.
2. **Record.** Create a file from [`TEMPLATE.md`](TEMPLATE.md), numbered
   `R-NNNN` (next free 4-digit id): `NNNN-short-name.md`.
3. **Accept.** When acceptance criteria are unambiguous, status → `Accepted`.
   Only then may a spec realize it.
4. **Realize.** One or more `SPEC-NNNN` in `specs/` implement the requirement.
5. **Verify.** The `qa` agent, scoped to this `R-NNNN`, confirms every
   acceptance criterion. Status → `Met`.

## Status values

`Draft` → `Accepted` → `Met` · (or `Superseded`)

## Relationship to specs

A requirement links forward to the spec(s) that realize it; a spec links back to
the requirement(s) it satisfies. The mapping is maintained in
[`ROADMAP.md`](../ROADMAP.md).

## Index

| Req | Title | Milestone | Status |
|-----|-------|-----------|--------|
| [R-0001](0001-eml-operator-core.md) | EML Operator Core | M1 | Met (the typed core R-0003 lowers into) |
| [R-0002](0002-geometric-algebra-core.md) | Geometric Algebra Core over G(3,0,0) — *via garust* | M1 | Accepted (built by the GA agent flow) |
| [R-0003](0003-sexpr-core.md) | Homoiconic S-Expression Core | M2 | Met (merged, PR #11) |
| [R-0004](0004-predicate-layer.md) | Predicate Layer (`⟦P⟧`) — the Checker | M3 | Met (merged, PR #14) |
| R-0005 | Value Conditional (`if`) | M3 | **Shelved** (language-build thread paused for the discovery pivot; on branch `R-0005-value-conditional`) |
| [R-0006](0006-integer-tensor-core.md) | Exact Integer-Tensor Core (`ufl-tensor`) | M5 — Discovery | Met (merged, PR #16) |
| [R-0007](0007-tensor-predicate.md) | Tensor-equality predicate (the Hehner-discharge bridge) | M5 — Discovery | Accepted (implementation in review) |
