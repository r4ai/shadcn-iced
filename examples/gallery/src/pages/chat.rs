//! Pages of the chat components, mirroring the examples of the docs.
use super::{DemoMessage, Page};
use crate::{Message, example};

use iced_ui::attachment::{self, State};
use iced_ui::bubble::{Align, Side, Variant, bubble_group};
use iced_ui::button::{Size, Variant as ButtonVariant};
use iced_ui::marker::Variant as MarkerVariant;
use iced_ui::message_scroller;
use iced_ui::prelude::*;

use iced::widget::{column, container, row, text};
use iced::{Element, Fill};

/// `w-full max-w-sm`
fn sm<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content).width(Fill).max_width(384).into()
}

pub const BUBBLE: Page = Page {
    id: "bubble",
    title: "Bubble",
    description: "A chat bubble with variants, alignment, grouping and reactions.",
    view: |_| {
        column![
            example(
                "Demo",
                sm(column![
                    bubble("Hey there! what's up?").align(Align::End),
                    bubble_group([
                        bubble("Hey! Want to see chat bubbles?")
                            .variant(Variant::Muted)
                            .into(),
                        bubble(
                            "I can group messages, switch sides, and keep the whole thread easy to scan."
                        )
                        .variant(Variant::Muted)
                        .reactions("👍")
                        .into(),
                    ]),
                    bubble("Sure. Hit me with your best demo.").align(Align::End),
                    bubble(
                        "Yes. You are reading a demo that is demoing itself. Very meta. Very on-brand."
                    )
                    .variant(Variant::Muted)
                    .reactions("👍 🔥 👀 +2"),
                ]
                .spacing(32))
            ),
            example(
                "Variants",
                sm(column![
                    bubble("This is the default primary bubble."),
                    bubble("This is the secondary variant.")
                        .variant(Variant::Secondary)
                        .align(Align::End),
                    bubble(
                        "This one is muted. It uses a lower emphasis color for the chat bubble."
                    )
                    .variant(Variant::Muted)
                    .reactions("👍"),
                    bubble(
                        "This one is tinted. The tint is a softer color derived from the primary color."
                    )
                    .variant(Variant::Tinted)
                    .align(Align::End),
                    bubble("We can also use an outlined variant.").variant(Variant::Outline),
                    bubble("Or a destructive variant with a reaction.")
                        .variant(Variant::Destructive)
                        .align(Align::End)
                        .reactions("🔥"),
                    bubble(
                        "Ghost bubbles work for assistant text, markdown, and other content that should not be framed.\n\nThis is perfect for assistant messages that should not have a frame and can take the full width of the container."
                    )
                    .variant(Variant::Ghost),
                ]
                .spacing(48))
            ),
            example(
                "Alignment",
                sm(column![
                    bubble("This bubble is aligned to the start. This is the default alignment.")
                        .variant(Variant::Muted),
                    bubble("This bubble is aligned to the end. Use this for user messages.")
                        .align(Align::End),
                ]
                .spacing(32))
            ),
            example(
                "Bubble Group",
                sm(column![
                    bubble("Can you tell me what's the issue?").variant(Variant::Muted),
                    bubble_group([
                        bubble("You tell me!").align(Align::End).into(),
                        bubble("It worked yesterday. You broke it!")
                            .align(Align::End)
                            .into(),
                        bubble("Find the bug and fix it.")
                            .align(Align::End)
                            .reactions("👀")
                            .reactions_align(Align::Start)
                            .into(),
                    ]),
                    bubble("Want me to diff yesterday's you against today's you? It's a bit embarrassing.")
                        .variant(Variant::Muted),
                ]
                .spacing(32))
            ),
            example(
                "Links and Buttons",
                sm(column![
                    bubble("How can I help you today?").variant(Variant::Muted),
                    bubble_group([
                        bubble("I forgot my password")
                            .variant(Variant::Tinted)
                            .align(Align::End)
                            .on_press(Message::Noop)
                            .into(),
                        bubble("I need help with my subscription")
                            .variant(Variant::Tinted)
                            .align(Align::End)
                            .on_press(Message::Noop)
                            .into(),
                        bubble("Something else. Talk to a human.")
                            .variant(Variant::Tinted)
                            .align(Align::End)
                            .on_press(Message::Noop)
                            .into(),
                    ]),
                ]
                .spacing(32))
            ),
            example(
                "Reactions",
                sm(column![
                    bubble("I don't need tests, I know my code works.")
                        .variant(Variant::Muted)
                        .align(Align::End)
                        .reactions("👍 😮")
                        .reactions_align(Align::Start),
                    bubble(
                        "Bold. Fine I'll add some tests. I'll let you know when they're done."
                    )
                    .variant(Variant::Muted)
                    .reactions("👀 🚀 +2"),
                    bubble("Tests passed on the first try. All 142 of them. Looking good!")
                        .align(Align::End)
                        .reactions("🎉 👏")
                        .reactions_side(Side::Top)
                        .reactions_align(Align::Start),
                    bubble("Are you sure I can run this command?")
                        .variant(Variant::Destructive)
                        .reactions_content(
                            button("Yes, run it")
                                .variant(ButtonVariant::Ghost)
                                .size(Size::Xs)
                                .on_press(Message::Noop)
                        ),
                ]
                .spacing(48))
            ),
        ]
        .into()
    },
};

pub const MESSAGE: Page = Page {
    id: "message",
    title: "Message",
    description: "A chat message with an avatar, a header, bubbles and a footer.",
    view: |_| {
        column![
            example(
                "Demo",
                sm(column![
                    message()
                        .align(Align::End)
                        .avatar(avatar("ME"))
                        .bubble(bubble("Deploying to prod real quick.")),
                    message()
                        .avatar(avatar("R"))
                        .bubble(bubble("It's 4:55 PM. On a Friday.").variant(Variant::Muted)),
                    message()
                        .align(Align::End)
                        .avatar(avatar("ME"))
                        .bubble(bubble("It's a one-line change."))
                        .footer("Delivered"),
                    message().avatar(avatar("R")).push(bubble_group([
                        bubble("It's always a one-line change 😭.")
                            .variant(Variant::Muted)
                            .into(),
                        bubble("Alright, let me take a look.")
                            .variant(Variant::Muted)
                            .reactions("👍")
                            .into(),
                    ])),
                    marker("Oliver is typing..."),
                ]
                .spacing(24))
            ),
            example(
                "Header and Footer",
                sm(column![
                    message()
                        .header("Olivia")
                        .bubble(bubble("I already checked the logs.").variant(Variant::Muted)),
                    message()
                        .align(Align::End)
                        .bubble(bubble(
                            "Send the report to the team. Ping @shadcn if you need help."
                        ))
                        .footer("Read Yesterday"),
                ]
                .spacing(32))
            ),
            example(
                "Actions",
                sm(column![
                    message()
                        .bubble(
                            bubble("The install failure is coming from the workspace package.")
                                .variant(Variant::Muted)
                        )
                        .footer_content(row![
                            ghost_icon(lucide::COPY, Size::Icon),
                            ghost_icon(lucide::THUMBS_UP, Size::Icon),
                            ghost_icon(lucide::THUMBS_DOWN, Size::Icon),
                        ]),
                    message()
                        .align(Align::End)
                        .bubble(bubble("Okay drop me a link. Taking a look..."))
                        .footer_content(
                            row![
                                text("Failed to send")
                                    .size(12)
                                    .style(|theme: &iced::Theme| {
                                        text::Style {
                                            color: Some(iced_ui::tokens(theme).palette.destructive),
                                        }
                                    }),
                                ghost_icon(lucide::REFRESH_CCW, Size::IconXs),
                            ]
                            .spacing(8)
                            .align_y(iced::Center)
                        ),
                ]
                .spacing(32))
            ),
            example(
                "Attachment",
                sm(message()
                    .align(Align::End)
                    .push(
                        attachment("quarterly-report.pdf")
                            .icon(lucide::FILE_TEXT)
                            .description("PDF · 2.4 MB")
                    )
                    .bubble(bubble("Here is the report you asked for."))),
            ),
        ]
        .into()
    },
};

fn ghost_icon<'a>(icon: iced_ui::Icon, size: Size) -> Element<'a, Message> {
    icon_button(icon)
        .variant(ButtonVariant::Ghost)
        .size(size)
        .on_press(Message::Noop)
        .into()
}

pub const MARKER: Page = Page {
    id: "marker",
    title: "Marker",
    description: "A small inline annotation in a conversation.",
    view: |_| {
        column![
            example(
                "Demo",
                sm(column![
                    marker("Switched to a new branch").icon(lucide::GIT_BRANCH),
                    marker("Thinking...").spinner(),
                    marker("Conversation compacted").variant(MarkerVariant::Separator),
                    marker("Explored 4 files").icon(lucide::SEARCH),
                ]
                .spacing(32))
            ),
            example(
                "Variants",
                sm(column![
                    marker("A default marker for inline notes."),
                    marker("A separator marker").variant(MarkerVariant::Separator),
                    marker("A border marker for row boundaries.").variant(MarkerVariant::Border),
                ]
                .spacing(32))
            ),
            example(
                "Status",
                sm(column![
                    marker("Compacting conversation").spinner(),
                    marker("Running tests")
                        .spinner()
                        .variant(MarkerVariant::Separator),
                ]
                .spacing(32))
            ),
            example(
                "With Icon",
                sm(column![
                    marker("Switched to a new branch").icon(lucide::GIT_BRANCH),
                    marker("Explored 4 files")
                        .icon(lucide::SEARCH)
                        .variant(MarkerVariant::Separator),
                    marker("Syncing completed")
                        .icon(lucide::BOOK_OPEN_CHECK)
                        .vertical(true),
                ]
                .spacing(48))
            ),
        ]
        .into()
    },
};

pub const ATTACHMENT: Page = Page {
    id: "attachment",
    title: "Attachment",
    description: "A file attached to a message.",
    view: |_| {
        let tile = |name: &'static str, meta: &'static str| -> Element<'static, Message> {
            attachment(name)
                .icon(lucide::IMAGE)
                .description(meta)
                .vertical(true)
                .into()
        };

        column![
            example(
                "Demo",
                sm(column![
                    attachment::attachment_group([
                        tile("workspace.png", "PNG · 820 KB"),
                        tile("desk-reference.jpg", "JPG · 1.1 MB"),
                        tile("office-reference.jpg", "JPG · 940 KB"),
                    ]),
                    attachment("sales-dashboard.pdf")
                        .spinner()
                        .description("Uploading · 64%")
                        .state(State::Uploading)
                        .width(Fill)
                        .action(attachment::remove_action(Message::Noop)),
                    attachment("message-renderer.tsx")
                        .icon(lucide::FILE_CODE)
                        .description("TypeScript · 12 KB")
                        .width(Fill)
                        .action(attachment::remove_action(Message::Noop)),
                ]
                .spacing(12))
            ),
            example(
                "States",
                sm(column![
                    attachment("selected-file.pdf")
                        .icon(lucide::CLOCK)
                        .description("Ready to upload")
                        .state(State::Idle)
                        .width(Fill)
                        .action(attachment::remove_action(Message::Noop)),
                    attachment("design-system.zip")
                        .spinner()
                        .description("Uploading · 64%")
                        .state(State::Uploading)
                        .width(Fill)
                        .action(attachment::remove_action(Message::Noop)),
                    attachment("market-research.pdf")
                        .icon(lucide::FILE_TEXT)
                        .description("Processing document")
                        .state(State::Processing)
                        .width(Fill)
                        .action(attachment::remove_action(Message::Noop)),
                    attachment("financial-model.xlsx")
                        .icon(lucide::FILE_WARNING)
                        .description("Upload failed. Try again.")
                        .state(State::Error)
                        .width(Fill)
                        .action(attachment::action(lucide::REFRESH_CW, Message::Noop))
                        .action(attachment::remove_action(Message::Noop)),
                    attachment("uploaded-report.pdf")
                        .icon(lucide::CHECK)
                        .description("Uploaded · 1.8 MB")
                        .width(Fill)
                        .action(attachment::remove_action(Message::Noop)),
                ]
                .spacing(8))
            ),
            example(
                "Sizes",
                sm(column![
                    attachment::<Message>("Default attachment")
                        .icon(lucide::FILE_TEXT)
                        .description("PDF · 2.4 MB")
                        .width(Fill),
                    attachment::<Message>("Small attachment")
                        .icon(lucide::FILE_TEXT)
                        .description("PDF · 2.4 MB")
                        .size(attachment::Size::Sm)
                        .width(Fill),
                    attachment::<Message>("Extra small attachment")
                        .icon(lucide::FILE_TEXT)
                        .size(attachment::Size::Xs)
                        .width(Fill),
                ]
                .spacing(12))
            ),
            example(
                "Group",
                sm(attachment::attachment_group(
                    [
                        ("briefing-notes.pdf", "PDF · 1.4 MB", lucide::FILE_TEXT),
                        ("workspace.png", "PNG · 820 KB", lucide::IMAGE),
                        ("customers.csv", "CSV · 18 KB", lucide::TABLE),
                        ("renderer.tsx", "TSX · 12 KB", lucide::FILE_CODE),
                    ]
                    .into_iter()
                    .map(|(name, meta, icon)| {
                        attachment(name)
                            .icon(icon)
                            .description(meta)
                            .width(256)
                            .action(attachment::remove_action(Message::Noop))
                            .into()
                    })
                ))
            ),
            example(
                "Trigger",
                sm(attachment("project-brief.pdf")
                    .icon(lucide::FILE_TEXT)
                    .description("Click to preview")
                    .width(Fill)
                    .on_press(Message::Noop)),
            ),
        ]
        .into()
    },
};

const CHAT: [(bool, &str); 6] = [
    (
        true,
        "I'm building a chat for our app and the scroll behavior is driving me nuts. Every time the AI streams a reply, the whole thread jumps around.",
    ),
    (
        false,
        "That's the classic streaming scroll problem. Wrap your message list in a message scroller: the viewport pins to the bottom as tokens arrive, so users always see the latest text land in place.\n\nThe important part: it only auto-scrolls while the reader is already at the bottom.",
    ),
    (
        true,
        "Okay, but when someone sends a new message the view still feels jarring, like the whole conversation reloads from the top.",
    ),
    (
        false,
        "Anchoring fixes that. The reply starts in view without that disorienting jump you get from a plain overflow container.",
    ),
    (
        true,
        "And if they've scrolled up to re-read an older answer? I don't want to yank them back down.",
    ),
    (
        false,
        "You won't. When there is content they haven't seen yet, the scroll button appears at the bottom of the viewport. One tap jumps them back to the newest message.",
    ),
];

pub const SCROLLER_ID: &str = "message-scroller-demo";

pub const MESSAGE_SCROLLER: Page = Page {
    id: "message-scroller",
    title: "Message Scroller",
    description: "A scrolling list of messages that sticks to the newest one.",
    view: |gallery| {
        let items = CHAT.iter().map(|(user, content)| {
            if *user {
                message().align(Align::End).bubble(bubble(*content)).into()
            } else {
                message()
                    .bubble(bubble(*content).variant(Variant::Ghost))
                    .into()
            }
        });

        let at_end = gallery.demo.scroller_at_end;

        let scroller = message_scroller(items)
            .id(SCROLLER_ID)
            .on_scroll(|viewport| {
                Message::Demo(DemoMessage::ScrollerAtEnd(message_scroller::is_at_end(
                    &viewport,
                )))
            })
            .button((!at_end).then_some(Message::Demo(DemoMessage::ScrollToEnd)));

        let header = column![
            text("New Chat").size(16).font(iced_ui::font::medium()),
            text("How can I help you today?")
                .size(14)
                .style(crate::muted),
        ]
        .spacing(4)
        .padding(16);

        example(
            "Demo",
            sm(container(column![
                header,
                iced_ui::separator::separator(),
                container(scroller).height(Fill),
            ])
            .height(480)
            .style(|theme: &iced::Theme| {
                let t = iced_ui::tokens(theme);
                container::Style {
                    background: Some(t.palette.card.into()),
                    text_color: Some(t.palette.card_foreground),
                    border: iced_ui::style::ring_border(&t, t.radius_xl()),
                    ..Default::default()
                }
            })),
        )
    },
};
