use super::Fixture;
use crate::store::{Msg, Store, set};

use shadcn_iced::select::{Size, select};

use iced::Element;
use iced::widget::column;

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "basic" | "reopen" | "keyboard" => Fixture::new(basic),
        "below-trigger" => Fixture::new(below_trigger),
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
