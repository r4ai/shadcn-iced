//! The fonts used by iced_ui.
//!
//! shadcn/ui's documentation uses [Geist](https://vercel.com/font) and
//! [Geist Mono](https://vercel.com/font); both are bundled (feature `geist`)
//! together with the [Lucide](https://lucide.dev) icon font (feature
//! `lucide`).
//!
//! Load them with [`ApplicationExt::iced_ui`]:
//!
//! ```no_run
//! use iced_ui_core::ApplicationExt;
//!
//! # fn update(_: &mut (), _: ()) {}
//! # fn view(_: &()) -> iced::Element<'_, ()> { iced::widget::text("").into() }
//! iced::application(|| (), update, view).iced_ui().run()
//! # ;
//! ```
use iced::Font;
use iced::font::{Family, Weight};

use std::borrow::Cow;

/// The sans-serif font (`font-sans`), Geist.
pub const SANS: Font = Font::with_name("Geist");

/// The monospace font (`font-mono`), Geist Mono.
pub const MONO: Font = Font::with_name("Geist Mono");

/// The icon font, Lucide.
pub const ICONS: Font = Font::with_name("lucide");

/// Returns the given font with a different weight.
pub const fn weight(font: Font, weight: Weight) -> Font {
    Font { weight, ..font }
}

/// `font-medium` (500) using the current default family.
pub const MEDIUM: Font = Font {
    family: Family::Name("Geist"),
    weight: Weight::Medium,
    ..Font::DEFAULT
};

/// `font-semibold` (600) using the default family.
pub const SEMIBOLD: Font = Font {
    family: Family::Name("Geist"),
    weight: Weight::Semibold,
    ..Font::DEFAULT
};

/// `font-bold` (700) using the default family.
pub const BOLD: Font = Font {
    family: Family::Name("Geist"),
    weight: Weight::Bold,
    ..Font::DEFAULT
};

/// The bytes of every bundled font.
pub fn all() -> Vec<Cow<'static, [u8]>> {
    let mut fonts: Vec<&'static [u8]> = Vec::new();

    #[cfg(feature = "geist")]
    fonts.extend([
        include_bytes!("../assets/fonts/Geist-Regular.ttf").as_slice(),
        include_bytes!("../assets/fonts/Geist-Medium.ttf").as_slice(),
        include_bytes!("../assets/fonts/Geist-SemiBold.ttf").as_slice(),
        include_bytes!("../assets/fonts/Geist-Bold.ttf").as_slice(),
        include_bytes!("../assets/fonts/GeistMono-Regular.ttf").as_slice(),
        include_bytes!("../assets/fonts/GeistMono-Medium.ttf").as_slice(),
    ]);

    #[cfg(feature = "lucide")]
    fonts.push(include_bytes!("../assets/fonts/lucide.ttf").as_slice());

    fonts.into_iter().map(Cow::Borrowed).collect()
}

/// Extends [`iced::Application`] with a method that loads the bundled fonts
/// and makes Geist the default font.
pub trait ApplicationExt: Sized {
    /// Loads the bundled fonts and sets Geist as the default font.
    fn iced_ui(self) -> Self;
}

impl<P: iced::Program> ApplicationExt for iced::Application<P> {
    fn iced_ui(self) -> Self {
        let app = all().into_iter().fold(self, |app, font| app.font(font));

        if cfg!(feature = "geist") {
            app.default_font(SANS)
        } else {
            app
        }
    }
}
