// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: 2026 Jason Lynch <jason@aexoden.com>

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Column, button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Subscription, Theme};

const BUTTON_FONT_SIZE: f32 = 22.0;
const BUTTON_PADDING: f32 = 16.0;
const DISPLAY_EXPRESSION_FONT_SIZE: f32 = 16.0;
const DISPLAY_RESULT_FONT_SIZE: f32 = 36.0;
const HISTORY_EXPRESSION_FONT_SIZE: f32 = 14.0;
const HISTORY_RESULT_FONT_SIZE: f32 = 16.0;
const KEYPAD_HEIGHT: f32 = 280.0;
const MAX_VISIBLE_HISTORY: usize = 50;
const SPACING: f32 = 4.0;

const RESULT_TEXT_COLOR: [f32; 3] = [0.8, 0.8, 0.85];
const EXPRESSION_TEXT_COLOR: [f32; 3] = [0.6, 0.6, 0.65];

#[derive(Debug, Clone)]
struct HistoryEntry {
    expression: String,
    result: String,
}

#[derive(Debug, Clone)]
enum Message {
    Unimplemented,
}

#[derive(Default)]
struct App {
    history: Vec<HistoryEntry>,
}

impl App {
    #[expect(clippy::unused_self)]
    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    #[expect(clippy::unused_self)]
    const fn theme(&self) -> Theme {
        Theme::TokyoNightStorm
    }

    #[expect(clippy::unused_self)]
    const fn update(&mut self, _message: Message) {}

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

    #[expect(clippy::unused_self)]
    fn view_display(&self) -> Element<'_, Message> {
        let expression_text = text("123 + 456")
            .size(DISPLAY_EXPRESSION_FONT_SIZE)
            .color(EXPRESSION_TEXT_COLOR)
            .width(Fill)
            .align_x(Horizontal::Right);

        let display_text = text("579")
            .size(DISPLAY_RESULT_FONT_SIZE)
            .font(Font::MONOSPACE)
            .color(RESULT_TEXT_COLOR)
            .width(Fill)
            .align_x(Horizontal::Right);

        container(column![expression_text, display_text])
            .width(Fill)
            .into()
    }

    fn view_history(&self) -> Element<'_, Message> {
        let entries: Vec<Element<'_, Message>> = self
            .history
            .iter()
            .rev()
            .take(MAX_VISIBLE_HISTORY)
            .rev()
            .flat_map(|entry| {
                [
                    text(&entry.expression)
                        .size(HISTORY_EXPRESSION_FONT_SIZE)
                        .color(EXPRESSION_TEXT_COLOR)
                        .width(Fill)
                        .align_x(Horizontal::Right)
                        .into(),
                    text(&entry.result)
                        .size(HISTORY_RESULT_FONT_SIZE)
                        .color(RESULT_TEXT_COLOR)
                        .width(Fill)
                        .align_x(Horizontal::Right)
                        .into(),
                ]
            })
            .collect();

        let history_column = Column::with_children(entries).spacing(2).padding(4);

        container(scrollable(history_column).width(Fill).anchor_bottom())
            .height(Fill)
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
                make_button("C", Message::Unimplemented),
                make_button("CE", Message::Unimplemented),
                make_button("%", Message::Unimplemented),
                make_button("÷", Message::Unimplemented)
            ]
            .spacing(SPACING),
            row![
                make_button("7", Message::Unimplemented),
                make_button("8", Message::Unimplemented),
                make_button("9", Message::Unimplemented),
                make_button("×", Message::Unimplemented)
            ]
            .spacing(SPACING),
            row![
                make_button("4", Message::Unimplemented),
                make_button("5", Message::Unimplemented),
                make_button("6", Message::Unimplemented),
                make_button("−", Message::Unimplemented)
            ]
            .spacing(SPACING),
            row![
                make_button("1", Message::Unimplemented),
                make_button("2", Message::Unimplemented),
                make_button("3", Message::Unimplemented),
                make_button("+", Message::Unimplemented)
            ]
            .spacing(SPACING),
            row![
                make_button("0", Message::Unimplemented),
                make_button(".", Message::Unimplemented),
                make_button("±", Message::Unimplemented),
                make_button("=", Message::Unimplemented)
            ]
            .spacing(SPACING),
        ]
        .spacing(SPACING);

        container(rows).height(KEYPAD_HEIGHT).width(Fill).into()
    }
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
