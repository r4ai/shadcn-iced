//! Design tokens ported from shadcn/ui and their bridge to [`iced::Theme`].
//!
//! shadcn/ui describes its look with a set of CSS variables (`--background`,
//! `--primary`, `--ring`, `--radius`, ...). This module mirrors them as a
//! [`Palette`] plus a radius, bundled together as [`Tokens`].
//!
//! Build an [`iced::Theme`] with [`Config`] and use it as your application
//! theme. Components then read the tokens back with [`tokens`]:
//!
//! ```no_run
//! use iced_ui_core::theme::{self, Accent, Mode};
//!
//! let theme: iced::Theme = theme::zinc(Mode::Dark).accent(Accent::Blue).build();
//! let tokens = theme::tokens(&theme);
//! assert!(tokens.is_dark());
//! ```
//!
//! Plain iced themes (e.g. [`iced::Theme::Dracula`]) are supported too: their
//! tokens are derived from the extended palette.
mod presets;

pub use presets::{ACCENT_COLORS, BASE_COLORS};

use iced::theme::palette::{Extended, Pair};
use iced::{Color, Theme};

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, LazyLock, RwLock};

/// Creates an opaque [`Color`] from 8-bit channels.
pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

/// Creates a [`Color`] from 8-bit channels and an alpha in `0.0..=1.0`.
pub const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::from_rgba8(r, g, b, a)
}

/// The color tokens of shadcn/ui, one field per CSS variable.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub background: Color,
    pub foreground: Color,
    pub card: Color,
    pub card_foreground: Color,
    pub popover: Color,
    pub popover_foreground: Color,
    pub primary: Color,
    pub primary_foreground: Color,
    pub secondary: Color,
    pub secondary_foreground: Color,
    pub muted: Color,
    pub muted_foreground: Color,
    pub accent: Color,
    pub accent_foreground: Color,
    pub destructive: Color,
    pub border: Color,
    pub input: Color,
    pub ring: Color,
    pub sidebar: Color,
    pub sidebar_foreground: Color,
    pub sidebar_primary: Color,
    pub sidebar_primary_foreground: Color,
    pub sidebar_accent: Color,
    pub sidebar_accent_foreground: Color,
    pub sidebar_border: Color,
    pub sidebar_ring: Color,
    pub chart: [Color; 5],
}

/// A partial [`Palette`], used by the accent color presets.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PaletteOverrides {
    pub background: Option<Color>,
    pub foreground: Option<Color>,
    pub card: Option<Color>,
    pub card_foreground: Option<Color>,
    pub popover: Option<Color>,
    pub popover_foreground: Option<Color>,
    pub primary: Option<Color>,
    pub primary_foreground: Option<Color>,
    pub secondary: Option<Color>,
    pub secondary_foreground: Option<Color>,
    pub muted: Option<Color>,
    pub muted_foreground: Option<Color>,
    pub accent: Option<Color>,
    pub accent_foreground: Option<Color>,
    pub destructive: Option<Color>,
    pub border: Option<Color>,
    pub input: Option<Color>,
    pub ring: Option<Color>,
    pub sidebar: Option<Color>,
    pub sidebar_foreground: Option<Color>,
    pub sidebar_primary: Option<Color>,
    pub sidebar_primary_foreground: Option<Color>,
    pub sidebar_accent: Option<Color>,
    pub sidebar_accent_foreground: Option<Color>,
    pub sidebar_border: Option<Color>,
    pub sidebar_ring: Option<Color>,
    pub chart: Option<[Color; 5]>,
}

impl Palette {
    /// Applies a function to every color of the palette.
    pub fn map(mut self, f: impl Fn(Color) -> Color) -> Self {
        macro_rules! map {
            ($($field:ident),*) => {
                $(self.$field = f(self.$field);)*
            };
        }

        map!(
            background,
            foreground,
            card,
            card_foreground,
            popover,
            popover_foreground,
            primary,
            primary_foreground,
            secondary,
            secondary_foreground,
            muted,
            muted_foreground,
            accent,
            accent_foreground,
            destructive,
            border,
            input,
            ring,
            sidebar,
            sidebar_foreground,
            sidebar_primary,
            sidebar_primary_foreground,
            sidebar_accent,
            sidebar_accent_foreground,
            sidebar_border,
            sidebar_ring
        );

        self.chart = self.chart.map(f);
        self
    }

    /// Applies the given overrides on top of this palette.
    pub fn apply(mut self, o: &PaletteOverrides) -> Self {
        macro_rules! apply {
            ($($field:ident),*) => {
                $(if let Some(value) = o.$field { self.$field = value; })*
            };
        }

        apply!(
            background,
            foreground,
            card,
            card_foreground,
            popover,
            popover_foreground,
            primary,
            primary_foreground,
            secondary,
            secondary_foreground,
            muted,
            muted_foreground,
            accent,
            accent_foreground,
            destructive,
            border,
            input,
            ring,
            sidebar,
            sidebar_foreground,
            sidebar_primary,
            sidebar_primary_foreground,
            sidebar_accent,
            sidebar_accent_foreground,
            sidebar_border,
            sidebar_ring,
            chart
        );

        self
    }
}

/// Whether a theme is light or dark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    /// A light theme.
    #[default]
    Light,
    /// A dark theme.
    Dark,
}

impl Mode {
    /// Returns the opposite mode.
    pub fn toggle(self) -> Self {
        match self {
            Mode::Light => Mode::Dark,
            Mode::Dark => Mode::Light,
        }
    }
}

/// The full set of design tokens a component needs to style itself.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tokens {
    /// The color tokens.
    pub palette: Palette,
    /// The base radius (`--radius`), in logical pixels. shadcn/ui's default
    /// is `0.625rem`, i.e. `10.0`.
    pub radius: f32,
    /// Whether the tokens describe a light or a dark theme.
    pub mode: Mode,
}

impl Tokens {
    /// The default radius of shadcn/ui (`0.625rem`).
    pub const DEFAULT_RADIUS: f32 = 10.0;

    /// Returns `true` if the tokens describe a dark theme.
    pub fn is_dark(&self) -> bool {
        self.mode == Mode::Dark
    }

    /// `--radius-sm`: `radius - 4px`.
    pub fn radius_sm(&self) -> f32 {
        (self.radius - 4.0).max(0.0)
    }

    /// `--radius-md`: `radius - 2px`.
    pub fn radius_md(&self) -> f32 {
        (self.radius - 2.0).max(0.0)
    }

    /// `--radius-lg`: `radius`.
    pub fn radius_lg(&self) -> f32 {
        self.radius
    }

    /// `--radius-xl`: `radius + 4px`.
    pub fn radius_xl(&self) -> f32 {
        self.radius + 4.0
    }
}

/// The base colors of shadcn/ui.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BaseColor {
    #[default]
    Neutral,
    Stone,
    Zinc,
    Mauve,
    Olive,
    Mist,
    Taupe,
}

impl BaseColor {
    /// Every base color.
    pub const ALL: &'static [Self] = &[
        Self::Neutral,
        Self::Stone,
        Self::Zinc,
        Self::Mauve,
        Self::Olive,
        Self::Mist,
        Self::Taupe,
    ];

    /// The lowercase name of the base color, as used by shadcn/ui.
    pub fn name(self) -> &'static str {
        BASE_COLORS[self as usize].0
    }

    /// The palette of the base color in the given mode.
    pub fn palette(self, mode: Mode) -> Palette {
        let (_, light, dark) = BASE_COLORS[self as usize];

        match mode {
            Mode::Light => light,
            Mode::Dark => dark,
        }
    }
}

impl std::fmt::Display for BaseColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name();
        let mut chars = name.chars();
        if let Some(first) = chars.next() {
            write!(f, "{}{}", first.to_ascii_uppercase(), chars.as_str())?;
        }
        Ok(())
    }
}

/// The accent colors of shadcn/ui, applied on top of a [`BaseColor`].
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Accent {
    Amber,
    Blue,
    Cyan,
    Emerald,
    Fuchsia,
    Green,
    Indigo,
    Lime,
    Orange,
    Pink,
    Purple,
    Red,
    Rose,
    Sky,
    Teal,
    Violet,
    Yellow,
}

impl Accent {
    /// Every accent color.
    pub const ALL: &'static [Self] = &[
        Self::Amber,
        Self::Blue,
        Self::Cyan,
        Self::Emerald,
        Self::Fuchsia,
        Self::Green,
        Self::Indigo,
        Self::Lime,
        Self::Orange,
        Self::Pink,
        Self::Purple,
        Self::Red,
        Self::Rose,
        Self::Sky,
        Self::Teal,
        Self::Violet,
        Self::Yellow,
    ];

    /// The lowercase name of the accent color, as used by shadcn/ui.
    pub fn name(self) -> &'static str {
        ACCENT_COLORS[self as usize].0
    }

    /// The overrides of the accent color in the given mode.
    pub fn overrides(self, mode: Mode) -> PaletteOverrides {
        let (_, light, dark) = ACCENT_COLORS[self as usize];

        match mode {
            Mode::Light => light,
            Mode::Dark => dark,
        }
    }
}

impl std::fmt::Display for Accent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name();
        let mut chars = name.chars();
        if let Some(first) = chars.next() {
            write!(f, "{}{}", first.to_ascii_uppercase(), chars.as_str())?;
        }
        Ok(())
    }
}

/// A builder for an [`iced::Theme`] carrying shadcn/ui [`Tokens`].
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    name: Option<String>,
    base: BaseColor,
    accent: Option<Accent>,
    tokens: Tokens,
}

/// The default light theme (`neutral`).
pub fn light() -> Config {
    Config::new(BaseColor::Neutral, Mode::Light)
}

/// The default dark theme (`neutral`).
pub fn dark() -> Config {
    Config::new(BaseColor::Neutral, Mode::Dark)
}

macro_rules! base_fns {
    ($($fn:ident => $base:ident),*) => {
        $(
            #[doc = concat!("A theme using the `", stringify!($fn), "` base color.")]
            pub fn $fn(mode: Mode) -> Config {
                Config::new(BaseColor::$base, mode)
            }
        )*
    };
}

base_fns!(
    neutral => Neutral,
    stone => Stone,
    zinc => Zinc,
    mauve => Mauve,
    olive => Olive,
    mist => Mist,
    taupe => Taupe
);

impl Config {
    /// Creates a theme from a base color and a mode.
    pub fn new(base: BaseColor, mode: Mode) -> Self {
        Self {
            name: None,
            base,
            accent: None,
            tokens: Tokens {
                palette: base.palette(mode),
                radius: Tokens::DEFAULT_RADIUS,
                mode,
            },
        }
    }

    /// Creates a theme from arbitrary tokens.
    pub fn from_tokens(name: impl Into<String>, tokens: Tokens) -> Self {
        Self {
            name: Some(name.into()),
            base: BaseColor::Neutral,
            accent: None,
            tokens,
        }
    }

    /// Applies an accent color.
    pub fn accent(mut self, accent: Accent) -> Self {
        self.accent = Some(accent);
        self.tokens.palette = self
            .base
            .palette(self.tokens.mode)
            .apply(&accent.overrides(self.tokens.mode));
        self
    }

    /// Sets the base radius (`--radius`) in logical pixels.
    pub fn radius(mut self, radius: f32) -> Self {
        self.tokens.radius = radius;
        self
    }

    /// Sets the display name of the theme.
    ///
    /// Themes are identified by name: two different configurations must not
    /// share a name. By default, a unique name is derived from the tokens.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Customizes the palette.
    pub fn palette(mut self, f: impl FnOnce(&mut Palette)) -> Self {
        f(&mut self.tokens.palette);
        self
    }

    /// Returns the tokens of the configuration.
    pub fn tokens(&self) -> Tokens {
        self.tokens
    }

    /// Builds the [`iced::Theme`].
    ///
    /// The tokens are registered globally under the theme name so that
    /// [`tokens`] can retrieve them later. The extended palette of the theme
    /// is derived from the tokens too, so built-in iced widgets blend in.
    pub fn build(self) -> Theme {
        let tokens = self.tokens;
        let name = self.name.clone().unwrap_or_else(|| self.default_name());

        let _ = REGISTRY
            .write()
            .map(|mut registry| registry.insert(name.clone(), tokens));

        let p = tokens.palette;
        let base = iced::theme::Palette {
            background: p.background,
            text: p.foreground,
            primary: p.primary,
            success: rgb(0x16, 0xa3, 0x4a),
            warning: rgb(0xf5, 0x9e, 0x0b),
            danger: p.destructive,
        };

        Theme::custom_with_fn(name, base, move |base| extended(base, &tokens))
    }

    fn default_name(&self) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        format!("{:?}", self.tokens).hash(&mut hasher);

        let mode = match self.tokens.mode {
            Mode::Light => "Light",
            Mode::Dark => "Dark",
        };

        match self.accent {
            Some(accent) => format!(
                "{} {accent} {mode} #{:08x}",
                self.base,
                hasher.finish() as u32
            ),
            None => {
                format!("{} {mode} #{:08x}", self.base, hasher.finish() as u32)
            }
        }
    }
}

impl From<Config> for Theme {
    fn from(config: Config) -> Self {
        config.build()
    }
}

static REGISTRY: LazyLock<RwLock<HashMap<String, Tokens>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

thread_local! {
    static LAST: std::cell::RefCell<Option<(Arc<iced::theme::Custom>, Tokens)>> =
        const { std::cell::RefCell::new(None) };
}

thread_local! {
    static FADED: std::cell::RefCell<HashMap<(String, u32), Theme>> =
        std::cell::RefCell::new(HashMap::new());
}

/// The number of opacity levels faded themes are quantized to.
const FADE_STEPS: f32 = 24.0;

/// Returns a copy of the theme whose colors are multiplied by `opacity`, or
/// `None` if the opacity is (almost) 1. Used to fade widgets in and out.
///
/// Faded themes are cached per opacity level.
pub fn faded(theme: &Theme, opacity: f32) -> Option<Theme> {
    let step = (opacity.clamp(0.0, 1.0) * FADE_STEPS).round() as u32;

    if step as f32 >= FADE_STEPS {
        return None;
    }

    let name = iced::theme::Base::name(theme).to_owned();
    let key = (name.clone(), step);

    if let Some(theme) = FADED.with_borrow(|cache| cache.get(&key).cloned()) {
        return Some(theme);
    }

    let mut tokens = tokens(theme);
    let opacity = step as f32 / FADE_STEPS;
    tokens.palette = tokens.palette.map(|color| alpha(color, opacity));

    let faded = Config::from_tokens(format!("{name} ~{step}"), tokens).build();

    FADED.with_borrow_mut(|cache| {
        // Themes are rebuilt when customized; keep the cache bounded.
        if cache.len() > 512 {
            cache.clear();
        }

        cache.insert(key, faded.clone());
    });

    Some(faded)
}

/// Returns the [`Tokens`] of the given theme.
///
/// Themes created with [`Config::build`] return their exact tokens. Any other
/// theme gets tokens derived from its extended palette.
pub fn tokens(theme: &Theme) -> Tokens {
    if let Theme::Custom(custom) = theme {
        let cached = LAST.with_borrow(|last| {
            last.as_ref()
                .and_then(|(theme, tokens)| Arc::ptr_eq(theme, custom).then_some(*tokens))
        });

        if let Some(tokens) = cached {
            return tokens;
        }

        let registered = REGISTRY
            .read()
            .ok()
            .and_then(|registry| registry.get(&custom.to_string()).copied());

        if let Some(tokens) = registered {
            LAST.with_borrow_mut(|last| {
                *last = Some((custom.clone(), tokens));
            });

            return tokens;
        }
    }

    derive(theme.extended_palette())
}

fn extended(base: iced::theme::Palette, tokens: &Tokens) -> Extended {
    let p = &tokens.palette;
    let mut extended = Extended::generate(base);

    let pair = |color, text| Pair { color, text };

    extended.background.base = pair(p.background, p.foreground);
    extended.background.weakest = pair(p.card, p.card_foreground);
    extended.background.weaker = pair(p.muted, p.foreground);
    extended.background.weak = pair(p.accent, p.accent_foreground);
    extended.background.neutral = pair(p.border, p.foreground);
    extended.background.strong = pair(p.input, p.foreground);
    extended.background.stronger = pair(p.ring, p.foreground);
    extended.background.strongest = pair(p.muted_foreground, p.background);

    extended.primary.base = pair(p.primary, p.primary_foreground);
    extended.primary.weak = pair(mix(p.primary, p.background, 0.4), p.primary_foreground);
    extended.primary.strong = pair(mix(p.primary, p.background, 0.1), p.primary_foreground);

    extended.secondary.base = pair(p.secondary, p.secondary_foreground);
    extended.secondary.weak = pair(p.muted, p.muted_foreground);
    extended.secondary.strong = pair(p.accent, p.accent_foreground);

    extended.danger.base = pair(p.destructive, Color::WHITE);
    extended.is_dark = tokens.is_dark();

    extended
}

fn derive(extended: &Extended) -> Tokens {
    let background = extended.background.base.color;
    let foreground = extended.background.base.text;
    let muted_foreground = mix(foreground, background, 0.45);
    let primary = extended.primary.base;
    let secondary = extended.background.weak;

    Tokens {
        palette: Palette {
            background,
            foreground,
            card: background,
            card_foreground: foreground,
            popover: background,
            popover_foreground: foreground,
            primary: primary.color,
            primary_foreground: primary.text,
            secondary: secondary.color,
            secondary_foreground: secondary.text,
            muted: secondary.color,
            muted_foreground,
            accent: secondary.color,
            accent_foreground: secondary.text,
            destructive: extended.danger.base.color,
            border: extended.background.strong.color,
            input: extended.background.strong.color,
            ring: mix(primary.color, background, 0.5),
            sidebar: extended.background.weakest.color,
            sidebar_foreground: foreground,
            sidebar_primary: primary.color,
            sidebar_primary_foreground: primary.text,
            sidebar_accent: secondary.color,
            sidebar_accent_foreground: secondary.text,
            sidebar_border: extended.background.strong.color,
            sidebar_ring: mix(primary.color, background, 0.5),
            chart: [
                primary.color,
                extended.success.base.color,
                extended.warning.base.color,
                extended.danger.base.color,
                extended.secondary.base.color,
            ],
        },
        radius: Tokens::DEFAULT_RADIUS,
        mode: if extended.is_dark {
            Mode::Dark
        } else {
            Mode::Light
        },
    }
}

/// Linearly mixes two colors in sRGB space. `factor` is the weight of `b`.
pub fn mix(a: Color, b: Color, factor: f32) -> Color {
    let inv = 1.0 - factor;

    Color {
        r: a.r * inv + b.r * factor,
        g: a.g * inv + b.g * factor,
        b: a.b * inv + b.b * factor,
        a: a.a * inv + b.a * factor,
    }
}

/// Returns the color with its alpha multiplied by `alpha`, like Tailwind's
/// `bg-primary/90` syntax.
pub fn alpha(color: Color, alpha: f32) -> Color {
    Color {
        a: color.a * alpha,
        ..color
    }
}

/// Composites `top` over `bottom`, returning an opaque result if `bottom` is
/// opaque.
pub fn over(top: Color, bottom: Color) -> Color {
    let a = top.a + bottom.a * (1.0 - top.a);

    if a <= 0.0 {
        return Color::TRANSPARENT;
    }

    let blend = |t: f32, b: f32| (t * top.a + b * bottom.a * (1.0 - top.a)) / a;

    Color {
        r: blend(top.r, bottom.r),
        g: blend(top.g, bottom.g),
        b: blend(top.b, bottom.b),
        a,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_tokens_round_trip() {
        let config = zinc(Mode::Dark).accent(Accent::Blue).radius(6.0);
        let expected = config.tokens();
        let theme = config.build();

        assert_eq!(tokens(&theme), expected);
        assert_eq!(tokens(&theme), expected);
    }

    #[test]
    fn builtin_themes_derive_tokens() {
        let tokens = tokens(&Theme::Dark);

        assert!(tokens.is_dark());
        assert_eq!(
            tokens.palette.background,
            Theme::Dark.extended_palette().background.base.color
        );
    }

    #[test]
    fn names_are_unique_per_tokens() {
        let a = light().build();
        let b = light().radius(4.0).build();

        assert_ne!(a.to_string(), b.to_string());
        assert_eq!(tokens(&b).radius, 4.0);
    }
}
