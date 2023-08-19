use iced::{
    alignment::{Alignment, Horizontal},
    widget::{Button, Column, Container, Text, TextInput},
    Element, Length,
};

use bambu::ConnectOpts;

use crate::Message;

/// Controls for gui
pub struct Controls {
    pub opts: ConnectOpts,
    pub connected: bool,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            opts: Default::default(),
            connected: false,
        }
    }
}

impl Controls {
    pub fn view(&self) -> Element<'_, Message> {
        let mut connect_ctl = Column::new()
            .spacing(10)
            .padding(20)
            .align_items(Alignment::Center);

        connect_ctl = connect_ctl.push(Text::new("Printer connection"));

        connect_ctl = connect_ctl.push(
            TextInput::new("hostname", &self.opts.hostname)
                .on_input(Message::SetHostname)
                .width(Length::Fill),
        );

        connect_ctl = connect_ctl.push(
            TextInput::new("access code", &self.opts.access_code)
                .on_input(Message::SetAccessCode)
                .width(Length::Fill),
        );

        if !self.connected {
            connect_ctl = connect_ctl.push(
                Button::new(Text::new("connect").horizontal_alignment(Horizontal::Center))
                    .on_press(Message::Connect(self.opts.clone()))
                    .width(Length::Fill),
            )
        } else {
            connect_ctl = connect_ctl.push(
                Button::new(Text::new("disconnect").horizontal_alignment(Horizontal::Center))
                    .on_press(Message::Disconnect)
                    .width(Length::Fill),
            )
        }

        Container::new(connect_ctl).into()
    }
}
