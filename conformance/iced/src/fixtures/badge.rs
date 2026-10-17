use super::Fixture;
use crate::store::Msg;

use shadcn_iced::badge::{Badge, Variant, badge};
use shadcn_iced::prelude::lucide;

use iced::widget::row;

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "variants" => Fixture::new(|_| {
            row![
                variant("Default", Variant::Default),
                variant("Secondary", Variant::Secondary),
                variant("Destructive", Variant::Destructive),
                variant("Outline", Variant::Outline),
                variant("Ghost", Variant::Ghost),
                variant("Link", Variant::Link),
            ]
            .spacing(8)
            .into()
        }),
        "with-icon" => Fixture::new(|_| {
            row![
                variant("Verified", Variant::Secondary).icon(lucide::BADGE_CHECK),
                variant("Bookmark", Variant::Outline).icon(lucide::BOOKMARK),
            ]
            .spacing(8)
            .into()
        }),
        "link" => Fixture::new(|_| {
            variant("Open Link", Variant::Default)
                .trailing_icon(lucide::ARROW_UP_RIGHT)
                .on_press(Msg::Noop)
                .into()
        }),
        _ => return None,
    })
}

fn variant(label: &str, variant: Variant) -> Badge<'_, Msg> {
    badge(label).variant(variant)
}
