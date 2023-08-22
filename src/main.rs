use std::{collections::HashMap, time};

use async_std::task::sleep;

use iced::{
    executor,
    keyboard::{KeyCode, Modifiers},
    subscription,
    widget::{row, Button, Column, Container, Text, TextInput},
    window, Application, Command, Settings, Subscription, Theme,
};

#[derive(Debug, Default)]
struct ApplicationState {
    connections: HashMap<String, String>,
    input_address: String,
}

#[derive(Debug, Clone)]
enum Message {
    InputAddressChanged(String),
    AsyncTest(String, String),
    AddAddress,
    RemoveAddress(String),
    Exit,
}

async fn some_async_fun(_key: String) -> &'static str {
    sleep(time::Duration::from_secs(2)).await;

    "async resolved"
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
            Message::InputAddressChanged(current) => {
                self.input_address = current;
                Command::none()
            }
            Message::AddAddress => {
                let address = self.input_address.clone();
                self.input_address.clear();

                self.connections
                    .insert(address.clone(), "Not resolved".into());

                Command::perform(some_async_fun(address.clone()), |arg| {
                    Message::AsyncTest(address, arg.into())
                })
            }
            Message::RemoveAddress(key) => {
                self.connections.remove(&key);
                Command::none()
            }

            Message::AsyncTest(key, resolved) => {
                if let Some(value) = self.connections.get_mut(&key) {
                    *value = resolved;
                }

                Command::none()
            }

            Message::Exit => window::close(),
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

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let mut col = {
            let input = TextInput::new("Connect to", &self.input_address)
                .on_input(Message::InputAddressChanged)
                .on_submit(Message::AddAddress);

            Column::new().padding(5).push(input)
        };

        for (address, connected_status) in self.connections.iter() {
            let ip_text_widget =
                Text::new(format!("{address} - {connected_status}")).width(iced::Length::Fill);
            let delete_button =
                Button::new("Remove").on_press(Message::RemoveAddress(address.clone()));

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
