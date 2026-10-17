use super::Fixture;
use crate::store::{Msg, set};

use shadcn_iced::button::{Variant, button};
use shadcn_iced::input::input;
use shadcn_iced::tokens;

use iced::widget::{column, container, row, text};
use iced::{Center, Fill, Theme};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|store| {
            narrow({
                input("Email", store.str("email"))
                    .on_input(set("email"))
                    .into()
            })
        }),
        "typing" => Fixture::new(|store| {
            narrow({
                let value = store.str("email");

                column![
                    input("Email", value).on_input(set("email")),
                    text(if value.is_empty() {
                        "Nothing typed".to_owned()
                    } else {
                        format!("You typed: {value}")
                    })
                    .size(14)
                    .style(|theme: &Theme| text::Style {
                        color: Some(tokens(theme).palette.muted_foreground),
                    }),
                ]
                .spacing(12)
                .into()
            })
        }),
        "states" => Fixture::new(|store| {
            narrow({
                column![
                    input("Name", store.str("name")).on_input(set("name")),
                    input("Email", ""),
                    input("Email", store.str("invalid"))
                        .on_input(set("invalid"))
                        .invalid(true),
                    input("Password", store.str("password"))
                        .on_input(set("password"))
                        .secure(true),
                ]
                .spacing(12)
                .into()
            })
        })
        .init(|store| {
            store
                .set("name", "Evil Rabbit")
                .set("invalid", "invalid@")
                .set("password", "secret");
        }),
        "with-button" => Fixture::new(|store| {
            narrow({
                let content = row![
                    input("Email", store.str("email"))
                        .on_input(set("email"))
                        .width(Fill),
                    button("Subscribe")
                        .variant(Variant::Outline)
                        .on_press(Msg::Noop),
                ]
                .spacing(8)
                .align_y(Center)
                .width(Fill);

                container(content).width(Fill).max_width(384).into()
            })
        }),
        _ => return None,
    })
}

/// The page of the fixture is 768px wide (the `md` breakpoint), in a `w-80`.
fn narrow(content: iced::Element<'_, Msg>) -> iced::Element<'_, Msg> {
    iced::widget::container(content).width(320).into()
}
