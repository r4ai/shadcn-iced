use dashboard::Dashboard;
use shadcn_iced::prelude::*;
use shadcn_iced::theme::{BaseColor, Config};

fn main() -> iced::Result {
    // `tab_order` moves the focus with Tab, like in a browser.
    fn view(dashboard: &Dashboard) -> iced::Element<'_, dashboard::Message> {
        tab_order(dashboard.view()).into()
    }

    iced::application(Dashboard::default, Dashboard::update, view)
        .title("Dashboard")
        .theme(|_: &Dashboard| Config::new(BaseColor::Neutral, Mode::Light).build())
        .window_size((1400.0, 960.0))
        .shadcn_iced()
        .run()
}
