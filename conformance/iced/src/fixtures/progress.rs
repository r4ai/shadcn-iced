use super::Fixture;

use shadcn_iced::progress::{progress, progress_with_label};

use iced::widget::{column, container, row, space};
use iced::{FillPortion, Length};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        // `w-[60%]`
        "demo" => Fixture::new(|_| {
            row![
                container(progress(66.0)).width(FillPortion(3)),
                space().width(FillPortion(2)),
            ]
            .width(Length::Fill)
            .into()
        }),
        "label" => Fixture::new(|_| {
            container(progress_with_label("Upload progress", 56.0))
                .max_width(384)
                .into()
        }),
        "values" => Fixture::new(|_| {
            column![progress(0.0), progress(25.0), progress(100.0)]
                .spacing(16)
                .max_width(384)
                .into()
        }),
        _ => return None,
    })
}
