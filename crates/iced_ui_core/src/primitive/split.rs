//! Two panes separated by a draggable divider.
//!
//! [`Split`] is the headless building block behind resizable panel groups.
//! It remembers the position of the divider by itself, unless you control it
//! with [`Split::ratio`] and [`Split::on_resize`].
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::{
    Background, Border, Color, Element, Event, Length, Point, Rectangle, Size, Vector, border,
    mouse, touch,
};

/// The direction in which the panes are laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Axis {
    /// Side by side, with a vertical divider.
    #[default]
    Horizontal,
    /// Stacked, with a horizontal divider.
    Vertical,
}

/// The look of the divider of a [`Split`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The color of the divider line.
    pub divider: Color,
    /// The color of the line while hovered or dragged.
    pub divider_active: Color,
    /// The background of the grip, if displayed.
    pub grip: Color,
    /// The border of the grip.
    pub grip_border: Color,
}

/// Two panes separated by a draggable divider.
pub struct Split<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    first: Element<'a, Message, Theme, Renderer>,
    second: Element<'a, Message, Theme, Renderer>,
    axis: Axis,
    ratio: Option<f32>,
    default_ratio: f32,
    min_ratio: f32,
    max_ratio: f32,
    grip: bool,
    width: Length,
    height: Length,
    on_resize: Option<Box<dyn Fn(f32) -> Message + 'a>>,
    style: Box<dyn Fn(&Theme) -> Style + 'a>,
}

impl<'a, Message, Theme, Renderer> Split<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    /// Creates a [`Split`] with two panes.
    pub fn new(
        first: impl Into<Element<'a, Message, Theme, Renderer>>,
        second: impl Into<Element<'a, Message, Theme, Renderer>>,
        style: impl Fn(&Theme) -> Style + 'a,
    ) -> Self {
        Self {
            first: first.into(),
            second: second.into(),
            axis: Axis::Horizontal,
            ratio: None,
            default_ratio: 0.5,
            min_ratio: 0.1,
            max_ratio: 0.9,
            grip: false,
            width: Length::Fill,
            height: Length::Fill,
            on_resize: None,
            style: Box::new(style),
        }
    }

    /// Sets the axis.
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    /// Controls the size of the first pane, as a ratio of the total size.
    pub fn ratio(mut self, ratio: f32) -> Self {
        self.ratio = Some(ratio);
        self
    }

    /// Sets the initial ratio of a self-managed [`Split`].
    pub fn default_ratio(mut self, ratio: f32) -> Self {
        self.default_ratio = ratio;
        self
    }

    /// Sets the bounds of the ratio.
    pub fn limits(mut self, min: f32, max: f32) -> Self {
        self.min_ratio = min;
        self.max_ratio = max;
        self
    }

    /// Shows a grip on the divider (`withHandle`).
    pub fn grip(mut self, grip: bool) -> Self {
        self.grip = grip;
        self
    }

    /// Sets the message produced while dragging the divider.
    pub fn on_resize(mut self, f: impl Fn(f32) -> Message + 'a) -> Self {
        self.on_resize = Some(Box::new(f));
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
}

#[derive(Debug)]
struct State {
    ratio: f32,
    dragging: bool,
}

const HIT: f32 = 8.0;

impl<Message, Theme, Renderer> Split<'_, Message, Theme, Renderer> {
    fn current(&self, state: &State) -> f32 {
        self.ratio
            .unwrap_or(state.ratio)
            .clamp(self.min_ratio, self.max_ratio)
    }

    fn divider(&self, bounds: Rectangle, ratio: f32) -> Rectangle {
        match self.axis {
            Axis::Horizontal => Rectangle {
                x: bounds.x + bounds.width * ratio - HIT / 2.0,
                y: bounds.y,
                width: HIT,
                height: bounds.height,
            },
            Axis::Vertical => Rectangle {
                x: bounds.x,
                y: bounds.y + bounds.height * ratio - HIT / 2.0,
                width: bounds.width,
                height: HIT,
            },
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Split<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            ratio: self.default_ratio,
            dragging: false,
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.first), Tree::new(&self.second)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.first.as_widget(), self.second.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let ratio = self.current(tree.state.downcast_ref::<State>());
        let size = limits.resolve(self.width, self.height, Size::ZERO);

        let (first, second, offset) = match self.axis {
            Axis::Horizontal => {
                let first = (size.width * ratio).round();
                (
                    Size::new(first, size.height),
                    Size::new(size.width - first - 1.0, size.height),
                    Vector::new(first + 1.0, 0.0),
                )
            }
            Axis::Vertical => {
                let first = (size.height * ratio).round();
                (
                    Size::new(size.width, first),
                    Size::new(size.width, size.height - first - 1.0),
                    Vector::new(0.0, first + 1.0),
                )
            }
        };

        let first = self.first.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(first, first),
        );

        let second = self
            .second
            .as_widget_mut()
            .layout(
                &mut tree.children[1],
                renderer,
                &layout::Limits::new(second, second),
            )
            .move_to(Point::ORIGIN + offset);

        layout::Node::with_children(size, vec![first, second])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let mut children = layout.children();
        let first = children.next().unwrap();
        let second = children.next().unwrap();

        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.first
                .as_widget_mut()
                .operate(&mut tree.children[0], first, renderer, operation);
            self.second
                .as_widget_mut()
                .operate(&mut tree.children[1], second, renderer, operation);
        });
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
        let bounds = layout.bounds();
        let ratio = self.current(tree.state.downcast_ref::<State>());
        let divider = self.divider(bounds, ratio);

        {
            let state = tree.state.downcast_mut::<State>();

            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. })
                    if cursor.is_over(divider) =>
                {
                    state.dragging = true;
                    shell.capture_event();
                    return;
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerLifted { .. })
                    if state.dragging =>
                {
                    state.dragging = false;
                    shell.capture_event();
                    return;
                }
                Event::Mouse(mouse::Event::CursorMoved { .. })
                | Event::Touch(touch::Event::FingerMoved { .. })
                    if state.dragging =>
                {
                    if let Some(position) = cursor.position() {
                        let next = match self.axis {
                            Axis::Horizontal => (position.x - bounds.x) / bounds.width,
                            Axis::Vertical => (position.y - bounds.y) / bounds.height,
                        }
                        .clamp(self.min_ratio, self.max_ratio);

                        if self.ratio.is_none() {
                            state.ratio = next;
                            shell.invalidate_layout();
                        }

                        if let Some(on_resize) = &self.on_resize {
                            shell.publish(on_resize(next));
                        }

                        shell.request_redraw();
                    }

                    shell.capture_event();
                    return;
                }
                _ => {}
            }
        }

        let mut children = layout.children();
        let first = children.next().unwrap();
        let second = children.next().unwrap();

        self.first.as_widget_mut().update(
            &mut tree.children[0],
            event,
            first,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        self.second.as_widget_mut().update(
            &mut tree.children[1],
            event,
            second,
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
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let ratio = self.current(state);
        let mut children = layout.children();
        let first = children.next().unwrap();
        let second = children.next().unwrap();

        for (element, tree, layout) in [
            (&self.first, &tree.children[0], first),
            (&self.second, &tree.children[1], second),
        ] {
            renderer.with_layer(layout.bounds(), |renderer| {
                element
                    .as_widget()
                    .draw(tree, renderer, theme, style, layout, cursor, viewport);
            });
        }

        let appearance = (self.style)(theme);
        let active = state.dragging || cursor.is_over(self.divider(bounds, ratio));

        let line = match self.axis {
            Axis::Horizontal => Rectangle {
                x: first.bounds().x + first.bounds().width,
                y: bounds.y,
                width: 1.0,
                height: bounds.height,
            },
            Axis::Vertical => Rectangle {
                x: bounds.x,
                y: first.bounds().y + first.bounds().height,
                width: bounds.width,
                height: 1.0,
            },
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: line,
                ..renderer::Quad::default()
            },
            Background::Color(if active {
                appearance.divider_active
            } else {
                appearance.divider
            }),
        );

        if self.grip {
            let (width, height) = match self.axis {
                Axis::Horizontal => (12.0, 16.0),
                Axis::Vertical => (16.0, 12.0),
            };
            let center = line.center();

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: center.x - width / 2.0,
                        y: center.y - height / 2.0,
                        width,
                        height,
                    },
                    border: Border {
                        color: appearance.grip_border,
                        width: 1.0,
                        radius: border::radius(2),
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(appearance.grip),
            );

            // Six dots, like Lucide's `grip-vertical`.
            for i in 0..3 {
                for j in 0..2 {
                    let (dx, dy) = match self.axis {
                        Axis::Horizontal => (j as f32 * 4.0 - 2.0, i as f32 * 4.0 - 4.0),
                        Axis::Vertical => (i as f32 * 4.0 - 4.0, j as f32 * 4.0 - 2.0),
                    };

                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: center.x + dx - 0.75,
                                y: center.y + dy - 0.75,
                                width: 1.5,
                                height: 1.5,
                            },
                            border: border::rounded(1),
                            ..renderer::Quad::default()
                        },
                        Background::Color(appearance.divider_active),
                    );
                }
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        if state.dragging || cursor.is_over(self.divider(bounds, self.current(state))) {
            return match self.axis {
                Axis::Horizontal => mouse::Interaction::ResizingHorizontally,
                Axis::Vertical => mouse::Interaction::ResizingVertically,
            };
        }

        let mut children = layout.children();
        let first = children.next().unwrap();
        let second = children.next().unwrap();

        self.first
            .as_widget()
            .mouse_interaction(&tree.children[0], first, cursor, viewport, renderer)
            .max(self.second.as_widget().mouse_interaction(
                &tree.children[1],
                second,
                cursor,
                viewport,
                renderer,
            ))
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let mut children = layout.children();
        let first = children.next().unwrap();
        let second = children.next().unwrap();
        let (a, b) = tree.children.split_at_mut(1);

        let overlays: Vec<_> = self
            .first
            .as_widget_mut()
            .overlay(&mut a[0], first, renderer, viewport, translation)
            .into_iter()
            .chain(self.second.as_widget_mut().overlay(
                &mut b[0],
                second,
                renderer,
                viewport,
                translation,
            ))
            .collect();

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

impl<'a, Message, Theme, Renderer> From<Split<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(widget: Split<'a, Message, Theme, Renderer>) -> Self {
        Element::new(widget)
    }
}
