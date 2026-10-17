use super::Fixture;
use crate::store::{Msg, Store};

use shadcn_iced::bubble::{Align, Variant, bubble};
use shadcn_iced::message::message;
use shadcn_iced::message_scroller::{self, message_scroller};

use iced::widget::container;
use iced::{Element, Fill};

const MESSAGES: [(bool, &str); 7] = [
    (true, "How do I keep a chat pinned to the newest message?"),
    (
        false,
        "Wrap the message list in a message scroller. The viewport stays at the bottom as new text arrives, so readers always see the latest reply land in place.",
    ),
    (
        true,
        "And if the reader scrolled up to re-read an older answer?",
    ),
    (
        false,
        "Then it does not yank them back down. A scroll button appears at the bottom of the viewport, and one press returns to the newest message.",
    ),
    (true, "Does it work while streaming?"),
    (
        false,
        "Yes. It only follows the stream while the reader is already at the bottom of the conversation.",
    ),
    (true, "That works, thanks!"),
];

/// `h-80 w-full max-w-sm`
fn demo(store: &Store) -> Element<'_, Msg> {
    let items = MESSAGES.iter().map(|(user, content)| {
        if *user {
            message().align(Align::End).bubble(bubble(*content)).into()
        } else {
            message()
                .bubble(bubble(*content).variant(Variant::Ghost))
                .into()
        }
    });

    let scroller = message_scroller(items)
        .on_scroll(|viewport| Msg::Set("at_end", message_scroller::is_at_end(&viewport).into()))
        // The button is shown while the reader is away from the newest message.
        .button((!store.bool("at_end")).then_some(Msg::Noop));

    container(scroller)
        .width(Fill)
        .max_width(384)
        .height(320)
        .into()
}

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "bottom" | "scroll-to-start" => Fixture::new(demo).init(|store| {
            let _ = store.set("at_end", true);
        }),
        _ => return None,
    })
}
