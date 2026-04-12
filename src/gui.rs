// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: 2026 Jason Lynch <jason@aexoden.com>

use iced::widget::{column, container, text};
use iced::{Element, Fill, Subscription, Theme};

const SPACING: f32 = 4.0;

#[derive(Debug, Clone)]
enum Message {}

#[derive(Default)]
struct App;

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

    #[expect(clippy::unused_self)]
    fn view(&self) -> Element<'_, Message> {
        let content = column![text("Hello, world!")].spacing(SPACING).padding(8);

        container(content).width(Fill).height(Fill).into()
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
