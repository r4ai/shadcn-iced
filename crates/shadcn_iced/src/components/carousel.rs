//! A carousel with motion and swipe built using iced.
//!
//! Port of <https://ui.shadcn.com/docs/components/carousel>. The current
//! index lives in your state. Once the carousel was clicked (like the focus
//! within its region on the web), <kbd>ArrowLeft</kbd> and
//! <kbd>ArrowRight</kbd> go to the previous and next items.
//!
//! ```no_run
//! # use shadcn_iced::carousel::carousel;
//! # #[derive(Clone)] enum Message { Slide(usize) }
//! # fn view<'a>(index: usize) -> iced::Element<'a, Message> {
//! carousel(index, Message::Slide)
//!     .item(iced::widget::text("1"))
//!     .item(iced::widget::text("2"))
//!     .into()
//! # }
//! ```
use super::button::{self, Variant, icon_button};

use shadcn_iced_core::Element;
use shadcn_iced_core::icon::lucide;
use shadcn_iced_core::primitive::track;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::keyboard::{self, key};
use iced::widget::{Column, Row, container};
use iced::{Alignment, Event, Length, Rectangle, Size, Theme, Vector, mouse};

/// The scrolling direction of a [`Carousel`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Orientation {
    /// Items side by side, previous/next on the left/right.
    #[default]
    Horizontal,
    /// Items stacked, previous/next above/below.
    Vertical,
}

/// A carousel. See the [module documentation](self).
pub struct Carousel<'a, Message> {
    index: usize,
    on_change: Box<dyn Fn(usize) -> Message + 'a>,
    items: Vec<Element<'a, Message>>,
    per_view: usize,
    spacing: f32,
    orientation: Orientation,
    looping: bool,
    height: Option<f32>,
}

/// Creates a [`Carousel`] showing the item at `index` first.
pub fn carousel<'a, Message: Clone + 'a>(
    index: usize,
    on_change: impl Fn(usize) -> Message + 'a,
) -> Carousel<'a, Message> {
    Carousel {
        index,
        on_change: Box::new(on_change),
        items: Vec::new(),
        per_view: 1,
        spacing: 16.0,
        orientation: Orientation::Horizontal,
        looping: false,
        height: None,
    }
}

impl<'a, Message: Clone + 'a> Carousel<'a, Message> {
    /// Appends an item.
    pub fn item(mut self, item: impl Into<Element<'a, Message>>) -> Self {
        self.items.push(item.into());
        self
    }

    /// Appends several items.
    pub fn items(mut self, items: impl IntoIterator<Item = Element<'a, Message>>) -> Self {
        self.items.extend(items);
        self
    }

    /// Sets how many items are visible at once (`basis-1/3`...).
    pub fn per_view(mut self, count: usize) -> Self {
        self.per_view = count.max(1);
        self
    }

    /// Sets the space between visible items (`pl-4` by default).
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the orientation.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Wraps around at both ends (`loop: true`).
    pub fn looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// Gives a vertical carousel a fixed height (`h-[200px]` on the
    /// `CarouselContent`): the items keep their own height and the ones
    /// that do not fit are clipped, instead of showing exactly
    /// [`per_view`](Self::per_view) items.
    pub fn height(mut self, height: impl Into<iced::Pixels>) -> Self {
        self.height = Some(height.into().0);
        self
    }
}

impl<'a, Message: Clone + 'a> From<Carousel<'a, Message>> for Element<'a, Message> {
    fn from(carousel: Carousel<'a, Message>) -> Self {
        let count = carousel.items.len();
        let last = count.saturating_sub(carousel.per_view);
        let index = carousel.index.min(last);
        let horizontal = carousel.orientation == Orientation::Horizontal;

        let previous = if index > 0 {
            Some(index - 1)
        } else if carousel.looping && count > 0 {
            Some(last)
        } else {
            None
        };

        let next = if index < last {
            Some(index + 1)
        } else if carousel.looping && count > 0 {
            Some(0)
        } else {
            None
        };

        // `variant="outline" size="icon-sm" rounded-full`
        let nav = |icon, target: Option<usize>| {
            icon_button(icon)
                .variant(Variant::Outline)
                .size(button::Size::IconSm)
                .disabled(target.is_none())
                .on_press_maybe(target.map(|target| (carousel.on_change)(target)))
                .position(shadcn_iced_core::style::Position::Alone, false)
        };

        let previous_message = previous.map(|target| (carousel.on_change)(target));
        let next_message = next.map(|target| (carousel.on_change)(target));

        let (before, after) = if horizontal {
            (lucide::CHEVRON_LEFT, lucide::CHEVRON_RIGHT)
        } else {
            (lucide::CHEVRON_UP, lucide::CHEVRON_DOWN)
        };
        let before = round(nav(before, previous));
        let after = round(nav(after, next));

        // All the items, sliding to the current one (and draggable).
        let on_change = carousel.on_change;
        let slides = track(
            carousel
                .items
                .into_iter()
                .map(|item| container(item).width(Length::Fill).into()),
            index,
        )
        .per_view(carousel.per_view)
        .spacing(carousel.spacing)
        .vertical(!horizontal)
        .height(carousel.height)
        .on_change(on_change);

        let content: Element<'a, Message> = if horizontal {
            Row::new()
                .push(before)
                .push(slides)
                .push(after)
                // `-left-12` / `-right-12`: 48px from the slides, minus
                // the 28px button.
                .spacing(20)
                .align_y(Alignment::Center)
                .into()
        } else {
            Column::new()
                .push(before)
                .push(slides)
                .push(after)
                // `-top-12` / `-bottom-12`
                .spacing(20)
                .align_x(Alignment::Center)
                .into()
        };

        Element::new(Keys {
            content,
            previous: previous_message,
            next: next_message,
        })
    }
}

/// Goes to the previous or next item with <kbd>ArrowLeft</kbd> and
/// <kbd>ArrowRight</kbd> (`onKeyDownCapture` on the carousel region), once
/// the carousel was clicked. iced has no focus within a region: a click
/// inside of it stands for the focus of its buttons, and a click outside of
/// it for their blur.
struct Keys<'a, Message> {
    content: Element<'a, Message>,
    previous: Option<Message>,
    next: Option<Message>,
}

#[derive(Debug, Default)]
struct KeysState {
    active: bool,
}

impl<Message: Clone> Widget<Message, Theme, iced::Renderer> for Keys<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<KeysState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(KeysState::default())
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

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        layout::Node::with_children(child.size(), vec![child])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
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
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<KeysState>();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(_)) => {
                state.active = cursor.is_over(layout.bounds());
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) if state.active => {
                let message = match key.as_ref() {
                    keyboard::Key::Named(key::Named::ArrowLeft) => Some(&self.previous),
                    keyboard::Key::Named(key::Named::ArrowRight) => Some(&self.next),
                    _ => None,
                };

                if let Some(message) = message {
                    if let Some(message) = message {
                        shell.publish(message.clone());
                    }

                    shell.capture_event();
                    return;
                }
            }
            _ => {}
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
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
        renderer: &mut iced::Renderer,
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
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
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
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

/// Makes the navigation buttons fully rounded (`rounded-full`).
fn round<'a, Message: Clone + 'a>(
    button: super::button::Button<'a, Message>,
) -> Element<'a, Message> {
    button.rounded(true).into()
}
