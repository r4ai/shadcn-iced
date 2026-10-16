//! Content displayed on top of the whole window.
//!
//! [`Layer`] is the headless building block behind dialogs, alert dialogs,
//! sheets, drawers and toasts. It places some content relative to the window
//! (centered, against an edge or in a corner), optionally dims everything
//! behind it with a backdrop and blocks interactions with the rest of the
//! interface.
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::keyboard::{self, key};
use iced::{
    Background, Color, Element, Event, Length, Padding, Point, Rectangle, Size, Vector, alignment,
    mouse, touch, window,
};

/// Content displayed on top of the whole window, over a `base` element.
pub struct Layer<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    base: Element<'a, Message, Theme, Renderer>,
    content: Element<'a, Message, Theme, Renderer>,
    open: bool,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
    padding: Padding,
    backdrop: Option<Box<dyn Fn(&Theme) -> Color + 'a>>,
    modal: bool,
    on_dismiss: Option<Message>,
    dismiss_on_backdrop: bool,
}

impl<'a, Message, Theme, Renderer> Layer<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    /// Creates a new [`Layer`] showing `content` over `base` when `open`.
    ///
    /// By default, the layer is modal, centered and dims the window with a
    /// translucent black backdrop.
    pub fn new(
        base: impl Into<Element<'a, Message, Theme, Renderer>>,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        open: bool,
    ) -> Self {
        Self {
            base: base.into(),
            content: content.into(),
            open,
            align_x: alignment::Horizontal::Center,
            align_y: alignment::Vertical::Center,
            padding: Padding::ZERO,
            backdrop: Some(Box::new(|_| Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
            modal: true,
            on_dismiss: None,
            dismiss_on_backdrop: true,
        }
    }

    /// Sets the horizontal placement of the content in the window.
    pub fn align_x(mut self, align: impl Into<alignment::Horizontal>) -> Self {
        self.align_x = align.into();
        self
    }

    /// Sets the vertical placement of the content in the window.
    pub fn align_y(mut self, align: impl Into<alignment::Vertical>) -> Self {
        self.align_y = align.into();
        self
    }

    /// Sets the space kept between the content and the edges of the window.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the color of the backdrop, or removes it.
    pub fn backdrop(mut self, backdrop: Option<Color>) -> Self {
        self.backdrop = backdrop
            .map(|color| Box::new(move |_: &Theme| color) as Box<dyn Fn(&Theme) -> Color + 'a>);
        self
    }

    /// Sets the color of the backdrop with a function of the theme.
    pub fn backdrop_with(mut self, f: impl Fn(&Theme) -> Color + 'a) -> Self {
        self.backdrop = Some(Box::new(f));
        self
    }

    /// Sets whether the layer blocks interactions with the rest of the
    /// interface. Non-modal layers let events outside of their content
    /// through.
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    /// Sets the message produced when the user wants to close the layer:
    /// <kbd>Escape</kbd> or a click on the backdrop.
    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.on_dismiss = Some(message);
        self
    }

    /// Sets the message produced on dismissal, if any.
    pub fn on_dismiss_maybe(mut self, message: Option<Message>) -> Self {
        self.on_dismiss = message;
        self
    }

    /// Sets whether a click on the backdrop dismisses the layer.
    pub fn dismiss_on_backdrop(mut self, dismiss: bool) -> Self {
        self.dismiss_on_backdrop = dismiss;
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Layer<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.base), Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.base.as_widget(), self.content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        self.base.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.base.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.base
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
        self.base
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
        self.base.as_widget_mut().update(
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
        self.base.as_widget().draw(
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
        self.base.as_widget().mouse_interaction(
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
        let (base_tree, content_tree) = tree.children.split_at_mut(1);

        let base = self.base.as_widget_mut().overlay(
            &mut base_tree[0],
            layout,
            renderer,
            viewport,
            translation,
        );

        let layer = self.open.then(|| {
            overlay::Element::new(Box::new(Overlay {
                content: &mut self.content,
                tree: &mut content_tree[0],
                align_x: self.align_x,
                align_y: self.align_y,
                padding: self.padding,
                backdrop: self.backdrop.as_deref(),
                modal: self.modal,
                on_dismiss: self.on_dismiss.as_ref(),
                dismiss_on_backdrop: self.dismiss_on_backdrop,
            }))
        });

        match (base, layer) {
            (None, None) => None,
            (Some(overlay), None) | (None, Some(overlay)) => Some(overlay),
            (Some(base), Some(layer)) => {
                Some(overlay::Group::with_children(vec![base, layer]).overlay())
            }
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Layer<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(layer: Layer<'a, Message, Theme, Renderer>) -> Self {
        Element::new(layer)
    }
}

/// Creates a new [`Layer`]. See [`Layer::new`].
pub fn layer<'a, Message, Theme, Renderer>(
    base: impl Into<Element<'a, Message, Theme, Renderer>>,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    open: bool,
) -> Layer<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    Layer::new(base, content, open)
}

struct Overlay<'a, 'b, Message, Theme, Renderer> {
    content: &'b mut Element<'a, Message, Theme, Renderer>,
    tree: &'b mut Tree,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
    padding: Padding,
    backdrop: Option<&'b (dyn Fn(&Theme) -> Color + 'a)>,
    modal: bool,
    on_dismiss: Option<&'b Message>,
    dismiss_on_backdrop: bool,
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'_, '_, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds).shrink(self.padding);

        let node = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);

        let size = node.size();
        let available = limits.max();

        let x = self.padding.left
            + match self.align_x {
                alignment::Horizontal::Left => 0.0,
                alignment::Horizontal::Center => (available.width - size.width) / 2.0,
                alignment::Horizontal::Right => available.width - size.width,
            };

        let y = self.padding.top
            + match self.align_y {
                alignment::Vertical::Top => 0.0,
                alignment::Vertical::Center => (available.height - size.height) / 2.0,
                alignment::Vertical::Bottom => available.height - size.height,
            };

        layout::Node::with_children(
            bounds,
            vec![node.move_to(Point::new(x.max(0.0), y.max(0.0)))],
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();

        if let Some(backdrop) = self.backdrop {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    ..renderer::Quad::default()
                },
                Background::Color(backdrop(theme)),
            );
        }

        let content = layout.children().next().unwrap();

        renderer.with_layer(bounds, |renderer| {
            self.content
                .as_widget()
                .draw(self.tree, renderer, theme, style, content, cursor, &bounds);
        });
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content.as_widget_mut().operate(
            self.tree,
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let bounds = layout.bounds();
        let content = layout.children().next().unwrap();
        let content_bounds = content.bounds();
        let was_captured = shell.is_event_captured();

        self.content.as_widget_mut().update(
            self.tree, event, content, cursor, renderer, clipboard, shell, &bounds,
        );

        if !was_captured && shell.is_event_captured() {
            return;
        }

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Escape),
                ..
            }) if self.modal => {
                if let Some(message) = self.on_dismiss {
                    shell.publish(message.clone());
                }

                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonPressed(_))
            | Event::Touch(touch::Event::FingerPressed { .. })
                if self.modal && !cursor.is_over(content_bounds) =>
            {
                if self.dismiss_on_backdrop
                    && let Some(message) = self.on_dismiss
                {
                    shell.publish(message.clone());
                }

                shell.capture_event();
            }
            Event::Window(window::Event::RedrawRequested(_)) => {}
            _ if self.modal => {
                shell.capture_event();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let content = layout.children().next().unwrap();

        let interaction = self
            .content
            .as_widget()
            .mouse_interaction(self.tree, content, cursor, &bounds, renderer);

        if self.modal || cursor.is_over(content.bounds()) {
            interaction.max(mouse::Interaction::Idle)
        } else {
            interaction
        }
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'c>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
        let bounds = layout.bounds();

        self.content.as_widget_mut().overlay(
            self.tree,
            layout.children().next().unwrap(),
            renderer,
            &bounds,
            Vector::ZERO,
        )
    }

    fn index(&self) -> f32 {
        if self.modal { 2.0 } else { 1.5 }
    }
}
