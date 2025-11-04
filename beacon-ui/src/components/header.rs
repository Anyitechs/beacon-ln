use crate::Message;
use iced::widget::{Container, Rule, container, horizontal_space, row, text};
use iced::{Alignment, Length};

pub fn header<'a>() -> Container<'a, Message> {
    let left = row![
        text("⚡ Beacon LN").size(20),
        horizontal_space(),
        Rule::vertical(1),
        horizontal_space(),
        text("● Network Connected"),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let right = row![
        text("Block Height: 815,432").size(12),
        horizontal_space(),
        text("⚙"),
        horizontal_space(),
        text("🔔"),
        horizontal_space(),
        text("U").size(14),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    container(row![left, horizontal_space(), right])
        .width(Length::Fill)
        .height(Length::Fixed(70.0))
        .padding(12)
}
