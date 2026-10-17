use super::Fixture;
use crate::store::Msg;

use shadcn_iced::prelude::lucide;
use shadcn_iced::trace::{Status, trace, trace_line};

use iced::widget::{container, text};
use iced::{Element, Fill};

/// `w-full max-w-sm`
fn sm(content: Element<'_, Msg>) -> Element<'_, Msg> {
    container(content).width(Fill).max_width(384).into()
}

/// The content of a `TraceCollapsibleContent`: `leading-relaxed`, inheriting
/// the 16px text of the page.
fn content(label: &str) -> iced::widget::Text<'_> {
    text(label).size(16).line_height(1.625)
}

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "lines" => Fixture::new(|_| {
            sm(trace([
                trace_line("Ran pnpm install")
                    .icon(lucide::CIRCLE_CHECK)
                    .description("2.1s")
                    .into(),
                trace_line("Failed to run pnpm lint")
                    .icon(lucide::CIRCLE_ALERT)
                    .status(Status::Error)
                    .into(),
                trace_line("Run pnpm build")
                    .icon(lucide::TERMINAL)
                    .status(Status::Pending)
                    .into(),
            ])
            .into())
        }),
        "collapsible" => Fixture::new(|store| {
            sm(trace([
                trace_line("Thought for 5s")
                    .icon(lucide::CIRCLE_CHECK)
                    .collapsible(content("I should check what the last commit changed."))
                    .open(store.bool("thought"), crate::store::set("thought"))
                    .into(),
                trace_line("Ran pnpm --version")
                    .icon(lucide::TERMINAL)
                    .collapsible(content("Exit code 0, no output."))
                    .open(store.bool("ran"), crate::store::set("ran"))
                    .into(),
            ])
            .into())
        }),
        _ => return None,
    })
}
