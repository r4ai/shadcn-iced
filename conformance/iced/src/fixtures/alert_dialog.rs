use super::Fixture;
use crate::store::{Msg, Store};

use shadcn_iced::alert_dialog::{self, alert_dialog};
use shadcn_iced::button::{Variant, button};
use shadcn_iced::prelude::lucide;

use iced::Element;

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" | "cancel" | "outside-click" | "narrow" => Fixture::new(confirm),
        "small-media" => Fixture::new(small),
        _ => return None,
    })
}

fn open(value: bool) -> Msg {
    Msg::Set("open", value.into())
}

fn confirm(store: &Store) -> Element<'_, Msg> {
    alert_dialog(
        button("Show Dialog")
            .variant(Variant::Outline)
            .on_press(open(true)),
        store.bool("open"),
    )
    .title("Are you absolutely sure?")
    .description(
        "This action cannot be undone. This will permanently delete your account from our servers.",
    )
    .cancel("Cancel", open(false))
    .action("Continue", open(false))
    .into()
}

fn small(store: &Store) -> Element<'_, Msg> {
    alert_dialog(
        button("Show Small Dialog")
            .variant(Variant::Outline)
            .on_press(open(true)),
        store.bool("open"),
    )
    .size(alert_dialog::Size::Sm)
    .media(lucide::BLUETOOTH)
    .title("Allow accessory to connect?")
    .description("Do you want to allow the USB accessory to connect to this device?")
    .cancel("Don't allow", open(false))
    .action("Allow", open(false))
    .into()
}
