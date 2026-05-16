# Specs

UFL is built **spec-driven**. Before any feature is implemented, it is described
here as a numbered spec: what it must do, what it must *not* do, and how we know
it is done. The spec is the contract; the implementation references it.

## Process

1. **Draft.** A new spec is created from [`TEMPLATE.md`](TEMPLATE.md), numbered
   with the next free 4-digit id (`NNNN-short-name.md`).
2. **Discuss.** Gustavo and Claude refine requirements and acceptance criteria
   until both agree the spec is unambiguous.
3. **Accept.** The spec's status moves to `Accepted`. Only then does
   implementation begin.
4. **Implement.** Code is written to satisfy exactly the accepted spec. The
   implementation cites its spec id.
5. **Verify.** Acceptance criteria are checked. The spec status moves to
   `Implemented`.

A spec may later move to `Superseded` (replaced by a newer spec) or `Revised`
(amended in place, with a changelog entry).

## Status values

`Draft` → `Accepted` → `Implemented` · (or `Superseded` / `Revised`)

## Build order

There is no phased timeline. The dependency-ordered sequence of specs to build
is in [`ROADMAP.md`](ROADMAP.md). We work through them one at a time.

## Index

| Spec | Title | Status |
|------|-------|--------|
| _none yet_ | | |
