# Specs

A **spec** states *how* a UFL feature is built — the technical design that
realizes one or more requirements. Requirements ([`requirements/`](../requirements/))
are the WHAT; specs are the HOW.

UFL is built spec-first: before any Rust is written, the feature is described
here as a numbered spec — design, code outline, non-goals, acceptance mapping —
and reviewed by the `architect` agent.

## Process

1. **Draft.** Once the governing requirement is `Accepted`, create a spec from
   [`TEMPLATE.md`](TEMPLATE.md), numbered `SPEC-NNNN`: `NNNN-short-name.md`.
2. **Design review.** The `architect` agent reviews the design and code outline
   against the requirement (`CLAUDE.md` §4, step 2).
3. **Accept.** When the design is sound and unambiguous, status → `Accepted`.
   Only then does implementation begin.
4. **Implement.** Code satisfies exactly the accepted spec and cites its id.
5. **Verify.** Acceptance criteria are checked; status → `Implemented`.

A spec may later become `Superseded` or `Revised` (amended in place, logged).

## Status values

`Draft` → `Accepted` → `Implemented` · (or `Superseded` / `Revised`)

## Relationship to requirements

Every spec links back to the requirement(s) it realizes via its **Realizes**
field. The build order across requirements and specs is in
[`ROADMAP.md`](../ROADMAP.md).

## Index

| Spec | Title | Realizes | Status |
|------|-------|----------|--------|
| [SPEC-0001](0001-eml-operator-core.md) | EML Operator Core | R-0001 | Implemented |
| [SPEC-0003](0003-sexpr-core.md) | Homoiconic S-Expression Core | R-0003 | Implemented (merged) |

| [SPEC-0004](0004-predicate-layer.md) | Predicate Layer (`⟦P⟧`) — the Checker | R-0004 | Implemented (merged) |

| [SPEC-0006](0006-integer-tensor-core.md) | Exact Integer-Tensor Core (`ufl-tensor`) | R-0006 | Draft |

SPEC-0002 (G(3,0,0) GA core, garust-based) is written by the separate GA agent
flow against [R-0002](../requirements/0002-geometric-algebra-core.md).
