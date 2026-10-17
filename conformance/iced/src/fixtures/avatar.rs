use super::Fixture;

use shadcn_iced::avatar::{self, Size, avatar, avatar_group};
use shadcn_iced::prelude::lucide;

use iced::widget::{column, row};
use iced::{Center, Color};

/// `bg-green-600`
const GREEN: Color = Color::from_rgb(
    0x16 as f32 / 255.0,
    0xa3 as f32 / 255.0,
    0x4a as f32 / 255.0,
);

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "sizes" => Fixture::new(|_| {
            row![
                avatar("CN").size(Size::Sm),
                avatar("CN"),
                avatar("CN").size(Size::Lg),
            ]
            .spacing(16)
            .align_y(Center)
            .into()
        }),
        "badge" => Fixture::new(|_| {
            row![
                avatar("CN").badge(Some(GREEN)),
                avatar("PP").badge_icon(lucide::PLUS),
            ]
            .spacing(16)
            .into()
        }),
        "group" => Fixture::new(|_| {
            column![
                avatar_group([
                    avatar("CN"),
                    avatar("LR"),
                    avatar("ER"),
                    avatar::avatar_group_count("+3"),
                ]),
                avatar_group([
                    avatar("CN"),
                    avatar("LR"),
                    avatar("ER"),
                    avatar::avatar_group_count_icon(lucide::PLUS),
                ]),
            ]
            .spacing(16)
            .into()
        }),
        _ => return None,
    })
}
