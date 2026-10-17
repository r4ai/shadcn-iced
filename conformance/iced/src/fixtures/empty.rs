use super::Fixture;
use crate::store::Msg;

use shadcn_iced::avatar::avatar;
use shadcn_iced::button::{Button, Size, Variant, button};
use shadcn_iced::empty::empty;
use shadcn_iced::prelude::lucide;

use iced::widget::{row, text};
use iced::{Center, Theme};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "demo" => Fixture::new(|_| {
            empty::<Msg>("No Projects Yet")
                .icon(lucide::FOLDER_CODE)
                .description(
                    "You haven't created any projects yet. Get started by creating your first project.",
                )
                .content(
                    row![
                        button("Create Project").on_press(Msg::Noop),
                        button("Import Project")
                            .variant(Variant::Outline)
                            .on_press(Msg::Noop),
                    ]
                    .spacing(8),
                )
                .extra(
                    Button::new(
                        row![
                            text("Learn More")
                                .size(12.8)
                                .font(shadcn_iced::font::medium())
                                .style(muted),
                            lucide::ARROW_UP_RIGHT.sized(14).style(muted),
                        ]
                        .spacing(4)
                        .align_y(Center),
                    )
                    .variant(Variant::Link)
                    .size(Size::Sm)
                    .on_press(Msg::Noop),
                )
                .into()
        }),
        "outline" => Fixture::new(|_| {
            empty::<Msg>("Cloud Storage Empty")
                .icon(lucide::CLOUD)
                .description("Upload files to your cloud storage to access them anywhere.")
                .content(
                    button("Upload Files")
                        .variant(Variant::Outline)
                        .size(Size::Sm)
                        .on_press(Msg::Noop),
                )
                .bordered(true)
                .into()
        }),
        "background" => Fixture::new(|_| {
            empty::<Msg>("No Notifications")
                .icon(lucide::BELL)
                .description("You're all caught up. New notifications will appear here.")
                .content(
                    button("Refresh")
                        .icon(lucide::REFRESH_CCW)
                        .variant(Variant::Outline)
                        .on_press(Msg::Noop),
                )
                .muted(true)
                .into()
        }),
        "avatar" => Fixture::new(|_| {
            empty::<Msg>("User Offline")
                .media(avatar("LR").diameter(48.0))
                .description(
                    "This user is currently offline. You can leave a message to notify them or try again later.",
                )
                .content(button("Leave Message").size(Size::Sm).on_press(Msg::Noop))
                .into()
        }),
        _ => return None,
    })
}

/// `text-muted-foreground`
fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(shadcn_iced::tokens(theme).palette.muted_foreground),
    }
}
