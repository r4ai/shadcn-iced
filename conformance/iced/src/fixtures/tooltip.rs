use super::Fixture;

use shadcn_iced::button::{Variant, button};
use shadcn_iced::primitive::Side;
use shadcn_iced::tooltip::tooltip;

use iced::widget::{container, row};
use iced::{Element, Padding};

use crate::store::Msg;

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        // `pt-12`
        "default" => Fixture::new(|_| {
            container(tooltip(
                button("Hover").variant(Variant::Outline),
                "Add to library",
            ))
            .padding(Padding::ZERO.top(48))
            .into()
        }),
        // `flex gap-2 pt-16 pl-32`
        "sides" => Fixture::new(|_| {
            container(
                row![
                    side("Left", Side::Left),
                    side("Top", Side::Top),
                    side("Bottom", Side::Bottom),
                    side("Right", Side::Right),
                ]
                .spacing(8),
            )
            .padding(Padding::ZERO.top(64).left(128))
            .into()
        }),
        _ => return None,
    })
}

fn side(label: &'static str, side: Side) -> Element<'static, Msg> {
    tooltip(button(label).variant(Variant::Outline), "Add to library")
        .side(side)
        .into()
}
