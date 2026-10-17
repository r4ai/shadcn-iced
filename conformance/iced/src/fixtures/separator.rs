use super::Fixture;
use crate::store::Msg;

use shadcn_iced::font;
use shadcn_iced::separator::{separator, vertical_separator};

use iced::widget::{column, row, space, text};
use iced::{Center, Element, Pixels, Theme};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "demo" => Fixture::new(|_| {
            column![
                column![
                    text("shadcn/ui")
                        .size(14)
                        .line_height(Pixels(14.0))
                        .font(font::medium()),
                    sm("The Foundation for your Design System").style(muted),
                ]
                .spacing(6),
                separator(),
                sm(
                    "A set of beautifully designed components that you can customize, extend, and build on."
                ),
            ]
            .spacing(16)
            .max_width(384)
            .into()
        }),
        "vertical" => Fixture::new(|_| {
            row![
                sm("Blog"),
                vertical_separator(),
                sm("Docs"),
                vertical_separator(),
                sm("Source"),
            ]
            .spacing(16)
            .height(20)
            .align_y(Center)
            .into()
        }),
        "list" => Fixture::new(|_| {
            column![
                entry("Item 1", "Value 1"),
                separator(),
                entry("Item 2", "Value 2"),
                separator(),
                entry("Item 3", "Value 3"),
            ]
            .spacing(8)
            .max_width(384)
            .into()
        }),
        _ => return None,
    })
}

/// `text-sm`
fn sm<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a> {
    text(content).size(14).line_height(Pixels(20.0))
}

/// `text-muted-foreground`
fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(shadcn_iced::tokens(theme).palette.muted_foreground),
    }
}

/// `<dl className="flex items-center justify-between">`
fn entry<'a>(key: &'a str, value: &'a str) -> Element<'a, Msg> {
    row![sm(key), space::horizontal(), sm(value).style(muted)]
        .align_y(Center)
        .into()
}
