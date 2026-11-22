use iced::time::{Duration, Instant};
use iced::{Color, Point, Rectangle, Size, Vector};
use proptest::prelude::*;

use shadcn_iced_core::animation::{
    self, Appearance, Transition, css_ease_out, cubic_bezier, ease, ease_in, ease_in_out, ease_out,
    linear,
};
use shadcn_iced_core::highlight::{Highlighter, Language, Token, color, highlight};
use shadcn_iced_core::primitive::anchored::{self, Align, Side};
use shadcn_iced_core::primitive::box_shadow::{self, Layer};
use shadcn_iced_core::primitive::listbox::{Typeahead, next_match};
use shadcn_iced_core::primitive::plot::{
    self, Bars, Categories, Linear, Offset, Pie, Sector, Spread,
};
use shadcn_iced_core::primitive::questionnaire::{Event as QEvent, Question, State as QState};
use shadcn_iced_core::style::{self, Position};
use shadcn_iced_core::theme::{self, Mode};

fn arb_color() -> impl Strategy<Value = Color> {
    (0.0f32..=1.0, 0.0f32..=1.0, 0.0f32..=1.0, 0.0f32..=1.0)
        .prop_map(|(r, g, b, a)| Color::from_rgba(r, g, b, a))
}

fn arb_language() -> impl Strategy<Value = Language> {
    prop_oneof![
        Just(Language::Rust),
        Just(Language::Toml),
        Just(Language::Shell),
        Just(Language::JavaScript),
        Just(Language::Json),
    ]
}

fn arb_side() -> impl Strategy<Value = Side> {
    prop_oneof![
        Just(Side::Top),
        Just(Side::Right),
        Just(Side::Bottom),
        Just(Side::Left),
    ]
}

fn arb_align() -> impl Strategy<Value = Align> {
    prop_oneof![Just(Align::Start), Just(Align::Center), Just(Align::End)]
}

proptest! {
    // -------------------------------------------------------------------------
    // highlight
    // -------------------------------------------------------------------------

    #[test]
    fn highlight_partitions_arbitrary_utf8_without_panicking(
        lang in arb_language(),
        code in "\\PC{0,120}",
    ) {
        let lines = highlight(lang, &code);
        let raw_lines: Vec<&str> = code.split('\n').collect();
        prop_assert_eq!(lines.len(), raw_lines.len());

        for (spans, line) in lines.iter().zip(&raw_lines) {
            let mut cursor = 0;
            let mut prev_token: Option<Token> = None;

            for (range, token) in spans {
                prop_assert!(range.start < range.end, "empty span {:?} in {:?}", range, line);
                prop_assert_eq!(range.start, cursor, "gap/overlap in {:?}", line);
                prop_assert!(line.is_char_boundary(range.start));
                prop_assert!(line.is_char_boundary(range.end));
                prop_assert_ne!(
                    prev_token,
                    Some(*token),
                    "adjacent duplicate token {:?}",
                    token
                );
                cursor = range.end;
                prev_token = Some(*token);
            }

            prop_assert_eq!(cursor, line.len(), "spans did not cover full line {:?}", line);
        }
    }

    #[test]
    fn highlight_incremental_matches_batch(
        lang in arb_language(),
        code in "([a-zA-Z0-9_ '\"`/#*$→🦀{}\\[\\]():=,.\n]{0,80})",
    ) {
        let batch = highlight(lang, &code);
        let mut h = Highlighter::new(lang);
        let incremental: Vec<_> = code.split('\n').map(|l| h.line(l)).collect();
        prop_assert_eq!(batch, incremental);
    }

    // -------------------------------------------------------------------------
    // theme
    // -------------------------------------------------------------------------

    #[test]
    fn theme_mix_endpoints_and_idempotence(a in arb_color(), b in arb_color(), t in 0.0f32..=1.0) {
        let at0 = theme::mix(a, b, 0.0);
        let at1 = theme::mix(a, b, 1.0);
        let same = theme::mix(a, a, t);

        prop_assert!((at0.r - a.r).abs() < 1e-6 && (at0.g - a.g).abs() < 1e-6);
        prop_assert!((at0.b - a.b).abs() < 1e-6 && (at0.a - a.a).abs() < 1e-6);
        prop_assert!((at1.r - b.r).abs() < 1e-6 && (at1.g - b.g).abs() < 1e-6);
        prop_assert!((at1.b - b.b).abs() < 1e-6 && (at1.a - b.a).abs() < 1e-6);
        prop_assert!((same.r - a.r).abs() < 1e-5 && (same.a - a.a).abs() < 1e-5);
    }

    #[test]
    fn theme_over_compositing_invariants(top in arb_color(), bottom in arb_color()) {
        let out = theme::over(top, bottom);
        prop_assert!(out.r.is_finite() && out.g.is_finite() && out.b.is_finite());
        prop_assert!(out.a >= -1e-6 && out.a <= 1.0 + 1e-6);

        // Opaque top completely covers bottom.
        let opaque_top = Color { a: 1.0, ..top };
        let covered = theme::over(opaque_top, bottom);
        prop_assert!((covered.r - top.r).abs() < 1e-5);
        prop_assert!((covered.g - top.g).abs() < 1e-5);
        prop_assert!((covered.b - top.b).abs() < 1e-5);
        prop_assert!((covered.a - 1.0).abs() < 1e-5);

        // Transparent top leaves bottom unchanged (when bottom has positive alpha).
        if bottom.a > 0.0 {
            let transparent = Color { a: 0.0, ..top };
            let unchanged = theme::over(transparent, bottom);
            prop_assert!((unchanged.r - bottom.r).abs() < 1e-5);
            prop_assert!((unchanged.g - bottom.g).abs() < 1e-5);
            prop_assert!((unchanged.b - bottom.b).abs() < 1e-5);
            prop_assert!((unchanged.a - bottom.a).abs() < 1e-5);
        }
    }

    #[test]
    fn theme_alpha_preserves_rgb_and_scales_a(c in arb_color(), factor in 0.0f32..=1.0) {
        let out = theme::alpha(c, factor);
        prop_assert_eq!((out.r, out.g, out.b), (c.r, c.g, c.b));
        prop_assert!((out.a - c.a * factor).abs() < 1e-6);
    }

    // -------------------------------------------------------------------------
    // animation
    // -------------------------------------------------------------------------

    #[test]
    fn animation_easings_are_monotone_and_bounded(t1 in 0.0f32..=1.0, t2 in 0.0f32..=1.0) {
        let (a, b) = if t1 <= t2 { (t1, t2) } else { (t2, t1) };
        for easing in [linear, ease, css_ease_out, ease_out, ease_in, ease_in_out] {
            let va = easing(a);
            let vb = easing(b);
            prop_assert!((0.0..=1.0).contains(&va));
            prop_assert!((0.0..=1.0).contains(&vb));
            prop_assert!(va <= vb + 1e-3, "non-monotone: {va} > {vb} for {a}..={b}");
            prop_assert_eq!(easing(-0.5), 0.0);
            prop_assert_eq!(easing(1.5), 1.0);
        }
    }

    #[test]
    fn animation_cubic_bezier_endpoints(
        x1 in 0.0f32..=1.0,
        y1 in -0.5f32..=1.5,
        x2 in 0.0f32..=1.0,
        y2 in -0.5f32..=1.5,
        t in -1.0f32..=2.0,
    ) {
        let v = cubic_bezier(x1, y1, x2, y2, t);
        prop_assert!(v.is_finite());
        if t <= 0.0 {
            prop_assert_eq!(v, 0.0);
        } else if t >= 1.0 {
            prop_assert_eq!(v, 1.0);
        }
    }

    #[test]
    fn animation_transition_bounds_and_settlement(
        from in -1000.0f32..=1000.0,
        to in -1000.0f32..=1000.0,
        ms in 1u64..=1000,
        elapsed in 0u64..=1500,
    ) {
        animation::set_enabled(true);
        let start = Instant::now();
        let duration = Duration::from_millis(ms);
        let mut tr = Transition::new(from);
        tr.go(to, duration, linear, start);

        let now = start + Duration::from_millis(elapsed);
        let val = tr.value(now);
        let lo = from.min(to);
        let hi = from.max(to);
        prop_assert!(val >= lo - 1e-3 && val <= hi + 1e-3, "{val} outside {lo}..={hi}");

        if elapsed >= ms {
            prop_assert_eq!(val, to);
            prop_assert!(!tr.is_animating(now));
        }
    }

    // -------------------------------------------------------------------------
    // style
    // -------------------------------------------------------------------------

    #[test]
    fn style_position_of_never_panics_and_matches_bounds(
        index in prop_oneof![0usize..20, Just(usize::MAX)],
        count in 0usize..20,
    ) {
        let pos = Position::of(index, count);
        if count <= 1 {
            prop_assert_eq!(pos, Position::Alone);
        } else if index == 0 {
            prop_assert_eq!(pos, Position::First);
        } else if index >= count - 1 {
            prop_assert_eq!(pos, Position::Last);
        } else {
            prop_assert_eq!(pos, Position::Middle);
        }
    }

    // -------------------------------------------------------------------------
    // plot
    // -------------------------------------------------------------------------

    #[test]
    fn plot_ticks_cover_domain_and_are_strictly_increasing(
        a in -1e6f32..=1e6,
        b in -1e6f32..=1e6,
        count in 0usize..=12,
    ) {
        let ticks = plot::ticks(a, b, count);
        prop_assert!(ticks.len() >= 2);
        prop_assert!(ticks.iter().all(|v| v.is_finite()));

        for w in ticks.windows(2) {
            prop_assert!(w[0] < w[1], "ticks not strictly increasing: {ticks:?}");
        }

        let lo = a.min(b);
        let hi = a.max(b);
        let tol = (hi - lo).abs() * 1e-4 + 1e-4;
        prop_assert!(ticks[0] <= lo + tol, "first tick {} > lo {lo}", ticks[0]);
        prop_assert!(*ticks.last().unwrap() >= hi - tol, "last tick {} < hi {hi}", ticks.last().unwrap());

        if lo <= 0.0 && hi >= 0.0 {
            prop_assert!(ticks.contains(&0.0), "missing 0 in {ticks:?} for {lo}..={hi}");
        }
    }

    #[test]
    fn plot_stack_preserves_totals(
        s1 in prop::collection::vec(0.0f32..1000.0, 1..10),
        s2 in prop::collection::vec(0.0f32..1000.0, 1..10),
    ) {
        let count = s1.len().max(s2.len());
        let stacked = plot::stack([s1.as_slice(), s2.as_slice()], count, Offset::None);
        for i in 0..count {
            let expected = plot::value(&s1, i) + plot::value(&s2, i);
            prop_assert_eq!(stacked[0][i][0], 0.0);
            prop_assert_eq!(stacked[0][i][1], stacked[1][i][0]);
            prop_assert!((stacked[1][i][1] - expected).abs() < 1e-3);
        }

        let expanded = plot::stack([s1.as_slice(), s2.as_slice()], count, Offset::Expand);
        for (i, bar) in expanded[1].iter().enumerate().take(count) {
            let total = plot::value(&s1, i) + plot::value(&s2, i);
            if total > 0.0 {
                prop_assert!((bar[1] - 1.0).abs() < 1e-4);
            }
        }
    }

    #[test]
    fn plot_categories_position_and_hit_test_round_trip(
        count in 1usize..=20,
        width in 20.0f32..=1000.0,
        height in 20.0f32..=1000.0,
        vertical in any::<bool>(),
        band in any::<bool>(),
    ) {
        let rect = Rectangle::new(Point::new(10.0, 20.0), Size::new(width, height));
        let spread = if band { Spread::Band } else { Spread::Point };
        let scale = if vertical {
            Categories::new(rect, count, spread).vertical()
        } else {
            Categories::new(rect, count, spread)
        };

        for i in 0..count {
            let pos = scale.position(i);
            let pt = if vertical {
                Point::new(rect.center_x(), pos.clamp(rect.y, rect.y + rect.height - 1e-3))
            } else {
                Point::new(pos.clamp(rect.x, rect.x + rect.width - 1e-3), rect.center_y())
            };
            prop_assert_eq!(scale.at(pt), Some(i));
        }

        prop_assert!(scale.label_step(50.0) >= 1);
    }

    #[test]
    fn plot_linear_ticks_are_finite_even_for_zero(ticks in 0usize..=10) {
        let rect = Rectangle::new(Point::new(0.0, 10.0), Size::new(200.0, 100.0));
        let linear = Linear::new(rect, 100.0);
        let ys: Vec<f32> = linear.ticks(ticks).collect();
        prop_assert_eq!(ys.len(), ticks + 1);
        prop_assert!(ys.iter().all(|y| y.is_finite()));
    }

    #[test]
    fn plot_polar_and_angle_round_trip(
        radius in 1.0f32..=500.0,
        deg in 0.0f32..360.0,
    ) {
        let center = Point::new(50.0, 60.0);
        let pt = plot::polar(center, radius, deg);
        let dist = plot::distance(center, pt);
        let recovered = plot::angle(center, pt);

        prop_assert!((dist - radius).abs() < 1e-3);
        let diff = (recovered - deg).rem_euclid(360.0);
        prop_assert!(diff < 1e-2 || (360.0 - diff) < 1e-2, "deg={deg}, recovered={recovered}");
    }

    #[test]
    fn plot_pie_sectors_are_finite_and_partition_sweep(
        values in prop::collection::vec(0.0f32..=100.0, 1..8),
        start in -360.0f32..=360.0,
        sweep in -360.0f32..=360.0,
    ) {
        let rect = Rectangle::new(Point::ORIGIN, Size::new(200.0, 200.0));
        let pie = Pie::new(rect, 0.4);
        let sectors = pie.sectors(&values, start, start + sweep);
        let positive_sum: f32 = values.iter().copied().filter(|&v| v > 0.0).sum();

        if positive_sum <= 0.0 {
            prop_assert!(sectors.is_empty());
        } else {
            prop_assert_eq!(sectors.len(), values.len());
            prop_assert!((sectors[0].start - start).abs() < 1e-3);
            prop_assert!((sectors.last().unwrap().end - (start + sweep)).abs() < 1e-2);
            for w in sectors.windows(2) {
                prop_assert!((w[0].end - w[1].start).abs() < 1e-3);
            }
        }
    }

    #[test]
    fn plot_place_never_precedes_start(
        anchor in 0.0f32..=500.0,
        size in 1.0f32..=200.0,
        offset in 0.0f32..=30.0,
        start in 0.0f32..=100.0,
        length in 50.0f32..=500.0,
    ) {
        let pos = plot::place(anchor, size, offset, start, length);
        prop_assert!(pos >= start);
    }

    // -------------------------------------------------------------------------
    // anchored
    // -------------------------------------------------------------------------

    #[test]
    fn anchored_position_invariants(
        ax in 0.0f32..=700.0,
        ay in 0.0f32..=500.0,
        aw in 10.0f32..=100.0,
        ah in 10.0f32..=100.0,
        cw in 10.0f32..=200.0,
        ch in 10.0f32..=200.0,
        side in arb_side(),
        align in arb_align(),
        offset in 0.0f32..=20.0,
        padding in 0.0f32..=20.0,
    ) {
        let viewport = Size::new(800.0, 600.0);
        let anchor = Rectangle::new(Point::new(ax, ay), Size::new(aw, ah));
        let size = Size::new(cw, ch);

        let (pt, resolved_side) =
            anchored::position(anchor, size, viewport, side, align, offset, 0.0, padding);

        prop_assert!(resolved_side == side || resolved_side == side.opposite());
        prop_assert_eq!(side.opposite().opposite(), side);

        // Cross-axis stays within viewport padding when content fits.
        match resolved_side {
            Side::Top | Side::Bottom if cw + 2.0 * padding <= viewport.width => {
                prop_assert!(pt.x >= padding - 1e-4);
                prop_assert!(pt.x + cw <= viewport.width - padding + 1e-4);
            }
            Side::Left | Side::Right if ch + 2.0 * padding <= viewport.height => {
                prop_assert!(pt.y >= padding - 1e-4);
                prop_assert!(pt.y + ch <= viewport.height - padding + 1e-4);
            }
            _ => {}
        }
    }

    // -------------------------------------------------------------------------
    // box_shadow
    // -------------------------------------------------------------------------

    #[test]
    fn box_shadow_quads_have_non_negative_radius(
        w in 0.0f32..=200.0,
        h in 0.0f32..=200.0,
        radius in 0.0f32..=50.0,
        spread in -100.0f32..=50.0,
        opacity in 0.0f32..=1.0,
    ) {
        let layer = Layer {
            color: Color::BLACK,
            offset: Vector::new(0.0, 2.0),
            blur_radius: 4.0,
            spread,
        };
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(w, h));
        let quads: Vec<_> = box_shadow::quads(&[layer], bounds, radius, opacity).collect();
        prop_assert_eq!(quads.len(), 1);
        let r = quads[0].border.radius;
        prop_assert!(r.top_left >= 0.0 && r.top_right >= 0.0 && r.bottom_left >= 0.0 && r.bottom_right >= 0.0);
    }

    // -------------------------------------------------------------------------
    // questionnaire state machine
    // -------------------------------------------------------------------------

    #[test]
    fn questionnaire_state_machine_invariants(
        ops in prop::collection::vec(0u8..8, 1..40),
    ) {
        let questions = vec![
            Question::new("Q1").choice("A").choice("B").required(true),
            Question::new("Q2").choice("C").choice("D").multiple(true).required(false),
            Question::new("Q3").choice("E").input("Other").required(true),
        ];
        let mut state = QState::new();

        for op in ops {
            let prev_idx = state.current();
            let event = match op {
                0 => QEvent::Toggle(0),
                1 => QEvent::Toggle(1),
                2 => QEvent::Input("hello".to_owned()),
                3 => QEvent::Input(String::new()),
                4 => QEvent::Previous,
                5 => QEvent::Skip,
                6 => QEvent::Next,
                _ => QEvent::Submit,
            };
            let submitted = state.update(&questions, event.clone());

            prop_assert!(state.current() < questions.len());
            prop_assert_eq!(state.answers(&questions).len(), questions.len());

            // Selected indices are always sorted and strictly unique.
            for q in 0..questions.len() {
                let sel = state.selected(q);
                for w in sel.windows(2) {
                    prop_assert!(w[0] < w[1]);
                }
            }

            // Required question cannot be skipped.
            if matches!(event, QEvent::Skip) && questions[prev_idx].required {
                prop_assert_eq!(state.current(), prev_idx);
                prop_assert!(!submitted);
            }
        }
    }

    // -------------------------------------------------------------------------
    // listbox typeahead
    // -------------------------------------------------------------------------

    #[test]
    fn listbox_next_match_never_returns_current(
        search in "[a-zA-Z]{0,5}",
        current in prop::option::of(0usize..5),
    ) {
        let items = [
            (0, "Apple"),
            (1, "Apricot"),
            (2, "Banana"),
            (3, "Blueberry"),
            (4, "Cherry"),
        ];
        let matched = next_match(&search, items, current);
        if let Some(idx) = matched {
            prop_assert_ne!(Some(idx), current);
            prop_assert!(items.iter().any(|(i, _)| *i == idx));
        }
    }
}

#[test]
fn highlight_color_and_from_name_smoke() {
    assert_eq!(Language::from_name("RS"), Some(Language::Rust));
    assert_eq!(
        Language::from_name("TypeScript"),
        Some(Language::JavaScript)
    );
    assert_eq!(Language::from_name("json5"), Some(Language::Json));
    assert_eq!(Language::from_name("unknown"), None);

    let light = theme::neutral(Mode::Light).build();
    let dark = theme::neutral(Mode::Dark).build();
    for token in [
        Token::Plain,
        Token::Keyword,
        Token::String,
        Token::Number,
        Token::Constant,
        Token::Comment,
        Token::Function,
        Token::Type,
        Token::Variable,
        Token::Property,
    ] {
        assert_ne!(color(&light, token), color(&dark, token));
    }
}

#[test]
fn plot_pie_handles_non_finite_values() {
    let pie = Pie::new(Rectangle::new(Point::ORIGIN, Size::new(100.0, 100.0)), 0.0);
    let sectors = pie.sectors(&[f32::INFINITY, 10.0, f32::NAN], 0.0, 360.0);
    assert!(
        sectors
            .iter()
            .all(|s| s.start.is_finite() && s.end.is_finite())
    );
}

#[test]
fn appearance_identity_and_transformation() {
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(100.0, 50.0));
    assert!(Appearance::VISIBLE.is_identity());
    assert!(Appearance::VISIBLE.transformation(bounds).is_none());

    let scaled = Appearance {
        scale: 0.95,
        ..Appearance::VISIBLE
    };
    assert!(!scaled.is_identity());
    assert!(scaled.transformation(bounds).is_some());

    let shifted = Appearance {
        offset: Vector::new(4.0, 0.0),
        ..Appearance::VISIBLE
    };
    assert!(!shifted.is_identity());
    assert!(shifted.transformation(bounds).is_some());

    let faded = Appearance {
        opacity: 0.5,
        ..Appearance::VISIBLE
    };
    assert!(!faded.is_identity());
    assert!(faded.transformation(bounds).is_none());
}

#[test]
fn style_radii_and_leading() {
    let r = 8.0;
    assert_eq!(Position::Alone.radius(r), iced::border::radius(r));
    assert_eq!(Position::Middle.radius(r), iced::border::radius(0));
    assert_eq!(Position::First.radius(r).top_left, r);
    assert_eq!(Position::First.radius(r).top_right, 0.0);
    assert_eq!(Position::Last.radius(r).top_right, r);
    assert_eq!(Position::Last.radius(r).top_left, 0.0);

    assert_eq!(Position::First.vertical_radius(r).top_right, r);
    assert_eq!(Position::First.vertical_radius(r).bottom_left, 0.0);
    assert_eq!(Position::Last.vertical_radius(r).bottom_left, r);
    assert_eq!(Position::Last.vertical_radius(r).top_left, 0.0);

    assert_eq!(style::spacing(4.0), 16.0);
    assert_eq!(
        style::text::leading(style::text::SM),
        iced::widget::text::LineHeight::Absolute(iced::Pixels(20.0))
    );
    assert_eq!(
        style::text::leading(10.0),
        iced::widget::text::LineHeight::Absolute(iced::Pixels(15.0))
    );
}

#[test]
fn plot_bars_and_sector_contains() {
    let bars = Bars::new(100.0, 2, 0.1, 4.0);
    assert_eq!(bars.offset, 10.0);
    assert_eq!(bars.gap, 4.0);
    assert_eq!(bars.width, 38.0);
    assert_eq!(bars.x(0.0, 0), 10.0);
    assert_eq!(bars.x(0.0, 1), 52.0);

    // When bars don't fit, gap drops to 0.
    let tight = Bars::new(10.0, 4, 0.1, 10.0);
    assert_eq!(tight.gap, 0.0);

    let sector = Sector {
        center: Point::new(0.0, 0.0),
        inner: 10.0,
        outer: 50.0,
        start: 0.0,
        end: 90.0,
    };
    assert_eq!(sector.middle(), 45.0);
    assert!(sector.contains(Point::new(20.0, -20.0))); // 45 deg
    assert!(!sector.contains(Point::new(5.0, 0.0))); // inside inner
    assert!(!sector.contains(Point::new(60.0, 0.0))); // outside outer
    assert!(!sector.contains(Point::new(-20.0, -20.0))); // 135 deg
}

#[test]
fn typeahead_timeout_resets_search() {
    let mut ta = Typeahead::default();
    let t0 = Instant::now();
    assert!(!ta.is_typing(t0));
    assert_eq!(ta.push("a", t0), "a");
    assert!(ta.is_typing(t0 + Duration::from_millis(500)));
    assert_eq!(ta.push("b", t0 + Duration::from_millis(500)), "ab");
    assert!(!ta.is_typing(t0 + Duration::from_millis(2000)));
    assert_eq!(ta.push("c", t0 + Duration::from_millis(2000)), "c");
}

#[test]
fn highlight_remaining_constructs() {
    assert_eq!(Language::from_name("toml"), Some(Language::Toml));

    // Multiline block comment spanning 3 lines
    let block = highlight(Language::Rust, "/* a\nb\nc */ let x = 1;");
    assert_eq!(block[1], vec![(0..1, Token::Comment)]);
    assert_eq!(block[2][0], (0..4, Token::Comment));

    // Triple-quoted TOML string
    let toml = highlight(Language::Toml, "s = \"\"\"\nline\n\"\"\"");
    assert_eq!(toml[1], vec![(0..4, Token::String)]);

    // Shell command substitution and sudo/env
    let sh = highlight(Language::Shell, "sudo env echo $(ls)");
    assert!(sh[0].iter().any(|(_, t)| *t == Token::Function));
}

#[test]
fn animation_transition_edge_cases_and_fade() {
    use shadcn_iced_core::animation::Fade;

    let start = Instant::now();
    let mut tr = Transition::default();
    assert_eq!(tr.target(), 0.0);

    // Same target is a no-op
    tr.go(0.0, Duration::from_millis(100), linear, start);
    assert!(!tr.is_animating(start));

    // Zero duration settles immediately
    tr.go(5.0, Duration::ZERO, linear, start);
    assert_eq!(tr.value(start), 5.0);
    assert!(!tr.is_animating(start));

    // set() jumps immediately
    tr.set(9.0);
    assert_eq!(tr.target(), 9.0);
    assert_eq!(tr.value(start), 9.0);

    // Theme fading: >= 1.0 returns None, < 1.0 caches and scales alpha
    let base = theme::neutral(Mode::Light).build();
    assert!(base.faded(1.0).is_none());
    let faded_a = base.faded(0.5).expect("faded theme");
    let faded_b = base.faded(0.5).expect("cached faded theme");
    assert_eq!(faded_a, faded_b);
    let orig_fg = theme::tokens(&base).palette.foreground;
    let faded_fg = theme::tokens(&faded_a).palette.foreground;
    assert!(faded_fg.a < orig_fg.a);
}

#[test]
fn questionnaire_builders_invalid_and_empty() {
    use shadcn_iced_core::primitive::questionnaire::Choice;

    let choice = Choice::new("A").description("desc");
    assert_eq!(choice.description.as_deref(), Some("desc"));

    let q = Question::new("Title")
        .description("Help")
        .choice(choice)
        .required(true);
    assert_eq!(q.description.as_deref(), Some("Help"));

    let mut state = QState::new();
    // Empty questions list is a no-op
    assert!(!state.update(&[], QEvent::Next));
    assert!(!state.is_invalid());

    // Advancing past unanswered required question sets invalid
    assert!(!state.update(std::slice::from_ref(&q), QEvent::Next));
    assert!(state.is_invalid());
    assert!(!state.update(std::slice::from_ref(&q), QEvent::Toggle(0)));
    assert!(!state.is_invalid());
}

#[test]
fn plot_curves_sector_paths_and_extent() {
    use iced::widget::canvas::path::Builder;
    use plot::Curve;

    let pts = [
        Point::new(0.0, 10.0),
        Point::new(10.0, 20.0),
        Point::new(20.0, 15.0),
        Point::new(30.0, 25.0),
    ];

    for kind in [Curve::Natural, Curve::Monotone, Curve::Linear, Curve::Step] {
        let mut b = Builder::new();
        plot::curve(&mut b, &[], kind, true);
        plot::curve(&mut b, &pts[..1], kind, true);
        plot::curve(&mut b, &pts[..2], kind, false);
        plot::curve(&mut b, &pts, kind, true);
        plot::curve(&mut b, &pts, kind, false);
        let _ = b.build();
    }

    // Sector path with and without corners and inner radius
    for (inner, corner) in [(0.0, 0.0), (0.0, 4.0), (20.0, 0.0), (20.0, 4.0)] {
        let sector = Sector {
            center: Point::new(50.0, 50.0),
            inner,
            outer: 40.0,
            start: 0.0,
            end: 90.0,
        };
        let mut b = Builder::new();
        sector.path(&mut b, corner);
        let _ = b.build();
    }

    let rect = Rectangle::new(Point::ORIGIN, Size::new(100.0, 100.0));
    let cats = Categories::band(rect, 4);
    assert_eq!(cats.band_width(), 25.0);
    assert_eq!(cats.band_start(2), 50.0);

    let flat = Linear::vertical(rect, 5.0, 5.0);
    assert_eq!(flat.position(5.0), 100.0);

    let (lo, hi) = plot::extent(
        [&[10.0, -5.0][..], &[20.0, -10.0][..]],
        2,
        Some(Offset::Sign),
    );
    assert_eq!((lo, hi), (-15.0, 30.0));
}

#[test]
fn style_and_theme_helpers() {
    let light = theme::neutral(Mode::Light).tokens();
    let dark = theme::neutral(Mode::Dark).tokens();

    assert_eq!(style::shadow_xs().blur_radius, 2.0);
    assert_eq!(style::shadow_sm().blur_radius, 3.0);
    assert_eq!(style::shadow_md().blur_radius, 6.0);
    assert_eq!(style::shadow_lg().blur_radius, 15.0);

    assert_eq!(style::focus_ring(&light).blur_radius, 6.0);
    assert_eq!(style::ring(&light, 6.0).width, 3.0);
    assert!(style::focus_visible(&light, 6.0).ring.is_some());
    assert!(
        style::invalid_outset_ring(&dark, 4.0).color.a
            > style::invalid_outset_ring(&light, 4.0).color.a
    );
    assert!(style::invalid_ring(&dark).color.a > style::invalid_ring(&light).color.a);
    assert_eq!(style::rounded(4.0).width, 0.0);
    assert_eq!(style::ring_border(&light, 4.0).width, 1.0);

    for (sz, expected) in [
        (style::text::XS, 16.0),
        (style::text::BASE, 24.0),
        (style::text::LG, 28.0),
        (style::text::XL2, 32.0),
        (style::text::XL3, 36.0),
        (style::text::XL4, 40.0),
    ] {
        assert_eq!(
            style::text::leading(sz),
            iced::widget::text::LineHeight::Absolute(iced::Pixels(expected))
        );
    }

    assert_eq!(theme::rgba(255, 0, 0, 0.5).a, 0.5);
    assert_eq!(
        theme::over(Color::TRANSPARENT, Color::TRANSPARENT),
        Color::TRANSPARENT
    );
    assert!(!theme::tokens(&iced::Theme::Light).is_dark());
}

#[test]
fn pointer_mouse_and_touch_lifecycle() {
    use iced::{Event, mouse, touch};
    use shadcn_iced_core::primitive::pointer::{self, Gesture, Press};

    let down = Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(1),
        position: Point::new(10.0, 10.0),
    });
    pointer::observe(&down);
    assert!(pointer::is_touch());
    assert!(pointer::is_touching());

    let lost = Event::Touch(touch::Event::FingerLost {
        id: touch::Finger(1),
        position: Point::new(10.0, 10.0),
    });
    let mut touch_press = Press::start(
        &down,
        mouse::Cursor::Available(Point::new(10.0, 10.0)),
        |_| true,
    )
    .unwrap();
    assert_eq!(
        touch_press.update(&lost, mouse::Cursor::Unavailable),
        Some(Gesture::Released)
    );
    pointer::observe(&lost);
    assert!(!pointer::is_touching());

    // Mouse press -> move -> tap and cursor left
    let mouse_down = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
    let cursor = mouse::Cursor::Available(Point::new(20.0, 20.0));
    let mut mp = Press::start(&mouse_down, cursor, |_| true).unwrap();
    let moved = Event::Mouse(mouse::Event::CursorMoved {
        position: Point::new(25.0, 20.0),
    });
    assert!(matches!(mp.update(&moved, cursor), Some(Gesture::Moved(_))));
    let up = Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left));
    assert!(matches!(mp.update(&up, cursor), Some(Gesture::Tapped(_))));

    let mut mp2 = Press::start(&mouse_down, cursor, |_| true).unwrap();
    assert_eq!(
        mp2.update(
            &Event::Mouse(mouse::Event::CursorLeft),
            mouse::Cursor::Unavailable
        ),
        Some(Gesture::Released)
    );
}
