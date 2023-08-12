use iced::{
    widget::{Column, Container, Text, TextInput},
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
        let label = Text::new(format!("List of {} IP addresses!", self.ip.len()));

        let input = TextInput::new("put your IP address here", &self.input_address)
            .on_input(|text| Message::InputAddressChanged(text))
            .on_submit(Message::AddAddress);

        let mut col = Column::new().push(label).push(input);

        for ip in &self.ip {
            let ip_text_widget = Text::new(ip);
            col = col.push(ip_text_widget);
        }

        Container::new(col)
            .center_x()
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
}

fn main() -> Result<(), iced::Error> {
    let mut settings = Settings::default();
    settings.window.min_size = Some((250, 100));
    IPAddresses::run(settings)
}
