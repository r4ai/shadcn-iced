use settings::{Message, Settings};
use shadcn_iced::prelude::*;
use shadcn_iced::theme::{BaseColor, Config};

use iced::{Element, Fill, Theme};

fn view(settings: &Settings) -> Element<'_, Message> {
    tab_order(scroll_area(settings.view()).height(Fill)).into()
}

/// The theme of the saved appearance.
fn theme(settings: &Settings) -> Theme {
    Config::new(BaseColor::Neutral, settings.applied_mode()).build()
}

fn main() -> iced::Result {
    iced::application(Settings::default, Settings::update, view)
        .title("Settings")
        .theme(theme)
        .window_size((1100.0, 860.0))
        .shadcn_iced()
        .run()
}
