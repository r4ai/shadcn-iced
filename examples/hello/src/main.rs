//! The example of the README.
use iced_ui::prelude::*;

#[derive(Debug, Clone)]
enum Message {
    Save,
}

fn view(_state: &()) -> iced::Element<'_, Message> {
    iced::widget::center(
        card()
            .title("Create project")
            .description("Deploy your new project in one-click.")
            .footer(button("Deploy").on_press(Message::Save))
            .width(350),
    )
    .into()
}

fn main() -> iced::Result {
    iced::application(|| (), |_: &mut (), _: Message| {}, view)
        // Any shadcn/ui preset, e.g. zinc in dark mode:
        .theme(|_: &()| iced_ui::theme::zinc(Mode::Dark).build())
        // Loads Geist and the Lucide icons.
        .iced_ui()
        .run()
}
