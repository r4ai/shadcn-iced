//! Accessible resizable panel groups and layouts with keyboard support.
//!
//! Port of <https://ui.shadcn.com/docs/components/resizable>. Nest
//! [`resizable`] calls for more than two panels.
//!
//! ```no_run
//! # use iced_ui::resizable::resizable;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! resizable(iced::widget::text("One"), iced::widget::text("Two")).grip(true).into()
//! # }
//! ```
use iced_ui_core::primitive::Split;
use iced_ui_core::primitive::split::Style;
use iced_ui_core::{Element, tokens};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::{Background, Event, Length, Rectangle, Size, Theme, Vector, border, mouse};

pub use iced_ui_core::primitive::split::Axis;

/// Two resizable panels. See the [module documentation](self).
pub struct Resizable<'a, Message> {
    split: Split<'a, Message>,
    axis: Axis,
    grip: bool,
}

/// Creates two resizable panels separated by a draggable handle
/// (`ResizableHandle`: a `w-px bg-border` line).
pub fn resizable<'a, Message: 'a>(
    first: impl Into<Element<'a, Message>>,
    second: impl Into<Element<'a, Message>>,
) -> Resizable<'a, Message> {
    Resizable {
        split: Split::new(first, second, style),
        axis: Axis::Horizontal,
        grip: false,
    }
}

impl<'a, Message: 'a> Resizable<'a, Message> {
    /// Sets the axis.
    pub fn axis(mut self, axis: Axis) -> Self {
        self.split = self.split.axis(axis);
        self.axis = axis;
        self
    }

    /// Controls the size of the first panel, as a ratio of the total size.
    pub fn ratio(mut self, ratio: f32) -> Self {
        self.split = self.split.ratio(ratio);
        self
    }

    /// Sets the initial ratio of self-managed panels.
    pub fn default_ratio(mut self, ratio: f32) -> Self {
        self.split = self.split.default_ratio(ratio);
        self
    }

    /// Sets the bounds of the ratio.
    pub fn limits(mut self, min: f32, max: f32) -> Self {
        self.split = self.split.limits(min, max);
        self
    }

    /// Shows a grip on the handle (`withHandle`): a `h-6 w-1 rounded-lg
    /// bg-border` pill.
    pub fn grip(mut self, grip: bool) -> Self {
        self.grip = grip;
        self
    }

    /// Sets the message produced while dragging the handle.
    pub fn on_resize(mut self, f: impl Fn(f32) -> Message + 'a) -> Self {
        self.split = self.split.on_resize(f);
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.split = self.split.width(width);
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.split = self.split.height(height);
        self
    }
}

/// The style of the resize handle: `bg-border`, unchanged on hover.
pub fn style(theme: &Theme) -> Style {
    let p = tokens(theme).palette;

    Style {
        divider: p.border,
        divider_active: p.border,
        grip: p.border,
        grip_border: p.border,
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Resizable<'_, Message> {
    fn tag(&self) -> tree::Tag {
        Widget::<Message, Theme, iced::Renderer>::tag(&self.split)
    }

    fn state(&self) -> tree::State {
        Widget::<Message, Theme, iced::Renderer>::state(&self.split)
    }

    fn children(&self) -> Vec<Tree> {
        Widget::<Message, Theme, iced::Renderer>::children(&self.split)
    }

    fn diff(&self, tree: &mut Tree) {
        Widget::<Message, Theme, iced::Renderer>::diff(&self.split, tree);
    }

    fn size(&self) -> Size<Length> {
        Widget::<Message, Theme, iced::Renderer>::size(&self.split)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        Widget::<Message, Theme, iced::Renderer>::layout(&mut self.split, tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        Widget::<Message, Theme, iced::Renderer>::operate(
            &mut self.split,
            tree,
            layout,
            renderer,
            operation,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        Widget::<Message, Theme, iced::Renderer>::update(
            &mut self.split,
            tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;

        Widget::<Message, Theme, iced::Renderer>::draw(
            &self.split,
            tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );

        if !self.grip {
            return;
        }

        // The handle sits at the end of the first panel.
        let Some(first) = layout.children().next() else {
            return;
        };

        let first = first.bounds();
        let bounds = layout.bounds();

        // `h-6 w-1 rounded-lg`, rotated for a vertical group.
        let grip = match self.axis {
            Axis::Horizontal => Rectangle {
                x: first.x + first.width + 0.5 - 2.0,
                y: bounds.center_y() - 12.0,
                width: 4.0,
                height: 24.0,
            },
            Axis::Vertical => Rectangle {
                x: bounds.center_x() - 12.0,
                y: first.y + first.height + 0.5 - 2.0,
                width: 24.0,
                height: 4.0,
            },
        };

        renderer.with_layer(*viewport, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: grip,
                    border: border::rounded(tokens(theme).radius_lg()),
                    ..renderer::Quad::default()
                },
                Background::Color(tokens(theme).palette.border),
            );
        });
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        Widget::<Message, Theme, iced::Renderer>::mouse_interaction(
            &self.split,
            tree,
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        Widget::<Message, Theme, iced::Renderer>::overlay(
            &mut self.split,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: 'a> From<Resizable<'a, Message>> for Element<'a, Message> {
    fn from(resizable: Resizable<'a, Message>) -> Self {
        Element::new(resizable)
    }
}
