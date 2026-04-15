// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: 2026 Jason Lynch <jason@aexoden.com>

use std::collections::VecDeque;
use std::fmt;

use big_rational_str::BigRationalExt as _;
use iced::alignment::{Horizontal, Vertical};
use iced::keyboard;
use iced::widget::{Column, button, column, container, row, scrollable, text};
use iced::{Background, Border, Color, Element, Fill, Font, Subscription, Theme};
use num::{BigRational, Zero as _};
use thiserror::Error;

const BUTTON_FONT_SIZE: f32 = 22.0;
const BUTTON_PADDING: f32 = 16.0;
const DISPLAY_EXPRESSION_FONT_SIZE: f32 = 16.0;
const DISPLAY_RESULT_FONT_SIZE: f32 = 36.0;
const HISTORY_EXPRESSION_FONT_SIZE: f32 = 14.0;
const HISTORY_RESULT_FONT_SIZE: f32 = 16.0;
const KEYPAD_HEIGHT: f32 = 280.0;
const MAX_VISIBLE_HISTORY: usize = 50;
const SPACING: f32 = 4.0;

const HISTORY_BACKGROUND: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.15);
const RESULT_TEXT_COLOR: [f32; 3] = [0.8, 0.8, 0.85];
const EXPRESSION_TEXT_COLOR: [f32; 3] = [0.6, 0.6, 0.65];

#[derive(Debug, Clone)]
struct HistoryEntry {
    expression: String,
    result: String,
}

#[derive(Debug, Error)]
enum MathError {
    #[error("division by zero")]
    DivisionByZero,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Answer,
    Backspace,
    Clear,
    ClearEntry,
    Decimal,
    Digit(char),
    Equals,
    Negate,
    Operator(Operator),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Operator {
    Add,
    Divide,
    Multiply,
    Subtract,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Divide => write!(f, "÷"),
            Self::Multiply => write!(f, "×"),
            Self::Subtract => write!(f, "−"),
        }
    }
}

#[derive(Debug)]
enum DisplayState {
    Editing(String),
    Error,
    Result(BigRational),
}

impl Default for DisplayState {
    fn default() -> Self {
        Self::Editing(String::new())
    }
}

#[derive(Debug)]
struct PendingOperation {
    left: BigRational,
    operator: Operator,
}

#[derive(Default)]
struct App {
    display: DisplayState,
    history: VecDeque<HistoryEntry>,
    last_expression: String,
    last_result: Option<BigRational>,
    pending: Option<PendingOperation>,
}

impl App {
    fn answer(&mut self) {
        if let Some(result) = &self.last_result {
            self.display = DisplayState::Result(result.clone());
        }
    }

    fn apply_operator(
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

    fn backspace(&mut self) {
        if let DisplayState::Editing(value) = &mut self.display {
            value.pop();
        }
    }

    fn clear(&mut self) {
        self.display = DisplayState::default();
        self.last_expression.clear();
        self.last_result = None;
        self.pending = None;
    }

    fn clear_entry(&mut self) {
        if let DisplayState::Editing(value) = &mut self.display {
            value.clear();
        } else {
            self.display = DisplayState::default();
        }
    }

    fn current_value(&self) -> BigRational {
        match &self.display {
            DisplayState::Editing(value) => BigRational::from_dec_str(value)
                .unwrap_or_else(|_| BigRational::from_integer(0.into())),
            DisplayState::Error => BigRational::from_integer(0.into()),
            DisplayState::Result(value) => value.clone(),
        }
    }

    fn display_value(&self) -> String {
        match &self.display {
            DisplayState::Editing(value) => value.clone().replacen('-', "\u{2212}", 1),
            DisplayState::Error => "Error".to_owned(),
            DisplayState::Result(value) => format_rational(value),
        }
    }

    fn ensure_editing(&mut self) {
        if matches!(self.display, DisplayState::Result(_)) {
            self.display = DisplayState::default();
        }
    }

    fn evaluate(&mut self) {
        if let Some(pending) = &self.pending {
            let right = self.current_value();
            let expression = format!("{}{} =", self.last_expression, format_rational(&right));

            let result = Self::apply_operator(pending.left.clone(), pending.operator, right);

            let result_str = if let Ok(result_value) = &result {
                let result_str = format_rational(result_value);
                self.display = DisplayState::Result(result_value.clone());
                self.last_result = Some(result_value.clone());

                result_str
            } else {
                self.display = DisplayState::Error;

                String::from("Error")
            };

            self.history.push_back(HistoryEntry {
                expression: expression.clone(),
                result: result_str,
            });

            if self.history.len() > MAX_VISIBLE_HISTORY {
                self.history.pop_front();
            }

            self.last_expression = expression;
            self.pending = None;
        }
    }

    fn input_decimal(&mut self) {
        self.ensure_editing();

        if let DisplayState::Editing(value) = &mut self.display
            && !value.contains('.')
        {
            if value.is_empty() {
                value.push('0');
            }

            value.push('.');
        }
    }

    fn input_digit(&mut self, c: char) {
        if !c.is_ascii_digit() {
            return;
        }

        self.ensure_editing();

        if let DisplayState::Editing(value) = &mut self.display {
            if value == "0" {
                value.clear();
            }

            value.push(c);
        }
    }

    fn input_operator(&mut self, op: Operator) {
        if let Some(pending) = &mut self.pending {
            if matches!(&self.display, DisplayState::Editing(v) if v.is_empty()) {
                pending.operator = op;
                self.last_expression = format!("{} {op} ", format_rational(&pending.left));
                return;
            }

            self.evaluate();
        }

        let current = self.current_value();
        self.last_expression = format!("{} {op} ", format_rational(&current));
        self.pending = Some(PendingOperation {
            left: current,
            operator: op,
        });

        self.display = DisplayState::Editing(String::new());
    }

    fn negate(&mut self) {
        if let DisplayState::Result(value) = &self.display {
            let negated_value = -value.clone();

            if self.pending.is_none() {
                let result_str = format_rational(&negated_value);
                let expression = format!("\u{2212}({}) =", format_rational(value));

                self.history.push_back(HistoryEntry {
                    expression: expression.clone(),
                    result: result_str,
                });

                if self.history.len() > MAX_VISIBLE_HISTORY {
                    self.history.pop_front();
                }

                self.last_expression = expression;
                self.last_result = Some(negated_value.clone());
            }

            self.display = DisplayState::Result(negated_value);
        } else if let DisplayState::Editing(value) = &mut self.display {
            if let Some(stripped) = value.strip_prefix('-') {
                *value = stripped.to_owned();
            } else if value != "0" {
                value.insert(0, '-');
            }
        }
    }

    #[expect(clippy::unused_self)]
    fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().filter_map(|event| {
            let keyboard::Event::KeyPressed {
                modified_key,
                repeat: false,
                ..
            } = event
            else {
                return None;
            };

            match modified_key.as_ref() {
                keyboard::Key::Character("0") => Some(Message::Digit('0')),
                keyboard::Key::Character("1") => Some(Message::Digit('1')),
                keyboard::Key::Character("2") => Some(Message::Digit('2')),
                keyboard::Key::Character("3") => Some(Message::Digit('3')),
                keyboard::Key::Character("4") => Some(Message::Digit('4')),
                keyboard::Key::Character("5") => Some(Message::Digit('5')),
                keyboard::Key::Character("6") => Some(Message::Digit('6')),
                keyboard::Key::Character("7") => Some(Message::Digit('7')),
                keyboard::Key::Character("8") => Some(Message::Digit('8')),
                keyboard::Key::Character("9") => Some(Message::Digit('9')),
                keyboard::Key::Character("+") => Some(Message::Operator(Operator::Add)),
                keyboard::Key::Character("-") => Some(Message::Operator(Operator::Subtract)),
                keyboard::Key::Character("*") => Some(Message::Operator(Operator::Multiply)),
                keyboard::Key::Character("/") => Some(Message::Operator(Operator::Divide)),
                keyboard::Key::Character(".") => Some(Message::Decimal),
                keyboard::Key::Character("=")
                | keyboard::Key::Named(keyboard::key::Named::Enter) => Some(Message::Equals),
                keyboard::Key::Character("x") => Some(Message::Answer),
                keyboard::Key::Named(keyboard::key::Named::Backspace) => Some(Message::Backspace),
                keyboard::Key::Named(keyboard::key::Named::Escape) => Some(Message::Clear),
                keyboard::Key::Named(keyboard::key::Named::Delete) => Some(Message::ClearEntry),
                keyboard::Key::Named(keyboard::key::Named::F9) => Some(Message::Negate),
                keyboard::Key::Named(_)
                | keyboard::Key::Character(_)
                | keyboard::Key::Unidentified => None,
            }
        })
    }

    #[expect(clippy::unused_self)]
    const fn theme(&self) -> Theme {
        Theme::TokyoNightStorm
    }

    fn update(&mut self, message: Message) {
        if matches!(self.display, DisplayState::Error) && !matches!(message, Message::Clear) {
            return;
        }

        match message {
            Message::Answer => self.answer(),
            Message::Backspace => self.backspace(),
            Message::Clear => self.clear(),
            Message::ClearEntry => self.clear_entry(),
            Message::Digit(c) => self.input_digit(c),
            Message::Decimal => self.input_decimal(),
            Message::Equals => self.evaluate(),
            Message::Negate => self.negate(),
            Message::Operator(op) => self.input_operator(op),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = column![
            self.view_history(),
            self.view_display(),
            Self::view_keypad()
        ]
        .spacing(SPACING)
        .padding(8);

        container(content).width(Fill).height(Fill).into()
    }

    fn view_display(&self) -> Element<'_, Message> {
        let expression_text = text(&self.last_expression)
            .size(DISPLAY_EXPRESSION_FONT_SIZE)
            .font(Font::MONOSPACE)
            .color(EXPRESSION_TEXT_COLOR)
            .width(Fill)
            .align_x(Horizontal::Right);

        let display_text = container(
            scrollable(
                text(self.display_value())
                    .size(DISPLAY_RESULT_FONT_SIZE)
                    .font(Font::MONOSPACE)
                    .color(RESULT_TEXT_COLOR),
            )
            .horizontal()
            .anchor_right()
            .spacing(4),
        )
        .align_right(Fill)
        .height(60.0);

        container(
            column![expression_text, display_text]
                .spacing(4)
                .padding(12),
        )
        .width(Fill)
        .into()
    }

    fn view_history(&self) -> Element<'_, Message> {
        let entries: Vec<Element<'_, Message>> = self
            .history
            .iter()
            .flat_map(|entry| {
                [
                    text(&entry.expression)
                        .size(HISTORY_EXPRESSION_FONT_SIZE)
                        .font(Font::MONOSPACE)
                        .color(EXPRESSION_TEXT_COLOR)
                        .width(Fill)
                        .align_x(Horizontal::Right)
                        .into(),
                    text(&entry.result)
                        .size(HISTORY_RESULT_FONT_SIZE)
                        .font(Font::MONOSPACE)
                        .color(RESULT_TEXT_COLOR)
                        .width(Fill)
                        .align_x(Horizontal::Right)
                        .into(),
                ]
            })
            .collect();

        let history_column = Column::with_children(entries).spacing(2).padding(4);

        container(
            scrollable(history_column)
                .spacing(4)
                .width(Fill)
                .anchor_bottom(),
        )
        .height(Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(HISTORY_BACKGROUND)),
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..container::Style::default()
        })
        .padding(4)
        .into()
    }

    fn view_keypad<'a>() -> Element<'a, Message> {
        let make_button = |label: &'a str, msg: Message| -> Element<'a, Message> {
            button(
                text(label)
                    .size(BUTTON_FONT_SIZE)
                    .width(Fill)
                    .height(Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center),
            )
            .on_press(msg)
            .padding(BUTTON_PADDING)
            .width(Fill)
            .height(Fill)
            .into()
        };

        let rows = column![
            row![
                make_button("C", Message::Clear),
                make_button("CE", Message::ClearEntry),
                make_button("Ans", Message::Answer),
                make_button("÷", Message::Operator(Operator::Divide))
            ]
            .spacing(SPACING),
            row![
                make_button("7", Message::Digit('7')),
                make_button("8", Message::Digit('8')),
                make_button("9", Message::Digit('9')),
                make_button("×", Message::Operator(Operator::Multiply))
            ]
            .spacing(SPACING),
            row![
                make_button("4", Message::Digit('4')),
                make_button("5", Message::Digit('5')),
                make_button("6", Message::Digit('6')),
                make_button("−", Message::Operator(Operator::Subtract))
            ]
            .spacing(SPACING),
            row![
                make_button("1", Message::Digit('1')),
                make_button("2", Message::Digit('2')),
                make_button("3", Message::Digit('3')),
                make_button("+", Message::Operator(Operator::Add))
            ]
            .spacing(SPACING),
            row![
                make_button("0", Message::Digit('0')),
                make_button(".", Message::Decimal),
                make_button("±", Message::Negate),
                make_button("=", Message::Equals)
            ]
            .spacing(SPACING),
        ]
        .spacing(SPACING);

        container(rows).height(KEYPAD_HEIGHT).width(Fill).into()
    }
}

// TODO: This is a placeholder. We need to support a max digit limit and add visually distinct rounding
fn format_rational(value: &BigRational) -> String {
    value.to_dec_string().replacen('-', "\u{2212}", 1)
}

/// Launches the Exactulator GUI application.
///
/// This initializes an [Iced](https://iced.rs) application with the default
/// [`App`] state and runs the event loop, blocking until the window is closed.
///
/// # Errors
///
/// Returns an error if the application fails to start (e.g. the windowing
/// system is unavailable).
///
/// # Examples
///
/// ```no_run
/// use exactulator::gui;
///
/// fn main() -> iced::Result {
///     gui::run()
/// }
/// ```
pub fn run() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .subscription(App::subscription)
        .title("Exactulator")
        .theme(App::theme)
        .window_size((380.0, 600.0))
        .run()
}

#[cfg(test)]
mod tests {
    use super::*;

    //
    // Helper functions
    //

    /// Converts a string of digits and `.` into a sequence of messages.
    fn input(s: &str) -> Vec<Message> {
        s.chars()
            .map(|c| match c {
                '.' => Message::Decimal,
                '0'..='9' => Message::Digit(c),
                _ => panic!("invalid character in input: {c}"),
            })
            .collect()
    }

    /// Runs a sequence of messages through a fresh `App` and returns it.
    fn run_app(messages: &[Message]) -> App {
        let mut app = App::default();
        for &msg in messages {
            app.update(msg);
        }
        app
    }

    /// Runs a sequence of messages and returns the display string.
    fn eval(messages: &[Message]) -> String {
        run_app(messages).display_value()
    }

    /// Builds a binary operation (left op right =) and returns the display string.
    fn simple_binop(left: &str, op: Operator, right: &str) -> String {
        let mut msgs: Vec<Message> = input(left);
        msgs.push(Message::Operator(op));
        msgs.extend(input(right));
        msgs.push(Message::Equals);
        eval(&msgs)
    }

    fn rational(num: i64, den: i64) -> BigRational {
        BigRational::new(num.into(), den.into())
    }

    //
    // format_rational
    //

    #[test]
    fn format_rational_zero() {
        assert_eq!(format_rational(&rational(0, 1)), "0");
    }

    #[test]
    fn format_rational_integer() {
        assert_eq!(format_rational(&rational(42, 1)), "42");
    }

    #[test]
    fn format_rational_negative_integer() {
        assert_eq!(format_rational(&rational(-7, 1)), "\u{2212}7");
    }

    // TODO: Update this test once repeating decimal format is determined
    #[test]
    fn format_rational_repeating_fraction() {
        let s = format_rational(&rational(1, 3));
        assert!(s.starts_with("0."), "expected decimal, got {s}");
    }

    //
    // apply_operator (pure function tests with unique values from integration tests)
    //

    #[test]
    fn apply_operator_basic_cases() {
        #[expect(
            clippy::type_complexity,
            reason = "tuple of test cases (left, operator, right, expected)"
        )]
        let cases: &[(i64, Operator, i64, Option<(i64, i64)>)] = &[
            (3, Operator::Add, 4, Some((7, 1))),
            (10, Operator::Subtract, 7, Some((3, 1))),
            (8, Operator::Multiply, 9, Some((72, 1))),
            (10, Operator::Divide, 4, Some((5, 2))),
            (1, Operator::Divide, 0, None),
        ];

        for &(l, op, r, expected) in cases {
            let result = App::apply_operator(rational(l, 1), op, rational(r, 1));
            match expected {
                Some((num, den)) => {
                    assert_eq!(
                        result.unwrap(),
                        rational(num, den),
                        "{l} {op} {r} should be {num}/{den}"
                    );
                }
                None => assert!(result.is_err(), "{l} {op} {r} should be an error"),
            }
        }
    }

    #[test]
    fn apply_operator_exact_fraction_addition() {
        let result = App::apply_operator(rational(1, 3), Operator::Add, rational(1, 6)).unwrap();
        assert_eq!(result, rational(1, 2));
    }

    //
    // Arithmetic integration tests (through the UI message pipeline)
    //

    #[test]
    fn arithmetic_basic_operations() {
        let cases: &[(&str, Operator, &str, &str)] = &[
            ("2", Operator::Add, "3", "5"),
            ("2.5", Operator::Add, "3.12", "5.62"),
            ("9", Operator::Subtract, "4", "5"),
            ("6", Operator::Multiply, "7", "42"),
            ("2.5", Operator::Multiply, "3.1", "7.75"),
            ("10", Operator::Divide, "2", "5"),
            ("1", Operator::Divide, "4", "0.25"),
            ("5", Operator::Divide, "0", "Error"),
        ];

        for &(left, op, right, expected) in cases {
            assert_eq!(
                simple_binop(left, op, right),
                expected,
                "{left} {op} {right} should be {expected}"
            );
        }
    }

    #[test]
    fn arithmetic_with_negated_input() {
        let mut msgs = input("15");
        msgs.push(Message::Negate);
        msgs.push(Message::Operator(Operator::Add));
        msgs.extend(input("5"));
        msgs.push(Message::Equals);
        assert_eq!(eval(&msgs), "\u{2212}10");
    }

    // TODO: This test should be updated once we determine the behavior for repeating decimals
    #[test]
    fn division_repeating() {
        let result = simple_binop("1", Operator::Divide, "3");
        assert!(result.starts_with("0."), "expected decimal, got {result}");
        assert!(result.contains('3'), "expected repeating 3s, got {result}");
    }

    //
    // Input tests
    //

    #[test]
    fn multi_digit_input() {
        assert_eq!(eval(&input("123")), "123");
    }

    #[test]
    fn decimal_input() {
        assert_eq!(eval(&input("1.5")), "1.5");
    }

    #[test]
    fn leading_decimal_input() {
        assert_eq!(eval(&input(".5")), "0.5");
    }

    #[test]
    fn duplicate_decimal_ignored() {
        assert_eq!(eval(&input("1.2.3")), "1.23");
    }

    #[test]
    fn leading_zero_replaced_by_digit() {
        assert_eq!(eval(&input("05")), "5");
    }

    #[test]
    fn invalid_digit_ignored() {
        let msgs = vec![
            Message::Digit('1'),
            Message::Digit('x'),
            Message::Digit('2'),
        ];
        assert_eq!(eval(&msgs), "12");
    }

    //
    // Negation
    //

    #[test]
    fn negate_positive_input() {
        let mut msgs = input("5");
        msgs.push(Message::Negate);
        assert_eq!(eval(&msgs), "\u{2212}5");
    }

    #[test]
    fn negate_result() {
        let mut msgs = input("3");
        msgs.push(Message::Operator(Operator::Add));
        msgs.extend(input("4"));
        msgs.push(Message::Equals);
        msgs.push(Message::Negate);
        assert_eq!(eval(&msgs), "\u{2212}7");
    }

    #[test]
    fn double_negate_cancels() {
        let mut msgs = input("5");
        msgs.push(Message::Negate);
        msgs.push(Message::Negate);
        assert_eq!(eval(&msgs), "5");
    }

    #[test]
    fn negate_zero_is_noop() {
        let mut msgs = input("0");
        msgs.push(Message::Negate);
        assert_eq!(eval(&msgs), "0");
    }

    #[test]
    fn negate_empty_input() {
        let msgs = vec![Message::Negate];
        assert_eq!(eval(&msgs), "\u{2212}");
    }

    //
    // Clear
    //

    #[test]
    fn clear_resets_everything() {
        let mut msgs = input("5");
        msgs.push(Message::Operator(Operator::Add));
        msgs.extend(input("3"));
        msgs.push(Message::Clear);
        assert_eq!(eval(&msgs), "");
    }

    #[test]
    fn clear_entry_only_resets_current_input() {
        let mut msgs = input("5");
        msgs.push(Message::Operator(Operator::Add));
        msgs.push(Message::Digit('3'));
        msgs.push(Message::ClearEntry);
        msgs.push(Message::Digit('7'));
        msgs.push(Message::Equals);
        assert_eq!(eval(&msgs), "12");
    }

    #[test]
    fn clear_entry_on_result_resets_display() {
        let app = run_app(&[
            Message::Digit('2'),
            Message::Operator(Operator::Add),
            Message::Digit('3'),
            Message::Equals,
            Message::ClearEntry,
        ]);
        assert_eq!(app.display_value(), "");
    }

    //
    // Backspace
    //

    #[test]
    fn backspace_removes_last_digit() {
        let msgs = vec![
            Message::Digit('1'),
            Message::Digit('2'),
            Message::Digit('3'),
            Message::Backspace,
        ];
        assert_eq!(eval(&msgs), "12");
    }

    #[test]
    fn backspace_on_empty_is_noop() {
        let msgs = vec![Message::Backspace];
        assert_eq!(eval(&msgs), "");
    }

    #[test]
    fn backspace_on_result_is_noop() {
        let app = run_app(&[
            Message::Digit('2'),
            Message::Operator(Operator::Add),
            Message::Digit('3'),
            Message::Equals,
            Message::Backspace,
        ]);
        assert_eq!(app.display_value(), "5");
    }

    #[test]
    fn backspace_removes_decimal_point() {
        let msgs = vec![Message::Digit('1'), Message::Decimal, Message::Backspace];
        assert_eq!(eval(&msgs), "1");
    }

    #[test]
    fn backspace_on_negated_removes_last_char() {
        let msgs = vec![
            Message::Digit('1'),
            Message::Digit('2'),
            Message::Negate,
            Message::Backspace,
        ];
        // "-12" → backspace → "-1"
        assert_eq!(eval(&msgs), "\u{2212}1");
    }

    #[test]
    fn backspace_all_digits_leaves_empty() {
        let msgs = vec![Message::Digit('5'), Message::Backspace];
        assert_eq!(eval(&msgs), "");
    }

    //
    // Chained operations
    //

    #[test]
    fn chained_addition() {
        let msgs = vec![
            Message::Digit('1'),
            Message::Operator(Operator::Add),
            Message::Digit('2'),
            Message::Operator(Operator::Add),
            Message::Digit('3'),
            Message::Equals,
        ];
        assert_eq!(eval(&msgs), "6");
    }

    #[test]
    fn chained_mixed_operators() {
        // 2 + 3 × 4 = should evaluate left-to-right: (2+3)×4 = 20
        let msgs = vec![
            Message::Digit('2'),
            Message::Operator(Operator::Add),
            Message::Digit('3'),
            Message::Operator(Operator::Multiply),
            Message::Digit('4'),
            Message::Equals,
        ];
        assert_eq!(eval(&msgs), "20");
    }

    #[test]
    fn operator_change() {
        let msgs = vec![
            Message::Digit('5'),
            Message::Operator(Operator::Add),
            Message::Operator(Operator::Subtract),
            Message::Digit('3'),
            Message::Equals,
        ];
        assert_eq!(eval(&msgs), "2");
    }

    #[test]
    fn result_used_as_left_operand() {
        // 2 + 3 = 5, then × 4 = should give 20
        let app = run_app(&[
            Message::Digit('2'),
            Message::Operator(Operator::Add),
            Message::Digit('3'),
            Message::Equals,
            Message::Operator(Operator::Multiply),
            Message::Digit('4'),
            Message::Equals,
        ]);
        assert_eq!(app.display_value(), "20");
    }

    //
    // Answer recall
    //

    #[test]
    fn answer_recall() {
        let app = run_app(&[
            // 2 + 3 = 5
            Message::Digit('2'),
            Message::Operator(Operator::Add),
            Message::Digit('3'),
            Message::Equals,
            // 9 - Ans =
            Message::Digit('9'),
            Message::Operator(Operator::Subtract),
            Message::Answer,
            Message::Equals,
        ]);
        assert_eq!(app.display_value(), "4");
    }

    #[test]
    fn answer_with_no_prior_result() {
        // Ans before any computation should not change the display
        let app = run_app(&[Message::Answer]);
        assert_eq!(app.display_value(), "");
    }

    //
    // Equals with no pending operation
    //

    #[test]
    fn equals_with_no_pending_op_is_noop() {
        let app = run_app(&[Message::Digit('5'), Message::Equals]);
        assert_eq!(app.display_value(), "5");
    }

    //
    // Error state
    //

    #[test]
    fn error_state_ignores_input_until_clear() {
        let mut app = run_app(&[
            Message::Digit('1'),
            Message::Operator(Operator::Divide),
            Message::Digit('0'),
            Message::Equals,
        ]);
        assert_eq!(app.display_value(), "Error");

        // Further input should be ignored
        app.update(Message::Digit('5'));
        assert_eq!(app.display_value(), "Error");

        // Clear should recover
        app.update(Message::Clear);
        assert_eq!(app.display_value(), "");
    }

    //
    // Exact rational arithmetic
    //

    #[test]
    fn exact_multiplication_avoids_floating_point_error() {
        assert_eq!(simple_binop("0.1", Operator::Multiply, "10"), "1");
    }

    #[test]
    fn exact_division_preserves_fractions() {
        // 1 / 7 × 7 should be exactly 1 (not 0.999...)
        let app = run_app(&[
            Message::Digit('1'),
            Message::Operator(Operator::Divide),
            Message::Digit('7'),
            Message::Operator(Operator::Multiply),
            Message::Digit('7'),
            Message::Equals,
        ]);
        assert_eq!(app.display_value(), "1");
    }

    //
    // Negate result with pending operation (no history entry)
    //

    #[test]
    fn negate_result_with_pending_does_not_add_history() {
        // 3 + 4 = 7, negate to -7. Negate on result with no pending adds history.
        let app = run_app(&[
            Message::Digit('3'),
            Message::Operator(Operator::Add),
            Message::Digit('4'),
            Message::Equals,
            Message::Negate,
        ]);
        // Should have 2 history entries: the original "3+4 =" and the "−(7) ="
        assert_eq!(app.history.len(), 2);

        // Now test: if there's a pending op, negate should NOT add history
        let app2 = run_app(&[
            Message::Digit('3'),
            Message::Operator(Operator::Add),
            Message::Digit('4'),
            Message::Equals,
            Message::Operator(Operator::Add),
            Message::Negate,
        ]);
        // Only 1 history entry: the original "3+4 ="
        assert_eq!(app2.history.len(), 1);
    }

    //
    // History
    //

    #[test]
    fn history_entry_added_on_evaluate() {
        let app = run_app(&[
            Message::Digit('2'),
            Message::Operator(Operator::Add),
            Message::Digit('3'),
            Message::Equals,
        ]);
        assert_eq!(app.history.len(), 1);
        assert_eq!(app.history.front().unwrap().result, "5");
    }

    #[test]
    fn history_respects_max_visible_limit() {
        let mut app = App::default();
        for _ in 0..MAX_VISIBLE_HISTORY + 5 {
            for &msg in &[
                Message::Digit('1'),
                Message::Operator(Operator::Add),
                Message::Digit('1'),
                Message::Equals,
                Message::Clear,
            ] {
                app.update(msg);
            }
        }
        assert_eq!(app.history.len(), MAX_VISIBLE_HISTORY);
    }
}
