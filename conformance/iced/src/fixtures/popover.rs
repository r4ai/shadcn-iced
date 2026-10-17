use super::Fixture;

use shadcn_iced::button::{Size, Variant, button};
use shadcn_iced::popover::{header, popover, popover_with_width};
use shadcn_iced::primitive::Align;

use iced::widget::{row, text};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|_| {
            popover(
                button("Open Popover").variant(Variant::Outline),
                header("Dimensions", "Set the dimensions for the layer."),
            )
            .align(Align::Start)
            .into()
        }),
        "align" => Fixture::new(|_| {
            let aligned = |label: &'static str, content: &'static str, align: Align| {
                popover_with_width(
                    button(label).variant(Variant::Outline).size(Size::Sm),
                    // `text-sm`
                    text(content).size(14).line_height(iced::Pixels(20.0)),
                    160.0,
                )
                .align(align)
            };

            row![
                aligned("Start", "Aligned to start", Align::Start),
                aligned("Center", "Aligned to center", Align::Center),
                aligned("End", "Aligned to end", Align::End),
            ]
            .spacing(8)
            .into()
        }),
        _ => return None,
    })
}
