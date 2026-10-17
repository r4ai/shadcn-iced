//! The example of the README.
use shadcn_iced::prelude::*;

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
        .theme(|_: &()| shadcn_iced::theme::zinc(Mode::Dark).build())
        // Loads Geist, Inter and the Lucide icons.
        .shadcn_iced()
        .run()
}
