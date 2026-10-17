use super::Fixture;
use crate::store::{Msg, Store};

use shadcn_iced::prelude::lucide;
use shadcn_iced::separator::vertical_separator;
use shadcn_iced::sidebar::{Item, Size, group, item, layout, sidebar, trigger};
use shadcn_iced::style::{border, rounded};
use shadcn_iced::{Icon, font, tokens};

use iced::widget::{column, container, row, text};
use iced::{Center, Element, Fill, Theme};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" | "collapse" => Fixture::new(view).init(|s| {
            s.set("active", "Playground");
            s.set("collapsed", false);
        }),
        _ => return None,
    })
}

/// A menu button that becomes the active one when pressed.
fn button(s: &Store, title: &'static str, icon: Icon) -> Item<'static, Msg> {
    item(title)
        .icon(icon)
        .active(s.str("active") == title)
        .on_press(Msg::Set("active", title.into()))
}

fn view(s: &Store) -> Element<'_, Msg> {
    let collapsed = s.bool("collapsed");

    let nav = sidebar()
        .header_item(
            item("Acme Inc")
                .size(Size::Lg)
                .icon_element(logo(lucide::GALLERY_VERTICAL_END))
                .content(two_lines(
                    "Acme Inc",
                    "Enterprise",
                    lucide::CHEVRONS_UP_DOWN,
                ))
                .on_press(Msg::Noop),
        )
        .group(
            group("Platform")
                .item(button(s, "Playground", lucide::SQUARE_TERMINAL))
                .item(button(s, "Models", lucide::BOT))
                .item(button(s, "Documentation", lucide::BOOK_OPEN))
                .item(button(s, "Settings", lucide::SETTINGS_2)),
        )
        .group(
            group("Projects")
                .item(button(s, "Design Engineering", lucide::FRAME))
                .item(button(s, "Sales & Marketing", lucide::CHART_PIE))
                .item(button(s, "Travel", lucide::MAP))
                .item(
                    item("More")
                        .icon(lucide::ELLIPSIS)
                        .badge("24")
                        .on_press(Msg::Set("active", "More".into())),
                ),
        )
        .footer_item(item("Support").icon(lucide::LIFE_BUOY).on_press(Msg::Noop))
        .collapsed(collapsed);

    // `flex h-12 items-center gap-2 px-4`
    let main = row![
        trigger(Msg::Toggle("collapsed")),
        container(vertical_separator()).height(16),
        text("Data Fetching").size(14),
    ]
    .spacing(8)
    .height(48)
    .padding([0, 16])
    .align_y(Center);

    // `h-[528px] overflow-hidden rounded-lg border`
    container(layout(nav, main))
        .width(Fill)
        .height(528)
        .padding(1)
        .clip(true)
        .style(|theme: &Theme| {
            let t = tokens(theme);

            container::Style {
                border: border(t.palette.border, t.radius_lg()),
                ..container::Style::default()
            }
        })
        .into()
}

/// Two lines of text in a large sidebar item
/// (`grid flex-1 text-left text-sm leading-tight`).
fn two_lines(title: &'static str, subtitle: &'static str, trailing: Icon) -> Element<'static, Msg> {
    row![
        column![
            text(title).size(14).font(font::medium()).line_height(1.25),
            text(subtitle).size(12).line_height(1.25),
        ]
        .width(Fill),
        trailing.sized(16),
    ]
    .align_y(Center)
    .into()
}

/// The square logo of the team switcher (`size-8 rounded-lg
/// bg-sidebar-primary`).
fn logo(icon: Icon) -> Element<'static, Msg> {
    container(icon.sized(16))
        .center(32)
        .style(|theme: &Theme| {
            let t = tokens(theme);

            container::Style {
                background: Some(t.palette.sidebar_primary.into()),
                text_color: Some(t.palette.sidebar_primary_foreground),
                border: rounded(t.radius_lg()),
                ..container::Style::default()
            }
        })
        .into()
}
