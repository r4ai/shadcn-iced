//! Floating content anchored to another element.
//!
//! [`Anchored`] is the headless building block behind popovers, dropdown
//! menus, selects, tooltips, hover cards and context menus. It takes care of:
//!
//! - positioning the floating content next to the anchor (on a [`Side`],
//!   with an [`Align`]ment and an offset), flipping it to the opposite side
//!   and shifting it along its alignment so it stays inside the window,
//! - opening it, either controlled by your application ([`Anchored::open`])
//!   or self-managed: on click ([`Anchored::new`]'s default), on hover
//!   ([`Anchored::hover`]), on right click ([`Anchored::context_menu`]) or
//!   like the popup of a combobox ([`Anchored::combobox`]),
//! - dismissing it on <kbd>Escape</kbd>, on a click outside, and (for menus)
//!   once an item has been activated, with a click or <kbd>Enter</kbd>,
//! - switching between the menus of a [`Group`] (a menubar) when another
//!   trigger is hovered or pressed, or with <kbd>←</kbd> and <kbd>→</kbd>,
//! - touch screens, which have no hover and no right click: a tap opens
//!   what a click opens (a drag that starts on the anchor scrolls instead),
//!   a long press what a right click opens, and hover content opens with a
//!   long press or a tap (see [`TouchOpen`]),
//! - the keyboard: <kbd>Enter</kbd>, <kbd>Space</kbd> and <kbd>↓</kbd> on a
//!   focused anchor open what a click opens, and focus its first control
//!   (or highlight the first item of a menu, see [`Anchored::menu_keys`]);
//!   <kbd>Tab</kbd> stays within open content, or closes a menu.
use super::{focus, pointer, tab};
use crate::animation::{self, Appearance, Fade, Transition};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};

use iced::keyboard::{self, key};
use iced::time::{Duration, Instant};
use iced::{
    Background, Border, Color, Element, Event, Length, Point, Rectangle, Size, Vector, mouse,
    touch, window,
};

use std::cell::Cell;

/// How long a tooltip opened with a long press stays once the finger lifted
/// (like the tooltips of Android).
const TOUCH_LINGER: Duration = Duration::from_millis(1500);

thread_local! {
    /// Incremented whenever a menu closes because one of its items was
    /// selected. Menus opened before compare it on every event and close
    /// too, which closes a whole chain of submenus at once. Per thread, like
    /// the interface that runs on it: the menus of another one (a test
    /// running in parallel) are not closed.
    static SELECT_EPOCH: Cell<u64> = const { Cell::new(0) };
}

fn select_epoch() -> u64 {
    SELECT_EPOCH.get()
}

/// Closes the menus opened before an item was selected.
fn bump_select_epoch() {
    SELECT_EPOCH.set(SELECT_EPOCH.get().wrapping_add(1));
}

/// The side of the anchor the floating content is placed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Side {
    /// Above the anchor.
    Top,
    /// On the right of the anchor.
    Right,
    /// Below the anchor.
    #[default]
    Bottom,
    /// On the left of the anchor.
    Left,
}

impl Side {
    /// Returns the opposite side.
    pub fn opposite(self) -> Self {
        match self {
            Side::Top => Side::Bottom,
            Side::Right => Side::Left,
            Side::Bottom => Side::Top,
            Side::Left => Side::Right,
        }
    }
}

/// The alignment of the floating content along the side of the anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Align {
    /// Aligned with the start (left or top) of the anchor.
    Start,
    /// Centered on the anchor.
    #[default]
    Center,
    /// Aligned with the end (right or bottom) of the anchor.
    End,
}

enum Trigger<Message> {
    Controlled {
        open: bool,
        on_dismiss: Option<Message>,
    },
    Click,
    Hover {
        open_delay: Duration,
        close_delay: Duration,
    },
    ContextMenu,
    Input,
}

/// How hover content ([`Anchored::hover`]) opens on a touch screen, which has
/// no hover: a tap on the anchor must still press it (a link, a button), so
/// a plain tap only opens content that is not a preview of what the anchor
/// does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TouchOpen {
    /// A long press shows the content while the finger rests, and a little
    /// longer (a tooltip). The next tap closes it.
    #[default]
    Hold,
    /// A long press opens the content, until a tap outside of it (a hover
    /// card, whose content can be pressed).
    LongPress,
    /// A tap on the anchor opens and closes the content, like a click opens
    /// a submenu or the panel of a navigation menu in Radix. The keyboard
    /// opens it too: <kbd>Enter</kbd>, <kbd>Space</kbd> or the arrow key
    /// towards the content, on a focused anchor or on an anchor that is
    /// the highlighted item of a list (see [`report_highlighted`]).
    Tap,
}

/// Floating content anchored to another element.
pub struct Anchored<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    anchor: Element<'a, Message, Theme, Renderer>,
    content: Element<'a, Message, Theme, Renderer>,
    trigger: Trigger<Message>,
    side: Side,
    align: Align,
    offset: f32,
    align_offset: f32,
    collision_padding: f32,
    outset: f32,
    match_anchor_width: bool,
    anchor_width: Option<f32>,
    fallback_offset: Option<f32>,
    close_on_select: bool,
    modal: bool,
    on_open_change: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    open_highlight: Option<Box<dyn Fn(&Theme) -> (Background, Border) + 'a>>,
    focus_ring: Option<Box<dyn Fn(&Theme) -> focus::Style + 'a>>,
    animate: bool,
    anchor_tracker: Option<AnchorTracker>,
    touch_open: TouchOpen,
    menu_keys: bool,
}

/// Where the anchor of open floating content is, for the content to point
/// at it (e.g. the arrow of a tooltip, which follows the trigger when the
/// content is shifted to stay inside the window).
///
/// Share a clone with [`Anchored::track_anchor`]: the bounds of the anchor,
/// in the coordinates of the content, are set when the content is laid out,
/// before it is drawn.
#[derive(Debug, Clone, Default)]
pub struct AnchorTracker(std::rc::Rc<std::cell::Cell<Option<Rectangle>>>);

impl AnchorTracker {
    /// Creates a tracker that knows no anchor yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// The bounds of the anchor of the open content, if any.
    pub fn anchor(&self) -> Option<Rectangle> {
        self.0.get()
    }
}

impl<'a, Message, Theme, Renderer> Anchored<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    /// Creates floating `content` anchored to `anchor`, toggled by clicking
    /// the anchor.
    pub fn new(
        anchor: impl Into<Element<'a, Message, Theme, Renderer>>,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            anchor: anchor.into(),
            content: content.into(),
            trigger: Trigger::Click,
            side: Side::Bottom,
            align: Align::Center,
            offset: 4.0,
            align_offset: 0.0,
            collision_padding: 8.0,
            outset: 0.0,
            anchor_width: None,
            fallback_offset: None,
            match_anchor_width: false,
            close_on_select: false,
            modal: false,
            on_open_change: None,
            open_highlight: None,
            focus_ring: None,
            animate: true,
            anchor_tracker: None,
            touch_open: TouchOpen::Hold,
            menu_keys: false,
        }
    }

    /// Tells `tracker` where the anchor is whenever the content is laid out.
    pub fn track_anchor(mut self, tracker: AnchorTracker) -> Self {
        self.anchor_tracker = Some(tracker);
        self
    }

    /// Controls whether the content is open from your application.
    ///
    /// Pair it with [`Anchored::on_dismiss`] to be notified when the user
    /// wants to close it.
    pub fn open(mut self, open: bool) -> Self {
        let on_dismiss = match std::mem::replace(&mut self.trigger, Trigger::Click) {
            Trigger::Controlled { on_dismiss, .. } => on_dismiss,
            _ => None,
        };

        self.trigger = Trigger::Controlled { open, on_dismiss };
        self
    }

    /// Sets the message produced when a controlled [`Anchored`] should be
    /// dismissed (<kbd>Escape</kbd>, click outside, item selected...).
    pub fn on_dismiss(mut self, message: Message) -> Self {
        let open = match self.trigger {
            Trigger::Controlled { open, .. } => open,
            _ => false,
        };

        self.trigger = Trigger::Controlled {
            open,
            on_dismiss: Some(message),
        };
        self
    }

    /// Opens the content while the anchor (or the content itself) is
    /// hovered, after the given delays.
    pub fn hover(mut self, open_delay: Duration, close_delay: Duration) -> Self {
        self.trigger = Trigger::Hover {
            open_delay,
            close_delay,
        };
        self
    }

    /// Sets how hover content opens on a touch screen. Defaults to
    /// [`TouchOpen::Hold`], like a tooltip.
    pub fn touch_open(mut self, touch_open: TouchOpen) -> Self {
        self.touch_open = touch_open;
        self
    }

    /// Makes the keyboard treat the content as a menu, like Radix: opened
    /// with the keyboard, its first item is highlighted (its last one with
    /// <kbd>↑</kbd>), <kbd>Tab</kbd> closes it and lets the focus move on,
    /// and <kbd>←</kbd> closes a submenu ([`Side::Right`]). Otherwise,
    /// content opened with the keyboard focuses its first control, and
    /// <kbd>Tab</kbd> cycles through its controls while it is open.
    pub fn menu_keys(mut self, menu: bool) -> Self {
        self.menu_keys = menu;
        self
    }

    /// Opens the content at the cursor position when the anchor is right
    /// clicked, or long pressed on a touch screen; <kbd>Shift</kbd> +
    /// <kbd>F10</kbd> and the context menu key open it at the anchor while
    /// the focus is within it.
    pub fn context_menu(mut self) -> Self {
        self.trigger = Trigger::ContextMenu;
        self.side = Side::Bottom;
        self.align = Align::Start;
        self.offset = 2.0;
        self.close_on_select = true;
        self.modal = true;
        self.menu_keys = true;
        self
    }

    /// Opens the content like the popup of a combobox: when a [`Toggle`]
    /// inside the anchor is pressed, or when the focused anchor receives
    /// text or <kbd>↓</kbd> / <kbd>↑</kbd> (which also highlight the first
    /// or last item). Pressing the rest of the anchor (the input) neither
    /// opens nor closes it.
    pub fn combobox(mut self) -> Self {
        self.trigger = Trigger::Input;
        self
    }

    /// Sets the side of the anchor the content is placed on.
    pub fn side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }

    /// Sets the alignment of the content along the side.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Sets the distance between the anchor and the content.
    pub fn offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    /// Sets an offset along the alignment axis.
    pub fn align_offset(mut self, offset: f32) -> Self {
        self.align_offset = offset;
        self
    }

    /// Sets the minimum distance kept between the content and the edges of
    /// the window.
    pub fn collision_padding(mut self, padding: f32) -> Self {
        self.collision_padding = padding;
        self
    }

    /// Sets the width of a ring drawn around the content, outside of the
    /// element it surrounds (`ring-1`): the content is placed, sized and
    /// kept inside the window as if it did not have it.
    pub fn outset(mut self, outset: f32) -> Self {
        self.outset = outset;
        self
    }

    /// Makes the content exactly as wide as the anchor, but at least `min`
    /// wide (`w-(--anchor-width) min-w-32`), whatever its own width.
    pub fn anchor_width(mut self, min: f32) -> Self {
        self.anchor_width = Some(min);
        self
    }

    /// Sets the offset used instead of [`Anchored::offset`] when the
    /// content placed with it would not fit in the window, e.g. a select
    /// list overlapping its trigger (`alignItemWithTrigger`) that falls
    /// back to opening below it.
    pub fn fallback_offset(mut self, offset: f32) -> Self {
        self.fallback_offset = Some(offset);
        self
    }

    /// Makes the content at least as wide as the anchor.
    pub fn match_anchor_width(mut self, match_width: bool) -> Self {
        self.match_anchor_width = match_width;
        self
    }

    /// Closes the content once one of its buttons has been pressed.
    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
        self
    }

    /// Blocks interactions with the rest of the interface while open.
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    /// Sets whether the content fades and zooms in and out
    /// (`animate-in fade-in-0 zoom-in-95 duration-100`). Defaults to `true`.
    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }

    /// Draws a background behind the anchor while the content is open,
    /// like `aria-expanded:bg-muted` on a trigger.
    pub fn open_highlight(mut self, f: impl Fn(&Theme) -> (Background, Border) + 'a) -> Self {
        self.open_highlight = Some(Box::new(f));
        self
    }

    /// Draws a focus ring around the anchor once the content has been
    /// dismissed with the keyboard (<kbd>Escape</kbd>, or <kbd>Enter</kbd>
    /// on an item), like the `focus-visible` style of a trigger the focus
    /// returns to. It lasts until the next press. Only the `ring` and
    /// `border` of the style are drawn, the border over the anchor.
    pub fn focus_ring(mut self, f: impl Fn(&Theme) -> focus::Style + 'a) -> Self {
        self.focus_ring = Some(Box::new(f));
        self
    }

    /// Sets a closure producing a message whenever a self-managed
    /// [`Anchored`] opens or closes.
    pub fn on_open_change(mut self, f: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_open_change = Some(Box::new(f));
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum Hover {
    #[default]
    Idle,
    Opening(Instant),
    Open,
    Closing(Instant),
}

#[derive(Debug, Default)]
struct State {
    presence: Transition,
    open: bool,
    epoch: u64,
    point: Option<Point>,
    hover: Hover,
    anchor_hovered: bool,
    /// Whether the pointer is over the anchor, grace area aside.
    anchor_over: bool,
    content_hovered: bool,
    /// Whether hover content was dismissed (<kbd>Escape</kbd>) and stays
    /// closed until the pointer leaves.
    suppressed: bool,
    /// Whether the content was dismissed with the keyboard, which leaves
    /// the anchor `focus-visible`.
    focus_visible: bool,
    /// The key event that opened the content, replayed into it once it is
    /// shown (<kbd>↓</kbd> highlights the first item).
    replay: Option<Event>,
    /// The bounds of the hover content, once shown.
    content_bounds: Option<Rectangle>,
    /// The area the pointer can cross on its way to the hover content.
    grace: Option<Grace>,
    /// The press of a finger on the anchor, which opens click content once
    /// it lifts as a tap.
    press: Option<pointer::Press>,
    /// The finger resting on the anchor, until it presses long.
    long_press: Option<pointer::LongPress>,
    /// The finger whose long press opened the content: its lift is not a
    /// tap on the anchor.
    swallow: Option<touch::Finger>,
    /// Whether hover content stays open until it is dismissed: opened with
    /// a tap, a long press or the keyboard.
    pinned: bool,
    /// Whether the content was pinned with the keyboard, which closes it
    /// when the focus leaves the anchor.
    pinned_by_keyboard: bool,
    /// Whether the finger of the long press that opened a tooltip rests.
    held: bool,
    /// When the tooltip opened with a long press closes, once the finger
    /// lifted.
    linger: Option<Instant>,
    /// Whether the anchor shows its keyboard focus (`:focus-visible`),
    /// which opens tooltips and hover cards.
    focused: bool,
    /// Whether the content was opened with the keyboard and focuses its
    /// first control once shown.
    focus_content: bool,
    /// Whether the anchor is the highlighted item of a list, as reported
    /// while it was last drawn (see [`report_highlighted`]).
    highlighted: Cell<bool>,
}

thread_local! {
    /// The [`Grace`] area the pointer is in, if any.
    static GRACE: std::cell::Cell<Option<Grace>> = const { std::cell::Cell::new(None) };

    /// Whether the anchor being drawn is highlighted, see
    /// [`report_highlighted`].
    static HIGHLIGHTED: Cell<Option<bool>> = const { Cell::new(None) };
}

/// Tells the [`Anchored`] whose anchor is being drawn whether the anchor is
/// the highlighted item of a [`Listbox`](super::listbox::Listbox), e.g.
/// from [`ItemRow::trailing_color`](super::listbox::ItemRow::trailing_color).
///
/// The highlight of a list is not focus: a submenu trigger learns this way
/// that <kbd>→</kbd>, <kbd>Enter</kbd> and <kbd>Space</kbd> are meant for it
/// (see [`TouchOpen::Tap`]).
pub fn report_highlighted(highlighted: bool) {
    HIGHLIGHTED.with(|cell| {
        if cell.get().is_some() {
            cell.set(Some(highlighted));
        }
    });
}

/// When `event` happens: the time of a redraw (which tests advance), or
/// now.
fn event_time(event: &Event) -> Instant {
    match event {
        Event::Window(window::Event::RedrawRequested(at)) => *at,
        _ => Instant::now(),
    }
}

/// A press of `key` without modifiers, e.g. to replay into content opened
/// with another key.
fn key_event(named: key::Named) -> Event {
    let key = keyboard::Key::Named(named);

    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::default(),
        text: None,
        repeat: false,
    })
}

/// The named key pressed by `event`, if it is pressed without the modifiers
/// of shortcuts.
fn plain_key(event: &Event) -> Option<key::Named> {
    match event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(named),
            modifiers,
            ..
        }) if !modifiers.command() && !modifiers.alt() && !modifiers.logo() => Some(*named),
        _ => None,
    }
}

/// Whether `position` is in the pointer grace area of an open submenu: the
/// triangle between the point where the pointer left its trigger and the
/// near edge of the submenu (Radix's "pointer grace area", also known as a
/// safe triangle).
///
/// The items under it are not highlighted and the submenus under it do not
/// open, so that the pointer can move diagonally to the submenu.
pub fn is_in_pointer_grace(position: Point) -> bool {
    GRACE
        .get()
        .is_some_and(|grace| Instant::now() < grace.deadline && grace.contains(position))
}

/// The pointer grace area of hover content (see [`is_in_pointer_grace`]).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Grace {
    triangle: [Point; 3],
    /// The pointer must reach the content before then (Radix's 300ms).
    deadline: Instant,
}

impl Grace {
    /// The grace area from `exit`, where the pointer left `anchor`, to the
    /// content on its left or right, if it is on one of those sides and the
    /// pointer did not leave past the other side of the anchor (moving
    /// away from the content, or jumping to another anchor).
    fn new(exit: Point, anchor: Rectangle, content: Rectangle, now: Instant) -> Option<Self> {
        // Moved back a little, so that the pointer can wobble.
        const BLEED: f32 = 5.0;

        let (x, apex) = if content.x >= exit.x && exit.x >= anchor.x - BLEED {
            (content.x, Point::new(exit.x - BLEED, exit.y))
        } else if content.x + content.width <= exit.x && exit.x <= anchor.x + anchor.width + BLEED {
            (
                content.x + content.width,
                Point::new(exit.x + BLEED, exit.y),
            )
        } else {
            return None;
        };

        Some(Self {
            triangle: [
                apex,
                Point::new(x, content.y),
                Point::new(x, content.y + content.height),
            ],
            deadline: now + Duration::from_millis(300),
        })
    }

    fn contains(&self, point: Point) -> bool {
        let [a, b, c] = self.triangle;
        let side =
            |p: Point, q: Point| (q.x - p.x) * (point.y - p.y) - (q.y - p.y) * (point.x - p.x);

        let (d1, d2, d3) = (side(a, b), side(b, c), side(c, a));
        let negative = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
        let positive = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;

        !(negative && positive)
    }
}

impl State {
    fn is_open<Message>(&self, trigger: &Trigger<Message>) -> bool {
        match trigger {
            Trigger::Controlled { open, .. } => *open,
            Trigger::Click | Trigger::ContextMenu | Trigger::Input => self.open,
            Trigger::Hover { .. } => {
                matches!(self.hover, Hover::Open | Hover::Closing(_))
            }
        }
    }

    fn tick<Message>(
        &mut self,
        now: Instant,
        open_delay: Duration,
        close_delay: Duration,
        shell: &mut Shell<'_, Message>,
    ) {
        if let Some(linger) = self.linger {
            if now < linger {
                shell.request_redraw_at(linger);
            } else {
                self.linger = None;
            }
        }

        // Content opened with a tap, a long press or the keyboard is open
        // until dismissed, and does not wait for the delay of a hover.
        let immediate = self.pinned || self.held;
        let mut hovered = self.anchor_hovered
            || self.content_hovered
            || self.focused
            || immediate
            || self.linger.is_some();

        if self.suppressed {
            self.suppressed = hovered;
            hovered = false;
        }

        let next = match self.hover {
            Hover::Idle if hovered => {
                if open_delay.is_zero() || immediate {
                    Hover::Open
                } else {
                    shell.request_redraw_at(now + open_delay);
                    Hover::Opening(now)
                }
            }
            Hover::Opening(_) if !hovered => Hover::Idle,
            Hover::Opening(_) if immediate => Hover::Open,
            Hover::Opening(at) if now >= at + open_delay => Hover::Open,
            Hover::Opening(at) => {
                shell.request_redraw_at(at + open_delay);
                Hover::Opening(at)
            }
            Hover::Open if !hovered => {
                if close_delay.is_zero() {
                    Hover::Idle
                } else {
                    shell.request_redraw_at(now + close_delay);
                    Hover::Closing(now)
                }
            }
            Hover::Closing(_) if hovered => Hover::Open,
            Hover::Closing(at) if now >= at + close_delay => Hover::Idle,
            Hover::Closing(at) => {
                shell.request_redraw_at(at + close_delay);
                Hover::Closing(at)
            }
            hover => hover,
        };

        let was_open = matches!(self.hover, Hover::Open | Hover::Closing(_));
        let is_open = matches!(next, Hover::Open | Hover::Closing(_));

        if is_open && !was_open {
            self.epoch = select_epoch();
        }

        self.hover = next;

        if was_open != is_open {
            shell.invalidate_layout();
            shell.request_redraw();
        }
    }

    /// Closes hover content right away, e.g. on a second tap on its anchor.
    fn close_hover<Message>(&mut self, shell: &mut Shell<'_, Message>) {
        self.pinned = false;
        self.pinned_by_keyboard = false;
        self.held = false;
        self.linger = None;
        self.hover = Hover::Idle;
        shell.invalidate_layout();
        shell.request_redraw();
    }
}

/// Opens self-managed click content.
fn open_click<Message>(
    state: &mut State,
    shell: &mut Shell<'_, Message>,
    on_open_change: Option<&dyn Fn(bool) -> Message>,
) {
    state.open = true;
    state.epoch = select_epoch();
    shell.invalidate_layout();
    shell.request_redraw();

    if let Some(f) = on_open_change {
        shell.publish(f(true));
    }
}

/// Opens a context menu at `position`.
fn open_at<Message>(
    state: &mut State,
    position: Point,
    shell: &mut Shell<'_, Message>,
    on_open_change: Option<&dyn Fn(bool) -> Message>,
) {
    let was_open = state.open;

    state.open = true;
    state.epoch = select_epoch();
    state.point = Some(position);
    shell.invalidate_layout();
    shell.request_redraw();

    if !was_open && let Some(f) = on_open_change {
        shell.publish(f(true));
    }
}

/// Prepares content opened with `key`: a menu highlights its first item
/// (its last one for <kbd>↑</kbd>), other content focuses its first control,
/// once shown.
fn opened_with_keyboard(state: &mut State, menu: bool, key: key::Named) {
    if menu {
        // `Home` and `End` rather than the arrows: a menu opened before may
        // still have an item highlighted.
        state.replay = Some(key_event(if key == key::Named::ArrowUp {
            key::Named::End
        } else {
            key::Named::Home
        }));
    } else {
        state.focus_content = true;
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Anchored<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Fade + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.anchor), Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.anchor.as_widget(), self.content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        self.anchor.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.anchor.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.anchor
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
        self.anchor
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
        pointer::observe(event);

        let was_captured = shell.is_event_captured();
        let bounds = layout.bounds();

        // The lift of the finger whose long press opened the content: the
        // anchor sees the finger lost instead, so that it is not pressed
        // (a long press on a link of a mobile browser does not follow it).
        let swallowed = match event {
            Event::Touch(
                touch::Event::FingerLifted { id, position }
                | touch::Event::FingerLost { id, position },
            ) if tree.state.downcast_ref::<State>().swallow == Some(*id) => {
                Some(Event::Touch(touch::Event::FingerLost {
                    id: *id,
                    position: *position,
                }))
            }
            _ => None,
        };

        self.anchor.as_widget_mut().update(
            &mut tree.children[0],
            swallowed.as_ref().unwrap_or(event),
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let state = tree.state.downcast_mut::<State>();

        if swallowed.is_some() {
            state.swallow = None;
            shell.capture_event();
        } else if let Event::Touch(touch::Event::FingerPressed { id, .. }) = event
            && state.swallow == Some(*id)
        {
            // The system reused the id of a finger whose lift never came.
            state.swallow = None;
        }

        if let Event::Mouse(mouse::Event::ButtonPressed(_))
        | Event::Touch(touch::Event::FingerPressed { .. })
        | Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key::Named::Tab),
            ..
        }) = event
            && state.focus_visible
        {
            state.focus_visible = false;
            shell.request_redraw();
        }

        // A finger resting on the anchor long enough: what a right click or
        // a hover opens with a mouse.
        let long_pressed = match state.long_press {
            Some(hold) => match hold.update(event) {
                pointer::Hold::Long => {
                    state.long_press = None;
                    Some(hold)
                }
                pointer::Hold::Ended => {
                    state.long_press = None;
                    None
                }
                pointer::Hold::Waiting => {
                    shell.request_redraw_at(hold.deadline);
                    None
                }
            },
            None => None,
        };

        if let Some(hold) = long_pressed {
            state.swallow = Some(hold.finger);

            // The anchor (a button) forgets the press now, so that the
            // content does not open on top of a pressed anchor.
            let lost = Event::Touch(touch::Event::FingerLost {
                id: hold.finger,
                position: hold.origin,
            });
            let mut messages = Vec::new();

            self.anchor.as_widget_mut().update(
                &mut tree.children[0],
                &lost,
                layout,
                cursor,
                renderer,
                clipboard,
                &mut Shell::new(&mut messages),
                viewport,
            );
        }

        let state = tree.state.downcast_mut::<State>();
        let anchor = &mut self.anchor;
        let anchor_tree = &mut tree.children[0];
        let mut anchor_focused = || {
            let mut focused = FindFocused(false);
            anchor
                .as_widget_mut()
                .operate(anchor_tree, layout, renderer, &mut focused);
            focused.0
        };

        match &self.trigger {
            Trigger::Controlled { .. } => {}
            Trigger::Click => {
                match event {
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                        if cursor.is_over(bounds) && !state.open =>
                    {
                        open_click(state, shell, self.on_open_change.as_deref());
                    }
                    // A finger opens the content once it lifts as a tap:
                    // a scroll that starts on the anchor does not, and the
                    // press is left to a parent scrollable.
                    Event::Touch(touch::Event::FingerPressed { .. }) => {
                        state.press = (!was_captured && !state.open)
                            .then(|| pointer::Press::start(event, cursor, |p| bounds.contains(p)))
                            .flatten();
                    }
                    _ => {
                        if let Some(press) = &mut state.press {
                            match press.update(event, cursor) {
                                Some(pointer::Gesture::Tapped(position)) => {
                                    state.press = None;

                                    if bounds.contains(position) && !state.open {
                                        open_click(state, shell, self.on_open_change.as_deref());
                                    }
                                }
                                Some(pointer::Gesture::Released | pointer::Gesture::Cancelled) => {
                                    state.press = None;
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // `Enter`, `Space` and the arrows open the content of a
                // focused trigger, like the trigger of a Radix menu or
                // select (a button opens a popover with the first two).
                if !was_captured
                    && !state.open
                    && let Some(
                        named @ (key::Named::Enter
                        | key::Named::Space
                        | key::Named::ArrowDown
                        | key::Named::ArrowUp),
                    ) = plain_key(event)
                    && anchor_focused()
                {
                    open_click(state, shell, self.on_open_change.as_deref());
                    opened_with_keyboard(state, self.menu_keys, named);
                    shell.capture_event();
                }
            }
            Trigger::ContextMenu => {
                if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) = event
                    && let Some(position) = cursor.position_over(bounds)
                {
                    open_at(state, position, shell, self.on_open_change.as_deref());
                    shell.capture_event();
                }

                // A long press is the right click of touch screens.
                if let Event::Touch(touch::Event::FingerPressed { .. }) = event
                    && !was_captured
                {
                    state.long_press = pointer::LongPress::start(event, |p| bounds.contains(p));

                    if let Some(hold) = state.long_press {
                        shell.request_redraw_at(hold.deadline);
                    }
                }

                if let Some(hold) = long_pressed {
                    open_at(state, hold.origin, shell, self.on_open_change.as_deref());
                }

                // `Shift+F10` and the context menu key open the menu at the
                // anchor, like the `contextmenu` event of browsers.
                if !was_captured
                    && let Event::Keyboard(keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(named),
                        modifiers,
                        ..
                    }) = event
                    && (*named == key::Named::ContextMenu
                        || (*named == key::Named::F10 && modifiers.shift()))
                    && anchor_focused()
                {
                    open_at(
                        state,
                        bounds.position(),
                        shell,
                        self.on_open_change.as_deref(),
                    );
                    opened_with_keyboard(state, true, key::Named::ArrowDown);
                    shell.capture_event();
                }
            }
            Trigger::Hover {
                open_delay,
                close_delay,
            } => {
                let now = event_time(event);
                // Touch screens have no hover: the cursor stays where a
                // finger lifted, over nothing.
                let touch = pointer::is_touch();
                let is_open = state.is_open(&self.trigger);

                match self.touch_open {
                    TouchOpen::Tap => {
                        if let Event::Touch(touch::Event::FingerPressed { .. }) = event {
                            state.press = (!was_captured)
                                .then(|| {
                                    pointer::Press::start(event, cursor, |p| bounds.contains(p))
                                })
                                .flatten();
                        } else if let Some(press) = &mut state.press {
                            match press.update(event, cursor) {
                                Some(pointer::Gesture::Tapped(position)) => {
                                    state.press = None;

                                    if bounds.contains(position) {
                                        if is_open {
                                            state.close_hover(shell);
                                        } else {
                                            state.pinned = true;
                                            state.pinned_by_keyboard = false;
                                        }
                                    }
                                }
                                Some(pointer::Gesture::Released | pointer::Gesture::Cancelled) => {
                                    state.press = None;
                                }
                                _ => {}
                            }
                        }

                        // The arrow towards the content (`ArrowRight` for a
                        // submenu), `Enter` and `Space` open it.
                        let towards = match self.side {
                            Side::Top => key::Named::ArrowUp,
                            Side::Right => key::Named::ArrowRight,
                            Side::Bottom => key::Named::ArrowDown,
                            Side::Left => key::Named::ArrowLeft,
                        };

                        if !was_captured
                            && !is_open
                            && let Some(named) = plain_key(event)
                            && (named == key::Named::Enter
                                || named == key::Named::Space
                                || named == towards)
                            && (state.highlighted.get() || anchor_focused())
                        {
                            state.pinned = true;
                            state.pinned_by_keyboard = true;
                            state.suppressed = false;
                            opened_with_keyboard(state, self.menu_keys, key::Named::ArrowDown);
                            shell.capture_event();
                        } else if state.pinned_by_keyboard
                            && let Event::Keyboard(_) = event
                            && !state.highlighted.get()
                            && !anchor_focused()
                        {
                            // The focus left the anchor (to another trigger).
                            state.pinned = false;
                            state.pinned_by_keyboard = false;
                        }
                    }
                    TouchOpen::Hold | TouchOpen::LongPress => {
                        if let Event::Touch(touch::Event::FingerPressed { .. }) = event {
                            // The next tap closes a tooltip shown by a long
                            // press.
                            state.linger = None;

                            if !was_captured {
                                state.long_press =
                                    pointer::LongPress::start(event, |p| bounds.contains(p));

                                if let Some(hold) = state.long_press {
                                    shell.request_redraw_at(hold.deadline);
                                }
                            }
                        }

                        if long_pressed.is_some() {
                            if self.touch_open == TouchOpen::Hold {
                                state.held = true;
                            } else {
                                state.pinned = true;
                            }
                        }

                        // The finger lifted (its lift may not reach the
                        // anchor): the tooltip stays a little longer.
                        if state.held && !pointer::is_touching() {
                            state.held = false;
                            state.linger = Some(now + TOUCH_LINGER);
                        }

                        // Like Radix, tooltips and hover cards open when
                        // their trigger receives the keyboard focus, not
                        // when it is clicked.
                        if let Event::Keyboard(_)
                        | Event::Window(window::Event::RedrawRequested(_))
                        | Event::Mouse(mouse::Event::ButtonPressed(_))
                        | Event::Touch(touch::Event::FingerPressed { .. }) = event
                        {
                            let mut visible = false;

                            focus::for_each_control(
                                anchor,
                                anchor_tree,
                                layout,
                                renderer,
                                |_, focus| {
                                    visible |= focus.is_visible();
                                },
                            );

                            state.focused = visible;
                        }
                    }
                }

                if let Event::Mouse(_)
                | Event::Touch(_)
                | Event::Keyboard(_)
                | Event::Window(window::Event::RedrawRequested(_)) = event
                {
                    let over = !touch && cursor.is_over(bounds);

                    // The pointer leaves the anchor towards the open
                    // content: it stays open while the pointer moves
                    // through the triangle between them.
                    if state.anchor_over
                        && !over
                        && is_open
                        && !touch
                        && let Some(position) = cursor.position()
                        && let Some(content) = state.content_bounds
                    {
                        state.grace = Grace::new(position, bounds, content, now);

                        if let Some(grace) = state.grace {
                            shell.request_redraw_at(grace.deadline);
                        }
                    }

                    let in_grace = !over
                        && !touch
                        && state.grace.is_some_and(|grace| {
                            now < grace.deadline
                                && cursor
                                    .position()
                                    .is_some_and(|position| grace.contains(position))
                        });

                    if in_grace {
                        GRACE.set(state.grace);
                    } else if let Some(grace) = state.grace.take()
                        && GRACE.get() == Some(grace)
                    {
                        GRACE.set(None);
                    }

                    // Another anchor under the grace area (the trigger of a
                    // sibling submenu) does not open its content.
                    let blocked =
                        !is_open && !touch && cursor.position().is_some_and(is_in_pointer_grace);

                    // Content opened with the keyboard closes like hovered
                    // content once the mouse moves elsewhere.
                    if state.pinned_by_keyboard
                        && let Event::Mouse(mouse::Event::CursorMoved { .. }) = event
                        && !over
                        && !in_grace
                        && !state.content_hovered
                    {
                        state.pinned = false;
                        state.pinned_by_keyboard = false;
                    }

                    state.anchor_over = over;
                    state.anchor_hovered = (over && !blocked) || in_grace;
                    state.tick(now, *open_delay, *close_delay, shell);
                }
            }
            Trigger::Input => {
                let toggle = match event {
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                    | Event::Touch(touch::Event::FingerPressed { .. }) => {
                        cursor.position_over(bounds).is_some_and(|position| {
                            let mut toggles = FindToggles(Vec::new());

                            self.anchor.as_widget_mut().operate(
                                &mut tree.children[0],
                                layout,
                                renderer,
                                &mut toggles,
                            );

                            toggles.0.iter().any(|toggle| toggle.contains(position))
                        })
                    }
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key,
                        text,
                        modifiers,
                        ..
                    }) if !state.open => {
                        let arrow = matches!(
                            key,
                            keyboard::Key::Named(key::Named::ArrowDown | key::Named::ArrowUp)
                        );
                        let typed = !modifiers.command()
                            && (text
                                .as_ref()
                                .is_some_and(|text| !text.chars().any(char::is_control))
                                || matches!(
                                    key,
                                    keyboard::Key::Named(
                                        key::Named::Backspace | key::Named::Delete
                                    )
                                ));

                        let opens = (arrow || typed) && {
                            let mut focused = FindFocused(false);

                            self.anchor.as_widget_mut().operate(
                                &mut tree.children[0],
                                layout,
                                renderer,
                                &mut focused,
                            );

                            focused.0
                        };

                        if opens && arrow {
                            state.replay = Some(event.clone());
                            shell.capture_event();
                        }

                        opens
                    }
                    _ => false,
                };

                if toggle {
                    // Like a click on the trigger of a Base UI combobox, which
                    // focuses its input.
                    if !state.open {
                        self.anchor.as_widget_mut().operate(
                            &mut tree.children[0],
                            layout,
                            renderer,
                            &mut FocusFirst(false),
                        );
                    }

                    state.open = !state.open;
                    state.epoch = select_epoch();
                    shell.invalidate_layout();
                    shell.request_redraw();

                    if let Some(f) = &self.on_open_change {
                        shell.publish(f(state.open));
                    }
                }
            }
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
        let focus_ring = self
            .focus_ring
            .as_ref()
            .filter(|_| state.focus_visible && !state.is_open(&self.trigger))
            .map(|f| f(theme));

        // `focus-visible:ring-3`
        if let Some(ring) = focus_ring.and_then(|style| style.ring) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds().expand(ring.width),
                    border: ring.border(),
                    ..renderer::Quad::default()
                },
                Background::Color(Color::TRANSPARENT),
            );
        }

        if let Some(highlight) = &self.open_highlight
            && state.is_open(&self.trigger)
        {
            let (background, border) = highlight(theme);

            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border,
                    ..renderer::Quad::default()
                },
                background,
            );
        }

        // The anchor may tell whether it is highlighted while drawn.
        let outer = HIGHLIGHTED.replace(Some(false));

        self.anchor.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );

        if let Some(highlighted) = HIGHLIGHTED.replace(outer) {
            state.highlighted.set(highlighted);
        }

        // `focus-visible:border-ring`
        if let Some(style) = focus_ring
            && style.border.width > 0.0
        {
            renderer.with_layer(*viewport, |renderer| {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: layout.bounds(),
                        border: style.border,
                        ..renderer::Quad::default()
                    },
                    Background::Color(Color::TRANSPARENT),
                );
            });
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
        self.anchor.as_widget().mouse_interaction(
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
        self.overlay_in(tree, layout, renderer, viewport, translation, None)
    }
}

impl<'a, Message, Theme, Renderer> Anchored<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Fade + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn overlay_in<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
        group: Option<GroupLink<'b>>,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let Tree {
            state, children, ..
        } = tree;
        let state = state.downcast_mut::<State>();
        let (anchor_tree, content_tree) = children.split_at_mut(1);

        let anchor = self.anchor.as_widget_mut().overlay(
            &mut anchor_tree[0],
            layout,
            renderer,
            viewport,
            translation,
        );

        let now = Instant::now();
        let open = state.is_open(&self.trigger);

        state.presence.go(
            if open { 1.0 } else { 0.0 },
            // `duration-100`
            if self.animate {
                Duration::from_millis(100)
            } else {
                Duration::ZERO
            },
            // `animate-in` / `animate-out`: `var(--tw-ease, ease)`
            animation::ease,
            now,
        );

        // Closed content stays mounted while it fades out.
        let floating = if open || state.presence.is_animating(now) {
            let anchor_bounds = layout.bounds() + translation;
            let point = match self.trigger {
                Trigger::ContextMenu => state.point.map(|point| point + translation),
                _ => None,
            };

            Some(overlay::Element::new(Box::new(Floating {
                content: &mut self.content,
                tree: &mut content_tree[0],
                state,
                trigger: &self.trigger,
                anchor_bounds,
                point,
                side: self.side,
                align: self.align,
                offset: self.offset,
                align_offset: self.align_offset,
                collision_padding: self.collision_padding,
                outset: self.outset,
                anchor_width: self.anchor_width,
                fallback_offset: self.fallback_offset,
                match_anchor_width: self.match_anchor_width,
                close_on_select: self.close_on_select,
                modal: self.modal,
                on_open_change: self.on_open_change.as_deref(),
                closing: !open,
                resolved_side: self.side,
                group: group.filter(|_| open),
                anchor_tracker: self.anchor_tracker.as_ref(),
                menu_keys: self.menu_keys,
            })))
        } else {
            None
        };

        match (anchor, floating) {
            (None, None) => None,
            (Some(overlay), None) | (None, Some(overlay)) => Some(overlay),
            (Some(anchor), Some(floating)) => {
                Some(overlay::Group::with_children(vec![anchor, floating]).overlay())
            }
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Anchored<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Fade + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(anchored: Anchored<'a, Message, Theme, Renderer>) -> Self {
        Element::new(anchored)
    }
}

/// Creates floating `content` anchored to `anchor`. See [`Anchored`].
pub fn anchored<'a, Message, Theme, Renderer>(
    anchor: impl Into<Element<'a, Message, Theme, Renderer>>,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Anchored<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    Anchored::new(anchor, content)
}

struct Floating<'a, 'b, Message, Theme, Renderer> {
    content: &'b mut Element<'a, Message, Theme, Renderer>,
    tree: &'b mut Tree,
    state: &'b mut State,
    trigger: &'b Trigger<Message>,
    anchor_bounds: Rectangle,
    point: Option<Point>,
    side: Side,
    align: Align,
    offset: f32,
    align_offset: f32,
    collision_padding: f32,
    outset: f32,
    match_anchor_width: bool,
    anchor_width: Option<f32>,
    fallback_offset: Option<f32>,
    close_on_select: bool,
    modal: bool,
    on_open_change: Option<&'b (dyn Fn(bool) -> Message + 'a)>,
    closing: bool,
    resolved_side: Side,
    group: Option<GroupLink<'b>>,
    anchor_tracker: Option<&'b AnchorTracker>,
    menu_keys: bool,
}

/// What the open content of a [`Group`] knows about its siblings.
struct GroupLink<'b> {
    /// The index of the [`Anchored`] in its group.
    index: usize,
    /// The bounds of the anchors of the group.
    anchors: Vec<Rectangle>,
    /// Where to ask the group to open another [`Anchored`], and whether
    /// the keyboard asks.
    request: &'b mut Option<(usize, bool)>,
}

impl<Message, Theme, Renderer> Floating<'_, '_, Message, Theme, Renderer>
where
    Message: Clone,
{
    fn dismiss(&mut self, shell: &mut Shell<'_, Message>) {
        match self.trigger {
            Trigger::Controlled { on_dismiss, .. } => {
                if let Some(message) = on_dismiss {
                    shell.publish(message.clone());
                }
            }
            Trigger::Click | Trigger::ContextMenu | Trigger::Input => {
                if self.state.open {
                    self.state.open = false;
                    shell.invalidate_layout();
                    shell.request_redraw();

                    if let Some(f) = self.on_open_change {
                        shell.publish(f(false));
                    }
                }
            }
            Trigger::Hover { .. } => {
                self.state.hover = Hover::Idle;
                self.state.pinned = false;
                self.state.pinned_by_keyboard = false;
                self.state.held = false;
                self.state.linger = None;
                // Until the pointer leaves, or it would reopen right away.
                self.state.suppressed = true;
                // The content is gone and stops tracking the pointer: left
                // hovered, it would keep the anchor suppressed for good
                // (an item selected in a submenu closing the whole menu).
                self.state.content_hovered = false;
                shell.invalidate_layout();
                shell.request_redraw();
            }
        }
    }
}

/// Places content of the given size next to the anchor, without keeping it
/// inside the window.
fn place(
    anchor: Rectangle,
    size: Size,
    side: Side,
    align: Align,
    offset: f32,
    align_offset: f32,
) -> Point {
    let cross_x = || match align {
        Align::Start => anchor.x + align_offset,
        Align::Center => anchor.center_x() - size.width / 2.0,
        Align::End => anchor.x + anchor.width - size.width - align_offset,
    };
    let cross_y = || match align {
        Align::Start => anchor.y + align_offset,
        Align::Center => anchor.center_y() - size.height / 2.0,
        Align::End => anchor.y + anchor.height - size.height - align_offset,
    };

    match side {
        Side::Bottom => Point::new(cross_x(), anchor.y + anchor.height + offset),
        Side::Top => Point::new(cross_x(), anchor.y - size.height - offset),
        Side::Right => Point::new(anchor.x + anchor.width + offset, cross_y()),
        Side::Left => Point::new(anchor.x - size.width - offset, cross_y()),
    }
}

/// Computes the position of floating content of the given size.
///
/// Exposed for components that want to position things themselves.
pub fn position(
    anchor: Rectangle,
    size: Size,
    viewport: Size,
    side: Side,
    align: Align,
    offset: f32,
    align_offset: f32,
    padding: f32,
) -> (Point, Side) {
    let place = |side: Side| place(anchor, size, side, align, offset, align_offset);

    let overflow = |side: Side, point: Point| match side {
        Side::Bottom => point.y + size.height - (viewport.height - padding),
        Side::Top => padding - point.y,
        Side::Right => point.x + size.width - (viewport.width - padding),
        Side::Left => padding - point.x,
    };

    let mut side = side;
    let mut point = place(side);

    let current = overflow(side, point);

    if current > 0.0 {
        let opposite = side.opposite();
        let flipped = place(opposite);

        if overflow(opposite, flipped) < current {
            side = opposite;
            point = flipped;
        }
    }

    let clamp = |value: f32, length: f32, max: f32| {
        let upper = max - padding - length;

        if upper < padding {
            padding.min(max - length).max(0.0)
        } else {
            value.clamp(padding, upper)
        }
    };

    // Like Base UI (`shift()` without `crossAxis`), the content only
    // shifts along its alignment axis: on its side, it stays next to the
    // anchor and may overflow the window when neither side has room.
    let point = match side {
        Side::Top | Side::Bottom => Point::new(clamp(point.x, size.width, viewport.width), point.y),
        Side::Left | Side::Right => {
            Point::new(point.x, clamp(point.y, size.height, viewport.height))
        }
    };

    (point, side)
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Floating<'_, '_, Message, Theme, Renderer>
where
    Theme: Fade,
    Message: Clone,
    Renderer: iced::advanced::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let padding = self.collision_padding;
        let outset = self.outset;
        let max = Size::new(
            (bounds.width - padding * 2.0 + outset * 2.0).max(0.0),
            (bounds.height - padding * 2.0 + outset * 2.0).max(0.0),
        );

        let (min_width, max) = match self.anchor_width {
            Some(min) if self.point.is_none() => {
                let width = (self.anchor_bounds.width.max(min) + outset * 2.0).min(max.width);

                (width, Size::new(width, max.height))
            }
            _ if self.match_anchor_width && self.point.is_none() => (
                (self.anchor_bounds.width + outset * 2.0).min(max.width),
                max,
            ),
            _ => (0.0, max),
        };

        let node = self.content.as_widget_mut().layout(
            self.tree,
            renderer,
            &layout::Limits::new(Size::new(min_width, 0.0), max),
        );

        let anchor = match self.point {
            Some(point) => Rectangle::new(point, Size::ZERO),
            None => self.anchor_bounds,
        };

        if let Some(tracker) = self.anchor_tracker {
            tracker.0.set(Some(anchor));
        }

        let size = node.size();
        let size = Size::new(
            (size.width - outset * 2.0).max(0.0),
            (size.height - outset * 2.0).max(0.0),
        );

        let offset = match self.fallback_offset {
            Some(fallback) => {
                let point = place(
                    anchor,
                    size,
                    self.side,
                    self.align,
                    self.offset,
                    self.align_offset,
                );
                let fits = point.x >= padding
                    && point.y >= padding
                    && point.x + size.width <= bounds.width - padding
                    && point.y + size.height <= bounds.height - padding;

                if fits { self.offset } else { fallback }
            }
            None => self.offset,
        };

        let (position, side) = position(
            anchor,
            size,
            bounds,
            self.side,
            self.align,
            offset,
            self.align_offset,
            padding,
        );

        self.resolved_side = side;

        node.move_to(position - Vector::new(outset, outset))
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
        let progress = self.state.presence.value(Instant::now());

        // `fade-in-0 zoom-in-95 slide-in-from-{side}-2`
        let distance = 8.0 * (1.0 - progress);
        let offset = match self.resolved_side {
            Side::Bottom => Vector::new(0.0, -distance),
            Side::Top => Vector::new(0.0, distance),
            Side::Right => Vector::new(-distance, 0.0),
            Side::Left => Vector::new(distance, 0.0),
        };

        animation::draw(
            self.content,
            self.tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            &bounds,
            Appearance {
                opacity: progress,
                scale: 0.95 + 0.05 * progress,
                offset,
            },
        );
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(self.tree, layout, renderer, operation);
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
        pointer::observe(event);

        let bounds = layout.bounds();
        let was_captured = shell.is_event_captured();

        if let Event::Window(window::Event::RedrawRequested(now)) = event
            && self.state.presence.is_animating(*now)
        {
            shell.request_redraw();
        }

        // Content fading out is only a picture of what it was.
        if self.closing {
            return;
        }

        if self.close_on_select
            && !matches!(self.trigger, Trigger::Controlled { .. })
            && self.state.epoch != select_epoch()
        {
            self.dismiss(shell);
            return;
        }

        // The key that opened the content (`ArrowDown` in a combobox) acts
        // on it too.
        if let Some(replay) = self.state.replay.take() {
            let mut messages = Vec::new();
            let mut inner = Shell::new(&mut messages);

            self.content.as_widget_mut().update(
                self.tree, &replay, layout, cursor, renderer, clipboard, &mut inner, &bounds,
            );

            // Not merged: the replayed event must not capture this one.
            let invalid = inner.is_layout_invalid();

            for message in messages {
                shell.publish(message);
            }

            if invalid {
                shell.invalidate_layout();
            }

            shell.request_redraw();
        }

        // Content opened with the keyboard focuses its first control, like
        // the `autoFocus` of a Radix popover.
        if std::mem::take(&mut self.state.focus_content) {
            let content = self.content.as_widget_mut();
            let tree = &mut *self.tree;

            let _ =
                tab::focus_first(|operation| content.operate(tree, layout, renderer, operation));
            shell.request_redraw();
        }

        // Another trigger of the group is hovered, or pressed (a finger
        // hovers nothing): open its menu instead.
        if let Some(group) = &mut self.group
            && let Some(position) = match event {
                Event::Mouse(
                    mouse::Event::CursorMoved { .. }
                    | mouse::Event::ButtonPressed(mouse::Button::Left),
                ) => cursor.position(),
                Event::Touch(touch::Event::FingerPressed { position, .. }) => Some(*position),
                _ => None,
            }
            && !bounds.contains(position)
            && let Some(other) = group
                .anchors
                .iter()
                .position(|anchor| anchor.contains(position))
            && other != group.index
        {
            *group.request = Some((other, false));
            self.dismiss(shell);
            shell.capture_event();
            return;
        }

        self.content.as_widget_mut().update(
            self.tree, event, layout, cursor, renderer, clipboard, shell, &bounds,
        );

        let captured_by_content = !was_captured && shell.is_event_captured();

        // `ArrowLeft` and `ArrowRight` move to the previous and next menus of
        // the group.
        if let Some(group) = &mut self.group
            && !captured_by_content
            && let Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(named @ (key::Named::ArrowLeft | key::Named::ArrowRight)),
                ..
            }) = event
            && group.anchors.len() > 1
        {
            let count = group.anchors.len();

            let other = if *named == key::Named::ArrowRight {
                (group.index + 1) % count
            } else {
                (group.index + count - 1) % count
            };

            *group.request = Some((other, true));
            self.dismiss(shell);
            shell.capture_event();
            return;
        }

        if let Trigger::Hover {
            open_delay,
            close_delay,
        } = *self.trigger
            && let Event::Mouse(_)
            | Event::Touch(_)
            | Event::Keyboard(_)
            | Event::Window(window::Event::RedrawRequested(_)) = event
        {
            // A finger on the content does not hover it: what opened it with
            // a tap or a long press keeps it open.
            self.state.content_hovered = !pointer::is_touch() && cursor.is_over(bounds);
            self.state.content_bounds = Some(bounds);
            self.state
                .tick(event_time(event), open_delay, close_delay, shell);
        }

        if !captured_by_content && let Some(backwards) = tab::tab_pressed(event) {
            if self.menu_keys {
                // Like Radix, `Tab` closes a menu, and the focus moves on
                // from its trigger.
                self.dismiss(shell);
            } else if !matches!(self.trigger, Trigger::Input) {
                let content = self.content.as_widget_mut();
                let tree = &mut *self.tree;
                let mut operate = |operation: &mut dyn Operation| {
                    content.operate(tree, layout, renderer, operation)
                };

                // The focus cycles through the controls of open content (a
                // popover), instead of going on to the page behind it.
                // Hover content keeps it only once it has it (a navigation
                // menu opened with the keyboard).
                if matches!(self.trigger, Trigger::Click | Trigger::Controlled { .. })
                    || tab::has_focus(&mut operate)
                {
                    let _ = tab::cycle(&mut operate, backwards);
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
        }

        // `ArrowLeft` closes a submenu, and the highlight stays on its
        // trigger.
        if self.menu_keys
            && !captured_by_content
            && matches!(self.trigger, Trigger::Hover { .. })
            && self.side == Side::Right
            && plain_key(event) == Some(key::Named::ArrowLeft)
        {
            self.dismiss(shell);
            shell.capture_event();
            return;
        }

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Escape),
                ..
            }) => {
                self.dismiss(shell);
                self.state.focus_visible = true;
                shell.capture_event();

                // The focus returns to the anchor, which kept it.
                if !self.menu_keys {
                    self.content
                        .as_widget_mut()
                        .operate(self.tree, layout, renderer, &mut Unfocus);
                }
            }
            // `Alt+ArrowUp` closes the list of a combobox (APG), the focus
            // staying in its input.
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::ArrowUp),
                modifiers,
                ..
            }) if modifiers.alt() && matches!(self.trigger, Trigger::Input) => {
                self.dismiss(shell);
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonPressed(_))
            | Event::Touch(touch::Event::FingerPressed { .. })
                if !captured_by_content && !cursor.is_over(bounds) =>
            {
                let over_anchor = self.point.is_none() && cursor.is_over(self.anchor_bounds);

                if matches!(self.trigger, Trigger::Input) {
                    // The anchor handles its own presses (its toggle).
                    if !over_anchor {
                        self.dismiss(shell);
                    }
                } else if matches!(self.trigger, Trigger::Hover { .. }) {
                    if !over_anchor {
                        self.dismiss(shell);
                    }
                } else {
                    self.dismiss(shell);

                    if self.modal || over_anchor {
                        shell.capture_event();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
                if captured_by_content && self.close_on_select =>
            {
                bump_select_epoch();
                self.state.epoch = select_epoch();
                self.dismiss(shell);
            }
            // An item activated with the keyboard.
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Enter),
                ..
            }) if captured_by_content && self.close_on_select => {
                bump_select_epoch();
                self.state.epoch = select_epoch();
                self.dismiss(shell);
                self.state.focus_visible = true;
            }
            Event::Mouse(_) | Event::Touch(_) if self.modal => {
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
        if self.closing {
            return mouse::Interaction::None;
        }

        let bounds = layout.bounds();
        let interaction = self
            .content
            .as_widget()
            .mouse_interaction(self.tree, layout, cursor, &bounds, renderer);

        if self.modal || cursor.is_over(bounds) {
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

        self.content
            .as_widget_mut()
            .overlay(self.tree, layout, renderer, &bounds, Vector::ZERO)
    }
}

/// A row of [`Anchored`] menus that behave as one, like a menubar: while
/// one of them is open, hovering another trigger opens its menu instead,
/// and <kbd>←</kbd> / <kbd>→</kbd> move to the previous or next menu.
///
/// Menus opened this way do not call their [`Anchored::on_open_change`].
pub struct Group<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    children: Vec<Anchored<'a, Message, Theme, Renderer>>,
    spacing: f32,
}

/// Creates a [`Group`] of [`Anchored`] menus.
pub fn group<'a, Message, Theme, Renderer>(
    children: impl IntoIterator<Item = Anchored<'a, Message, Theme, Renderer>>,
) -> Group<'a, Message, Theme, Renderer> {
    Group {
        children: children.into_iter().collect(),
        spacing: 0.0,
    }
}

impl<Message, Theme, Renderer> Group<'_, Message, Theme, Renderer> {
    /// Sets the horizontal space between the anchors.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

#[derive(Debug, Default)]
struct GroupState {
    /// The menu to open, asked for by the open one, and whether the
    /// keyboard asked.
    request: Option<(usize, bool)>,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Group<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Fade + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<GroupState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(GroupState::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.children
            .iter()
            .map(|child| Tree::new(child as &dyn Widget<Message, Theme, Renderer>))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(
            &self
                .children
                .iter()
                .map(|child| child as &dyn Widget<Message, Theme, Renderer>)
                .collect::<Vec<_>>(),
        );
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let loose = layout::Limits::new(Size::ZERO, limits.max());
        let mut nodes: Vec<layout::Node> = self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .map(|(child, tree)| child.layout(tree, renderer, &loose))
            .collect();

        let height = nodes
            .iter()
            .map(|node| node.size().height)
            .fold(0.0, f32::max);
        let mut x = 0.0;

        for node in &mut nodes {
            let size = node.size();
            node.move_to_mut(Point::new(x, (height - size.height) / 2.0));
            x += size.width + self.spacing;
        }

        let width = if nodes.is_empty() {
            0.0
        } else {
            x - self.spacing
        };

        layout::Node::with_children(
            limits.resolve(Length::Shrink, Length::Shrink, Size::new(width, height)),
            nodes,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            for ((child, tree), layout) in self
                .children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
            {
                child.operate(tree, layout, renderer, operation);
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
        for ((child, tree), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
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
        for ((child, tree), layout) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            child.draw(tree, renderer, theme, style, layout, cursor, viewport);
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
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), layout)| {
                child.mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let Tree {
            state, children, ..
        } = tree;
        let group = state.downcast_mut::<GroupState>();

        // Opens the menu the previously open one asked for.
        if let Some((index, keyboard)) = group.request.take()
            && let Some(tree) = children.get_mut(index)
        {
            let state = tree.state.downcast_mut::<State>();
            state.open = true;
            state.epoch = select_epoch();

            // Like Radix, a menu reached with the keyboard highlights its
            // first item, and its trigger takes the focus (to return it on
            // `Escape`).
            if keyboard {
                state.replay = Some(key_event(key::Named::Home));

                for (i, ((child, tree), layout)) in self
                    .children
                    .iter_mut()
                    .zip(children.iter_mut())
                    .zip(layout.children())
                    .enumerate()
                {
                    child.anchor.as_widget_mut().operate(
                        &mut tree.children[0],
                        layout,
                        renderer,
                        &mut focus::Controls {
                            index: 0,
                            f: |_, focus: &mut focus::Focus| {
                                if i == index {
                                    focus.focus_with_keyboard();
                                } else {
                                    focus.blur();
                                }
                            },
                        },
                    );
                }
            }
        }

        let anchors: Vec<Rectangle> = layout
            .children()
            .map(|layout| layout.bounds() + translation)
            .collect();
        let mut request = Some(&mut group.request);

        let overlays: Vec<_> = self
            .children
            .iter_mut()
            .zip(children.iter_mut())
            .zip(layout.children())
            .enumerate()
            .filter_map(|(index, ((child, tree), layout))| {
                let link = if tree.state.downcast_ref::<State>().is_open(&child.trigger) {
                    request.take().map(|request| GroupLink {
                        index,
                        anchors: anchors.clone(),
                        request,
                    })
                } else {
                    None
                };

                child.overlay_in(tree, layout, renderer, viewport, translation, link)
            })
            .collect();

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

impl<'a, Message, Theme, Renderer> From<Group<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Fade + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(group: Group<'a, Message, Theme, Renderer>) -> Self {
        Element::new(group)
    }
}

/// The part of the anchor of a [`combobox`](Anchored::combobox) that opens
/// and closes the content when pressed, e.g. a chevron button.
pub struct Toggle<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
}

/// Creates a [`Toggle`].
pub fn toggle<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Toggle<'a, Message, Theme, Renderer> {
    Toggle {
        content: content.into(),
    }
}

/// The state of a [`Toggle`], which [`Anchored`] looks for.
#[derive(Debug, Default)]
struct ToggleMarker;

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Toggle<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ToggleMarker>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(ToggleMarker)
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
        operation.custom(
            None,
            layout.bounds(),
            tree.state.downcast_mut::<ToggleMarker>(),
        );
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

impl<'a, Message, Theme, Renderer> From<Toggle<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(toggle: Toggle<'a, Message, Theme, Renderer>) -> Self {
        Element::new(toggle)
    }
}

/// Collects the bounds of the [`Toggle`]s of a widget tree.
struct FindToggles(Vec<Rectangle>);

impl Operation for FindToggles {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn custom(
        &mut self,
        _id: Option<&widget::Id>,
        bounds: Rectangle,
        state: &mut dyn std::any::Any,
    ) {
        if state.is::<ToggleMarker>() {
            self.0.push(bounds);
        }
    }
}

/// Focuses the first focusable widget of a widget tree.
struct FocusFirst(bool);

impl Operation for FocusFirst {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(
        &mut self,
        _id: Option<&widget::Id>,
        _bounds: Rectangle,
        state: &mut dyn widget::operation::Focusable,
    ) {
        if !self.0 {
            state.focus();
            self.0 = true;
        }
    }
}

/// Takes the keyboard focus away from a widget tree.
struct Unfocus;

impl Operation for Unfocus {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(
        &mut self,
        _id: Option<&widget::Id>,
        _bounds: Rectangle,
        state: &mut dyn widget::operation::Focusable,
    ) {
        if state.is_focused() {
            state.unfocus();
        }
    }
}

/// Finds out whether a widget tree holds the keyboard focus.
struct FindFocused(bool);

impl Operation for FindFocused {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(
        &mut self,
        _id: Option<&widget::Id>,
        _bounds: Rectangle,
        state: &mut dyn widget::operation::Focusable,
    ) {
        self.0 |= state.is_focused();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORT: Size = Size::new(800.0, 600.0);

    #[test]
    fn places_below_and_centered() {
        let anchor = Rectangle::new(Point::new(100.0, 100.0), Size::new(80.0, 36.0));
        let (point, side) = position(
            anchor,
            Size::new(200.0, 100.0),
            VIEWPORT,
            Side::Bottom,
            Align::Center,
            4.0,
            0.0,
            8.0,
        );

        assert_eq!(side, Side::Bottom);
        assert_eq!(point, Point::new(40.0, 140.0));
    }

    #[test]
    fn flips_when_overflowing() {
        let anchor = Rectangle::new(Point::new(100.0, 550.0), Size::new(80.0, 36.0));
        let (point, side) = position(
            anchor,
            Size::new(200.0, 100.0),
            VIEWPORT,
            Side::Bottom,
            Align::Start,
            4.0,
            0.0,
            8.0,
        );

        assert_eq!(side, Side::Top);
        assert_eq!(point, Point::new(100.0, 446.0));
    }

    #[test]
    fn shifts_inside_viewport() {
        let anchor = Rectangle::new(Point::new(780.0, 100.0), Size::new(10.0, 10.0));
        let (point, _) = position(
            anchor,
            Size::new(200.0, 100.0),
            VIEWPORT,
            Side::Bottom,
            Align::Start,
            4.0,
            0.0,
            8.0,
        );

        assert_eq!(point.x, 592.0);
    }

    #[test]
    fn grace_area_leads_to_the_submenu() {
        let now = Instant::now();
        // A submenu on the right of a trigger left at (100, 50).
        let content = Rectangle::new(Point::new(200.0, 0.0), Size::new(150.0, 200.0));
        let anchor = Rectangle::new(Point::new(0.0, 30.0), Size::new(100.0, 20.0));
        let grace = Grace::new(Point::new(100.0, 50.0), anchor, content, now).expect("grace area");

        // Diagonally down to the submenu, over the items below the trigger.
        assert!(grace.contains(Point::new(150.0, 100.0)));
        assert!(grace.contains(Point::new(199.0, 190.0)));
        // Away from it.
        assert!(!grace.contains(Point::new(150.0, 180.0)));
        assert!(!grace.contains(Point::new(80.0, 50.0)));

        // Content overlapping the exit point has no grace area.
        assert!(Grace::new(Point::new(250.0, 50.0), anchor, content, now).is_none());
    }

    #[test]
    fn no_grace_area_when_leaving_away_from_the_content() {
        let now = Instant::now();
        // A tooltip on the left of its trigger, and the pointer jumping to
        // the next trigger on the right.
        let anchor = Rectangle::new(Point::new(144.0, 80.0), Size::new(48.0, 32.0));
        let content = Rectangle::new(Point::new(42.0, 82.0), Size::new(98.0, 28.0));

        assert!(Grace::new(Point::new(222.0, 96.0), anchor, content, now).is_none());
        // Leaving through its left edge still has one.
        assert!(Grace::new(Point::new(143.0, 96.0), anchor, content, now).is_some());
    }
}
