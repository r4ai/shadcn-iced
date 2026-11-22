use super::Fixture;
use crate::store::{Msg, Store, set};

use shadcn_iced::select::{Size, select};

use iced::widget::{column, container};
use iced::{Element, Padding};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "basic" | "reopen" | "keyboard" => Fixture::new(basic),
        "below-trigger" => Fixture::new(below_trigger),
        "long-top" => Fixture::new(|s| long(s, 48, "12")),
        "long-middle" => Fixture::new(|s| long(s, 168, "2")),
        "long-bottom" => Fixture::new(|s| long(s, 304, "30")),
        "trigger-states" => Fixture::new(trigger_states),
        _ => return None,
    })
}

fn basic(s: &Store) -> Element<'_, Msg> {
    // `w-full max-w-48`
    select(s.opt("fruit"), set("fruit"))
        .placeholder("Select a fruit")
        .label("Fruits")
        .option("apple", "Apple")
        .option("banana", "Banana")
        .option("blueberry", "Blueberry")
        .option("grapes", "Grapes")
        .option("pineapple", "Pineapple")
        .width(192)
        .into()
}

fn below_trigger(s: &Store) -> Element<'_, Msg> {
    select(s.opt("fruit"), set("fruit"))
        .placeholder("Select a fruit")
        .option("apple", "Apple")
        .option("banana", "Banana")
        .option("blueberry", "Blueberry")
        .separator()
        .option("grapes", "Grapes")
        .disabled_option("pineapple", "Pineapple")
        .align_item_with_trigger(false)
        .width(192)
        .into()
}

/// 30 items, which do not fit in the window, `top` pixels down the page.
fn long<'a>(s: &'a Store, top: u32, default: &'static str) -> Element<'a, Msg> {
    let selected = s.opt("item").unwrap_or(default);

    container(
        (1..=30)
            .fold(select(Some(selected), set("item")), |select, item| {
                select.option(ITEMS[item - 1], format!("Item {item}"))
            })
            .width(192),
    )
    .padding(Padding::ZERO.top(top))
    .into()
}

const ITEMS: [&str; 30] = [
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17",
    "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30",
];

fn trigger_states(_: &Store) -> Element<'_, Msg> {
    column![
        select(None, set("a"))
            .placeholder("Default")
            .option("apple", "Apple")
            .width(192),
        select(None, set("b"))
            .placeholder("Small")
            .option("apple", "Apple")
            .size(Size::Sm)
            .width(192),
        select(Some("banana"), set("c"))
            .option("banana", "Banana")
            .width(192),
        select(None, set("d"))
            .placeholder("Disabled")
            .option("apple", "Apple")
            .disabled(true)
            .width(192),
    ]
    .spacing(8)
    .into()
}
