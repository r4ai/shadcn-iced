use super::Fixture;
use crate::store::{Msg, Store, Value};

use shadcn_iced::pagination::pagination;

use iced::Element;

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(default).init(|s| {
            s.set("page", 2.0);
        }),
        "simple" => Fixture::new(simple).init(|s| {
            s.set("page", 3.0);
        }),
        _ => return None,
    })
}

fn go(page: usize) -> Msg {
    Msg::Set("page", Value::Num(page as f32))
}

/// 10 pages, one sibling, previous and next.
fn default(s: &Store) -> Element<'_, Msg> {
    pagination(s.num("page") as usize, 10, go).into()
}

/// 5 pages, page numbers only.
fn simple(s: &Store) -> Element<'_, Msg> {
    pagination(s.num("page") as usize, 5, go)
        .siblings(5)
        .controls(false)
        .into()
}
