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
struct ConnectionState {
    connection: Option<Connection>,
    interfaces: Vec<String>,
}

impl From<Connection> for ConnectionState {
    fn from(connection: Connection) -> Self {
        Self {
            connection: Some(connection),
            interfaces: Vec::default(),
        }
    }
}

#[derive(Default, Debug)]
pub struct Application {
    connections: HashMap<Arc<str>, ConnectionState>,
    failed_connection: HashMap<Arc<str>, connection::Error>,
    input_address: String,
}

#[derive(Debug)]
pub enum Message {
    Connect(Arc<str>, ConnectionResult),
    ListOfInterfaces(Arc<str>, Connection, Vec<String>),
    UserInteraction(Interaction),
    Exit,
}

#[derive(Debug, Clone)]
pub enum Interaction {
    InputAddressChanged(String),
    AddAddress,
    RemoveAddress(Arc<str>),
}

impl Application {
    fn fail_connection(&mut self, key: Arc<str>, error: connection::Error) {
        self.failed_connection.insert(key, error);
    }
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
            Message::Connect(key, result) => match result {
                Ok(connection) => {
                    self.connections
                        .insert(key.clone(), ConnectionState::default());

                    Command::perform(connection.list_interfaces(), |result| match result {
                        Ok((connection, list)) => Message::ListOfInterfaces(key, connection, list),
                        Err(error) => Message::Connect(key, Err(error)),
                    })
                }
                Err(error) => {
                    self.fail_connection(key, error);
                    Command::none()
                }
            },

            Message::ListOfInterfaces(key, connection, interfaces) => {
                if let Some(the_connection) = self.connections.get_mut(&key) {
                    the_connection.connection = Some(connection);
                    the_connection.interfaces = interfaces;
                }
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
            let connection_text = format!(
                "Connected ! - {:?} - {}",
                connection_state.interfaces,
                connection_state.interfaces.len()
            );

            let ip_text_widget =
                Text::new(format!("{address} - {connection_text}")).width(iced::Length::Fill);
            let delete_button = iced::Element::from(
                Button::new("Remove").on_press(Interaction::RemoveAddress(address.clone())),
            )
            .map(Message::UserInteraction);

            let row = row![ip_text_widget, delete_button].padding(5);

            col = col.push(row);
        }

        for (address, failed) in &self.failed_connection {
            col = col.push(Text::new(format!("{address} - {failed:?}")));
        }

        Container::new(col)
            .center_x()
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .padding(5)
            .into()
    }
}
