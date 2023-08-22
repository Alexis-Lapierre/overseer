use std::collections::HashMap;

use iced::{
    executor,
    keyboard::{KeyCode, Modifiers},
    subscription,
    widget::{row, Button, Column, Container, Text, TextInput},
    window, Application, Command, Settings, Subscription, Theme,
};

mod connection;

#[derive(Debug, Default)]
struct ApplicationState {
    connections: HashMap<String, Option<connection::Result>>,
    input_address: String,
}

#[derive(Debug)]
enum Message {
    ConnectionResult(String, connection::Result),
    Interaction(Interaction),
    Exit,
}

#[derive(Debug, Clone)]
enum Interaction {
    InputAddressChanged(String),
    AddAddress,

    RemoveAddress(String),
}

impl Application for ApplicationState {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        "Overseer".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::ConnectionResult(key, resolved) => {
                if self.connections.contains_key(&key) {
                    self.connections.insert(key, Some(resolved));
                }

                Command::none()
            }
            Message::Exit => window::close(),

            Message::Interaction(interaction) => match interaction {
                Interaction::InputAddressChanged(current) => {
                    self.input_address = current;
                    Command::none()
                }

                Interaction::AddAddress => {
                    let address = self.input_address.clone();
                    self.input_address.clear();

                    self.connections.insert(address.clone(), None);

                    let async_address = address.clone();
                    Command::perform(
                        async move { connection::try_connect(&async_address) },
                        |resolved| Message::ConnectionResult(address, resolved),
                    )
                }

                Interaction::RemoveAddress(key) => {
                    self.connections.remove(&key);
                    Command::none()
                }
            },
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        subscription::events_with(|event, _status| {
            if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key_code,
                modifiers,
            }) = event
            {
                if key_code == KeyCode::Q && modifiers == Modifiers::CTRL {
                    return Some(Message::Exit);
                }
            }

            None
        })
    }

    fn view(&self) -> iced::Element<Self::Message> {
        let mut col = {
            let input = iced::Element::from(
                TextInput::new("Connect to", &self.input_address)
                    .on_input(Interaction::InputAddressChanged)
                    .on_submit(Interaction::AddAddress),
            )
            .map(Message::Interaction);

            Column::new().padding(5).push(input)
        };

        for (address, connection_state) in &self.connections {
            let connection_text: String = match connection_state {
                Some(state) => match state {
                    Ok(_) => String::from("Connected !"),
                    Err(err) => format!("{err:?}"),
                },
                None => "Connected !".into(),
            };

            let ip_text_widget =
                Text::new(format!("{address} - {connection_text}")).width(iced::Length::Fill);
            let delete_button = iced::Element::from(
                Button::new("Remove").on_press(Interaction::RemoveAddress(address.clone())),
            )
            .map(Message::Interaction);

            let row = row![ip_text_widget, delete_button].padding(5);

            col = col.push(row);
        }

        Container::new(col)
            .center_x()
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .padding(5)
            .into()
    }
}

fn main() -> Result<(), iced::Error> {
    let window_settings = iced::window::Settings {
        min_size: Some((250, 100)),
        ..Default::default()
    };

    let settings = Settings {
        window: window_settings,
        ..Default::default()
    };

    ApplicationState::run(settings)
}
