//! The reader — text → [`Sexpr`] (SPEC-0003 §2.3).
//!
//! Tokenizer rule (normative): tokens are runs bounded by whitespace, `(`,
//! `)`, or `;` (line comment). A token that parses as a **finite** `f64`
//! becomes `Num`; every other token (including non-finite spellings like
//! `inf`/`nan`) becomes `Sym`. This closes the reader's image so the
//! round-trip invariant and `PartialEq` are total on it.

use crate::sexpr::Sexpr;

/// A failure while reading text into an [`Sexpr`] (SPEC-0003 §2.3 / AC2).
/// There is no invalid-token variant by construction: every non-delimiter run
/// is a valid `Num` or `Sym`.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ReadError {
    #[error("unbalanced parentheses: unexpected ')'")]
    UnexpectedClose,
    #[error("unbalanced parentheses: unclosed '(' at end of input")]
    UnclosedList,
    #[error("empty input — no s-expression to read")]
    EmptyInput,
    #[error("unexpected trailing tokens after the first s-expression")]
    TrailingTokens,
}

/// Read exactly one top-level s-expression from `src`.
pub fn read(_src: &str) -> Result<Sexpr, ReadError> {
    unimplemented!("R-0003 implementation — reader, see SPEC-0003 §2.3")
}
