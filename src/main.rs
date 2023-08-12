use iced::{
    widget::{row, Button, Column, Container, Text, TextInput},
    Sandbox, Settings,
};

#[derive(Debug)]
struct IPAddresses {
    ip: Vec<String>,
    input_address: String,
}

#[derive(Debug, Clone)]
enum Message {
    InputAddressChanged(String),
    AddAddress,
    RemoveAddressAt(usize),
}

impl Sandbox for IPAddresses {
    type Message = Message;

    fn new() -> Self {
        Self {
            ip: Vec::new(),
            input_address: String::new(),
        }
    }

    fn title(&self) -> String {
        "Xena Linux app".into()
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::InputAddressChanged(current) => {
                self.input_address = current;
            }
            Message::AddAddress => {
                self.ip.push(self.input_address.clone());
                self.input_address.clear();
            }
            Message::RemoveAddressAt(index) => {
                self.ip.swap_remove(index);
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
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
    let mut settings = Settings::default();
    settings.window.min_size = Some((250, 100));
    IPAddresses::run(settings)
}
