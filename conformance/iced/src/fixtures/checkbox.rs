use super::Fixture;
use crate::store::{Msg, Store, set};

use shadcn_iced::checkbox::checkbox;
use shadcn_iced::tokens;

use iced::widget::{column, text};
use iced::{Element, Theme};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|store| {
            checkbox(store.bool("checked"))
                .label("Accept terms and conditions")
                .on_toggle(set("checked"))
                .into()
        }),
        "states" => Fixture::new(|_| {
            column![
                checkbox(true)
                    .label("Accept terms and conditions")
                    .on_toggle(|_| Msg::Noop),
                checkbox(false)
                    .label("Accept terms and conditions")
                    .description("By clicking this checkbox, you agree to the terms.")
                    .on_toggle(|_| Msg::Noop),
                checkbox::<Msg>(false)
                    .label("Enable notifications")
                    .disabled(true),
                checkbox::<Msg>(true)
                    .label("Enable notifications")
                    .disabled(true),
                checkbox(false)
                    .label("Accept terms and conditions")
                    .invalid(true)
                    .on_toggle(|_| Msg::Noop),
                checkbox(true)
                    .label("Accept terms and conditions")
                    .invalid(true)
                    .on_toggle(|_| Msg::Noop),
            ]
            .spacing(12)
            .into()
        }),
        "keyboard" | "tab" => Fixture::new(|store| controlled(store, false)),
        "disabled" => Fixture::new(|store| controlled(store, true)),
        _ => return None,
    })
}

fn controlled(store: &Store, disabled: bool) -> Element<'_, Msg> {
    let checked = store.bool("checked");

    column![
        checkbox(checked)
            .label("Accept terms and conditions")
            .on_toggle(set("checked"))
            .disabled(disabled),
        text(if checked { "Accepted" } else { "Not accepted" })
            .size(14)
            .line_height(iced::Pixels(20.0))
            .style(|theme: &Theme| text::Style {
                color: Some(tokens(theme).palette.muted_foreground),
            }),
    ]
    .spacing(12)
    .into()
}
