use super::Fixture;
use crate::store::Msg;

use shadcn_iced::button::{Button, Variant};
use shadcn_iced::kbd::{kbd, kbd_group};

use iced::widget::{column, float, row, text};
use iced::{Center, Fill, Pixels, Theme, Vector};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "demo" => Fixture::new(|_| {
            column![
                kbd_group([kbd("⌘"), kbd("⇧"), kbd("⌥"), kbd("⌃")]),
                // The `<span>` inherits the 16px text of the page.
                kbd_group([
                    kbd("Ctrl"),
                    text("+").size(16).line_height(Pixels(24.0)).into(),
                    kbd("B"),
                ]),
            ]
            .spacing(16)
            .width(Fill)
            .align_x(Center)
            .into()
        }),
        "group" => Fixture::new(|_| {
            row![
                sm("Use").style(muted),
                kbd_group([kbd("Ctrl + B"), kbd("Ctrl + K")]),
                sm("to open the command palette").style(muted),
            ]
            .spacing(4)
            .align_y(Center)
            .into()
        }),
        // `<Kbd data-icon="inline-end" className="translate-x-0.5">`
        "button" => Fixture::new(|_| {
            Button::new(
                row![
                    text("Accept").size(14).font(shadcn_iced::font::medium()),
                    // `translate-x-0.5` moves the key without resizing the
                    // button.
                    float(kbd("⏎")).translate(|_, _| Vector::new(2.0, 0.0)),
                ]
                .spacing(6)
                .align_y(Center),
            )
            .variant(Variant::Outline)
            .inline_end(true)
            .on_press(Msg::Noop)
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
