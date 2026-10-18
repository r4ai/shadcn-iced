use super::Fixture;
use crate::store::{Msg, Store, Value};

use shadcn_iced::aspect_ratio::aspect_ratio;
use shadcn_iced::card::card;
use shadcn_iced::carousel::{Orientation, carousel};
use shadcn_iced::font;
use shadcn_iced::style::text::leading;

use iced::widget::{container, text};
use iced::{Element, Fill};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" | "keyboard" => Fixture::new(|s| {
            // `max-w-xs`, with the buttons outside of it (`px-12`).
            container(carousel(index(s), go).items((1..=5).map(|n| slide(n, 36.0, true))))
                .max_width(320 + 96)
                .into()
        }),
        "sizes" => Fixture::new(|s| {
            // `max-w-sm`, `basis-1/3`
            container(
                carousel(index(s), go)
                    .items((1..=5).map(|n| slide(n, 30.0, true)))
                    .per_view(3),
            )
            .max_width(384 + 96)
            .into()
        }),
        "vertical" => Fixture::new(|s| {
            // `max-w-xs`, `-mt-1 h-[200px]` (a 196px tall viewport), `pt-1
            // basis-1/2` (the cards are taller than half of it)
            container(
                carousel(index(s), go)
                    .items((1..=5).map(|n| slide(n, 30.0, false)))
                    .per_view(2)
                    .spacing(4.0)
                    .height(196)
                    .orientation(Orientation::Vertical),
            )
            .max_width(320)
            .padding([0, 0])
            .into()
        }),
        _ => return None,
    })
}

fn index(s: &Store) -> usize {
    s.num("slide") as usize
}

fn go(index: usize) -> Msg {
    Msg::Set("slide", Value::Num(index as f32))
}

/// `p-1` around a card with `flex aspect-square items-center justify-center
/// p-6` content (`flex items-center justify-center p-6` when not `square`).
fn slide(n: usize, size: f32, square: bool) -> Element<'static, Msg> {
    let number = container(
        text(n.to_string())
            .size(size)
            .line_height(leading(size))
            .font(font::semibold()),
    )
    .center_x(Fill);

    // With the `p-6`, a square content keeps the whole content square.
    let content: Element<'static, Msg> = if square {
        aspect_ratio(1.0, number.center_y(Fill)).into()
    } else {
        number.into()
    };

    container(card().content_padding(24).content(content))
        .padding(4)
        .into()
}
