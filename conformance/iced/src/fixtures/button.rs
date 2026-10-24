use super::Fixture;
use crate::store::Msg;

use shadcn_iced::button::{Size, Variant, button, icon_button};
use shadcn_iced::prelude::lucide;

use iced::Center;
use iced::widget::row;

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|_| button("Button").on_press(Msg::Noop).into()),
        "secondary" => Fixture::new(|_| variant("Secondary", Variant::Secondary)),
        "destructive" => Fixture::new(|_| variant("Destructive", Variant::Destructive)),
        "outline" => Fixture::new(|_| variant("Outline", Variant::Outline)),
        "ghost" => Fixture::new(|_| variant("Ghost", Variant::Ghost)),
        "link" => Fixture::new(|_| variant("Link", Variant::Link)),
        "sizes" => Fixture::new(|_| {
            let outline = |label, size| {
                button(label)
                    .variant(Variant::Outline)
                    .size(size)
                    .on_press(Msg::Noop)
            };

            row![
                outline("Extra Small", Size::Xs),
                outline("Small", Size::Sm),
                outline("Default", Size::Default),
                outline("Large", Size::Lg),
            ]
            .spacing(8)
            .align_y(Center)
            .into()
        }),
        "icon-sizes" => Fixture::new(|_| {
            let outline = |size| {
                icon_button(lucide::ARROW_UP_RIGHT)
                    .variant(Variant::Outline)
                    .size(size)
                    .on_press(Msg::Noop)
            };

            row![
                outline(Size::IconXs),
                outline(Size::IconSm),
                outline(Size::Icon),
                outline(Size::IconLg),
            ]
            .spacing(8)
            .align_y(Center)
            .into()
        }),
        "with-icon" => Fixture::new(|_| {
            row![
                button("New Branch")
                    .variant(Variant::Outline)
                    .icon(lucide::GIT_BRANCH)
                    .on_press(Msg::Noop),
                button("Send")
                    .trailing_icon(lucide::SEND)
                    .on_press(Msg::Noop),
            ]
            .spacing(8)
            .align_y(Center)
            .into()
        }),
        "disabled" => Fixture::new(|_| {
            row![
                button("Disabled").disabled(true),
                button("Outline").variant(Variant::Outline).disabled(true),
            ]
            .spacing(8)
            .align_y(Center)
            .into()
        }),
        "keyboard" => Fixture::new(|_| {
            row![
                button("Button").on_press(Msg::Noop),
                button("Outline")
                    .variant(Variant::Outline)
                    .on_press(Msg::Noop),
                button("Destructive")
                    .variant(Variant::Destructive)
                    .on_press(Msg::Noop),
            ]
            .spacing(8)
            .align_y(Center)
            .into()
        }),
        _ => return None,
    })
}

fn variant(label: &str, variant: Variant) -> iced::Element<'_, Msg> {
    button(label).variant(variant).on_press(Msg::Noop).into()
}
