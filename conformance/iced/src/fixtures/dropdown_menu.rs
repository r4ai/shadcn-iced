use super::Fixture;
use crate::store::{Msg, Store};

use shadcn_iced::button::{Variant, button};
use shadcn_iced::dropdown_menu::dropdown_menu;
use shadcn_iced::menu::{Menu, item, menu};
use shadcn_iced::prelude::lucide;

use iced::Element;

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "basic" | "submenu" | "keyboard" => Fixture::new(basic),
        "checkboxes" => Fixture::new(checkboxes).init(|s| {
            s.set("status_bar", true);
        }),
        "radio-group" => Fixture::new(radio_group).init(|s| {
            s.set("position", "bottom");
        }),
        "destructive" => Fixture::new(|_| {
            dropdown_menu(
                button("Destructive").variant(Variant::Outline),
                menu()
                    .item(item("Edit").icon(lucide::PENCIL).on_select(Msg::Noop))
                    .item(item("Share").icon(lucide::SHARE).on_select(Msg::Noop))
                    .separator()
                    .item(
                        item("Delete")
                            .icon(lucide::TRASH)
                            .destructive()
                            .on_select(Msg::Noop),
                    ),
            )
            .into()
        }),
        _ => return None,
    })
}

fn basic(_: &Store) -> Element<'_, Msg> {
    let noop = |label| item(label).on_select(Msg::Noop);

    dropdown_menu(
        button("Open").variant(Variant::Outline),
        menu()
            .label("My Account")
            .item(noop("Profile").shortcut("⇧⌘P"))
            .item(noop("Billing").shortcut("⌘B"))
            .item(noop("Settings").shortcut("⌘S"))
            .separator()
            .item(noop("Team"))
            .sub(
                "Invite users",
                menu()
                    .item(noop("Email"))
                    .item(noop("Message"))
                    .separator()
                    .item(noop("More...")),
            )
            .item(noop("New Team").shortcut("⌘+T"))
            .separator()
            .item(noop("GitHub"))
            .item(noop("Support"))
            .item(item("API"))
            .separator()
            .item(noop("Log out").shortcut("⇧⌘Q"))
            // `w-40`
            .width(160.0),
    )
    .into()
}

fn checkboxes(s: &Store) -> Element<'_, Msg> {
    let menu: Menu<'_, Msg> = menu()
        .label("Appearance")
        .checkbox(
            "Status Bar",
            s.bool("status_bar"),
            Msg::Toggle("status_bar"),
        )
        .checkbox_maybe("Activity Bar", false, None)
        .checkbox("Panel", s.bool("panel"), Msg::Toggle("panel"))
        // `w-40`
        .width(160.0);

    dropdown_menu(button("Checkboxes").variant(Variant::Outline), menu).into()
}

fn radio_group(s: &Store) -> Element<'_, Msg> {
    let radio = |menu: Menu<'static, Msg>, value: &'static str, label: &'static str| {
        menu.radio(
            label,
            s.str("position") == value,
            Msg::Set("position", value.into()),
        )
    };

    let menu = menu().label("Panel Position");
    let menu = radio(menu, "top", "Top");
    let menu = radio(menu, "bottom", "Bottom");
    let menu = radio(menu, "right", "Right");

    // `w-32`
    dropdown_menu(
        button("Radio Group").variant(Variant::Outline),
        menu.width(128.0),
    )
    .into()
}
