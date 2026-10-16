use gallery::Gallery;
use iced_ui::ApplicationExt;

fn main() -> iced::Result {
    iced::application(Gallery::default, Gallery::update, Gallery::view)
        .title("iced-ui gallery")
        .theme(Gallery::theme)
        .window_size((1200.0, 820.0))
        .iced_ui()
        .run()
}
