//! Responsive variants (`sm:`, `md:`...) for the content of overlays.
//!
//! Tailwind's responsive variants depend on the width of the window, which
//! iced widgets do not know. The content of an overlay, though, is laid out
//! with the size of the window minus a known padding. [`Breakpoint::measure`]
//! wraps such content and records, on every layout, whether the window is at
//! least as wide as the breakpoint; the [`Breakpoint::either`] widgets nested
//! in it then show their wide or their narrow variant.
//!
//! ```no_run
//! # use shadcn_iced_core::primitive::Breakpoint;
//! # use iced::widget::{column, row, text};
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! let sm = Breakpoint::sm();
//!
//! // `flex flex-col-reverse sm:flex-row`
//! let buttons = sm.either(row![text("Cancel"), text("OK")], column![text("OK"), text("Cancel")]);
//!
//! // The content of an overlay placed with a padding of 16.
//! sm.measure(buttons, 32.0).max_width(384.0, f32::INFINITY).into()
//! # }
//! ```
use std::cell::Cell;
use std::rc::Rc;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::{Element, Event, Length, Rectangle, Size, Vector, mouse};

/// The `sm` breakpoint of Tailwind CSS (40rem).
pub const SM: f32 = 640.0;
/// The `md` breakpoint of Tailwind CSS (48rem).
pub const MD: f32 = 768.0;

/// Whether the window is at least as wide as a breakpoint, shared by the
/// widgets that depend on it. See the [module documentation](self).
#[derive(Debug, Clone)]
pub struct Breakpoint {
    min_width: f32,
    wide: Rc<Cell<bool>>,
}

impl Breakpoint {
    /// A breakpoint at the given window width. It is considered reached
    /// until a [`Breakpoint::measure`] measures the window.
    pub fn new(min_width: f32) -> Self {
        Self {
            min_width,
            wide: Rc::new(Cell::new(true)),
        }
    }

    /// The `sm` breakpoint (640px).
    pub fn sm() -> Self {
        Self::new(SM)
    }

    /// Whether the window was at least as wide as the breakpoint during the
    /// last layout.
    pub fn is_wide(&self) -> bool {
        self.wide.get()
    }

    /// Wraps the root of the content of an overlay, laid out with the width
    /// of the window minus `inset` (the horizontal padding of the overlay),
    /// to measure the window.
    pub fn measure<'a, Message, Theme, Renderer>(
        &self,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        inset: f32,
    ) -> Measure<'a, Message, Theme, Renderer> {
        Measure {
            breakpoint: self.clone(),
            content: content.into(),
            inset,
            max_width: (f32::INFINITY, f32::INFINITY),
        }
    }

    /// Shows `wide` when the window is at least as wide as the breakpoint,
    /// and `narrow` otherwise. It must be nested in a
    /// [`Breakpoint::measure`] of the same breakpoint.
    pub fn either<'a, Message, Theme, Renderer>(
        &self,
        wide: impl Into<Element<'a, Message, Theme, Renderer>>,
        narrow: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Either<'a, Message, Theme, Renderer> {
        Either {
            breakpoint: self.clone(),
            wide: wide.into(),
            narrow: narrow.into(),
        }
    }
}

/// Measures the window for a [`Breakpoint`]. See [`Breakpoint::measure`].
pub struct Measure<'a, Message, Theme, Renderer> {
    breakpoint: Breakpoint,
    content: Element<'a, Message, Theme, Renderer>,
    inset: f32,
    max_width: (f32, f32),
}

impl<Message, Theme, Renderer> Measure<'_, Message, Theme, Renderer> {
    /// Limits the width of the content, from the breakpoint on (`wide`) and
    /// below it (`narrow`), e.g. `max-w-[calc(100%-2rem)] sm:max-w-sm`.
    pub fn max_width(mut self, wide: f32, narrow: f32) -> Self {
        self.max_width = (wide, narrow);
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Measure<'_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let wide = limits.max().width + self.inset >= self.breakpoint.min_width;
        self.breakpoint.wide.set(wide);

        let max_width = if wide {
            self.max_width.0
        } else {
            self.max_width.1
        };

        self.content
            .as_widget_mut()
            .layout(tree, renderer, &limits.max_width(max_width))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

impl<'a, Message, Theme, Renderer> From<Measure<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(measure: Measure<'a, Message, Theme, Renderer>) -> Self {
        Element::new(measure)
    }
}

/// One of two variants of some content, depending on a [`Breakpoint`]. See
/// [`Breakpoint::either`].
pub struct Either<'a, Message, Theme, Renderer> {
    breakpoint: Breakpoint,
    wide: Element<'a, Message, Theme, Renderer>,
    narrow: Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> Either<'a, Message, Theme, Renderer> {
    fn index(&self) -> usize {
        if self.breakpoint.is_wide() { 0 } else { 1 }
    }

    fn active(&self) -> &Element<'a, Message, Theme, Renderer> {
        if self.breakpoint.is_wide() {
            &self.wide
        } else {
            &self.narrow
        }
    }

    fn active_mut(&mut self) -> &mut Element<'a, Message, Theme, Renderer> {
        if self.breakpoint.is_wide() {
            &mut self.wide
        } else {
            &mut self.narrow
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Either<'_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.wide), Tree::new(&self.narrow)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.wide, &self.narrow]);
    }

    fn size(&self) -> Size<Length> {
        self.active().as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.active().as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let index = self.index();
        let child =
            self.active_mut()
                .as_widget_mut()
                .layout(&mut tree.children[index], renderer, limits);

        layout::Node::with_children(child.size(), vec![child])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let index = self.index();

        self.active_mut().as_widget_mut().operate(
            &mut tree.children[index],
            layout.children().next().unwrap(),
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
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let index = self.index();

        self.active_mut().as_widget_mut().update(
            &mut tree.children[index],
            event,
            layout.children().next().unwrap(),
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
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.active().as_widget().draw(
            &tree.children[self.index()],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.active().as_widget().mouse_interaction(
            &tree.children[self.index()],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let index = self.index();

        self.active_mut().as_widget_mut().overlay(
            &mut tree.children[index],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Either<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(either: Either<'a, Message, Theme, Renderer>) -> Self {
        Element::new(either)
    }
}
