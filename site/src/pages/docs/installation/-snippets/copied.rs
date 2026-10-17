mod ui;

fn view(state: &State) -> iced::Element<'_, Message> {
    ui::button::button("Click me").on_press(Message::Click).into()
}
