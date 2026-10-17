use super::Fixture;
use crate::store::Msg;

use shadcn_iced::attachment::{self, Size, State, attachment, attachment_group};
use shadcn_iced::prelude::lucide;

use iced::widget::{column, container, text};
use iced::{Element, Fill};

/// `w-full max-w-sm`
fn sm(content: Element<'_, Msg>) -> Element<'_, Msg> {
    container(content).width(Fill).max_width(384).into()
}

fn tile<'a>(name: &'static str, meta: &'static str) -> Element<'a, Msg> {
    attachment(name)
        .icon(lucide::IMAGE)
        .description(meta)
        .vertical(true)
        .into()
}

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "states" => Fixture::new(|_| {
            sm(column![
                attachment("selected-file.pdf")
                    .icon(lucide::CLOCK)
                    .description("Ready to upload")
                    .state(State::Idle)
                    .width(Fill)
                    .action(attachment::remove_action(Msg::Noop)),
                attachment("design-system.zip")
                    .spinner()
                    .description("Uploading · 64%")
                    .state(State::Uploading)
                    .width(Fill)
                    .action(attachment::remove_action(Msg::Noop)),
                attachment("market-research.pdf")
                    .icon(lucide::FILE_TEXT)
                    .description("Processing document")
                    .state(State::Processing)
                    .width(Fill)
                    .action(attachment::remove_action(Msg::Noop)),
                attachment("financial-model.xlsx")
                    .icon(lucide::FILE_WARNING)
                    .description("Upload failed. Try again.")
                    .state(State::Error)
                    .width(Fill)
                    .action(attachment::action(lucide::REFRESH_CW, Msg::Noop))
                    .action(attachment::remove_action(Msg::Noop)),
                attachment("uploaded-report.pdf")
                    .icon(lucide::CHECK)
                    .description("Uploaded · 1.8 MB")
                    .width(Fill)
                    .action(attachment::remove_action(Msg::Noop)),
            ]
            .spacing(8)
            .into())
        }),
        "sizes" => Fixture::new(|_| {
            sm(column![
                attachment::<Msg>("Default attachment")
                    .icon(lucide::FILE_TEXT)
                    .description("PDF · 2.4 MB")
                    .width(Fill),
                attachment::<Msg>("Small attachment")
                    .icon(lucide::FILE_TEXT)
                    .description("PDF · 2.4 MB")
                    .size(Size::Sm)
                    .width(Fill),
                attachment::<Msg>("Extra small attachment")
                    .icon(lucide::FILE_TEXT)
                    .size(Size::Xs)
                    .width(Fill),
            ]
            .spacing(12)
            .into())
        }),
        "group" => Fixture::new(|_| {
            sm(attachment_group([
                tile("workspace.png", "PNG · 820 KB"),
                tile("desk-reference.jpg", "JPG · 1.1 MB"),
                tile("office-reference.jpg", "JPG · 940 KB"),
            ]))
        }),
        "trigger" => Fixture::new(|store| {
            sm(column![
                attachment("project-brief.pdf")
                    .icon(lucide::FILE_TEXT)
                    .description("Click to preview")
                    .width(Fill)
                    .on_press(Msg::Set("opened", true.into())),
                text(if store.bool("opened") {
                    "Opened"
                } else {
                    "Not opened"
                })
                // `text-sm`
                .size(14)
                .line_height(iced::Pixels(20.0)),
            ]
            .spacing(12)
            .into())
        }),
        _ => return None,
    })
}
