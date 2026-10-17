use super::Fixture;
use crate::store::{Msg, Store, Value};

use shadcn_iced::native_select::{NativeSelect, invalid, native_select, small};

use iced::widget::column;

const OPTIONS: &[&str] = &["Todo", "In Progress", "Done", "Cancelled"];

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|store| status(store).into()),
        "selected" => Fixture::new(|store| status(store).into()).init(|store| {
            store.set("status", "In Progress");
        }),
        "sizes" => Fixture::new(|store| {
            column![status(store), small(status(store))]
                .spacing(12)
                .into()
        })
        .init(|store| {
            store.set("status", "Todo");
        }),
        "keyboard" => Fixture::new(|store| status(store).into()),
        "states" => Fixture::new(|store| {
            column![status(store).disabled(true), invalid(status(store))]
                .spacing(12)
                .into()
        })
        .init(|store| {
            store.set("status", "Todo");
        }),
        _ => return None,
    })
}

type Select<'a> = NativeSelect<'a, &'static str, Msg>;

fn selected(store: &Store) -> Option<&'static str> {
    let value = store.opt("status")?;

    OPTIONS.iter().copied().find(|option| *option == value)
}

fn select(value: &'static str) -> Msg {
    Msg::Set("status", Value::from(value))
}

fn status(store: &Store) -> Select<'static> {
    native_select(OPTIONS, selected(store), select).placeholder("Select status")
}
