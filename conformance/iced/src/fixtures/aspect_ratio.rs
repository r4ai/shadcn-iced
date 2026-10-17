use super::Fixture;
use crate::store::Msg;

use shadcn_iced::aspect_ratio::aspect_ratio;

use iced::widget::{container, text};
use iced::{Element, Fill, Pixels, Theme};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "video" => Fixture::new(|_| {
            container(aspect_ratio(16.0 / 9.0, placeholder("16 : 9")))
                .max_width(384)
                .into()
        }),
        "square" => Fixture::new(|_| {
            container(aspect_ratio(1.0, placeholder("1 : 1")))
                .max_width(192)
                .into()
        }),
        _ => return None,
    })
}

/// `flex size-full items-center justify-center rounded-lg bg-muted text-sm
/// text-muted-foreground`, standing in for an image.
fn placeholder(label: &str) -> Element<'_, Msg> {
    container(
        text(label)
            .size(14)
            .line_height(Pixels(20.0))
            .style(|theme: &Theme| text::Style {
                color: Some(shadcn_iced::tokens(theme).palette.muted_foreground),
            }),
    )
    .center(Fill)
    .style(|theme: &Theme| {
        let t = shadcn_iced::tokens(theme);

        container::Style {
            background: Some(t.palette.muted.into()),
            border: shadcn_iced::style::rounded(t.radius_lg()),
            ..Default::default()
        }
    })
    .into()
}
