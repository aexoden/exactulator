// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: 2026 Jason Lynch <jason@aexoden.com>

use std::collections::VecDeque;
use std::fmt;

use big_rational_str::BigRationalExt as _;
use iced::alignment::{Horizontal, Vertical};
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
        Subscription::none()
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

// TODO: This is a placeholder. We need to support a max digit limit and add visually distinct rounding, as well as
// use the correct unicode minus sign.
fn format_rational(value: &BigRational) -> String {
    value.to_dec_string()
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
