use iced::{
    executor,
    keyboard::{KeyCode, Modifiers},
    subscription,
    widget::{row, Button, Column, Container, Text, TextInput},
    window, Application, Command, Settings, Subscription, Theme,
};

#[derive(Debug, Default)]
struct IPAddresses {
    ip: Vec<String>,
    input_address: String,
}

#[derive(Debug, Clone)]
enum Message {
    InputAddressChanged(String),
    AddAddress,
    RemoveAddressAt(usize),
    Exit,
}

impl Application for IPAddresses {
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
                self.ip.push(self.input_address.clone());
                self.input_address.clear();
                Command::none()
            }
            Message::RemoveAddressAt(index) => {
                self.ip.swap_remove(index);
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
            let label = Text::new(format!("List of {} IP addresses!", self.ip.len()));

            let input = TextInput::new("put your IP address here", &self.input_address)
                .on_input(Message::InputAddressChanged)
                .on_submit(Message::AddAddress);

            Column::new().padding(5).push(label).push(input)
        };

        for (index, ip_address) in self.ip.iter().enumerate() {
            let ip_text_widget = Text::new(ip_address).width(iced::Length::Fill);
            let delete_button = Button::new("Remove").on_press(Message::RemoveAddressAt(index));

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

    IPAddresses::run(settings)
}
