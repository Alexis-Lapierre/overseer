use iced::Application;

mod connection;

mod application;
use application::Application as Overseer;

fn main() -> Result<(), iced::Error> {
    let window_settings = iced::window::Settings {
        min_size: Some(iced::Size {
            width: 250.,
            height: 100.,
        }),
        ..Default::default()
    };

    let settings = iced::Settings {
        window: window_settings,
        ..Default::default()
    };

    Overseer::run(settings)
}
