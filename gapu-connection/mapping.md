# GAPU ↔ UFL Mapping

> Status: stub. See proposal [§5](../docs/ufl-first-draft.md).

The GAPU three-layer architecture maps onto UFL's pillars:

| GAPU Layer | Timescale | UFL Pillar | Role |
|------------|-----------|------------|------|
| System 1 (analog reservoir) | nanoseconds | Log-Arithmetic + GA | Fast geometric transform in log-domain analog circuits |
| System 2 (digital observer) | microseconds | Hehner predicates | Constrains System 1 output; triggers correction |
| System 0 (Hamiltonian meta) | seconds–hours | Orchestrator | Recompiles substrate bindings from accumulated error |

The falsifiable conjecture — noise covariance scales as σᵏ for grade-k objects
— becomes a UFL typing rule: higher-grade expressions carry higher uncertainty
budgets and need tighter predicates to compile to hardware.

_To be formalized once specs 0003–0008 land._
