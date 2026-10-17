use super::Fixture;
use crate::store::Msg;

use shadcn_iced::marker::{Variant, marker};
use shadcn_iced::prelude::lucide;

use iced::widget::{column, container};
use iced::{Element, Fill};

/// `w-full max-w-sm`
fn sm(content: Element<'_, Msg>) -> Element<'_, Msg> {
    container(content).width(Fill).max_width(384).into()
}

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "variants" => Fixture::new(|_| {
            sm(column![
                marker("A default marker for inline notes."),
                marker("A separator marker").variant(Variant::Separator),
                marker("A border marker for row boundaries.").variant(Variant::Border),
            ]
            .spacing(32)
            .into())
        }),
        "with-icon" => Fixture::new(|_| {
            sm(column![
                marker("Switched to a new branch").icon(lucide::GIT_BRANCH),
                marker("Thinking...").spinner(),
                marker("Explored 4 files")
                    .icon(lucide::SEARCH)
                    .variant(Variant::Separator),
            ]
            .spacing(32)
            .into())
        }),
        _ => return None,
    })
}
