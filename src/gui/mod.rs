pub mod app;
pub mod controller;

pub use app::CallibriApp;

pub fn run() -> iced::Result {
    iced::application("Callibri GUI", CallibriApp::update, CallibriApp::view)
        .subscription(CallibriApp::subscription)
        .run_with(CallibriApp::init)
}
