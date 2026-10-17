use super::Fixture;
use crate::store::{Msg, Store, Value};

use shadcn_iced::button::{Variant, button};
use shadcn_iced::toast::{Kind, Toast, toaster};

use iced::Element;
use iced::widget::row;

fn hide(key: &'static str) -> Msg {
    Msg::Set(key, Value::Bool(false))
}

fn show(key: &'static str) -> Msg {
    Msg::Set(key, Value::Bool(true))
}

/// The toast of each store key; the shadcn/ui fixtures create them on click.
fn toasts(store: &Store) -> Vec<Toast<Msg>> {
    let mut toasts = Vec::new();

    if store.bool("default") {
        toasts.push(
            Toast::new(1, "Event created")
                .description("Sunday, December 3 at 9:00 AM")
                .action("Undo", hide("default")),
        );
    }

    if store.bool("simple") {
        toasts.push(Toast::new(2, "Event created"));
    }

    if store.bool("success") {
        toasts.push(Toast::new(3, "Event created").kind(Kind::Success));
    }

    if store.bool("error") {
        toasts.push(Toast::new(4, "Event failed").kind(Kind::Error));
    }

    toasts
}

fn close(id: u64) -> Msg {
    hide(match id {
        1 => "default",
        2 => "simple",
        3 => "success",
        _ => "error",
    })
}

fn outline(label: &'static str, on_press: Msg) -> Element<'static, Msg> {
    button(label)
        .variant(Variant::Outline)
        .on_press(on_press)
        .into()
}

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|store| {
            toaster(
                outline("Show Toast", show("default")),
                &toasts(store),
                close,
            )
            .into()
        }),
        "auto-dismiss" => Fixture::new(|store| {
            toaster(outline("Show Toast", show("simple")), &toasts(store), close).into()
        }),
        "types" => Fixture::new(|store| {
            toaster(
                row![
                    outline("Success", show("success")),
                    outline("Error", show("error")),
                ]
                .spacing(8),
                &toasts(store),
                close,
            )
            .into()
        }),
        _ => return None,
    })
}
