use super::Fixture;
use crate::store::Msg;

use shadcn_iced::button::{Button, Size, Variant, button, icon_button};
use shadcn_iced::button_group::{Orientation, button_group};
use shadcn_iced::prelude::lucide;

use iced::widget::{column, row};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|_| {
            row![
                button_group([icon_button(lucide::ARROW_LEFT)
                    .variant(Variant::Outline)
                    .on_press(Msg::Noop)]),
                button_group([
                    outline("Archive", Size::Default),
                    outline("Report", Size::Default),
                ]),
                button_group([
                    outline("Snooze", Size::Default),
                    icon_button(lucide::ELLIPSIS)
                        .variant(Variant::Outline)
                        .on_press(Msg::Noop),
                ]),
            ]
            .spacing(8)
            .into()
        }),
        "orientation" => Fixture::new(|_| {
            button_group([
                icon_button(lucide::PLUS)
                    .variant(Variant::Outline)
                    .on_press(Msg::Noop),
                icon_button(lucide::MINUS)
                    .variant(Variant::Outline)
                    .on_press(Msg::Noop),
            ])
            .orientation(Orientation::Vertical)
            .into()
        }),
        "size" => Fixture::new(|_| {
            let plus = |size| {
                icon_button(lucide::PLUS)
                    .variant(Variant::Outline)
                    .size(size)
                    .on_press(Msg::Noop)
            };

            column![
                button_group([
                    outline("Small", Size::Sm),
                    outline("Button", Size::Sm),
                    outline("Group", Size::Sm),
                    plus(Size::IconSm),
                ]),
                button_group([
                    outline("Default", Size::Default),
                    outline("Button", Size::Default),
                    outline("Group", Size::Default),
                    plus(Size::Icon),
                ]),
                button_group([
                    outline("Large", Size::Lg),
                    outline("Button", Size::Lg),
                    outline("Group", Size::Lg),
                    plus(Size::IconLg),
                ]),
            ]
            .spacing(32)
            .into()
        }),
        "separator" => Fixture::new(|_| {
            button_group([secondary("Copy", Size::Sm)])
                .separator()
                .push(secondary("Paste", Size::Sm))
                .into()
        }),
        "split" => Fixture::new(|_| {
            button_group([secondary("Button", Size::Default)])
                .separator()
                .push(
                    icon_button(lucide::PLUS)
                        .variant(Variant::Secondary)
                        .on_press(Msg::Noop),
                )
                .into()
        }),
        "text" => Fixture::new(|_| {
            button_group([outline("Button", Size::Default)])
                .text("Text")
                .push(outline("Button", Size::Default))
                .into()
        }),
        _ => return None,
    })
}

fn outline(label: &'static str, size: Size) -> Button<'static, Msg> {
    button(label)
        .variant(Variant::Outline)
        .size(size)
        .on_press(Msg::Noop)
}

fn secondary(label: &'static str, size: Size) -> Button<'static, Msg> {
    button(label)
        .variant(Variant::Secondary)
        .size(size)
        .on_press(Msg::Noop)
}
