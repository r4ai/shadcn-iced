//! The fonts used by iced_ui.
//!
//! shadcn/ui's default preset (Nova) uses [Geist](https://vercel.com/font)
//! and [Geist Mono](https://vercel.com/font) (feature `geist`).
//! [Inter](https://rsms.me/inter/) (feature `inter`) and the
//! [Lucide](https://lucide.dev) icon font (feature `lucide`) are bundled too.
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
//!
//! Like the `font` and `fontHeading` options of shadcn/ui, the families are
//! configurable: call [`set_sans`] (before [`ApplicationExt::iced_ui`], which
//! makes it the default font), [`set_heading`] and [`set_mono`] with any
//! bundled family or a font you load yourself.
use iced::Font;
use iced::font::{Family, Weight};

use std::borrow::Cow;
use std::sync::RwLock;

/// The Geist family (bundled with the `geist` feature).
pub const GEIST: &str = "Geist";

/// The Inter family (bundled with the `inter` feature).
pub const INTER: &str = "Inter";

/// The Geist Mono family (bundled with the `geist` feature).
pub const GEIST_MONO: &str = "Geist Mono";

/// The default sans-serif font (`font-sans`), Geist.
///
/// Components use [`sans`] instead, which honors [`set_sans`].
pub const SANS: Font = Font::with_name(GEIST);

/// The default monospace font (`font-mono`), Geist Mono.
///
/// Components use [`mono`] instead, which honors [`set_mono`].
pub const MONO: Font = Font::with_name(GEIST_MONO);

/// The icon font, Lucide.
pub const ICONS: Font = Font::with_name("lucide");

/// Returns the given font with a different weight.
pub const fn weight(font: Font, weight: Weight) -> Font {
    Font { weight, ..font }
}

/// `font-medium` (500) using the default family. Prefer [`medium`].
pub const MEDIUM: Font = weight(SANS, Weight::Medium);

/// `font-semibold` (600) using the default family. Prefer [`semibold`].
pub const SEMIBOLD: Font = weight(SANS, Weight::Semibold);

/// `font-bold` (700) using the default family. Prefer [`bold`].
pub const BOLD: Font = weight(SANS, Weight::Bold);

struct Families {
    sans: &'static str,
    heading: Option<&'static str>,
    mono: &'static str,
}

static FAMILIES: RwLock<Families> = RwLock::new(Families {
    sans: GEIST,
    heading: None,
    mono: GEIST_MONO,
});

fn families<T>(f: impl FnOnce(&Families) -> T) -> T {
    match FAMILIES.read() {
        Ok(families) => f(&families),
        Err(poisoned) => f(&poisoned.into_inner()),
    }
}

fn update(f: impl FnOnce(&mut Families)) {
    match FAMILIES.write() {
        Ok(mut families) => f(&mut families),
        Err(poisoned) => f(&mut poisoned.into_inner()),
    }
}

/// Sets the sans-serif family (`font`), e.g. [`INTER`].
pub fn set_sans(family: &'static str) {
    update(|families| families.sans = family);
}

/// Sets the family of titles (`fontHeading`); `None` inherits the
/// sans-serif family.
pub fn set_heading(family: Option<&'static str>) {
    update(|families| families.heading = family);
}

/// Sets the monospace family.
pub fn set_mono(family: &'static str) {
    update(|families| families.mono = family);
}

/// The sans-serif font (`font-sans`) with a regular weight.
pub fn sans() -> Font {
    Font::with_name(families(|families| families.sans))
}

/// `font-medium` (500).
pub fn medium() -> Font {
    weight(sans(), Weight::Medium)
}

/// `font-semibold` (600).
pub fn semibold() -> Font {
    weight(sans(), Weight::Semibold)
}

/// `font-bold` (700).
pub fn bold() -> Font {
    weight(sans(), Weight::Bold)
}

/// The heading font (`cn-font-heading`) with the given weight.
pub fn heading(weight: Weight) -> Font {
    Font {
        family: Family::Name(families(|families| {
            families.heading.unwrap_or(families.sans)
        })),
        weight,
        ..Font::DEFAULT
    }
}

/// The monospace font (`font-mono`).
pub fn mono() -> Font {
    Font::with_name(families(|families| families.mono))
}

/// The bytes of every bundled font.
pub fn all() -> Vec<Cow<'static, [u8]>> {
    let mut fonts: Vec<&'static [u8]> = Vec::new();

    #[cfg(feature = "inter")]
    fonts.extend([
        include_bytes!("../assets/fonts/Inter-Regular.ttf").as_slice(),
        include_bytes!("../assets/fonts/Inter-Medium.ttf").as_slice(),
        include_bytes!("../assets/fonts/Inter-SemiBold.ttf").as_slice(),
        include_bytes!("../assets/fonts/Inter-Bold.ttf").as_slice(),
    ]);

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
/// and makes the sans-serif family (Geist by default) the default font.
pub trait ApplicationExt: Sized {
    /// Loads the bundled fonts and sets [`sans`] as the default font.
    fn iced_ui(self) -> Self;
}

impl<P: iced::Program> ApplicationExt for iced::Application<P> {
    fn iced_ui(self) -> Self {
        let app = all().into_iter().fold(self, |app, font| app.font(font));

        app.default_font(sans())
    }
}

impl<P: iced::Program> ApplicationExt for iced::Daemon<P> {
    fn iced_ui(self) -> Self {
        let daemon = all()
            .into_iter()
            .fold(self, |daemon, font| daemon.font(font));

        daemon.default_font(sans())
    }
}
