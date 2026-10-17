//! Smooth wheel scrolling (`shadcn_iced::primitive::wheel`).
use iced::time::{Duration, Instant};
use iced::widget::{Column, scrollable, text};
use iced::{Element, Event, mouse, window};
use iced_test::simulator::Simulator;

#[derive(Debug, Clone)]
enum Message {
    Scrolled(f32, f32),
}

fn view(horizontal: bool, notify: bool) -> Element<'static, Message> {
    let items = (0..60).map(|i| text(format!("Item {i}")).into());

    let content: Element<'static, Message> = if horizontal {
        iced::widget::Row::with_children(items).spacing(40).into()
    } else {
        Column::with_children(items).into()
    };

    let area = scrollable(content)
        .direction(if horizontal {
            scrollable::Direction::Horizontal(scrollable::Scrollbar::default())
        } else {
            scrollable::Direction::Vertical(scrollable::Scrollbar::default())
        })
        .width(300)
        .height(200);

    // iced reports the first viewport as a scroll: without `on_scroll`, a
    // wheel that scrolls nothing is not captured.
    let area = if notify {
        area.on_scroll(|viewport| {
            let offset = viewport.absolute_offset();
            Message::Scrolled(offset.x, offset.y)
        })
    } else {
        area
    };

    shadcn_iced::primitive::wheel(area).into()
}

/// Scrolls with `delta` at the center of the area, then lets `frames`
/// frames pass, returning the offsets reported and the status of the wheel.
fn scroll(
    horizontal: bool,
    notify: bool,
    shift: bool,
    delta: mouse::ScrollDelta,
    frames: u32,
) -> (Vec<(f32, f32)>, iced::event::Status) {
    let mut ui = Simulator::with_size(
        iced::Settings::default(),
        (400.0, 300.0),
        view(horizontal, notify),
    );
    ui.point_at((150.0, 100.0));

    if shift {
        let _ = ui.simulate([Event::Keyboard(iced::keyboard::Event::ModifiersChanged(
            iced::keyboard::Modifiers::SHIFT,
        ))]);
    }

    let status = ui.simulate([Event::Mouse(mouse::Event::WheelScrolled { delta })]);
    let start = Instant::now();
    let _ = ui.simulate((1..=frames).map(|frame| {
        Event::Window(window::Event::RedrawRequested(
            start + Duration::from_millis(16 * u64::from(frame)),
        ))
    }));

    let offsets = ui
        .into_messages()
        .map(|Message::Scrolled(x, y)| (x, y))
        .collect();

    (offsets, status[0])
}

#[test]
fn a_notch_scrolls_smoothly() {
    shadcn_iced::animation::set_enabled(true);

    let (offsets, status) = scroll(
        false,
        true,
        false,
        mouse::ScrollDelta::Pixels { x: 0.0, y: -100.0 },
        30,
    );

    assert_eq!(status, iced::event::Status::Captured);
    assert!(offsets.len() > 3, "not spread over frames: {offsets:?}");
    assert!(offsets.windows(2).all(|w| w[1].1 >= w[0].1), "{offsets:?}");
    assert!(
        (offsets.last().unwrap().1 - 100.0).abs() < 1.0,
        "{offsets:?}"
    );
}

#[test]
fn shift_scrolls_horizontally() {
    shadcn_iced::animation::set_enabled(true);

    let (offsets, status) = scroll(
        true,
        true,
        true,
        mouse::ScrollDelta::Pixels { x: 0.0, y: -100.0 },
        30,
    );

    assert_eq!(status, iced::event::Status::Captured);
    assert!(
        (offsets.last().unwrap().0 - 100.0).abs() < 1.0,
        "{offsets:?}"
    );
}

#[test]
fn the_end_lets_the_wheel_through() {
    shadcn_iced::animation::set_enabled(true);

    let mut ui = Simulator::with_size(
        iced::Settings::default(),
        (400.0, 300.0),
        view(false, false),
    );
    ui.point_at((150.0, 100.0));

    let up = || {
        Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 0.0, y: 100.0 },
        })
    };
    let start = Instant::now();
    let frames = |from: u64| {
        (from..from + 20).map(move |frame| {
            Event::Window(window::Event::RedrawRequested(
                start + Duration::from_millis(16 * frame),
            ))
        })
    };

    // iced reports the first viewport of a scrollable as a scroll, so the
    // first wheel is captured either way.
    let _ = ui.simulate([up()]);
    let _ = ui.simulate(frames(1));

    // iced also keeps sending the wheel to the scrollable that scrolled
    // last for 1.5s (in real time).
    std::thread::sleep(std::time::Duration::from_millis(1600));

    // Upwards at the top: nothing to scroll.
    let status = ui.simulate([up()]);

    assert_eq!(status[0], iced::event::Status::Ignored);
}
