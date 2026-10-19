//! Text with the typographic features that iced's text does not expose.
//!
//! iced shapes text with [cosmic-text], which can space letters, turn on
//! OpenType features and slant a font that has no italic face, but
//! [`Text`](iced::widget::Text) has no setting for any of them. [`Typeset`]
//! shapes its own [`cosmic_text::Buffer`] and draws it like iced draws its
//! paragraphs:
//!
//! - [`Typeset::letter_spacing`]: `tracking-*` (CSS `letter-spacing`, after
//!   every character, the last one included),
//! - [`Typeset::feature`]: OpenType features, e.g. [`TABULAR_NUMS`]
//!   (`tabular-nums`),
//! - [`Typeset::italic`]: the italic face of the font, or its regular face
//!   slanted like browsers synthesize one (`font-synthesis-style`).
//!
//! ```no_run
//! # use shadcn_iced_core::primitive::typeset::{self, typeset};
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! typeset("Question 1 of 3")
//!     .size(12.0)
//!     .feature(typeset::TABULAR_NUMS)
//!     .into()
//! # }
//! ```
//!
//! [cosmic-text]: https://github.com/pop-os/cosmic-text
use iced::advanced::graphics::text::{self as graphics, cosmic_text};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::text::Renderer as _;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::alignment;
use iced::widget::text::{self, LineHeight, Wrapping};
use iced::{Element, Event, Font, Length, Pixels, Rectangle, Size, mouse};

use std::sync::Arc;

/// An OpenType feature: its tag and its value (`1` turns it on).
pub type Feature = ([u8; 4], u32);

/// `tabular-nums`: digits of a single width.
pub const TABULAR_NUMS: Feature = (*b"tnum", 1);

/// Text shaped with letter spacing, OpenType features or a synthesized
/// italic. See the [module documentation](self).
pub struct Typeset<'a, Theme = iced::Theme> {
    content: String,
    size: Option<f32>,
    line_height: LineHeight,
    font: Option<Font>,
    letter_spacing: f32,
    features: Vec<Feature>,
    italic: bool,
    width: Length,
    height: Length,
    align_x: text::Alignment,
    wrapping: Wrapping,
    style: Box<dyn Fn(&Theme) -> text::Style + 'a>,
}

/// Creates [`Typeset`] text.
pub fn typeset<'a, Theme>(content: impl Into<String>) -> Typeset<'a, Theme> {
    Typeset {
        content: content.into(),
        size: None,
        line_height: LineHeight::default(),
        font: None,
        letter_spacing: 0.0,
        features: Vec::new(),
        italic: false,
        width: Length::Shrink,
        height: Length::Shrink,
        align_x: text::Alignment::Default,
        wrapping: Wrapping::default(),
        style: Box::new(|_| text::Style::default()),
    }
}

impl<'a, Theme> Typeset<'a, Theme> {
    /// Sets the size of the text.
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into().0);
        self
    }

    /// Sets the line height.
    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    /// Sets the font. Defaults to the default font of the renderer.
    pub fn font(mut self, font: impl Into<Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Adds space after every character, in pixels (`tracking-tight` is
    /// `-0.025em`: `-0.025 * size`).
    pub fn letter_spacing(mut self, spacing: f32) -> Self {
        self.letter_spacing = spacing;
        self
    }

    /// Turns on an OpenType feature, e.g. [`TABULAR_NUMS`].
    pub fn feature(mut self, feature: Feature) -> Self {
        self.features.push(feature);
        self
    }

    /// Draws the text in italic: the italic face of the font, or its
    /// regular face slanted when it has none.
    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the horizontal alignment of the lines.
    pub fn align_x(mut self, alignment: impl Into<text::Alignment>) -> Self {
        self.align_x = alignment.into();
        self
    }

    /// Centers the lines.
    pub fn center(self) -> Self {
        self.align_x(alignment::Horizontal::Center)
    }

    /// Sets how the lines wrap.
    pub fn wrapping(mut self, wrapping: Wrapping) -> Self {
        self.wrapping = wrapping;
        self
    }

    /// Sets the style (the color; the inherited one without it).
    pub fn style(mut self, style: impl Fn(&Theme) -> text::Style + 'a) -> Self {
        self.style = Box::new(style);
        self
    }

    /// Sets the color.
    pub fn color(self, color: impl Into<iced::Color>) -> Self {
        let color = color.into();

        self.style(move |_| text::Style { color: Some(color) })
    }
}

/// What the shaped buffer was shaped for.
#[derive(Debug, Clone, PartialEq)]
struct Key {
    content: String,
    size: f32,
    line_height: f32,
    font: Font,
    letter_spacing: f32,
    features: Vec<Feature>,
    italic: bool,
    max_width: Option<f32>,
    align_x: text::Alignment,
    wrapping: Wrapping,
    version: graphics::Version,
}

#[derive(Default)]
struct State {
    key: Option<Key>,
    buffer: Option<Arc<cosmic_text::Buffer>>,
    size: Size,
}

/// Whether the family of `font` has an italic or oblique face.
fn has_italic(font_system: &mut cosmic_text::FontSystem, font: Font) -> bool {
    let iced::font::Family::Name(name) = font.family else {
        // Generic families resolve to fonts of the system: trust them.
        return true;
    };

    font_system.db().faces().any(|face| {
        face.style != cosmic_text::fontdb::Style::Normal
            && face.families.iter().any(|(family, _)| family == name)
    })
}

impl Key {
    fn shape(&self) -> (cosmic_text::Buffer, Size) {
        let mut font_system = graphics::font_system().write().expect("Write font system");
        let font_system = font_system.raw();

        let mut buffer = cosmic_text::Buffer::new(
            font_system,
            cosmic_text::Metrics::new(self.size, self.line_height),
        );

        buffer.set_size(font_system, self.max_width, None);
        buffer.set_wrap(font_system, graphics::to_wrap(self.wrapping));

        let mut attrs = graphics::to_attributes(self.font);

        // cosmic-text spaces letters in ems.
        if self.letter_spacing != 0.0 && self.size > 0.0 {
            attrs = attrs.letter_spacing(self.letter_spacing / self.size);
        }

        if !self.features.is_empty() {
            let mut features = cosmic_text::FontFeatures::new();

            for (tag, value) in &self.features {
                let _ = features.set(cosmic_text::FeatureTag::new(tag), *value);
            }

            attrs = attrs.font_features(features);
        }

        if self.italic {
            attrs = if has_italic(font_system, self.font) {
                attrs.style(cosmic_text::Style::Italic)
            } else {
                attrs.cache_key_flags(cosmic_text::CacheKeyFlags::FAKE_ITALIC)
            };
        }

        buffer.set_text(
            font_system,
            &self.content,
            &attrs,
            cosmic_text::Shaping::Advanced,
            None,
        );

        let size = graphics::align(&mut buffer, font_system, self.align_x);

        (buffer, size)
    }
}

impl<'a, Message, Theme> Widget<Message, Theme, iced::Renderer> for Typeset<'a, Theme> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        let limits = limits.width(self.width).height(self.height);
        let size = self.size.unwrap_or(renderer.default_size().0);
        let max_width = limits.max().width;

        let key = Key {
            content: self.content.clone(),
            size,
            line_height: self.line_height.to_absolute(Pixels(size)).0,
            font: self.font.unwrap_or_else(|| renderer.default_font()),
            letter_spacing: self.letter_spacing,
            features: self.features.clone(),
            italic: self.italic,
            max_width: (self.wrapping != Wrapping::None && max_width.is_finite())
                .then_some(max_width),
            align_x: self.align_x,
            wrapping: self.wrapping,
            version: graphics::font_system()
                .read()
                .expect("Read font system")
                .version(),
        };

        if state.key.as_ref() != Some(&key) {
            let (buffer, size) = key.shape();

            state.buffer = Some(Arc::new(buffer));
            state.size = size;
            state.key = Some(key);
        }

        let size = limits.resolve(self.width, self.height, state.size);

        // Aligned lines are aligned in the final width.
        if self.align_x != text::Alignment::Default
            && size.width != state.size.width
            && let Some(buffer) = &mut state.buffer
        {
            let mut font_system = graphics::font_system().write().expect("Write font system");

            Arc::make_mut(buffer).set_size(font_system.raw(), Some(size.width), None);
        }

        layout::Node::new(size)
    }

    fn operate(
        &mut self,
        _tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.text(None, layout.bounds(), &self.content);
    }

    fn update(
        &mut self,
        _tree: &mut Tree,
        _event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::graphics::text::Renderer as _;

        let state = tree.state.downcast_ref::<State>();

        let Some(buffer) = &state.buffer else {
            return;
        };

        let color = (self.style)(theme).color.unwrap_or(style.text_color);

        renderer.fill_raw(graphics::Raw {
            buffer: Arc::downgrade(buffer),
            position: layout.bounds().position(),
            color,
            clip_bounds: *viewport,
        });
    }
}

impl<'a, Message, Theme: 'a> From<Typeset<'a, Theme>>
    for Element<'a, Message, Theme, iced::Renderer>
{
    fn from(typeset: Typeset<'a, Theme>) -> Self {
        Element::new(typeset)
    }
}
