//! Wheel scrolling like in browsers: smooth, and horizontal with
//! <kbd>Shift</kbd>.
//!
//! iced's scrollables jump by the whole delta of a mouse wheel notch and
//! only turn <kbd>Shift</kbd> + wheel into a horizontal scroll for line
//! deltas (not for the pixel deltas of the web). [`Wheel`] wraps any content
//! (usually the whole view of an application) and, before its scrollables
//! see them, rewrites the wheel events:
//!
//! - <kbd>Shift</kbd> swaps the axes (except on macOS, which does it),
//! - a notch of a mouse wheel (a line delta, or a large pixel delta) is
//!   spread over a few frames with an ease-out, like the smooth scrolling of
//!   browsers. Small pixel deltas (touchpads) apply right away.
//!
//! The scrolled scrollable stays the one under the pointer when the scroll
//! started, and a scroll that reaches the end of every scrollable is not
//! captured, so the scroll still reaches whatever is outside of the
//! content (e.g. the page around a web preview).
//!
//! ```no_run
//! # fn view<'a>(content: iced::Element<'a, ()>) -> iced::Element<'a, ()> {
//! shadcn_iced_core::primitive::wheel(content).into()
//! # }
//! ```
use crate::animation;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::time::Instant;
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector, keyboard, mouse, window};

/// The pixels of a line delta (iced's scrollables use the same factor).
const LINE: f32 = 60.0;

/// Pixel deltas from this size are wheel notches (browsers report about
/// 100px per notch), smaller ones come from touchpads.
const NOTCH: f32 = 50.0;

/// The time constant of the ease-out: about 95% of a scroll is done after
/// three of them.
const TAU: f32 = 0.04;

/// Smooth wheel scrolling for some content. See the
/// [module documentation](self).
pub struct Wheel<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    smooth: bool,
}

/// Wraps `content` in a [`Wheel`].
pub fn wheel<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Wheel<'a, Message, Theme, Renderer> {
    Wheel {
        content: content.into(),
        smooth: true,
    }
}

impl<Message, Theme, Renderer> Wheel<'_, Message, Theme, Renderer> {
    /// Sets whether wheel notches scroll smoothly (on by default, and off
    /// while animations are disabled).
    pub fn smooth(mut self, smooth: bool) -> Self {
        self.smooth = smooth;
        self
    }
}

#[derive(Debug, Default)]
struct State {
    modifiers: keyboard::Modifiers,
    /// The distance left to scroll, in pixels (wheel direction).
    remaining: Vector,
    /// Where the pointer was when the scroll started.
    position: Option<Point>,
    last_frame: Option<Instant>,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Wheel<'_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
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
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
            .operate(&mut tree.children[0], layout, renderer, operation);
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
        let state = tree.state.downcast_mut::<State>();

        if let Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) = event {
            state.modifiers = *modifiers;
        }

        let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event else {
            if let Event::Window(window::Event::RedrawRequested(now)) = event {
                self.content.as_widget_mut().update(
                    &mut tree.children[0],
                    event,
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );

                frame(
                    &mut self.content,
                    tree,
                    *now,
                    layout,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            } else {
                self.content.as_widget_mut().update(
                    &mut tree.children[0],
                    event,
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }

            return;
        };

        if shell.is_event_captured() {
            return;
        }

        let (pixels, notch) = match *delta {
            mouse::ScrollDelta::Lines { x, y } => (Vector::new(x, y) * LINE, true),
            mouse::ScrollDelta::Pixels { x, y } => {
                (Vector::new(x, y), x.abs().max(y.abs()) >= NOTCH)
            }
        };

        // macOS swaps the axes of `Shift` + wheel itself.
        let pixels = if state.modifiers.shift() && !cfg!(target_os = "macos") {
            Vector::new(pixels.y, pixels.x)
        } else {
            pixels
        };

        let smooth = self.smooth && notch && animation::enabled();

        // A new scroll in another direction or somewhere else stops the
        // one in progress.
        if !smooth || cursor.position() != state.position {
            let remaining = std::mem::take(&mut state.remaining);
            let position = state.position.take();

            if remaining != Vector::ZERO
                && let Some(position) = position
            {
                let _ = scroll(
                    &mut self.content,
                    &mut tree.children[0],
                    remaining,
                    position,
                    layout,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }
        }

        let state = tree.state.downcast_mut::<State>();

        if !smooth {
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                &Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Pixels {
                        x: pixels.x,
                        y: pixels.y,
                    },
                }),
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );

            return;
        }

        let Some(position) = cursor.position() else {
            return;
        };

        // A scroll in progress goes on further.
        if state.remaining != Vector::ZERO {
            state.remaining += pixels;
            shell.capture_event();
            shell.request_redraw();
            return;
        }

        // The first frame of the scroll, right away: whether something
        // captures it tells whether anything can scroll.
        let first = pixels * 0.25;
        let captured = scroll(
            &mut self.content,
            &mut tree.children[0],
            first,
            position,
            layout,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let state = tree.state.downcast_mut::<State>();

        if captured {
            state.remaining += pixels - first;
            state.position = Some(position);
            state.last_frame = Some(Instant::now());
            shell.capture_event();
            shell.request_redraw();
        }
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
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
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
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
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
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

/// Scrolls a frame of the scroll in progress, if any.
fn frame<Message, Theme, Renderer>(
    content: &mut Element<'_, Message, Theme, Renderer>,
    tree: &mut Tree,
    now: Instant,
    layout: Layout<'_>,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    viewport: &Rectangle,
) where
    Renderer: iced::advanced::Renderer,
{
    let state = tree.state.downcast_mut::<State>();

    let Some(position) = state.position else {
        return;
    };

    let elapsed = state
        .last_frame
        .map_or(0.016, |last| (now - last).as_secs_f32())
        .clamp(0.0, 0.1);

    let mut step = state.remaining * (1.0 - (-elapsed / TAU).exp());
    let left = state.remaining - step;

    if left.x.abs() < 0.5 && left.y.abs() < 0.5 {
        step = state.remaining;
    }

    state.remaining -= step;
    state.last_frame = Some(now);

    let done = state.remaining == Vector::ZERO;

    // A scroll that reaches the end stops.
    let scrolled = step == Vector::ZERO
        || scroll(
            content,
            &mut tree.children[0],
            step,
            position,
            layout,
            renderer,
            clipboard,
            shell,
            viewport,
        );

    let state = tree.state.downcast_mut::<State>();

    if done || !scrolled {
        state.remaining = Vector::ZERO;
        state.position = None;
        state.last_frame = None;
    } else {
        shell.request_redraw();
    }
}

/// Scrolls the content under `position` by `pixels` (in the direction of
/// the wheel), returning whether something scrolled.
fn scroll<Message, Theme, Renderer>(
    content: &mut Element<'_, Message, Theme, Renderer>,
    tree: &mut Tree,
    pixels: Vector,
    position: Point,
    layout: Layout<'_>,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    viewport: &Rectangle,
) -> bool
where
    Renderer: iced::advanced::Renderer,
{
    let mut messages = Vec::new();
    let mut inner = Shell::new(&mut messages);

    content.as_widget_mut().update(
        tree,
        &Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels {
                x: pixels.x,
                y: pixels.y,
            },
        }),
        layout,
        mouse::Cursor::Available(position),
        renderer,
        clipboard,
        &mut inner,
        viewport,
    );

    let captured = inner.is_event_captured();

    // The capture is of the synthetic event, not of the current one.
    let redraw = inner.redraw_request();
    let layout_invalid = inner.is_layout_invalid();
    let widgets_invalid = inner.are_widgets_invalid();
    drop(inner);

    for message in messages {
        shell.publish(message);
    }

    if layout_invalid {
        shell.invalidate_layout();
    }

    if widgets_invalid {
        shell.invalidate_widgets();
    }

    match redraw {
        window::RedrawRequest::NextFrame => shell.request_redraw(),
        window::RedrawRequest::At(at) => shell.request_redraw_at(at),
        window::RedrawRequest::Wait => {}
    }

    captured
}

impl<'a, Message, Theme, Renderer> From<Wheel<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(wheel: Wheel<'a, Message, Theme, Renderer>) -> Self {
        Element::new(wheel)
    }
}
