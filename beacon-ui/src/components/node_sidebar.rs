use crate::{BeaconLN, Message, Node};
use iced::widget::{
    Container, Rule, button, column, container, horizontal_space, row, text, text_input,
};
use iced::{Border, Element, Length, Theme};

pub fn node_sidebar<'a>(beacon_ln: &'a BeaconLN) -> Container<'a, Message> {
    let header = row![
        text("Lightning Nodes").size(16),
        horizontal_space(),
        badge_text("3 Active"),
    ];

    let search = text_input("Search nodes...", beacon_ln.search_query.as_str())
        .on_input(Message::SearchChanged)
        .padding(8)
        .size(14);

    let filtered_nodes: Vec<(usize, &Node)> = beacon_ln
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

    container(nodes_list)
        .width(Length::Fixed(280.0))
        .height(Length::Fill)
        .padding(12)
}

fn node_list_item<'a>(index: usize, node: &'a Node, active_index: usize) -> Element<'a, Message> {
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
        .style(move |theme: &Theme, status| {
            let mut st = button::text(theme, status);
            st.border = Border {
                color: theme.extended_palette().success.weak.color,
                width: 0.0,
                radius: 10.0.into(),
            };
            if index == active_index {
                st.background = Some(theme.extended_palette().primary.weak.color.into());
            }
            st
        })
        .into()
}

fn badge_text<'a>(label: &'a str) -> Element<'a, Message> {
    container(text(label).size(12))
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
