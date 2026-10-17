//! A width that animates when it changes.
//!
//! [`AnimatedWidth`] gives its content a fixed width and, when that width
//! changes, eases from the previous one (`transition-[width]`), like a
//! sidebar collapsing to icons. The content is laid out with the larger of
//! the current and the target widths and clipped, so that it does not wrap
//! while it grows.
use crate::animation::{self, Transition};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::time::{Duration, Instant};
use iced::{Element, Event, Length, Rectangle, Size, Vector, mouse, window};

/// The duration of the transition (`duration-200`).
const DURATION: Duration = Duration::from_millis(200);

/// Content with an animated width. See the [module documentation](self).
pub struct AnimatedWidth<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    width: f32,
    height: Length,
}

/// Gives `content` the given width, animating its changes.
pub fn animated_width<'a, Message, Theme, Renderer>(
    width: f32,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> AnimatedWidth<'a, Message, Theme, Renderer> {
    AnimatedWidth {
        content: content.into(),
        width,
        height: Length::Shrink,
    }
}

impl<Message, Theme, Renderer> AnimatedWidth<'_, Message, Theme, Renderer> {
    /// Sets the height.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

#[derive(Debug, Default)]
struct State {
    width: Option<Transition>,
}

impl State {
    fn width(&mut self, target: f32, now: Instant) -> f32 {
        let width = self.width.get_or_insert_with(|| Transition::new(target));

        if (width.target() - target).abs() > f32::EPSILON {
            width.go(target, DURATION, animation::ease_in_out, now);
        }

        width.value(now)
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for AnimatedWidth<'_, Message, Theme, Renderer>
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
        Size::new(Length::Shrink, self.height)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let width = tree
            .state
            .downcast_mut::<State>()
            .width(self.width, Instant::now());
        let content_width = width.max(self.width);

        let limits = limits.height(self.height);
        let content = self.content.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &limits.width(content_width),
        );

        let size = limits.resolve(
            Length::Fixed(width),
            self.height,
            Size::new(width, content.size().height),
        );

        layout::Node::with_children(size, vec![content])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
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
        if let Event::Window(window::Event::RedrawRequested(now)) = event
            && let Some(width) = &tree.state.downcast_ref::<State>().width
            && width.is_animating(*now)
        {
            shell.invalidate_layout();
            shell.request_redraw();
        }

        let bounds = layout.bounds();

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            if cursor.is_over(bounds) {
                cursor
            } else {
                cursor.levitate()
            },
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
        let bounds = layout.bounds();

        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };

        renderer.with_layer(clip, |renderer| {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                &clip,
            );
        });
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if !cursor.is_over(layout.bounds()) {
            return mouse::Interaction::None;
        }

        self.content.as_widget().mouse_interaction(
            &tree.children[0],
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
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<AnimatedWidth<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(widget: AnimatedWidth<'a, Message, Theme, Renderer>) -> Self {
        Element::new(widget)
    }
}
