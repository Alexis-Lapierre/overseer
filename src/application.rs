use std::{collections::HashMap, sync::Arc};

use iced::{
    executor,
    keyboard::{KeyCode, Modifiers},
    subscription,
    widget::{row, Button, Column, Container, Text, TextInput},
    window, Command, Subscription, Theme,
};

use crate::connection;
use connection::Connection;

type ConnectionResult = Result<Connection, connection::Error>;

#[derive(Default, Debug)]
pub struct Application {
    connections: HashMap<Arc<str>, ConnectionResult>,
    input_address: String,
}

#[derive(Debug)]
pub enum Message {
    Connect(Arc<str>, ConnectionResult),
    UserInteraction(Interaction),
    Exit,
}

#[derive(Debug, Clone)]
pub enum Interaction {
    InputAddressChanged(String),
    AddAddress,
    RemoveAddress(Arc<str>),
}

impl iced::Application for Application {
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
            Message::Connect(key, result) => {
                self.connections.insert(key, result);
                Command::none()
            }

            Message::Exit => window::close(),

            Message::UserInteraction(interaction) => match interaction {
                Interaction::InputAddressChanged(current) => {
                    self.input_address = current;
                    Command::none()
                }

                Interaction::AddAddress => {
                    let address = std::mem::take(&mut self.input_address);

                    let address: Arc<str> = address.into();
                    Command::perform(Connection::connect(address.clone()), |resolved| {
                        Message::Connect(address, resolved)
                    })
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
            .map(Message::UserInteraction);

            Column::new().padding(5).push(input)
        };

        for (address, connection_state) in &self.connections {
            let connection_text: String = match connection_state {
                Ok(connection) => format!("Connected ! - {connection:?}"),
                Err(err) => format!("{err:?}"),
            };

            let ip_text_widget =
                Text::new(format!("{address} - {connection_text}")).width(iced::Length::Fill);
            let delete_button = iced::Element::from(
                Button::new("Remove").on_press(Interaction::RemoveAddress(address.clone())),
            )
            .map(Message::UserInteraction);

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
