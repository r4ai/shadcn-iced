use super::Fixture;
use crate::store::{Msg, Store, Value};

use shadcn_iced::button::{Variant, button};
use shadcn_iced::sheet::{self, sheet};

use iced::widget::{column, row};
use iced::{Element, Fill};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" | "close-button" | "footer" | "outside-click" => Fixture::new(single),
        "sides" => Fixture::new(sides),
        _ => return None,
    })
}

fn open(value: bool) -> Msg {
    Msg::Set("open", value.into())
}

/// The sheet of a `SheetContent`, with `trigger` as the base.
fn profile<'a>(
    trigger: impl Into<Element<'a, Msg>>,
    is_open: bool,
    side: sheet::Side,
    close: Msg,
) -> Element<'a, Msg> {
    sheet(trigger, is_open)
        .side(side)
        .title("Edit profile")
        .description("Make changes to your profile here. Click save when you're done.")
        .footer(
            column![
                button("Save changes").width(Fill).on_press(close.clone()),
                button("Close")
                    .variant(Variant::Outline)
                    .width(Fill)
                    .on_press(close.clone()),
            ]
            .spacing(8),
        )
        .on_close(close)
        .into()
}

fn single(store: &Store) -> Element<'_, Msg> {
    profile(
        button("Open")
            .variant(Variant::Outline)
            .on_press(open(true)),
        store.bool("open"),
        sheet::Side::Right,
        open(false),
    )
}

fn sides(store: &Store) -> Element<'_, Msg> {
    let open = |side: &'static str| Msg::Set("side", side.into());
    let trigger = |side: &'static str| button(side).variant(Variant::Outline).on_press(open(side));

    let current = store.opt("side");
    let side = match current {
        Some("top") => sheet::Side::Top,
        Some("bottom") => sheet::Side::Bottom,
        Some("left") => sheet::Side::Left,
        _ => sheet::Side::Right,
    };

    profile(
        row![
            trigger("top"),
            trigger("right"),
            trigger("bottom"),
            trigger("left")
        ]
        .spacing(8),
        current.is_some(),
        side,
        Msg::Set("side", Value::None),
    )
}
