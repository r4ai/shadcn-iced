//! [Lucide](https://lucide.dev) icons, rendered with the bundled icon font.
//!
//! ```no_run
//! use iced_ui_core::icon::{self, lucide};
//!
//! let element: iced::Element<'_, ()> = icon::icon(lucide::CHEVRON_DOWN).size(16).into();
//! ```
pub mod lucide;

use crate::font;

use iced::widget::{Text, text};
use iced::{Center, Pixels};

/// A glyph of the icon font.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Icon(char);

impl Icon {
    /// Creates an icon from a codepoint of the icon font.
    pub const fn new(codepoint: char) -> Self {
        Self(codepoint)
    }

    /// The codepoint of the icon.
    pub const fn codepoint(self) -> char {
        self.0
    }

    /// Looks an icon up by its kebab-case Lucide name (e.g. `"chevron-down"`).
    pub fn from_name(name: &str) -> Option<Self> {
        lucide::ALL
            .binary_search_by(|(candidate, _)| candidate.cmp(&name))
            .ok()
            .map(|index| lucide::ALL[index].1)
    }

    /// Turns the icon into a [`Text`] widget of the given size.
    pub fn sized<'a>(self, size: impl Into<Pixels>) -> Text<'a> {
        let size = size.into();

        text(self.0)
            .font(font::ICONS)
            .size(size)
            .line_height(1.0)
            .width(size)
            .height(size)
            .align_x(Center)
            .align_y(Center)
    }
}

/// Creates a 16x16 icon (`size-4`, the default icon size of shadcn/ui).
pub fn icon<'a>(icon: Icon) -> Text<'a> {
    icon.sized(16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_by_name() {
        assert_eq!(Icon::from_name("chevron-down"), Some(lucide::CHEVRON_DOWN));
        assert_eq!(Icon::from_name("does-not-exist"), None);
    }
}
