use super::Fixture;
use crate::store::{Msg, Store, Value};

use shadcn_iced::font;
use shadcn_iced::radio_group::{RadioGroup, radio_group};
use shadcn_iced::tokens;

use iced::Theme;
use iced::widget::{column, text};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|store| density(store).into()).init(init),
        "keyboard" => Fixture::new(|store| {
            let value = store.str("value");

            column![
                density(store),
                text(format!("Selected: {value}"))
                    .size(14)
                    .style(|theme: &Theme| text::Style {
                        color: Some(tokens(theme).palette.muted_foreground),
                    }),
            ]
            .spacing(12)
            .into()
        })
        .init(init),
        "description" => Fixture::new(|store| {
            radio_group(store.opt("value").and_then(known), select)
                .font(font::sans())
                .option_with_description(
                    "default",
                    "Default",
                    "Standard spacing for most use cases.",
                )
                .option_with_description(
                    "comfortable",
                    "Comfortable",
                    "More space between elements.",
                )
                .into()
        })
        .init(init),
        "disabled" => Fixture::new(|store| density(store).disabled(true).into()).init(init),
        "invalid" => Fixture::new(|store| density(store).invalid(true).into()).init(init),
        _ => return None,
    })
}

fn init(store: &mut Store) {
    store.set("value", "comfortable");
}

fn select(value: &'static str) -> Msg {
    Msg::Set("value", Value::from(value))
}

fn density(store: &Store) -> RadioGroup<'_, &'static str, Msg> {
    radio_group(store.opt("value").and_then(known), select)
        .font(font::sans())
        .option("default", "Default")
        .option("comfortable", "Comfortable")
        .option("compact", "Compact")
}

/// Maps the stored value back to one of the `'static` option values.
fn known(value: &str) -> Option<&'static str> {
    ["default", "comfortable", "compact"]
        .into_iter()
        .find(|known| *known == value)
}
