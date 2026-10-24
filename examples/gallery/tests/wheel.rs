//! Browser-like wheel scrolling (`shadcn_iced::primitive::wheel`).
use iced::time::{Duration, Instant};
use iced::widget::{Column, scrollable, text};
use iced::{Element, Event, mouse, window};
use iced_test::simulator::Simulator;

#[derive(Debug, Clone)]
enum Message {
    Scrolled(f32, f32),
}

fn view(horizontal: bool, notify: bool) -> Element<'static, Message> {
    shadcn_iced::primitive::wheel(area(horizontal, notify))
        .smooth(true)
        .into()
}

fn area(horizontal: bool, notify: bool) -> Element<'static, Message> {
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

    area.into()
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
    scroll_in(view(horizontal, notify), shift, delta, frames)
}

/// Like [`scroll`], in any view.
fn scroll_in(
    view: Element<'static, Message>,
    shift: bool,
    delta: mouse::ScrollDelta,
    frames: u32,
) -> (Vec<(f32, f32)>, iced::event::Status) {
    let mut ui = Simulator::with_size(iced::Settings::default(), (400.0, 300.0), view);
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

#[test]
fn smooth_scrolling_is_opt_in() {
    shadcn_iced::animation::set_enabled(true);

    let notch = mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 };
    let plain = || shadcn_iced::primitive::wheel(area(false, true)).into();

    // Off by default: a notch scrolls 100px at once.
    let (offsets, _) = scroll_in(plain(), false, notch, 30);
    assert_eq!(offsets.len(), 1, "{offsets:?}");
    assert!((offsets[0].1 - 100.0).abs() < 1.0, "{offsets:?}");

    // On for every wheel.
    shadcn_iced::primitive::wheel::set_smooth(true);
    let (offsets, _) = scroll_in(plain(), false, notch, 30);
    shadcn_iced::primitive::wheel::set_smooth(false);

    assert!(offsets.len() > 3, "not spread over frames: {offsets:?}");
    assert!(
        (offsets.last().unwrap().1 - 100.0).abs() < 1.0,
        "{offsets:?}"
    );
}

#[test]
fn a_notch_eases_in_and_out_within_200ms() {
    shadcn_iced::animation::set_enabled(true);

    let (offsets, _) = scroll(
        false,
        true,
        false,
        mouse::ScrollDelta::Pixels { x: 0.0, y: -100.0 },
        30,
    );

    // The first viewport, then a frame every 16ms.
    let steps: Vec<f32> = offsets[1..].windows(2).map(|w| w[1].1 - w[0].1).collect();
    let fastest = steps
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .unwrap()
        .0;

    assert!(steps.len() <= 13, "longer than 200ms: {offsets:?}");
    assert!(
        fastest > 0 && fastest < steps.len() - 1,
        "no ease-in-out: {steps:?}"
    );
}

#[test]
fn notches_during_a_scroll_add_up_without_stopping() {
    shadcn_iced::animation::set_enabled(true);

    let mut ui = Simulator::with_size(iced::Settings::default(), (400.0, 300.0), view(false, true));
    ui.point_at((150.0, 100.0));

    let notch = || {
        Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -100.0 },
        })
    };
    let start = Instant::now();
    let frames = |from: u64, count: u64| {
        (from..from + count).map(move |frame| {
            Event::Window(window::Event::RedrawRequested(
                start + Duration::from_millis(16 * frame),
            ))
        })
    };

    let _ = ui.simulate([notch()]);
    let _ = ui.simulate(frames(1, 4));
    let _ = ui.simulate([notch()]);
    let _ = ui.simulate(frames(5, 4));
    let _ = ui.simulate([notch()]);
    let _ = ui.simulate(frames(9, 30));

    let offsets: Vec<f32> = ui
        .into_messages()
        .map(|Message::Scrolled(_, y)| y)
        .collect();
    let steps: Vec<f32> = offsets.windows(2).map(|w| w[1] - w[0]).collect();

    assert!(steps.iter().all(|step| *step >= 0.0), "{offsets:?}");
    assert!((offsets.last().unwrap() - 300.0).abs() < 1.0, "{offsets:?}");
}

#[test]
fn a_smooth_area_in_a_plain_wheel() {
    shadcn_iced::animation::set_enabled(true);

    let view = || {
        shadcn_iced::primitive::wheel(shadcn_iced::primitive::smooth_scroll(area(true, true)))
            .into()
    };

    // Smooth, and `Shift` swaps the axes once.
    let (offsets, status) = scroll_in(
        view(),
        true,
        mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
        30,
    );

    assert_eq!(status, iced::event::Status::Captured);
    assert!(offsets.len() > 3, "not spread over frames: {offsets:?}");
    assert!(
        (offsets.last().unwrap().0 - 100.0).abs() < 1.0,
        "{offsets:?}"
    );
}
