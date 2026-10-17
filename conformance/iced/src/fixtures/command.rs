use super::Fixture;
use crate::store::{Msg, Store, set};

use shadcn_iced::command::{command, item};
use shadcn_iced::prelude::lucide;

use iced::Element;
use iced::widget::container;

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "basic" | "keyboard" | "filter" | "empty" => Fixture::new(basic),
        _ => return None,
    })
}

fn basic(s: &Store) -> Element<'_, Msg> {
    container(
        command(s.str("query"), set("query"))
            .group(
                "Suggestions",
                [
                    item("Calendar").icon(lucide::CALENDAR).on_select(Msg::Noop),
                    item("Search Emoji")
                        .icon(lucide::SMILE)
                        .on_select(Msg::Noop),
                    item("Calculator").icon(lucide::CALCULATOR),
                ],
            )
            .group(
                "Settings",
                [
                    item("Profile")
                        .icon(lucide::USER)
                        .shortcut("⌘P")
                        .on_select(Msg::Noop),
                    item("Billing")
                        .icon(lucide::CREDIT_CARD)
                        .shortcut("⌘B")
                        .on_select(Msg::Noop),
                    item("Settings")
                        .icon(lucide::SETTINGS)
                        .shortcut("⌘S")
                        .on_select(Msg::Noop),
                ],
            ),
    )
    // `max-w-sm`
    .max_width(384)
    .into()
}
