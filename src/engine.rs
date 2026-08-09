// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: 2026 Jason Lynch <jason@aexoden.com>

use big_rational_str::BigRationalExt as _;
use num::{BigRational, Zero as _};
use thiserror::Error;

#[non_exhaustive]
#[derive(Debug)]
pub enum DisplayState {
    Editing(String),
    Error,
    Result(BigRational),
}

impl Default for DisplayState {
    fn default() -> Self {
        Self::Editing(String::new())
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Eq, PartialEq, Error)]
pub enum MathError {
    #[error("division by zero")]
    DivisionByZero,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Operator {
    Add,
    Divide,
    Multiply,
    Subtract,
}

#[non_exhaustive]
#[derive(Debug)]
pub struct PendingOperation {
    pub left: BigRational,
    pub operator: Operator,
}

/// Applies a binary operator to two `BigRational` operands.
///
/// # Errors
///
/// Returns `MathError::DivisionByZero` if the operator is `Operator::Divide` and the right operand is zero.
pub fn apply_operator(
    left: BigRational,
    op: Operator,
    right: BigRational,
) -> Result<BigRational, MathError> {
    match op {
        Operator::Add => Ok(left + right),
        Operator::Divide => {
            if right.is_zero() {
                Err(MathError::DivisionByZero)
            } else {
                Ok(left / right)
            }
        }
        Operator::Multiply => Ok(left * right),
        Operator::Subtract => Ok(left - right),
    }
}

/// Parses a decimal string into a `BigRational`.
///
/// Input that is empty or otherwise unparseable yields zero.
#[must_use]
pub fn parse_decimal(value: &str) -> BigRational {
    BigRational::from_dec_str(value).unwrap_or_else(|_| BigRational::from_integer(0.into()))
}
