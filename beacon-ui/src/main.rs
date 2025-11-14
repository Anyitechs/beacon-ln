use iced::futures::stream::StreamExt;
use iced::widget::{Rule, button, column, container, horizontal_space, row, text};
use iced::{Alignment, Border, Color, Element, Font, Length, Subscription, Task, Theme, theme};
use iced_font_awesome::fa_icon_solid;
use tokio_stream::wrappers::BroadcastStream;

use crate::components::header::*;
use crate::components::node_sidebar::*;
use beacon_node::types::{Network, log};
use beacon_node::{
    BeaconNodeConfig, BeaconNodeManagerCommand, BeaconNodeManagerEvent, BeaconNodeManagerHandle,
    NodeDetails,
};
use std::path::PathBuf;

pub mod components;

fn main() -> iced::Result {
    env_logger::init();

    iced::application("Beacon", BeaconLN::update, BeaconLN::view)
        .font(include_bytes!("../assets/fonts/Inter-Regular.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/Inter-Bold.ttf").as_slice())
        .theme(|app| app.theme())
        // .default_font(Font::with_name(FONT_NAME))
        .default_font(Font {
            family: iced::font::Family::Name("Inter-Regular.ttf"),
            weight: iced::font::Weight::Normal,
            stretch: iced::font::Stretch::Normal,
            style: iced::font::Style::Normal,
        })
        .subscription(BeaconLN::subscription)
        .run_with(BeaconLN::new)
}

// Defines which view is active in the main content area
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Dashboard,
    Channels,
    Send,
    Receive,
    Transactions,
    Routing,
    Security,
    Analytics,
    Settings,
}

impl Default for View {
    fn default() -> Self {
        Self::Dashboard
    }
}

#[derive(Default)]
pub struct BeaconLN {
    manager_handle: Option<BeaconNodeManagerHandle>,
    nodes: Vec<NodeDetails>,
    active_node_index: usize,
    active_view: View,
    search_query: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    IcedReady,
    NodeSelected(usize),
    ViewSelected(View),
    SearchChanged(String),
    CreateNodePressed,
    ManagerEvent(BeaconNodeManagerEvent),
    NoOp,
}

impl BeaconLN {
    fn new() -> (Self, Task<Message>) {
        (
            BeaconLN {
                manager_handle: None,
                nodes: Vec::new(),
                active_node_index: 0,
                active_view: View::Dashboard,
                search_query: String::new(),
            },
            Task::perform(async {}, |_| Message::IcedReady),
        )
    }

    fn theme(&self) -> Theme {
        Theme::custom(
            String::from("Custome"),
            theme::Palette {
                background: Color::WHITE,
                danger: Color::from_rgb8(231, 76, 60),
                primary: Color::from_rgb8(0, 102, 255),
                success: Color::from_rgb8(46, 204, 113),
                text: Color::BLACK,
            },
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::IcedReady => {
                log::info!("Iced is ready!");
                let base_dir = PathBuf::from("./../beacon-node/beacon-data");
                self.manager_handle = Some(beacon_node::start_manager_actor(base_dir));

                if let Some(handle) = &self.manager_handle {
                    let sender = handle.command_sender.clone();

                    return Task::perform(
                        async move {
                            sender.send(BeaconNodeManagerCommand::ListNodes).await.ok();
                        },
                        |_| Message::NoOp,
                    );
                }
            }
            Message::NodeSelected(index) => {
                self.active_node_index = index;
            }
            Message::ViewSelected(view) => {
                self.active_view = view;
            }
            Message::SearchChanged(value) => {
                self.search_query = value;
            }
            Message::CreateNodePressed => {
                if let Some(handle) = &self.manager_handle {
                    log::info!("UI sending command to create node...");

                    let node_name = format!("Node {}", self.nodes.len() + 1);
                    let config = BeaconNodeConfig {
                        network: Network::Testnet,
                        chain_source: None,
                        gossip_source: None,
                        node_alias: Some(node_name.clone()),
                    };

                    let sender = handle.command_sender.clone();

                    return Task::perform(
                        async move {
                            sender
                                .send(BeaconNodeManagerCommand::CreateNode { node_name, config })
                                .await
                        },
                        |res| {
                            if res.is_err() {
                                Message::ManagerEvent(BeaconNodeManagerEvent::Error(
                                    "Failed to send command".into(),
                                ))
                            } else {
                                Message::NoOp
                            }
                        },
                    );
                }
            }
            Message::ManagerEvent(event) => {
                match event {
                    BeaconNodeManagerEvent::NodeListUpdated(nodes) => {
                        self.nodes = nodes;
                    }
                    BeaconNodeManagerEvent::NodeCreated(name) => {
                        log::info!("UI notified that node was created: {}", name);
                        // TODO: display a success toast here
                    }
                    BeaconNodeManagerEvent::Error(e) => {
                        log::error!("Manager error: {}", e);
                        // TODO: display an error toast here
                    }
                }
            }
            Message::NoOp => {}
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        if let Some(handle) = &self.manager_handle {
            struct ManagerEvents;

            // Creates a stream from the broadcast receiver.
            //  Calling `resubscribe` is important to get a fresh receiver for this subscription.
            let event_stream =
                BroadcastStream::new(handle.event_receiver.resubscribe()).map(|result| {
                    result
                        .map(Message::ManagerEvent)
                        .unwrap_or(Message::IcedReady)
                });

            return Subscription::run_with_id(
                std::any::TypeId::of::<ManagerEvents>(),
                event_stream,
            );
        }

        Subscription::none()
    }

    fn view<'a>(&'a self) -> Element<'a, Message> {
        let navigation_pane = {
            let nav_button = |icon_name: &str, label: &'a str, view: View, active_view: &View| {
                let is_active = &view == active_view;

                let mut icon = fa_icon_solid(icon_name).size(20.0);

                if !is_active {
                    icon = icon.color(Color::BLACK);
                }

                let content = row![icon, text(label).size(16.0)]
                    .spacing(10)
                    .align_y(Alignment::Center);

                button(content)
                    .on_press(Message::ViewSelected(view))
                    .width(Length::Fill)
                    .padding(15)
                    .style(move |theme: &Theme, _status| {
                        nav_button_style(theme, _status, is_active)
                    })
            };

            let active_node_name: String = self
                .nodes
                .get(self.active_node_index)
                .map_or(String::new(), |n| n.name.to_string());

            container(
                column![
                    text(active_node_name).size(24),
                    Rule::horizontal(10),
                    nav_button("grip", "Dashboard", View::Dashboard, &self.active_view),
                    nav_button(
                        "diagram-project",
                        "Channels",
                        View::Channels,
                        &self.active_view
                    ),
                    nav_button("paper-plane", "Send", View::Send, &self.active_view),
                    nav_button("download", "Receive", View::Receive, &self.active_view),
                    nav_button(
                        "list",
                        "Transactions",
                        View::Transactions,
                        &self.active_view
                    ),
                    nav_button("route", "Routing", View::Routing, &self.active_view),
                    nav_button(
                        "shield-halved",
                        "Security",
                        View::Security,
                        &self.active_view
                    ),
                    nav_button(
                        "chart-line",
                        "Analytics",
                        View::Analytics,
                        &self.active_view
                    ),
                    nav_button("gear", "Settings", View::Settings, &self.active_view),
                ]
                .spacing(10),
            )
            .width(Length::Fixed(240.0))
            .height(Length::Fill)
            .padding(10)
        };

        let content_area = {
            let content = match self.active_view {
                View::Dashboard => {
                    let kpis = row![
                        stat_card_with_color(
                            "Total Balance",
                            "₿ 0.05432100",
                            "+2.3%",
                            "On-chain: ₿ 0.02100000 | Channels: ₿ 0.03332100",
                            CardColor::Green
                        ),
                        stat_card_with_color(
                            "Inbound Liquidity (sats)",
                            "2,150,000",
                            "Available",
                            "",
                            CardColor::Blue
                        ),
                        stat_card_with_color(
                            "Outbound Liquidity (sats)",
                            "1,850,000",
                            "Available",
                            "",
                            CardColor::Orange
                        ),
                    ]
                    .spacing(16);

                    let lower = row![
                        panel_box(
                            "Payment Volume (24h)",
                            column![text("No data yet")].spacing(8)
                        ),
                        panel_box(
                            "Channel Utilization",
                            column![text("No data yet")].spacing(8)
                        ),
                    ]
                    .spacing(16);

                    column![kpis, lower].spacing(16)
                }
                _ => column![text(format!("{:?} View", self.active_view)).size(32)],
            };

            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(20)
        };

        column![
            header(),
            Rule::horizontal(10),
            row![
                node_sidebar(self),
                Rule::vertical(10),
                navigation_pane,
                Rule::vertical(10),
                content_area,
            ]
            .align_y(Alignment::Start)
        ]
        .into()
    }
}

#[derive(Clone, Copy)]
enum CardColor {
    Green,
    Blue,
    Orange,
}

fn stat_card<'a>(
    title: &'a str,
    value: &'a str,
    badge: &'a str,
    note: &'a str,
) -> Element<'a, Message> {
    let header = row![
        text(title).size(14),
        horizontal_space(),
        container(text(badge).size(12)).padding(8),
    ];

    let body = column![text(value).size(28), text(note).size(12),].spacing(6);

    container(column![header, body].spacing(12))
        .padding(16)
        .width(Length::Fill)
        .into()
}

fn stat_card_with_color<'a>(
    title: &'a str,
    value: &'a str,
    badge: &'a str,
    note: &'a str,
    color: CardColor,
) -> Element<'a, Message> {
    let base = stat_card(title, value, badge, note);
    container(base)
        .style(move |theme: &Theme| {
            let (bg, text_color) = match color {
                CardColor::Green => (
                    theme.extended_palette().success.weak.color,
                    theme.palette().success,
                ),
                CardColor::Blue => (
                    theme.extended_palette().primary.weak.color,
                    theme.palette().primary,
                ),
                CardColor::Orange => (
                    theme.extended_palette().secondary.weak.color,
                    theme.palette().background,
                ),
            };
            iced::widget::container::Style {
                background: Some(bg.into()),
                text_color: Some(text_color),
                border: Border {
                    color: bg,
                    width: 0.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

fn panel_box<'a>(
    title: &'a str,
    content: iced::widget::Column<'a, Message>,
) -> Element<'a, Message> {
    let header = text(title).size(16);
    container(column![header, Rule::horizontal(10), content].spacing(12))
        .padding(16)
        .width(Length::Fill)
        .into()
}

fn nav_button_style(theme: &Theme, _status: button::Status, is_active: bool) -> button::Style {
    if is_active {
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

        return style;
    }
    button::Style {
        background: Some(iced::Background::Color(Color::TRANSPARENT)),
        text_color: Color::BLACK,
        border: Border::rounded(Border::default(), iced::border::Radius::new(10.0)),
        ..Default::default()
    }
}
