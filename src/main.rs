// Hide console window on Windows (always, even in debug mode)
#![windows_subsystem = "windows"]

mod app;
mod models;
mod ui;
mod winget;

use app::AppState;
use iced::Theme;

fn main() -> iced::Result {
    let icon = ui::create_icon();

    iced::application("Windows Apps Updater", AppState::update, AppState::view)
        .theme(|_| Theme::Dark)
        .window(iced::window::Settings {
            size: iced::Size::new(1050.0, 700.0),
            min_size: Some(iced::Size::new(700.0, 450.0)),
            icon,
            ..Default::default()
        })
        .run_with(AppState::new)
}
