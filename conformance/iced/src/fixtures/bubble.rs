use super::Fixture;
use crate::store::Msg;

use shadcn_iced::bubble::{Align, Side, Variant, bubble, bubble_group};

use iced::widget::{column, container, text};
use iced::{Element, Fill};

/// `w-full max-w-sm`
fn sm(content: Element<'_, Msg>) -> Element<'_, Msg> {
    container(content).width(Fill).max_width(384).into()
}

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "variants" => Fixture::new(|_| {
            sm(column![
                bubble("This is the default variant."),
                bubble("This is the secondary variant.")
                    .variant(Variant::Secondary)
                    .align(Align::End),
                bubble("This is the muted variant.").variant(Variant::Muted),
                bubble("This is the tinted variant.")
                    .variant(Variant::Tinted)
                    .align(Align::End),
                bubble("This is the outline variant.").variant(Variant::Outline),
                bubble("This is the destructive variant.")
                    .variant(Variant::Destructive)
                    .align(Align::End),
                bubble("This is the ghost variant.").variant(Variant::Ghost),
            ]
            .spacing(16)
            .into())
        }),
        "group-and-reactions" => Fixture::new(|_| {
            sm(column![
                bubble_group([
                    bubble("Hey! Want to see chat bubbles?")
                        .variant(Variant::Muted)
                        .into(),
                    bubble("I can group messages and keep them easy to scan.")
                        .variant(Variant::Muted)
                        .reactions("👍")
                        .into(),
                ]),
                bubble("Tests passed on the first try.")
                    .align(Align::End)
                    .reactions("🎉 👏")
                    .reactions_side(Side::Top)
                    .reactions_align(Align::Start),
                bubble("Find the bug and fix it.")
                    .align(Align::End)
                    .reactions("👀")
                    .reactions_align(Align::Start),
            ]
            .spacing(32)
            .into())
        }),
        "button" => Fixture::new(|store| {
            sm(column![
                bubble_group([
                    bubble("I forgot my password")
                        .variant(Variant::Tinted)
                        .align(Align::End)
                        .on_press(Msg::Toggle("picked"))
                        .into(),
                    bubble("Something else. Talk to a human.")
                        .variant(Variant::Tinted)
                        .align(Align::End)
                        .on_press(Msg::Noop)
                        .into(),
                ]),
                text(if store.bool("picked") {
                    "Picked"
                } else {
                    "Nothing picked"
                })
                .size(14),
            ]
            .spacing(16)
            .into())
        }),
        _ => return None,
    })
}
