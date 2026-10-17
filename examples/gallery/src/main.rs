use gallery::Gallery;
use shadcn_iced::ApplicationExt;

fn main() -> iced::Result {
    // `GALLERY_FONT=inter` picks the body font (see the customizer).
    shadcn_iced::font::set_sans(gallery::customizer::body_font());

    iced::application(Gallery::default, Gallery::update, Gallery::view)
        .title("shadcn-iced gallery")
        .theme(Gallery::theme)
        .window_size((1200.0, 820.0))
        .shadcn_iced()
        .run()
}
