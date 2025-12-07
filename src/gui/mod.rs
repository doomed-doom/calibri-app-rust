pub mod app;
mod components;
pub mod controller;
mod theme;

pub use app::CallibriApp;

use theme::detect_theme;

pub fn run() -> iced::Result {
    let theme = detect_theme();

    iced::daemon("Callibri GUI", CallibriApp::update, CallibriApp::view)
        .theme(move |_, _| theme.clone())
        .subscription(CallibriApp::subscription)
        .run_with(CallibriApp::init)
}
