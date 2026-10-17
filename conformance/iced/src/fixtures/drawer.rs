use super::Fixture;
use crate::store::{Msg, Store, Value};

use shadcn_iced::button::{Variant, button};
use shadcn_iced::drawer::{self, drawer};

use iced::widget::{column, row};
use iced::{Element, Fill};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" | "footer" | "outside-click" => Fixture::new(|store| single(store, false)),
        "swipe-handle" => Fixture::new(|store| single(store, true)),
        "directions" => Fixture::new(directions),
        _ => return None,
    })
}

fn open(value: bool) -> Msg {
    Msg::Set("open", value.into())
}

/// The drawer of the "Are you absolutely sure?" example, with `trigger` as
/// the base.
fn confirm<'a>(
    trigger: impl Into<Element<'a, Msg>>,
    is_open: bool,
    direction: drawer::Direction,
    swipe_handle: bool,
    close: Msg,
) -> Element<'a, Msg> {
    drawer(trigger, is_open)
        .direction(direction)
        .swipe_handle(swipe_handle)
        .title("Are you absolutely sure?")
        .description("This action cannot be undone.")
        .footer(
            column![
                button("Submit").width(Fill).on_press(Msg::Noop),
                button("Cancel")
                    .variant(Variant::Outline)
                    .width(Fill)
                    .on_press(close.clone()),
            ]
            .spacing(8),
        )
        .on_close(close)
        .into()
}

fn single(store: &Store, swipe_handle: bool) -> Element<'_, Msg> {
    confirm(
        button("Open Drawer")
            .variant(Variant::Outline)
            .on_press(open(true)),
        store.bool("open"),
        drawer::Direction::Down,
        swipe_handle,
        open(false),
    )
}

fn directions(store: &Store) -> Element<'_, Msg> {
    let trigger = |label: &'static str| {
        button(label)
            .variant(Variant::Outline)
            .on_press(Msg::Set("direction", label.into()))
    };

    let current = store.opt("direction");
    let direction = match current {
        Some("up") => drawer::Direction::Up,
        Some("right") => drawer::Direction::Right,
        Some("left") => drawer::Direction::Left,
        _ => drawer::Direction::Down,
    };

    confirm(
        row![
            trigger("up"),
            trigger("right"),
            trigger("down"),
            trigger("left")
        ]
        .spacing(8),
        current.is_some(),
        direction,
        false,
        Msg::Set("direction", Value::None),
    )
}
