//! The store cannot hold a `Content` nor apply `Action`s, so the textareas
//! of these fixtures show a fixed content and ignore edits.
use super::Fixture;
use crate::store::Msg;

use shadcn_iced::textarea::{Content, textarea};
use shadcn_iced::tokens;

use iced::widget::{column, text};
use iced::{Element, Theme};

const PLACEHOLDER: &str = "Type your message here.";

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|_| narrow(editable("", false))),
        "typing" => Fixture::new(|store| {
            narrow({
                let value = store.str("message");

                column![
                    editable("", false),
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
        "states" => Fixture::new(|_| {
            narrow({
                column![
                    editable("Hello, world!", false),
                    textarea::<Msg>(content("")).placeholder(PLACEHOLDER),
                    editable("This message is invalid.", true),
                    editable("First line\nSecond line\nThird line\nFourth line", false),
                ]
                .spacing(12)
                .into()
            })
        }),
        _ => return None,
    })
}

/// Leaks the content: fixtures are views of a short-lived process.
fn content(value: &str) -> &'static Content {
    Box::leak(Box::new(Content::with_text(value)))
}

fn editable(value: &str, invalid: bool) -> Element<'static, Msg> {
    textarea(content(value))
        .placeholder(PLACEHOLDER)
        .on_action(|_| Msg::Noop)
        .invalid(invalid)
        .into()
}

/// The page of the fixture is 768px wide (the `md` breakpoint), in a `w-80`.
fn narrow(content: iced::Element<'_, Msg>) -> iced::Element<'_, Msg> {
    iced::widget::container(content).width(320).into()
}
