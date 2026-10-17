use super::Fixture;
use crate::store::{Msg, Store, edit};

use shadcn_iced::textarea::textarea;
use shadcn_iced::tokens;

use iced::widget::{column, text};
use iced::{Element, Theme};

const PLACEHOLDER: &str = "Type your message here.";

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => {
            Fixture::new(|store| narrow(editable(store, "message", false))).init(|store| {
                store.set_editor("message", "");
            })
        }
        "typing" => Fixture::new(|store| {
            let value = store.editor("message").text();
            let value = value.trim_end();

            narrow(
                column![
                    editable(store, "message", false),
                    text(if value.is_empty() {
                        "Nothing typed".to_owned()
                    } else {
                        format!("You typed: {value}")
                    })
                    .size(14)
                    .line_height(shadcn_iced::style::text::leading(14.0))
                    .style(|theme: &Theme| text::Style {
                        color: Some(tokens(theme).palette.muted_foreground),
                    }),
                ]
                .spacing(12)
                .into(),
            )
        })
        .init(|store| {
            store.set_editor("message", "");
        }),
        "states" => Fixture::new(|store| {
            narrow(
                column![
                    editable(store, "value", false),
                    textarea::<Msg>(store.editor("disabled")).placeholder(PLACEHOLDER),
                    editable(store, "invalid", true),
                    editable(store, "lines", false),
                ]
                .spacing(12)
                .into(),
            )
        })
        .init(|store| {
            store
                .set_editor("value", "Hello, world!")
                .set_editor("disabled", "")
                .set_editor("invalid", "This message is invalid.")
                .set_editor("lines", "First line\nSecond line\nThird line\nFourth line");
        }),
        _ => return None,
    })
}

fn editable<'a>(store: &'a Store, key: &'static str, invalid: bool) -> Element<'a, Msg> {
    textarea(store.editor(key))
        .placeholder(PLACEHOLDER)
        .on_action(edit(key))
        .invalid(invalid)
        .into()
}

/// The page of the fixture is 768px wide (the `md` breakpoint), in a `w-80`.
fn narrow(content: Element<'_, Msg>) -> Element<'_, Msg> {
    iced::widget::container(content).width(320).into()
}
