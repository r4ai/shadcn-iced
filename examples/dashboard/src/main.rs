use dashboard::Dashboard;
use shadcn_iced::prelude::*;
use shadcn_iced::theme::{BaseColor, Config};

fn main() -> iced::Result {
    iced::application(Dashboard::default, Dashboard::update, Dashboard::view)
        .title("Dashboard")
        .theme(|_: &Dashboard| Config::new(BaseColor::Neutral, Mode::Light).build())
        .window_size((1400.0, 960.0))
        .shadcn_iced()
        .run()
}
