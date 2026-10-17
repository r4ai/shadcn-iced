use super::Fixture;
use crate::store::{Msg, set};

use shadcn_iced::prelude::lucide;
use shadcn_iced::toggle::{Size, Variant, toggle};
use shadcn_iced::tokens;

use iced::widget::{column, row, text};
use iced::{Center, Theme};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|store| {
            toggle(store.bool("pressed"))
                .icon(lucide::ITALIC)
                .label("Italic")
                .on_toggle(set("pressed"))
                .into()
        }),
        "outline" => Fixture::new(|store| {
            toggle(store.bool("pressed"))
                .variant(Variant::Outline)
                .icon(lucide::ITALIC)
                .label("Italic")
                .on_toggle(set("pressed"))
                .into()
        }),
        "icon" => Fixture::new(|store| {
            toggle(store.bool("pressed"))
                .icon(lucide::BOLD)
                .on_toggle(set("pressed"))
                .into()
        }),
        "sizes" => Fixture::new(|_| {
            let outline = |label, size| {
                toggle(false)
                    .variant(Variant::Outline)
                    .size(size)
                    .label(label)
                    .on_toggle(|_| Msg::Noop)
            };

            row![
                outline("Small", Size::Sm),
                outline("Default", Size::Default),
                outline("Large", Size::Lg),
            ]
            .spacing(8)
            .align_y(Center)
            .into()
        }),
        "disabled" => Fixture::new(|_| {
            row![
                toggle::<Msg>(false).label("Disabled").disabled(true),
                toggle::<Msg>(true)
                    .variant(Variant::Outline)
                    .label("Pressed")
                    .disabled(true),
            ]
            .spacing(8)
            .align_y(Center)
            .into()
        }),
        "keyboard" => Fixture::new(|store| {
            let pressed = store.bool("pressed");

            column![
                toggle(pressed)
                    .icon(lucide::ITALIC)
                    .label("Italic")
                    .on_toggle(set("pressed")),
                text(if pressed {
                    "Italic is on"
                } else {
                    "Italic is off"
                })
                .size(14)
                .style(|theme: &Theme| text::Style {
                    color: Some(tokens(theme).palette.muted_foreground),
                }),
            ]
            .spacing(12)
            .into()
        }),
        _ => return None,
    })
}
