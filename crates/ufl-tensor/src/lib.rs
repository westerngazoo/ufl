//! UFL — exact integer-tensor core for matmul-decomposition discovery.
//!
//! Realizes [R-0006](../../../requirements/0006-integer-tensor-core.md) per
//! [SPEC-0006](../../../specs/0006-integer-tensor-core.md). Pure integer
//! arithmetic (no `Complex<f64>` EML core): build the matmul target tensor
//! `T_n`, represent a decomposition scheme, reconstruct it, and decide exact
//! equality.
//!
//! Conventions (SPEC-0006 §2.1): `d = n²`; `M[i][j]` flattens row-major to
//! `i·n+j`; `T_n[p,q,r] = 1` iff `p=i·n+j, q=j·n+k, r=i·n+k`. The map is
//! injective, so every entry is 0/1 and `error == 0` means exact equality.

#![forbid(unsafe_code)]

mod reconstruct;
mod scheme;
mod tensor;

pub use reconstruct::{is_valid, reconstruct, scheme_error};
pub use scheme::{Scheme, SchemeError, Triple};
pub use tensor::{error, target, Tensor};
