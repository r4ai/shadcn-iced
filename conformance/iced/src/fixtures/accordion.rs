use super::Fixture;
use crate::store::{Msg, Store, Value};

use shadcn_iced::accordion::accordion;

use iced::widget::{container, text};
use iced::{Element, Fill};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" | "keyboard" => Fixture::new(product).init(|s| {
            s.set("open", "0");
        }),
        "multiple" => Fixture::new(|_| {
            container(
                accordion()
                    .open_item(
                        "Product Information",
                        body("Our products are made to last."),
                    )
                    .item(
                        "Return Policy",
                        body("Returns are accepted within 30 days."),
                    ),
            )
            .width(Fill)
            .max_width(512)
            .into()
        }),
        "disabled" => Fixture::new(|_| {
            container(
                accordion()
                    .item("Account history", body("View your complete history."))
                    .disabled_item("Premium feature", body("Upgrade your plan to see this."))
                    .item("Update your email", body("Change it in your settings.")),
            )
            .width(Fill)
            .max_width(512)
            .into()
        }),
        "borders" => Fixture::new(|s| {
            container(
                accordion()
                    .single(open(s), set_open)
                    .bordered(true)
                    .item("How does billing work?", body("Monthly and annual plans."))
                    .item("Is my data secure?", body("Yes. Everything is encrypted."))
                    .item("Which integrations exist?", body("Over 500 popular tools.")),
            )
            .width(Fill)
            .max_width(512)
            .into()
        })
        .init(|s| {
            s.set("open", "0");
        }),
        _ => return None,
    })
}

/// `max-w-lg`, one item open at a time.
fn product(s: &Store) -> Element<'_, Msg> {
    container(
        accordion()
            .single(open(s), set_open)
            .item(
                "Product Information",
                body("We offer standard and express shipping."),
            )
            .item(
                "Return Policy",
                body("Returns are accepted within 30 days."),
            )
            .item("Contact Us", body("Reach us by email or chat.")),
    )
    .width(Fill)
    .max_width(512)
    .into()
}

/// The index of the open item.
fn open(s: &Store) -> Option<usize> {
    s.opt("open").and_then(|index| index.parse().ok())
}

fn set_open(open: Option<usize>) -> Msg {
    Msg::Set("open", Value::from(open.map(|index| index.to_string())))
}

/// `text-sm`
fn body(content: &'static str) -> iced::widget::Text<'static> {
    text(content).size(14).line_height(iced::Pixels(20.0))
}
