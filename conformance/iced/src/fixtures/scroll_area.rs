use super::Fixture;
use crate::store::Msg;

use shadcn_iced::scroll_area::{horizontal_scroll_area, scroll_area};
use shadcn_iced::separator::separator;
use shadcn_iced::style::border;
use shadcn_iced::{font, tokens};

use iced::widget::{column, container, row, space, text};
use iced::{Element, Fill, Padding, Pixels, Theme};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|_| {
            // `h-72 w-48 rounded-md border`
            // Every separator has a `my-2`, the last one included.
            let tags = column((1..=50).rev().map(|i| {
                column![
                    text(format!("v1.2.0-beta.{i}"))
                        .size(14)
                        .line_height(Pixels(20.0)),
                    separator()
                ]
                .spacing(8)
                .padding(Padding::default().bottom(8))
                .into()
            }));

            container(
                scroll_area(
                    column![
                        text("Tags")
                            .size(14)
                            .line_height(Pixels(14.0))
                            .font(font::medium()),
                        tags
                    ]
                    .spacing(16)
                    .padding(16)
                    .width(Fill),
                )
                .height(Fill),
            )
            .width(192)
            .height(288)
            .padding(1)
            .style(frame)
            .into()
        }),
        "horizontal" => Fixture::new(|_| {
            // `w-96 rounded-md border`, `flex w-max space-x-4 p-4`
            let works = row(["Ornella Binni", "Tom Byrom", "Vladimir Malyavko"].map(figure))
                .spacing(16)
                .padding(16);

            container(horizontal_scroll_area(works))
                .width(384)
                .padding(1)
                .style(frame)
                .into()
        }),
        _ => return None,
    })
}

/// `rounded-md border`
fn frame(theme: &Theme) -> container::Style {
    let t = tokens(theme);

    container::Style {
        border: border(t.palette.border, t.radius_md()),
        ..container::Style::default()
    }
}

/// A `h-[200px] w-[150px] rounded-md bg-muted` box with a caption in
/// `pt-2 text-xs`.
fn figure(artist: &'static str) -> Element<'static, Msg> {
    let image = container(space())
        .width(150)
        .height(200)
        .style(|theme: &Theme| {
            let t = tokens(theme);

            container::Style {
                background: Some(t.palette.muted.into()),
                border: shadcn_iced::style::rounded(t.radius_md()),
                ..container::Style::default()
            }
        });

    let caption = row![
        text("Photo by")
            .size(12)
            .line_height(Pixels(16.0))
            .style(|theme: &Theme| text::Style {
                color: Some(tokens(theme).palette.muted_foreground),
            }),
        text(artist)
            .size(12)
            .line_height(Pixels(16.0))
            .font(font::semibold()),
    ]
    .spacing(4);

    column![image, caption].spacing(8).into()
}
