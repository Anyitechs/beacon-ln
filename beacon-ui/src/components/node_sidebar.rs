use crate::{BeaconLN, Message};
use beacon_node::NodeDetails;
use iced::border::Radius;
use iced::widget::{
    Column, Rule, button, column, container, horizontal_space, row, text, text_input,
};
use iced::{Alignment, Border, Color, Element, Length, Theme, padding};
use iced_font_awesome::fa_icon_solid;

pub fn node_sidebar(beacon_ln: &BeaconLN) -> Column<'_, Message> {
    let active_nodes: String = beacon_ln.nodes.len().to_string();

    let header = row![
        text("Lightning Nodes").size(16),
        horizontal_space(),
        badge_text(active_nodes),
    ];

    let search = text_input("Search nodes...", beacon_ln.search_query.as_str())
        .on_input(Message::SearchChanged)
        .padding(8)
        .size(14);

    let filtered_nodes: Vec<(usize, &NodeDetails)> = beacon_ln
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| {
            n.name
                .to_lowercase()
                .contains(&beacon_ln.search_query.to_lowercase())
        })
        .collect();

    let nodes_list = filtered_nodes.into_iter().fold(
        column![header, search, Rule::horizontal(10)].spacing(8),
        |col, (i, node)| col.push(node_list_item(i, node, beacon_ln.active_node_index)),
    );

    let button_content = row![fa_icon_solid("plus"), text("Add New Node").size(16.0)]
        .spacing(10)
        .align_y(Alignment::Center);

    let create_node = container(
        button(button_content)
            .on_press(Message::CreateNodePressed)
            .width(Length::Fixed(280.0))
            .padding(12)
            .style(move |theme: &Theme, _status| node_button_style(theme, _status)),
    )
    .padding(padding::bottom(20))
    .align_x(Alignment::Center)
    .padding(10);

    let side_node_container = container(column![nodes_list])
        .width(Length::Fixed(280.0))
        .height(Length::Fill)
        .padding(12);

    column![side_node_container, create_node].spacing(8)
}

fn node_list_item(index: usize, node: &NodeDetails, active_index: usize) -> Element<'_, Message> {
    let status_text = if node.is_online {
        "● Online"
    } else {
        "● Offline"
    };
    let content = column![
        text(node.name.clone()).size(16),
        row![
            text(status_text).size(12),
            horizontal_space(),
            text(format!("{} Active", node.channels_active)).size(12),
        ]
        .spacing(8),
    ]
    .spacing(4);

    button(content)
        .on_press(Message::NodeSelected(index))
        .width(Length::Fill)
        .padding(10)
        .style(move |theme: &Theme, _status| {
            let mut st = button::Style {
                background: Some(iced::Background::Color(Color::from_rgb8(242, 242, 242))),
                text_color: Color::BLACK,
                border: Border::rounded(Border::default(), Radius::new(10.0)),
                ..Default::default()
            };
            st.border = Border {
                color: theme.extended_palette().success.weak.color,
                width: 0.0,
                radius: 10.0.into(),
            };
            if index == active_index {
                st.background = Some(theme.extended_palette().primary.weak.color.into());
                st.text_color = Color::WHITE;
            }
            st
        })
        .into()
}

fn badge_text(label: String) -> Element<'static, Message> {
    container(text(format!("{} Active", label)).size(12))
        .padding(8)
        .style(|theme: &Theme| {
            let mut style = iced::widget::container::Style::default();
            let bg = theme.extended_palette().primary.weak.color;
            style.background = Some(bg.into());
            style.text_color = Some(theme.palette().primary);
            style.border = Border {
                color: bg,
                width: 0.0,
                radius: 12.0.into(),
            };
            style
        })
        .into()
}

fn node_button_style(theme: &Theme, _status: button::Status) -> button::Style {
    let style = button::Style {
        background: Some(theme.extended_palette().primary.weak.color.into()),
        text_color: theme.extended_palette().primary.base.text,
        border: Border {
            color: theme.extended_palette().success.weak.color,
            width: 0.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    };

    style
}
