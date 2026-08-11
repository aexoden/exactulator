// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: 2026 Jason Lynch <jason@aexoden.com>

use std::collections::VecDeque;

use big_rational_str::BigRationalExt as _;
use num::{BigRational, Zero as _};
use thiserror::Error;

/// Maximum number of entries retained by a [`History`].
///
/// Older entries are discarded once this limit is reached.
pub const MAX_HISTORY_ENTRIES: usize = 50;

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

/// A bounded, chronological log of completed operations.
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct History {
    entries: VecDeque<HistoryEntry>,
}

impl History {
    /// Returns whether the history is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the entries, oldest first.
    pub fn iter(&self) -> impl Iterator<Item = &HistoryEntry> {
        self.entries.iter()
    }

    /// Returns the number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Appends an entry, discarding the oldest entry if [`MAX_HISTORY_ENTRIES`] is exceeded.
    pub fn push(&mut self, entry: HistoryEntry) {
        self.entries.push_back(entry);

        if self.entries.len() > MAX_HISTORY_ENTRIES {
            self.entries.pop_front();
        }
    }
}

/// A single completed operation and its result.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HistoryEntry {
    Binary {
        left: BigRational,
        operator: Operator,
        result: Result<BigRational, MathError>,
        right: BigRational,
    },
    Unary {
        operand: BigRational,
        operator: UnaryOperator,
        result: Result<BigRational, MathError>,
    },
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

/// An operator that takes a single operand.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UnaryOperator {
    Negate,
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

#[cfg(test)]
mod tests {
    use num::BigRational;

    use super::{History, HistoryEntry, MAX_HISTORY_ENTRIES, Operator};

    fn integer(value: i64) -> BigRational {
        BigRational::from_integer(value.into())
    }

    fn addition(left: i64, right: i64) -> HistoryEntry {
        HistoryEntry::Binary {
            left: integer(left),
            operator: Operator::Add,
            result: Ok(integer(left + right)),
            right: integer(right),
        }
    }

    #[test]
    fn history_starts_empty() {
        let history = History::default();
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn history_iterates_oldest_first() {
        let mut history = History::default();
        history.push(addition(1, 1));
        history.push(addition(2, 2));

        let entries: Vec<&HistoryEntry> = history.iter().collect();

        assert_eq!(entries, vec![&addition(1, 1), &addition(2, 2)]);
    }

    #[test]
    fn history_discards_oldest_entry_when_full() {
        let mut history = History::default();

        for i in 0..MAX_HISTORY_ENTRIES + 5 {
            #[expect(
                clippy::cast_possible_wrap,
                reason = "the loop bound is far below i64::MAX"
            )]
            history.push(addition(i as i64, 0));
        }

        assert_eq!(history.len(), MAX_HISTORY_ENTRIES);
        assert_eq!(history.iter().next(), Some(&addition(5, 0)));
    }
}
