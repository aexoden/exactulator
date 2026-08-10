// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: 2026 Jason Lynch <jason@aexoden.com>

use std::collections::HashMap;

use num::{BigInt, BigRational, Signed as _, Zero as _};

/// Number of fractional digits used by [`FormattedRational::new`].
pub const DEFAULT_MAX_DIGITS: usize = 12;

/// A rational rendered for display.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormattedRational {
    /// Whether `text` is a rounded rendering of the true value rather than an exact one.
    pub is_approximate: bool,
    /// The rendered value, using the Unicode minus sign for negatives.
    pub text: String,
}

impl FormattedRational {
    /// Formats a rational for display using [`DEFAULT_MAX_DIGITS`] fractional digits.
    ///
    /// See [`FormattedRational::with_max_digits`] for the exact rules.
    #[must_use]
    pub fn new(value: &BigRational) -> Self {
        Self::with_max_digits(value, DEFAULT_MAX_DIGITS)
    }

    /// Formats a rational for display, emitting at most `max_digits` fractional digits.
    ///
    /// Repeating decimals are rendered using parenthesis notation (e.g., `1/3` becomes `0.(3)` and `5/12` becomes
    /// `0.41(6)`). If the decimal expansion is longer than `max_digits`, the result is rounded half away from zero
    /// and `is_approximate` is set to `true`.
    #[must_use]
    pub fn with_max_digits(value: &BigRational, max_digits: usize) -> Self {
        let sign = if value.is_negative() { "\u{2212}" } else { "" };

        let magnitude = value.abs();
        let integer_part = magnitude.numer() / magnitude.denom();

        let mut remainder = magnitude.numer() % magnitude.denom();
        let mut digits = String::new();
        let mut seen: HashMap<BigInt, usize> = HashMap::new();
        let mut repetend_start: Option<usize> = None;
        let ten = BigInt::from(10);

        loop {
            if remainder.is_zero() {
                break;
            }

            if let Some(&start) = seen.get(&remainder) {
                repetend_start = Some(start);
                break;
            }

            if digits.len() >= max_digits {
                return Self {
                    is_approximate: true,
                    text: format!("{sign}{}", round_to_digits(&magnitude, max_digits)),
                };
            }

            seen.insert(remainder.clone(), digits.len());

            remainder *= &ten;
            let digit = &remainder / magnitude.denom();
            remainder %= magnitude.denom();

            digits.push_str(&digit.to_string());
        }

        let text = match repetend_start {
            None if digits.is_empty() => format!("{sign}{integer_part}"),
            None => format!("{sign}{integer_part}.{digits}"),
            Some(start) => {
                let prefix: String = digits.chars().take(start).collect();
                let repetend: String = digits.chars().skip(start).collect();
                format!("{sign}{integer_part}.{prefix}({repetend})")
            }
        };

        Self {
            is_approximate: false,
            text,
        }
    }
}

/// Renders a non-negative rational rounded to `max_digits` places, half away from zero.
///
/// Trailing zeroes are omitted, and a value that rounds to a whole number renders without
/// a decimal point.
///
/// The caller is responsible for the sign; the rounding here is applied to the magnitude,
/// which makes it half away from zero for negative values too.
fn round_to_digits(magnitude: &BigRational, max_digits: usize) -> String {
    let scale = num::pow(BigInt::from(10), max_digits);
    let denominator = magnitude.denom();

    let scaled = (magnitude.numer() * &scale * 2 + denominator) / (denominator * 2);

    let integer_part = &scaled / &scale;
    let fraction_part = scaled % &scale;
    let padded = format!("{fraction_part:0>max_digits$}");
    let digits = padded.trim_end_matches('0');

    if digits.is_empty() {
        format!("{integer_part}")
    } else {
        format!("{integer_part}.{digits}")
    }
}

#[cfg(test)]
mod tests {
    use num::BigRational;

    use super::{DEFAULT_MAX_DIGITS, FormattedRational};

    fn ratio(numerator: i64, denominator: i64) -> BigRational {
        BigRational::new(numerator.into(), denominator.into())
    }

    fn assert_formats(value: &BigRational, text: &str, is_approximate: bool) {
        let formatted = FormattedRational::new(value);

        assert_eq!(
            formatted,
            FormattedRational {
                is_approximate,
                text: text.to_owned(),
            }
        );
    }

    #[test]
    fn zero() {
        assert_formats(&ratio(0, 1), "0", false);
    }

    #[test]
    fn positive_integer() {
        assert_formats(&ratio(42, 1), "42", false);
    }

    #[test]
    fn negative_integer() {
        assert_formats(&ratio(-7, 1), "\u{2212}7", false);
    }

    #[test]
    fn large_integer_is_never_truncated() {
        let value = BigRational::from_integer("123456789012345678901234567890".parse().unwrap());

        assert_formats(&value, "123456789012345678901234567890", false);
    }

    #[test]
    fn terminating_decimal_within_max_digits() {
        assert_formats(&ratio(1, 8), "0.125", false);
        assert_formats(&ratio(-3, 4), "\u{2212}0.75", false);
        assert_formats(&ratio(7, 2), "3.5", false);
    }

    #[test]
    fn terminating_decimal_exactly_at_max_digits() {
        // 1/2^12 has exactly 12 fractional digits.
        assert_formats(&ratio(1, 4096), "0.000244140625", false);
    }

    #[test]
    fn terminating_decimal_longer_than_max_digits_is_rounded() {
        // 1/2^13 is 0.0001220703125: 13 fractional digits, one past the limit,
        // with the discarded tail exactly half a unit in the last place.
        assert_formats(&ratio(1, 8192), "0.000122070313", true);
    }

    #[test]
    fn repeating_third() {
        assert_formats(&ratio(1, 3), "0.(3)", false);
        assert_formats(&ratio(-1, 3), "\u{2212}0.(3)", false);
    }

    #[test]
    fn repeating_seventh() {
        assert_formats(&ratio(1, 7), "0.(142857)", false);
    }

    #[test]
    fn repetend_with_non_repeating_prefix() {
        assert_formats(&ratio(5, 12), "0.41(6)", false);
        assert_formats(&ratio(1, 6), "0.1(6)", false);
    }

    #[test]
    fn repeating_with_integer_part() {
        assert_formats(&ratio(7, 3), "2.(3)", false);
    }

    #[test]
    fn period_exactly_equal_to_max_digits_stays_exact() {
        // 1/9901 repeats with a period of exactly 12 digits.
        assert_formats(&ratio(1, 9901), "0.(000100999899)", false);
    }

    #[test]
    fn period_one_past_max_digits_is_rounded() {
        // 1/53 repeats with a period of 13 digits.
        assert_formats(&ratio(1, 53), "0.018867924528", true);

        // 1/17 is 0.0588235294117647... with a period of 16, so the last place
        // rounds up rather than truncating to ...411.
        assert_formats(&ratio(1, 17), "0.058823529412", true);
    }

    #[test]
    fn rounding_carries_into_the_integer_part() {
        assert_formats(&ratio(9_999_999_999_999, 10_000_000_000_000), "1", true);
    }

    #[test]
    fn trailing_zeros_from_rounding_are_stripped() {
        assert_formats(&ratio(4_999_999_999_999, 10_000_000_000_000), "0.5", true);
    }

    #[test]
    fn negative_values_round_away_from_zero() {
        assert_formats(&ratio(-1, 17), "\u{2212}0.058823529412", true);
    }

    #[test]
    fn large_denominator_returns_promptly() {
        let value = BigRational::new(1.into(), "99999999999999999989".parse().unwrap());

        // Rounds to zero at this digit limit, but is still not exact.
        assert_formats(&value, "0", true);
    }

    #[test]
    fn tiny_negative_keeps_its_sign_when_it_rounds_to_zero() {
        // The sign is retained deliberately: with the approximation marker, a
        // rendered "−0" says the value is a small negative rather than zero.
        let value = BigRational::new((-1).into(), "99999999999999999989".parse().unwrap());

        assert_formats(&value, "\u{2212}0", true);
    }

    #[test]
    fn max_digits_is_configurable() {
        assert_eq!(
            FormattedRational::with_max_digits(&ratio(1, 3), 3).text,
            "0.(3)"
        );
        assert_eq!(
            FormattedRational::with_max_digits(&ratio(1, 7), 4),
            FormattedRational {
                is_approximate: true,
                text: "0.1429".to_owned(),
            }
        );
        assert_eq!(
            FormattedRational::with_max_digits(&ratio(1, 53), 4),
            FormattedRational {
                is_approximate: true,
                text: "0.0189".to_owned(),
            }
        );
        assert_eq!(
            FormattedRational::with_max_digits(&ratio(1, 53), DEFAULT_MAX_DIGITS + 1),
            FormattedRational {
                is_approximate: false,
                text: "0.(0188679245283)".to_owned(),
            }
        );
    }

    #[test]
    fn zero_max_digits_rounds_to_a_whole_number() {
        assert_eq!(
            FormattedRational::with_max_digits(&ratio(7, 2), 0),
            FormattedRational {
                is_approximate: true,
                text: "4".to_owned(),
            }
        );
        assert_eq!(
            FormattedRational::with_max_digits(&ratio(4, 2), 0),
            FormattedRational {
                is_approximate: false,
                text: "2".to_owned(),
            }
        );
    }
}
