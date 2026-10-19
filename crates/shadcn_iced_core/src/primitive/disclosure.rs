//! A trigger that shows and hides some content.
//!
//! [`Disclosure`] is the headless building block behind collapsibles and
//! accordions. It keeps track of whether it is open by itself (clicking the
//! trigger toggles it) unless you control it with [`Disclosure::open`].
//!
//! Since triggers usually look different when open (e.g. a rotated chevron),
//! it takes one trigger element per state.
//!
//! When its trigger contains a focused control (e.g. a focusable
//! [`Pressable`](super::Pressable) without a message of its own),
//! <kbd>Enter</kbd> and <kbd>Space</kbd> toggle it too.
use super::focus::{self, Focus};
use crate::animation::{self, Transition};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::keyboard::{self, key::Named};

use iced::time::Instant;
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector, mouse, touch, window};

/// A trigger that shows and hides some content.
pub struct Disclosure<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    closed: Element<'a, Message, Theme, Renderer>,
    opened: Element<'a, Message, Theme, Renderer>,
    content: Element<'a, Message, Theme, Renderer>,
    open: Option<bool>,
    default_open: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    spacing: f32,
    width: Length,
    disabled: bool,
}

impl<'a, Message, Theme, Renderer> Disclosure<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    /// Creates a [`Disclosure`] with a trigger for each state and the content
    /// shown when open.
    pub fn new(
        closed: impl Into<Element<'a, Message, Theme, Renderer>>,
        opened: impl Into<Element<'a, Message, Theme, Renderer>>,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            closed: closed.into(),
            opened: opened.into(),
            content: content.into(),
            open: None,
            default_open: false,
            on_toggle: None,
            spacing: 0.0,
            width: Length::Fill,
            disabled: false,
        }
    }

    /// Controls whether the content is shown from your application.
    pub fn open(mut self, open: bool) -> Self {
        self.open = Some(open);
        self
    }

    /// Sets whether a self-managed [`Disclosure`] starts open.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// Sets the message produced when the trigger is pressed, with the
    /// requested state.
    pub fn on_toggle(mut self, f: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(f));
        self
    }

    /// Sets the space between the trigger and the content.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Prevents the trigger from toggling.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Debug)]
struct State {
    open: bool,
    pressed: bool,
    presence: Transition,
    /// The progress of the animation when the layout was last computed.
    laid_out: f32,
    /// Whether the open trigger was laid out last.
    laid_out_open: bool,
    /// The focus of the controls of the trigger that was replaced by the
    /// trigger of the given state, to hand over to it.
    focus: Option<(bool, Vec<Focus>)>,
}

impl<'a, Message, Theme, Renderer> Disclosure<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn is_open(&self, state: &State) -> bool {
        self.open.unwrap_or(state.open)
    }

    /// Toggles the disclosure (or asks the application to), handing the
    /// focus of the current trigger over to the other one.
    fn toggle(
        &mut self,
        tree: &mut Tree,
        trigger: Layout<'_>,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        open: bool,
    ) {
        let next = !open;
        let mut states = Vec::new();

        let (element, trigger_tree) = if open {
            (&mut self.opened, &mut tree.children[1])
        } else {
            (&mut self.closed, &mut tree.children[0])
        };

        focus::for_each_control(element, trigger_tree, trigger, renderer, |_, focus| {
            states.push(*focus);
            focus.blur();
        });

        let state = tree.state.downcast_mut::<State>();
        state.focus = Some((next, states));

        if self.open.is_none() {
            state.open = next;
            shell.invalidate_layout();
            shell.request_redraw();
        }

        if let Some(on_toggle) = &self.on_toggle {
            shell.publish(on_toggle(next));
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Disclosure<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        let open = self.open.unwrap_or(self.default_open);

        tree::State::new(State {
            open: self.default_open,
            pressed: false,
            presence: Transition::new(if open { 1.0 } else { 0.0 }),
            laid_out: if open { 1.0 } else { 0.0 },
            laid_out_open: open,
            focus: None,
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![
            Tree::new(&self.closed),
            Tree::new(&self.opened),
            Tree::new(&self.content),
        ]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[
            self.closed.as_widget(),
            self.opened.as_widget(),
            self.content.as_widget(),
        ]);
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let open = self.is_open(tree.state.downcast_ref::<State>());
        let limits = limits.width(self.width).height(Length::Shrink);
        let now = Instant::now();

        let (visible, progress) = {
            let state = tree.state.downcast_mut::<State>();

            // `animate-accordion-down` / `animate-accordion-up`, with the
            // `ease-out` keyword of CSS (not Tailwind's)
            state.presence.go(
                if open { 1.0 } else { 0.0 },
                animation::NORMAL,
                animation::css_ease_out,
                now,
            );

            let progress = state.presence.value(now);
            state.laid_out = progress;
            state.laid_out_open = open;

            (open || state.presence.is_animating(now), progress)
        };

        let trigger = if open {
            self.opened
                .as_widget_mut()
                .layout(&mut tree.children[1], renderer, &limits)
        } else {
            self.closed
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &limits)
        };

        let trigger_height = trigger.size().height;
        let mut width = trigger.size().width;
        let mut children = vec![trigger];
        let mut height = trigger_height;

        if visible {
            let content = self
                .content
                .as_widget_mut()
                .layout(&mut tree.children[2], renderer, &limits)
                .move_to(Point::new(0.0, trigger_height + self.spacing));

            width = width.max(content.size().width);
            height += (self.spacing + content.size().height) * progress;
            children.push(content);
        }

        let size = limits.resolve(self.width, Length::Shrink, Size::new(width, height));

        layout::Node::with_children(size, children)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let open = self.is_open(tree.state.downcast_ref::<State>());
        let mut children = layout.children();
        let trigger = children.next().unwrap();

        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            if open {
                self.opened.as_widget_mut().operate(
                    &mut tree.children[1],
                    trigger,
                    renderer,
                    operation,
                );

                if let Some(content) = layout.children().nth(1) {
                    self.content.as_widget_mut().operate(
                        &mut tree.children[2],
                        content,
                        renderer,
                        operation,
                    );
                }
            } else {
                self.closed.as_widget_mut().operate(
                    &mut tree.children[0],
                    trigger,
                    renderer,
                    operation,
                );
            }
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
        let open = self.is_open(tree.state.downcast_ref::<State>());

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            let state = tree.state.downcast_ref::<State>();

            if state.presence.is_animating(*now) {
                shell.invalidate_layout();
                shell.request_redraw();
            } else if state.laid_out != state.presence.value(*now) {
                // The layout is still a frame of an animation that has ended.
                shell.invalidate_layout();
                shell.request_redraw();
            }
        }
        let mut children = layout.children();
        let trigger = children.next().unwrap();

        // Once the other trigger is laid out, it takes over the focus.
        let state = tree.state.downcast_mut::<State>();

        if state.laid_out_open == open
            && let Some((_, states)) = state.focus.take_if(|(target, _)| *target == open)
        {
            let (element, trigger_tree) = if open {
                (&mut self.opened, &mut tree.children[1])
            } else {
                (&mut self.closed, &mut tree.children[0])
            };

            focus::for_each_control(element, trigger_tree, trigger, renderer, |index, focus| {
                if let Some(state) = states.get(index) {
                    *focus = *state;
                }
            });

            shell.request_redraw();
        }

        if open {
            self.opened.as_widget_mut().update(
                &mut tree.children[1],
                event,
                trigger,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );

            if let Some(content) = children.next() {
                self.content.as_widget_mut().update(
                    &mut tree.children[2],
                    event,
                    content,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }
        } else {
            self.closed.as_widget_mut().update(
                &mut tree.children[0],
                event,
                trigger,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        if shell.is_event_captured() || self.disabled {
            return;
        }

        // <kbd>Enter</kbd> and <kbd>Space</kbd> toggle a disclosure whose
        // trigger holds the focus (e.g. a focusable `Pressable`).
        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(Named::Enter | Named::Space),
            ..
        }) = event
        {
            let focused = if open {
                focus::is_focus_within(&mut self.opened, &mut tree.children[1], trigger, renderer)
            } else {
                focus::is_focus_within(&mut self.closed, &mut tree.children[0], trigger, renderer)
            };

            if focused {
                self.toggle(tree, trigger, renderer, shell, open);
                shell.capture_event();
            }

            return;
        }

        let over_trigger = cursor.is_over(trigger.bounds());

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
                if over_trigger =>
            {
                tree.state.downcast_mut::<State>().pressed = true;
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
                if tree.state.downcast_ref::<State>().pressed =>
            {
                tree.state.downcast_mut::<State>().pressed = false;

                if over_trigger {
                    self.toggle(tree, trigger, renderer, shell, open);
                    shell.capture_event();
                }
            }
            _ => {}
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
        let state = tree.state.downcast_ref::<State>();
        let open = self.is_open(state);
        let animating = state.presence.is_animating(Instant::now());
        let mut children = layout.children();
        let trigger = children.next().unwrap();

        if open {
            self.opened.as_widget().draw(
                &tree.children[1],
                renderer,
                theme,
                style,
                trigger,
                cursor,
                viewport,
            );
        } else {
            self.closed.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                trigger,
                cursor,
                viewport,
            );
        }

        let clipped = animating || state.laid_out < 1.0;

        if let Some(content) = children.next().filter(|_| open || animating) {
            let draw = |renderer: &mut Renderer| {
                self.content.as_widget().draw(
                    &tree.children[2],
                    renderer,
                    theme,
                    style,
                    content,
                    cursor,
                    viewport,
                );
            };

            if clipped {
                // The content slides out of a growing (or shrinking) box.
                let bounds = layout.bounds();
                let clip = Rectangle {
                    y: trigger.bounds().y + trigger.bounds().height,
                    height: (bounds.y + bounds.height
                        - trigger.bounds().y
                        - trigger.bounds().height)
                        .max(0.0),
                    ..bounds
                };

                renderer.with_layer(clip, draw);
            } else {
                draw(renderer);
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
        let open = self.is_open(tree.state.downcast_ref::<State>());
        let mut children = layout.children();
        let trigger = children.next().unwrap();

        let interaction = if open {
            let trigger_interaction = self.opened.as_widget().mouse_interaction(
                &tree.children[1],
                trigger,
                cursor,
                viewport,
                renderer,
            );

            let content_interaction = children.next().map(|content| {
                self.content.as_widget().mouse_interaction(
                    &tree.children[2],
                    content,
                    cursor,
                    viewport,
                    renderer,
                )
            });

            trigger_interaction.max(content_interaction.unwrap_or_default())
        } else {
            self.closed.as_widget().mouse_interaction(
                &tree.children[0],
                trigger,
                cursor,
                viewport,
                renderer,
            )
        };

        if interaction == mouse::Interaction::None
            && !self.disabled
            && cursor.is_over(trigger.bounds())
        {
            mouse::Interaction::Pointer
        } else {
            interaction
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let open = self.open.unwrap_or(tree.state.downcast_ref::<State>().open);
        let mut children = layout.children();
        let trigger = children.next().unwrap();
        let [closed, opened, content] = &mut tree.children[..] else {
            return None;
        };

        let mut overlays = Vec::new();

        if open {
            overlays.extend(self.opened.as_widget_mut().overlay(
                opened,
                trigger,
                renderer,
                viewport,
                translation,
            ));

            if let Some(layout) = children.next() {
                overlays.extend(self.content.as_widget_mut().overlay(
                    content,
                    layout,
                    renderer,
                    viewport,
                    translation,
                ));
            }
        } else {
            overlays.extend(self.closed.as_widget_mut().overlay(
                closed,
                trigger,
                renderer,
                viewport,
                translation,
            ));
        }

        match overlays.len() {
            0 => None,
            1 => overlays.pop(),
            _ => Some(overlay::Group::with_children(overlays).overlay()),
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Disclosure<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(widget: Disclosure<'a, Message, Theme, Renderer>) -> Self {
        Element::new(widget)
    }
}

/// Creates a new [`Disclosure`].
pub fn disclosure<'a, Message, Theme, Renderer>(
    closed: impl Into<Element<'a, Message, Theme, Renderer>>,
    opened: impl Into<Element<'a, Message, Theme, Renderer>>,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Disclosure<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    Disclosure::new(closed, opened, content)
}
