use super::Fixture;
use crate::store::{Msg, Store};

use shadcn_iced::navigation_menu::{icon_link, link, navigation_menu};
use shadcn_iced::prelude::lucide;

use iced::Element;
use iced::widget::{Column, Row, column};

const COMPONENTS: [(&str, &str); 6] = [
    (
        "Alert Dialog",
        "A modal dialog that interrupts the user with important content and expects a response.",
    ),
    (
        "Hover Card",
        "For sighted users to preview content available behind a link.",
    ),
    (
        "Progress",
        "Displays an indicator showing the completion progress of a task, typically displayed as a progress bar.",
    ),
    ("Scroll-area", "Visually or semantically separates content."),
    (
        "Tabs",
        "A set of layered sections of content—known as tab panels—that are displayed one at a time.",
    ),
    (
        "Tooltip",
        "A popup that displays information related to an element when the element receives keyboard focus or the mouse hovers over it.",
    ),
];

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "closed" | "getting-started" | "components" | "switch" => Fixture::new(example),
        _ => return None,
    })
}

fn example(_: &Store) -> Element<'_, Msg> {
    // `grid w-[500px] gap-2 md:grid-cols-2`
    let grid = Column::with_children(COMPONENTS.chunks(2).map(|pair| {
        Row::with_children(
            pair.iter()
                .map(|(title, description)| link(*title, *description, Msg::Noop)),
        )
        .spacing(8)
        .into()
    }))
    .spacing(8)
    .width(500);

    navigation_menu()
        .menu(
            "Getting started",
            // `w-96`
            column![
                link(
                    "Introduction",
                    "Re-usable components built with Tailwind CSS.",
                    Msg::Noop
                ),
                link(
                    "Installation",
                    "How to install dependencies and structure your app.",
                    Msg::Noop
                ),
                link(
                    "Typography",
                    "Styles for headings, paragraphs, lists...etc",
                    Msg::Noop
                ),
            ]
            .width(384),
        )
        .menu("Components", grid)
        .menu(
            "With Icon",
            // `w-[200px]`
            column![
                icon_link(lucide::CIRCLE_ALERT, "Backlog", Msg::Noop),
                icon_link(lucide::CIRCLE_DASHED, "To Do", Msg::Noop),
                icon_link(lucide::CIRCLE_CHECK, "Done", Msg::Noop),
            ]
            .width(200),
        )
        .link("Docs", Msg::Noop)
        .into()
}
