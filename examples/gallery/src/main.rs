use gallery::Gallery;
use iced_ui::ApplicationExt;

fn main() -> iced::Result {
    // `GALLERY_FONT=inter` picks the body font (see the customizer).
    iced_ui::font::set_sans(gallery::customizer::body_font());

    iced::application(Gallery::default, Gallery::update, Gallery::view)
        .title("iced-ui gallery")
        .theme(Gallery::theme)
        .window_size((1200.0, 820.0))
        .iced_ui()
        .run()
}
