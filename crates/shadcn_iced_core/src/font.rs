//! The fonts used by shadcn_iced.
//!
//! shadcn/ui's default preset (Nova) uses [Geist](https://vercel.com/font)
//! and [Geist Mono](https://vercel.com/font) (feature `geist`).
//! The [Lucide](https://lucide.dev) icon font (feature `lucide`) is bundled
//! too, and [Inter](https://rsms.me/inter/) with the feature `inter` (off by
//! default).
//! The keyboard symbols and arrows that Geist lacks (⌘ ⌥ ⌃ ⏎ ⌫ ...), used by
//! shortcuts, are bundled too (feature `symbols`), so that they do not depend
//! on the fonts of the system: see [`SYMBOLS`].
//!
//! Load them with [`ApplicationExt::shadcn_iced`]:
//!
//! ```no_run
//! use shadcn_iced_core::ApplicationExt;
//!
//! # fn update(_: &mut (), _: ()) {}
//! # fn view(_: &()) -> iced::Element<'_, ()> { iced::widget::text("").into() }
//! iced::application(|| (), update, view).shadcn_iced().run()
//! # ;
//! ```
//!
//! Like the `font` and `fontHeading` options of shadcn/ui, the families are
//! configurable: call [`set_sans`] (before [`ApplicationExt::shadcn_iced`], which
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

/// The Shadcn Iced Symbols family (bundled with the `symbols` feature): the
/// Arrows, Miscellaneous Technical, Control Pictures and Supplemental Arrows
/// blocks (⌘ ⌥ ⌃ ⇧ ⏎ ⌫ ⎋ ⇥ ␣ ...), merged from
/// [Noto Sans Symbols 2](https://fonts.google.com/noto/specimen/Noto+Sans+Symbols+2),
/// [Noto Sans Symbols](https://fonts.google.com/noto/specimen/Noto+Sans+Symbols)
/// and [Noto Sans Math](https://fonts.google.com/noto/specimen/Noto+Sans+Math)
/// by `scripts/gen_fonts.py`.
///
/// Text can fall back to it for the glyphs that its font lacks, even where no
/// font of the system has them (cosmic-text tries the fallback fonts of the
/// platform first).
pub const SYMBOLS: &str = "Shadcn Iced Symbols";

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

/// `font-extrabold` (800).
pub fn extrabold() -> Font {
    weight(sans(), Weight::ExtraBold)
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
        include_bytes!("../assets/fonts/Geist-ExtraBold.ttf").as_slice(),
        include_bytes!("../assets/fonts/GeistMono-Regular.ttf").as_slice(),
        include_bytes!("../assets/fonts/GeistMono-Medium.ttf").as_slice(),
        include_bytes!("../assets/fonts/GeistMono-SemiBold.ttf").as_slice(),
    ]);

    #[cfg(feature = "symbols")]
    fonts.push(include_bytes!("../assets/fonts/ShadcnIcedSymbols-Regular.ttf").as_slice());

    #[cfg(feature = "lucide")]
    fonts.push(include_bytes!("../assets/fonts/lucide.ttf").as_slice());

    fonts.into_iter().map(Cow::Borrowed).collect()
}

/// Extends [`iced::Application`] with a method that loads the bundled fonts
/// and makes the sans-serif family (Geist by default) the default font.
pub trait ApplicationExt: Sized {
    /// Loads the bundled fonts and sets [`sans`] as the default font.
    fn shadcn_iced(self) -> Self;
}

impl<P: iced::Program> ApplicationExt for iced::Application<P> {
    fn shadcn_iced(self) -> Self {
        let app = all().into_iter().fold(self, |app, font| app.font(font));

        app.default_font(sans())
    }
}

impl<P: iced::Program> ApplicationExt for iced::Daemon<P> {
    fn shadcn_iced(self) -> Self {
        let daemon = all()
            .into_iter()
            .fold(self, |daemon, font| daemon.font(font));

        daemon.default_font(sans())
    }
}
