use super::Fixture;
use crate::store::Msg;

use shadcn_iced::skeleton::skeleton;

use iced::widget::{column, row, space};
use iced::{Center, Element, Fill, FillPortion};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "avatar" => Fixture::new(|_| {
            row![
                skeleton(40, 40).rounded_full(),
                column![skeleton(150, 16), skeleton(100, 16)].spacing(8),
            ]
            .spacing(16)
            .align_y(Center)
            .into()
        }),
        "lines" => Fixture::new(|_| {
            column![
                line(16),
                // `w-2/3`
                portion(2, 3, 16),
                line(32),
                Element::from(skeleton(96, 32)),
            ]
            .spacing(12)
            .max_width(384)
            .into()
        }),
        _ => return None,
    })
}

/// `w-full`
fn line<'a>(height: u32) -> Element<'a, Msg> {
    skeleton(Fill, height as f32).into()
}

/// `w-2/3`, as portions of the available width.
fn portion<'a>(part: u16, whole: u16, height: u32) -> Element<'a, Msg> {
    row![
        skeleton(FillPortion(part), height as f32),
        space().width(FillPortion(whole - part)),
    ]
    .into()
}
