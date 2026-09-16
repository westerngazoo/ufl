# Flujos — ufl

## Flujo principal (Mermaid)

```mermaid
flowchart TB
SEXPR[s-expr] --> SYN[ufl-syntax]
SYN --> EML[ufl-core eval]
EML --> PRED[ufl-predicate]
PRED --> DISC[ufl-discovery]
GEO[ufl-geo GeoExpr] --> EVO[ufl-evolve]
EVO --> PRED
```

## Descripción paso a paso

1. **Syntax** — s-expr reader → `ufl-syntax` lowers to EML tree.
2. **Eval** — `ufl-core` evaluates over `Complex<f64>` log-domain arithmetic.
3. **Predicates** — Wrap in ⟦P⟧ → `ufl-predicate` checks pre/post state.
4. **Evolution** — `ufl-evolve` proposes GeoExpr mutations; verifier is honest gate.

## Secuencia (PlantUML)

Fuente: [`diagrams/flow-sequence.puml`](./diagrams/flow-sequence.puml)

```plantuml
@startuml
title ufl — secuencia principal

participant "Syntax" as Syntax0
participant "Eval" as Eval1
participant "Predicates" as Predicates2
participant "Evolution" as Evolution3

Syntax0 -> Eval1: `ufl-core` evaluates over `Complex<f64>` log-domain arithmetic.
Eval1 -> Predicates2: Wrap in ⟦P⟧ → `ufl-predicate` checks pre/post state.
Predicates2 -> Evolution3: `ufl-evolve` proposes GeoExpr mutations; verifier is honest gate.

@enduml
```

## Componentes / estados (PlantUML)

Fuente: [`diagrams/flow-architecture.puml`](./diagrams/flow-architecture.puml)

```plantuml
@startuml
title ufl — flujo de componentes
start
:SEXPRs-expr;
:SYNufl-syntax;
:SYN;
:EMLufl-core;
:EML;
:PREDufl-predicate;
:PRED;
:DISCufl-discovery;
:GEOufl-geo;
:EVOufl-evolve;
:EVO;
stop

@enduml
```

## Estados y casos borde

Consulta los tests de integración y los RFC/requirements del proyecto para flujos de error y recuperación.
