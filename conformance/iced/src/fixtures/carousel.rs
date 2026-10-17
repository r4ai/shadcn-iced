use super::Fixture;
use crate::store::{Msg, Store, Value};

use shadcn_iced::card::card;
use shadcn_iced::carousel::{Orientation, carousel};
use shadcn_iced::font;
use shadcn_iced::style::text::leading;

use iced::widget::{container, text};
use iced::{Element, Fill, Length};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" | "keyboard" => Fixture::new(|s| {
            // `max-w-xs`, with the buttons outside of it (`px-12`).
            container(carousel(index(s), go).items((1..=5).map(|n| slide(n, 36.0, Some(SQUARE)))))
                .max_width(320 + 96)
                .into()
        }),
        "sizes" => Fixture::new(|s| {
            // `max-w-sm`, `basis-1/3`
            container(
                carousel(index(s), go)
                    .items((1..=5).map(|n| slide(n, 30.0, Some(SQUARE_THIRD))))
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
                    .items((1..=5).map(|n| slide(n, 30.0, None)))
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

/// The width of the cards of a `max-w-xs` carousel (`p-1`).
const SQUARE: f32 = 320.0 - 8.0;
/// The width of the cards of a `max-w-sm` carousel of `basis-1/3` items
/// (`pl-4` and `p-1`).
const SQUARE_THIRD: f32 = (384.0 + 16.0) / 3.0 - 16.0 - 8.0;

/// `p-1` around a card with `flex aspect-square items-center justify-center
/// p-6` content.
///
/// The `p-6` of the content replaces the `px-4` of the card: the content is
/// as tall as the card is wide (`side`), whereas iced cards always pad
/// their content.
fn slide(n: usize, size: f32, side: Option<f32>) -> Element<'static, Msg> {
    let number = container(
        text(n.to_string())
            .size(size)
            .line_height(leading(size))
            .font(font::semibold()),
    )
    .center_x(Fill)
    .center_y(Fill)
    .padding(24);

    let content: Element<'static, Msg> = match side {
        Some(side) => number.height(side).into(),
        None => number.height(Length::Shrink).into(),
    };

    container(card().content(content)).padding(4).into()
}
