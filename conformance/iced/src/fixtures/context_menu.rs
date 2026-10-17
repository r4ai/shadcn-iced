use super::Fixture;
use crate::store::{Msg, Store};

use shadcn_iced::context_menu::context_menu;
use shadcn_iced::menu::{Menu, item, menu};

use iced::Element;
use iced::widget::{container, text};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "trigger" | "open" | "submenu" | "checkboxes" | "radio-group" => Fixture::new(example)
            .init(|s| {
                s.set("bookmarks", true);
                s.set("person", "pedro");
            }),
        _ => return None,
    })
}

fn example(s: &Store) -> Element<'_, Msg> {
    let noop = |label| item(label).on_select(Msg::Noop);

    let person = |menu: Menu<'static, Msg>, value: &'static str, label: &'static str| {
        menu.radio(
            label,
            s.str("person") == value,
            Msg::Set("person", value.into()),
        )
    };

    // `aspect-video w-full max-w-xs rounded-xl border border-dashed text-sm`
    let area = container(text("Right click here").size(14))
        .center_x(320)
        .center_y(180)
        .style(|theme: &iced::Theme| {
            let t = shadcn_iced::tokens(theme);

            container::Style {
                border: shadcn_iced::style::border(t.palette.border, t.radius_xl()),
                ..Default::default()
            }
        });

    let menu = menu()
        .item(noop("Back").shortcut("⌘["))
        .item(item("Forward").shortcut("⌘]"))
        .item(noop("Reload").shortcut("⌘R"))
        .sub(
            "More Tools",
            menu()
                .item(noop("Save Page..."))
                .item(noop("Create Shortcut..."))
                .item(noop("Name Window..."))
                .separator()
                .item(noop("Developer Tools"))
                .separator()
                .item(item("Delete").destructive().on_select(Msg::Noop))
                // `w-44`
                .min_width(176.0),
        )
        .separator()
        .checkbox(
            "Show Bookmarks",
            s.bool("bookmarks"),
            Msg::Toggle("bookmarks"),
        )
        .checkbox("Show Full URLs", s.bool("urls"), Msg::Toggle("urls"))
        .separator()
        .inset_label("People");

    let menu = person(menu, "pedro", "Pedro Duarte");
    // `w-48`
    let menu = person(menu, "colm", "Colm Tuite").min_width(192.0);

    context_menu(area, menu).into()
}
