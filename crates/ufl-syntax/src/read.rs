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

/// A lexical token. Comments and whitespace are consumed during tokenization
/// and never appear here.
enum Token {
    Open,
    Close,
    Atom(String),
}

/// Split `src` into tokens, dropping whitespace and `;` line comments.
fn tokenize(src: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = src.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ';' => {
                // line comment — consume through the end of the line
                for ch in chars.by_ref() {
                    if ch == '\n' {
                        break;
                    }
                }
            }
            '(' => {
                chars.next();
                tokens.push(Token::Open);
            }
            ')' => {
                chars.next();
                tokens.push(Token::Close);
            }
            c if c.is_whitespace() => {
                chars.next();
            }
            _ => {
                let mut atom = String::new();
                while let Some(&c) = chars.peek() {
                    if is_delimiter(c) {
                        break;
                    }
                    atom.push(c);
                    chars.next();
                }
                tokens.push(Token::Atom(atom));
            }
        }
    }
    tokens
}

/// True iff `c` bounds a token: whitespace, `(`, `)`, or the `;` comment start.
/// The single source of the delimiter set (shared by the tokenizer above and
/// [`is_reader_canonical_sym`]).
fn is_delimiter(c: char) -> bool {
    c.is_whitespace() || c == '(' || c == ')' || c == ';'
}

/// Classify an atom token: a finite `f64` is `Num`, everything else `Sym`
/// (so non-finite spellings like `inf`/`nan` are symbols — §2.3).
fn classify(atom: &str) -> Sexpr {
    match atom.parse::<f64>() {
        Ok(n) if n.is_finite() => Sexpr::Num(n),
        _ => Sexpr::Sym(atom.to_string()),
    }
}

/// Is `token` a symbol the reader produces verbatim — i.e. does
/// `read(token) == Ok(Sexpr::Sym(token))`? True iff `token` is non-empty,
/// contains no delimiter (`(`, `)`, `;`, whitespace), and is **not** a
/// finite-float spelling (which the reader would classify as `Num`).
///
/// This is the reader's own `Sym` acceptance rule, exposed so a generator can
/// draw `Sym` payloads from exactly the reader's canonical image without
/// duplicating (and drifting from) the classification logic (SPEC-0016 §2.5,
/// §5 open question 2).
pub fn is_reader_canonical_sym(token: &str) -> bool {
    !token.is_empty()
        && !token.chars().any(is_delimiter)
        && matches!(classify(token), Sexpr::Sym(_))
}

/// Read exactly one top-level s-expression from `src`.
///
/// **Iterative** (R-0017): an explicit stack of partial lists, so input of any
/// nesting depth parses without recursion and **without a depth cap**. The error
/// behaviour is unchanged — in particular the single-top-level-datum rule: once
/// a top-level datum completes, *any* further token is `TrailingTokens`.
pub fn read(src: &str) -> Result<Sexpr, ReadError> {
    let tokens = tokenize(src);
    if tokens.is_empty() {
        return Err(ReadError::EmptyInput);
    }

    let mut stack: Vec<Vec<Sexpr>> = Vec::new();
    let mut result: Option<Sexpr> = None;

    for token in &tokens {
        // Once the single top-level datum is complete, nothing may follow it.
        if result.is_some() && stack.is_empty() {
            return Err(ReadError::TrailingTokens);
        }
        match token {
            Token::Open => stack.push(Vec::new()),
            Token::Atom(atom) => {
                let expr = classify(atom);
                match stack.last_mut() {
                    Some(top) => top.push(expr),
                    None => result = Some(expr),
                }
            }
            Token::Close => {
                let Some(items) = stack.pop() else {
                    return Err(ReadError::UnexpectedClose);
                };
                let list = Sexpr::List(items);
                match stack.last_mut() {
                    Some(top) => top.push(list),
                    None => result = Some(list),
                }
            }
        }
    }

    if !stack.is_empty() {
        return Err(ReadError::UnclosedList);
    }
    result.ok_or(ReadError::UnclosedList)
}
