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
        let window = limits.max().width + self.inset;
        WINDOW_WIDTH.set(Some(window));

        let wide = window >= self.breakpoint.min_width;
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

thread_local! {
    /// The width of the window, as last seen by a widget of shadcn_iced.
    static WINDOW_WIDTH: Cell<Option<f32>> = const { Cell::new(None) };
}

/// The width of the window, as last seen by a widget of shadcn_iced: in a
/// window event (`Opened`, `Resized`) or by a [`Breakpoint::measure`].
/// `None` until then.
pub fn window_width() -> Option<f32> {
    WINDOW_WIDTH.get()
}

/// Records the width of the window from a window event, returning whether
/// it changed.
fn observe(event: &Event) -> bool {
    let width = match event {
        Event::Window(
            iced::window::Event::Opened { size, .. } | iced::window::Event::Resized(size),
        ) => size.width,
        _ => return false,
    };

    WINDOW_WIDTH.replace(Some(width)) != Some(width)
}

/// Whether the window is at least `min_width` wide, as far as it is known
/// (see [`window_width`]). An unknown window counts as wide.
pub fn is_window_wide(min_width: f32) -> bool {
    window_width().is_none_or(|width| width >= min_width)
}

/// A widget adapted to the width of the window, e.g. `text-base md:text-sm`
/// for an input. See [`adaptive`].
pub struct Adaptive<W> {
    widget: Option<W>,
    min_width: f32,
    adapt: fn(W, bool) -> W,
    /// Whether the widget was adapted to a wide window, once adapted.
    wide: Option<bool>,
}

/// Adapts `widget` to the width of the window before every layout, with
/// `adapt(widget, wide)`, where `wide` is whether the window is at least
/// `min_width` wide (see [`is_window_wide`]).
///
/// Unlike [`Breakpoint::measure`], it works anywhere: the window is known
/// from the window events that iced sends to every widget (`Opened`, then
/// `Resized`), and a resize lays the interface out again when it crosses
/// the breakpoint.
///
/// ```no_run
/// # use shadcn_iced_core::primitive::breakpoint::{self, adaptive};
/// # use iced::widget::text_input;
/// # fn view<'a>() -> iced::Element<'a, String> {
/// // `text-base md:text-sm`
/// adaptive(text_input("Email", ""), breakpoint::MD, |input, wide| {
///     input.size(if wide { 14 } else { 16 })
/// })
/// .into()
/// # }
/// ```
pub fn adaptive<W>(widget: W, min_width: f32, adapt: fn(W, bool) -> W) -> Adaptive<W> {
    Adaptive {
        widget: Some(widget),
        min_width,
        adapt,
        wide: None,
    }
}

impl<W> Adaptive<W> {
    fn widget(&self) -> &W {
        self.widget.as_ref().expect("adaptive widget")
    }

    fn widget_mut(&mut self) -> &mut W {
        self.widget.as_mut().expect("adaptive widget")
    }
}

impl<W, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Adaptive<W>
where
    W: Widget<Message, Theme, Renderer>,
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        self.widget().tag()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        self.widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let wide = is_window_wide(self.min_width);

        if self.wide != Some(wide) {
            let widget = self.widget.take().expect("adaptive widget");
            self.widget = Some((self.adapt)(widget, wide));
            self.wide = Some(wide);
        }

        self.widget_mut().layout(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.widget_mut().operate(tree, layout, renderer, operation);
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
        observe(event);

        if self
            .wide
            .is_some_and(|wide| wide != is_window_wide(self.min_width))
        {
            shell.invalidate_layout();
            shell.request_redraw();
        }

        self.widget_mut().update(
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
        self.widget()
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
        self.widget()
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
        self.widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

impl<'a, W, Message, Theme, Renderer> From<Adaptive<W>> for Element<'a, Message, Theme, Renderer>
where
    W: Widget<Message, Theme, Renderer> + 'a,
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(adaptive: Adaptive<W>) -> Self {
        Element::new(adaptive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_events_tell_the_width_of_the_window() {
        let resized = |width| Event::Window(iced::window::Event::Resized(Size::new(width, 600.0)));

        assert!(observe(&Event::Window(iced::window::Event::Opened {
            position: None,
            size: Size::new(500.0, 600.0),
        })));
        assert_eq!(window_width(), Some(500.0));
        assert!(!is_window_wide(MD));

        assert!(!observe(&resized(500.0)));
        assert!(observe(&resized(800.0)));
        assert!(is_window_wide(MD));
    }
}
