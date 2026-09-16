# Recorrido del código — ufl

Guía orientada a desarrolladores para entender dónde vive cada responsabilidad.

### 1. Syntax

s-expr reader → `ufl-syntax` lowers to EML tree.

### 2. Eval

`ufl-core` evaluates over `Complex<f64>` log-domain arithmetic.

### 3. Predicates

Wrap in ⟦P⟧ → `ufl-predicate` checks pre/post state.

### 4. Evolution

`ufl-evolve` proposes GeoExpr mutations; verifier is honest gate.

## Punto de entrada recomendado

Empieza por el README del proyecto y el módulo/crate principal listado en la documentación de arquitectura.
