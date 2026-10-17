use super::Fixture;
use crate::store::{Msg, Store};

use shadcn_iced::menu::{Menu, item, menu};
use shadcn_iced::menubar::menubar;

use iced::Element;

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "basic" | "submenu" | "switch-menus" | "keyboard" | "checkboxes" | "radio-group" => {
            Fixture::new(example).init(|s| {
                s.set("urls", true);
                s.set("profile", "benoit");
            })
        }
        _ => return None,
    })
}

fn example(s: &Store) -> Element<'_, Msg> {
    let noop = |label| item(label).on_select(Msg::Noop);

    let profile = |menu: Menu<'static, Msg>, value: &'static str, label: &'static str| {
        menu.radio(
            label,
            s.str("profile") == value,
            Msg::Set("profile", value.into()),
        )
    };

    let profiles = menu();
    let profiles = profile(profiles, "andy", "Andy");
    let profiles = profile(profiles, "benoit", "Benoit");
    let profiles = profile(profiles, "luis", "Luis");

    menubar()
        .menu(
            "File",
            menu()
                .item(noop("New Tab").shortcut("⌘T"))
                .item(noop("New Window").shortcut("⌘N"))
                .item(item("New Incognito Window"))
                .separator()
                .sub(
                    "Share",
                    menu()
                        .item(noop("Email link"))
                        .item(noop("Messages"))
                        .item(noop("Notes")),
                )
                .separator()
                .item(noop("Print...").shortcut("⌘P")),
        )
        .menu(
            "Edit",
            menu()
                .item(noop("Undo").shortcut("⌘Z"))
                .item(noop("Redo").shortcut("⇧⌘Z"))
                .separator()
                .sub(
                    "Find",
                    menu()
                        .item(noop("Search the web"))
                        .separator()
                        .item(noop("Find..."))
                        .item(noop("Find Next"))
                        .item(noop("Find Previous")),
                )
                .separator()
                .item(noop("Cut"))
                .item(noop("Copy"))
                .item(noop("Paste")),
        )
        .menu(
            "View",
            menu()
                .checkbox(
                    "Bookmarks Bar",
                    s.bool("bookmarks"),
                    Msg::Toggle("bookmarks"),
                )
                .checkbox("Full URLs", s.bool("urls"), Msg::Toggle("urls"))
                .separator()
                .item(noop("Reload").inset().shortcut("⌘R"))
                .item(item("Force Reload").inset().shortcut("⇧⌘R"))
                .separator()
                .item(noop("Toggle Fullscreen").inset())
                .separator()
                .item(noop("Hide Sidebar").inset())
                // `w-44`
                .width(176.0),
        )
        .menu(
            "Profiles",
            profiles
                .separator()
                .item(noop("Edit...").inset())
                .separator()
                .item(noop("Add Profile...").inset()),
        )
        .into()
}
