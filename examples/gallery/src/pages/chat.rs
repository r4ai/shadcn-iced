//! Pages of the chat components.
use super::Page;
use crate::{Message, example};

use iced_ui::attachment::{self, State};
use iced_ui::bubble::{Align, Variant};
use iced_ui::marker::Variant as MarkerVariant;
use iced_ui::prelude::*;

use iced::widget::{column, container, row};
use iced::Fill;

pub const BUBBLE: Page = Page {
    id: "bubble",
    title: "Bubble",
    description: "A chat bubble with variants and reactions.",
    view: |_| {
        example(
            "Variants",
            column![
                bubble("Default bubble"),
                bubble("Secondary bubble").variant(Variant::Secondary),
                bubble("Muted bubble").variant(Variant::Muted).align(Align::End),
                bubble("Tinted bubble").variant(Variant::Tinted),
                bubble("Outline bubble").variant(Variant::Outline).align(Align::End),
                bubble("Ghost bubble: no background, no padding.").variant(Variant::Ghost),
                bubble("Something went wrong.").variant(Variant::Destructive),
                bubble("Nice work on the release!").variant(Variant::Muted).reactions("🎉 3"),
            ]
            .spacing(12)
            .max_width(520),
        )
    },
};

fn conversation<'a>() -> Vec<iced::Element<'a, Message>> {
    vec![
        marker("Today").variant(MarkerVariant::Separator).into(),
        message()
            .avatar(avatar("OM"))
            .header("Olivia Martin")
            .bubble(bubble("Hey! Did you get a chance to look at the new components?").variant(Variant::Muted))
            .bubble(bubble("The overlays feel really snappy now.").variant(Variant::Muted))
            .into(),
        message()
            .align(Align::End)
            .bubble(bubble("Yes! Dropdowns, selects and dialogs all work."))
            .push(
                attachment("screenshot.png")
                    .icon(lucide::IMAGE)
                    .description("1.2 MB"),
            )
            .footer("Read 2m ago")
            .into(),
        marker("Olivia joined the call").icon(lucide::PHONE).into(),
        message()
            .avatar(avatar("OM"))
            .bubble(bubble("Amazing, ship it! 🚀").variant(Variant::Muted).reactions("❤️ 1"))
            .into(),
    ]
}

pub const MESSAGE: Page = Page {
    id: "message",
    title: "Message",
    description: "A chat message with an avatar, a header, bubbles and a footer.",
    view: |_| {
        example("Conversation", container(column(conversation()).spacing(24)).max_width(560))
    },
};

pub const MARKER: Page = Page {
    id: "marker",
    title: "Marker",
    description: "A small inline annotation in a conversation.",
    view: |_| {
        example(
            "Variants",
            column![
                marker("Olivia changed the topic").icon(lucide::PENCIL),
                marker("Yesterday").variant(MarkerVariant::Separator),
                marker("Earlier messages").variant(MarkerVariant::Border),
            ]
            .spacing(24)
            .max_width(480),
        )
    },
};

pub const ATTACHMENT: Page = Page {
    id: "attachment",
    title: "Attachment",
    description: "A file attached to a message.",
    view: |_| {
        column![
            example(
                "States",
                row![
                    attachment("report.pdf").icon(lucide::FILE_TEXT).description("2.4 MB")
                        .action(attachment::remove_action(Message::Noop)),
                    attachment("photo.jpg").icon(lucide::IMAGE).description("Uploading… 45%").state(State::Uploading),
                    attachment("data.csv").icon(lucide::FILE_SPREADSHEET).description("Upload failed").state(State::Error),
                ]
                .spacing(12)
                .wrap()
            ),
            example(
                "Sizes and orientation",
                row![
                    attachment::<Message>("notes.md").icon(lucide::FILE).size(attachment::Size::Sm),
                    attachment::<Message>("notes.md").icon(lucide::FILE).size(attachment::Size::Xs),
                    attachment::<Message>("design.fig").icon(lucide::FILE_IMAGE).description("8 MB").vertical(true),
                ]
                .spacing(12)
                .align_y(iced::Center)
            ),
        ]
        .into()
    },
};

pub const MESSAGE_SCROLLER: Page = Page {
    id: "message-scroller",
    title: "Message Scroller",
    description: "A scrolling list of messages that sticks to the newest one.",
    view: |_| {
        let mut items = conversation();
        items.extend(conversation());

        example(
            "Anchored to the bottom",
            container(message_scroller(items))
                .height(360)
                .width(Fill)
                .max_width(560)
                .style(|theme: &iced::Theme| {
                    let t = iced_ui::tokens(theme);
                    container::Style {
                        border: iced_ui::style::border(t.palette.border, t.radius_xl()),
                        ..Default::default()
                    }
                }),
        )
    },
};
