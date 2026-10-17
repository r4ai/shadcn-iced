use super::Fixture;
use crate::store::{Msg, Store};

use shadcn_iced::button::{Variant, button};
use shadcn_iced::dialog::dialog;

use iced::Element;
use iced::widget::row;

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" | "close-button" | "footer" | "outside-click" | "narrow" => Fixture::new(profile),
        _ => return None,
    })
}

fn profile(store: &Store) -> Element<'_, Msg> {
    let close = || Msg::Set("open", false.into());

    dialog(
        button("Open Dialog")
            .variant(Variant::Outline)
            .on_press(Msg::Set("open", true.into())),
        store.bool("open"),
    )
    .title("Edit profile")
    .description("Make changes to your profile here. Click save when you're done.")
    .footer(
        row![
            button("Cancel").variant(Variant::Outline).on_press(close()),
            button("Save changes").on_press(close()),
        ]
        .spacing(8),
    )
    .on_close(close())
    .into()
}
