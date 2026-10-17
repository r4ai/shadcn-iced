use super::Fixture;
use crate::store::{Msg, set};

use shadcn_iced::button::Variant;
use shadcn_iced::input::input;
use shadcn_iced::input_group::{group_button, input_group};
use shadcn_iced::prelude::lucide;

use iced::widget::column;

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "icon" => Fixture::new(|store| {
            narrow({
                input_group(input("Search...", store.str("query")).on_input(set("query")))
                    .leading_icon(lucide::SEARCH)
                    .trailing_text("12 results")
                    .into()
            })
        }),
        "text" => Fixture::new(|store| {
            narrow({
                input_group(input("example.com", store.str("url")).on_input(set("url")))
                    .leading_text("https://")
                    .trailing_text(".com")
                    .into()
            })
        }),
        "button" => Fixture::new(|store| {
            narrow({
                input_group(input("Type to search...", store.str("query")).on_input(set("query")))
                    .trailing_button(
                        group_button("Search")
                            .variant(Variant::Secondary)
                            .on_press(Msg::Noop),
                    )
                    .into()
            })
        }),
        "kbd" => Fixture::new(|store| {
            narrow({
                input_group(input("Search...", store.str("query")).on_input(set("query")))
                    .leading_icon(lucide::SEARCH)
                    .trailing_kbd("⌘K")
                    .into()
            })
        }),
        "states" => Fixture::new(|store| {
            narrow({
                column![
                    input_group(input("Search...", store.str("query")).on_input(set("query")))
                        .invalid(true)
                        .leading_icon(lucide::SEARCH)
                        .trailing_text("12 results"),
                    input_group(input::<Msg>("Search...", ""))
                        .disabled(true)
                        .leading_icon(lucide::SEARCH)
                        .trailing_text("12 results"),
                ]
                .spacing(12)
                .into()
            })
        }),
        _ => return None,
    })
}

/// The page of the fixture is 768px wide (the `md` breakpoint), in a `w-80`.
fn narrow(content: iced::Element<'_, Msg>) -> iced::Element<'_, Msg> {
    iced::widget::container(content).width(320).into()
}
