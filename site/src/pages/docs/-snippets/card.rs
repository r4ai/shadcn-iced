use shadcn_iced::prelude::*;

fn view(_state: &()) -> iced::Element<'_, Message> {
    card()
        .title("Create project")
        .description("Deploy your new project in one-click.")
        .footer(button("Deploy").on_press(Message::Save))
        .width(350)
        .into()
}
