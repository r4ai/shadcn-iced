//! The geometry of charts, like the parts of [d3](https://d3js.org) that
//! [Recharts](https://recharts.org) builds on.
//!
//! These are plain functions and types, not widgets. A cartesian chart lays
//! out its plot with a [`Categories`] scale and a [`Linear`] scale (either
//! may be vertical), finds the values of its axis with [`ticks`], stacks its
//! series with [`stack`], and draws its lines with [`curve`]. A polar chart
//! places its points with [`polar`], draws its slices, rings and radial bars
//! as [`Sector`]s, and lays out its pies with [`Pie`]. [`Categories::at`],
//! [`Sector::contains`] and [`angle`] find what the pointer is over. The
//! look (colors, strokes, labels, tooltips) stays in the chart.
//!
//! Angles are in degrees, counterclockwise from the right (3 o'clock), like
//! in Recharts: 90 is the top.
//!
//! ```
//! use shadcn_iced_core::primitive::plot::{self, Categories, Linear};
//! use iced::Rectangle;
//!
//! let plot = Rectangle::new([0.0, 0.0].into(), [300.0, 200.0].into());
//! let values = [186.0, 305.0, 237.0];
//!
//! let x = Categories::band(plot, values.len());
//! let ticks = plot::ticks(0.0, 305.0, 5);
//! let y = Linear::new(plot, *ticks.last().unwrap());
//!
//! assert_eq!(ticks, [0.0, 80.0, 160.0, 240.0, 320.0]);
//! assert_eq!(x.position(0), 50.0);
//! assert_eq!(y.position(0.0), 200.0);
//! assert_eq!(x.at([120.0, 10.0].into()), Some(1));
//! ```
use iced::widget::canvas::path;
use iced::{Point, Rectangle};

/// The values of the ticks of a value axis covering `min..=max`, `count` of
/// them at "nice" steps, like Recharts' `getNiceTickValues`. The first and
/// last ticks are the domain of the axis.
///
/// The ticks always include 0 when the range crosses it.
pub fn ticks(min: f32, max: f32, count: usize) -> Vec<f32> {
    let count = count.max(2);
    let (min, max) = (f64::from(min.min(max)), f64::from(max.max(min)));

    // A single value: an axis of unit steps around it.
    let (min, max) = if max - min <= f64::EPSILON {
        if min == 0.0 {
            (0.0, 1.0)
        } else {
            (min.min(0.0), min.max(0.0))
        }
    } else {
        (min, max)
    };

    let mut correction = 0.0;

    loop {
        let step = format_step((max - min) / (count - 1) as f64, correction);

        if step <= 0.0 {
            return vec![min as f32, max as f32];
        }

        let middle = if min <= 0.0 && max >= 0.0 {
            0.0
        } else {
            let middle = (min + max) / 2.0;
            middle - middle % step
        };

        let mut below = ((middle - min) / step - 1e-9).ceil().max(0.0) as usize;
        let mut above = ((max - middle) / step - 1e-9).ceil().max(0.0) as usize;
        let scale = below + above + 1;

        if scale > count {
            correction += 1.0;
            continue;
        }

        if max > 0.0 {
            above += count - scale;
        } else {
            below += count - scale;
        }

        let start = middle - below as f64 * step;

        return (0..=below + above)
            .map(|index| clean(start + index as f64 * step) as f32)
            .collect();
    }
}

/// A "nice" step close to `rough`: a multiple of 0.05 or 0.1 times a power
/// of 10, `correction` multiples larger.
fn format_step(rough: f64, correction: f64) -> f64 {
    if rough <= 0.0 || !rough.is_finite() {
        return 0.0;
    }

    let digits = rough.log10().floor() + 1.0;
    let magnitude = 10f64.powf(digits);
    let scale = if digits == 1.0 { 0.1 } else { 0.05 };

    clean((((rough / magnitude) / scale).ceil() + correction) * scale * magnitude)
}

/// Rounds away the error of floating point arithmetic (like `Decimal` does in
/// Recharts), so that ticks are e.g. `0.3` and not `0.30000000000000004`.
fn clean(value: f64) -> f64 {
    if value == 0.0 {
        return 0.0;
    }

    let digits = 12 - value.abs().log10().ceil() as i32;
    let factor = 10f64.powi(digits);

    (value * factor).round() / factor
}

/// The top of a value axis starting at 0: the last of 5 [`ticks`]. Returns 1
/// when `max` is not positive.
pub fn nice(max: f32) -> f32 {
    ticks(0.0, max.max(0.0), 5).last().copied().unwrap_or(1.0)
}

/// The value of a series at a category, 0 when it is missing.
pub fn value(values: &[f32], index: usize) -> f32 {
    values.get(index).copied().unwrap_or(0.0)
}

/// How [`stack`] piles series up (`stackOffset`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Offset {
    /// Every series starts where the previous one ends (`none`).
    #[default]
    None,
    /// Like [`Offset::None`], scaled so that every category adds up to 1
    /// (`expand`).
    Expand,
    /// Positive values pile up above 0 and negative values below it
    /// (`sign`).
    Sign,
}

/// Stacks series (`stackId`): where each series starts and ends at each
/// category, `[lower, upper]`.
pub fn stack<'a>(
    series: impl IntoIterator<Item = &'a [f32]>,
    count: usize,
    offset: Offset,
) -> Vec<Vec<[f32; 2]>> {
    let series: Vec<&[f32]> = series.into_iter().collect();
    let mut positive = vec![0.0; count];
    let mut negative = vec![0.0; count];

    let mut stacked: Vec<Vec<[f32; 2]>> = series
        .iter()
        .map(|values| {
            (0..count)
                .map(|i| {
                    let value = value(values, i);
                    let base = match offset {
                        Offset::Sign if value < 0.0 => &mut negative[i],
                        _ => &mut positive[i],
                    };
                    let lower = *base;
                    *base += value;

                    [lower, *base]
                })
                .collect()
        })
        .collect();

    if offset == Offset::Expand {
        for i in 0..count {
            let total: f32 = series.iter().map(|values| value(values, i)).sum();

            if total != 0.0 {
                for series in &mut stacked {
                    series[i] = [series[i][0] / total, series[i][1] / total];
                }
            }
        }
    }

    stacked
}

/// The smallest and largest values of `series`: of their ends when
/// `stacked`, else of the values. Always includes 0, like the default domain
/// of a value axis (`[0, 'auto']`).
pub fn extent<'a>(
    series: impl IntoIterator<Item = &'a [f32]>,
    count: usize,
    stacked: Option<Offset>,
) -> (f32, f32) {
    let series: Vec<&[f32]> = series.into_iter().collect();

    let values: Vec<f32> = match stacked {
        Some(offset) => stack(series.iter().copied(), count, offset)
            .into_iter()
            .flatten()
            .flatten()
            .collect(),
        None => series
            .iter()
            .flat_map(|values| values.iter().take(count).copied())
            .collect(),
    };

    values
        .into_iter()
        .filter(|value| value.is_finite())
        .fold((0.0, 0.0), |(min, max), value| {
            (min.min(value), max.max(value))
        })
}

/// How categories are spread along their axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Spread {
    /// Each category is a band, its position is the center of the band
    /// (`scaleBand`, for bars and pies).
    Band,
    /// The categories are spread edge to edge (`scalePoint`, for lines and
    /// areas).
    Point,
}

/// A scale mapping categories to positions along an axis of a plot:
/// horizontal (left to right), or vertical (top to bottom).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Categories {
    plot: Rectangle,
    count: usize,
    spread: Spread,
    vertical: bool,
}

impl Categories {
    /// Creates a horizontal scale of `count` categories over `plot`.
    pub fn new(plot: Rectangle, count: usize, spread: Spread) -> Self {
        Self {
            plot,
            count,
            spread,
            vertical: false,
        }
    }

    /// A horizontal [`Spread::Band`] scale.
    pub fn band(plot: Rectangle, count: usize) -> Self {
        Self::new(plot, count, Spread::Band)
    }

    /// A horizontal [`Spread::Point`] scale.
    pub fn point(plot: Rectangle, count: usize) -> Self {
        Self::new(plot, count, Spread::Point)
    }

    /// The same scale from the top to the bottom of the plot (the category
    /// axis of horizontal bars, `layout="vertical"`).
    pub fn vertical(self) -> Self {
        Self {
            vertical: true,
            ..self
        }
    }

    fn start(&self) -> f32 {
        if self.vertical {
            self.plot.y
        } else {
            self.plot.x
        }
    }

    fn length(&self) -> f32 {
        if self.vertical {
            self.plot.height
        } else {
            self.plot.width
        }
    }

    /// The size of a band.
    pub fn band_width(&self) -> f32 {
        self.length() / self.count.max(1) as f32
    }

    /// The start (left or top edge) of the band of a category.
    pub fn band_start(&self, index: usize) -> f32 {
        self.start() + self.band_width() * index as f32
    }

    /// The position of a category along the axis.
    pub fn position(&self, index: usize) -> f32 {
        let count = self.count.max(1);
        let (start, length) = (self.start(), self.length());

        match self.spread {
            Spread::Point if count == 1 => start + length / 2.0,
            Spread::Point => start + length * index as f32 / (count - 1) as f32,
            Spread::Band => start + length / count as f32 * (index as f32 + 0.5),
        }
    }

    /// The category under a point of the plot: the band it is in, or the
    /// closest point.
    pub fn at(&self, point: Point) -> Option<usize> {
        let count = self.count;

        if count == 0 || !self.plot.contains(point) {
            return None;
        }

        let offset = if self.vertical { point.y } else { point.x } - self.start();
        let length = self.length();

        let index = match self.spread {
            Spread::Point if count > 1 => (offset / (length / (count - 1) as f32) + 0.5).floor(),
            _ => (offset / (length / count as f32)).floor(),
        };

        Some((index.max(0.0) as usize).min(count - 1))
    }

    /// Every how many categories a label fits, when labels need `spacing`
    /// pixels each.
    pub fn label_step(&self, spacing: f32) -> usize {
        ((self.count as f32 * spacing) / self.length().max(1.0))
            .ceil()
            .max(1.0) as usize
    }
}

/// A scale mapping values from `min` to `max` to positions in a plot: from
/// the bottom to the top, or from the left to the right.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Linear {
    plot: Rectangle,
    min: f32,
    max: f32,
    horizontal: bool,
}

impl Linear {
    /// Creates a vertical scale from 0 to `max` over `plot`.
    pub fn new(plot: Rectangle, max: f32) -> Self {
        Self::vertical(plot, 0.0, max)
    }

    /// Creates a vertical scale from `min` (at the bottom) to `max` (at the
    /// top) over `plot`.
    pub fn vertical(plot: Rectangle, min: f32, max: f32) -> Self {
        Self {
            plot,
            min,
            max,
            horizontal: false,
        }
    }

    /// Creates a horizontal scale from `min` (on the left) to `max` (on the
    /// right) over `plot`.
    pub fn horizontal(plot: Rectangle, min: f32, max: f32) -> Self {
        Self {
            plot,
            min,
            max,
            horizontal: true,
        }
    }

    /// The share of the scale below a value, from 0 to 1.
    fn ratio(&self, value: f32) -> f32 {
        let range = self.max - self.min;

        if range == 0.0 {
            0.0
        } else {
            (value - self.min) / range
        }
    }

    /// The position of a value: vertical or horizontal, like the scale.
    pub fn position(&self, value: f32) -> f32 {
        let ratio = self.ratio(value);

        if self.horizontal {
            self.plot.x + ratio * self.plot.width
        } else {
            self.plot.y + self.plot.height - ratio * self.plot.height
        }
    }

    /// The vertical positions of `ticks + 1` evenly spaced ticks, from the
    /// top.
    pub fn ticks(&self, ticks: usize) -> impl Iterator<Item = f32> + use<> {
        let plot = self.plot;

        (0..=ticks).map(move |step| plot.y + plot.height * step as f32 / ticks as f32)
    }
}

/// Where the bars of a category go in its band.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bars {
    /// The space before the first bar (`barCategoryGap`).
    pub offset: f32,
    /// The width of each bar.
    pub width: f32,
    /// The space between bars (`barGap`).
    pub gap: f32,
}

impl Bars {
    /// Fits `groups` bars side by side in a band, like Recharts with a
    /// `barCategoryGap` of `category_gap` (a fraction of the band) and a
    /// `barGap` of `gap`, which goes away when the bars do not fit. Bars
    /// wider than a pixel get rounded widths (Recharts 3.10).
    pub fn new(band: f32, groups: usize, category_gap: f32, gap: f32) -> Self {
        let groups = groups.max(1) as f32;
        let offset = band * category_gap;
        let mut gap = gap;

        if band - 2.0 * offset - (groups - 1.0) * gap <= 0.0 {
            gap = 0.0;
        }

        let mut width = (band - 2.0 * offset - (groups - 1.0) * gap) / groups;
        if width > 1.0 {
            width = width.round();
        }

        Self { offset, width, gap }
    }

    /// The start of the bar in `slot`, in a band starting at `start`.
    pub fn x(&self, start: f32, slot: usize) -> f32 {
        start + self.offset + slot as f32 * (self.width + self.gap)
    }
}

/// The interpolation between the points of a line or an area (`type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Curve {
    /// A natural cubic spline (`natural`, d3's `curveNatural`).
    #[default]
    Natural,
    /// A smooth curve that never overshoots the points (`monotone`, d3's
    /// `curveMonotoneX`).
    Monotone,
    /// Straight segments (`linear`).
    Linear,
    /// Horizontal and vertical segments, changing halfway between the
    /// points (`step`, d3's `curveStep`).
    Step,
}

/// Appends a [`Curve`] through `points` to a path. Starts with a `move_to`
/// when `move_first`, else continues the current subpath with a `line_to`,
/// which joins the upper and lower edges of an area.
pub fn curve(builder: &mut path::Builder, points: &[Point], curve: Curve, move_first: bool) {
    match curve {
        Curve::Natural => natural(builder, points, move_first),
        Curve::Monotone => monotone(builder, points, move_first),
        Curve::Linear | Curve::Step => {
            let Some(first) = points.first() else { return };

            if move_first {
                builder.move_to(*first);
            } else {
                builder.line_to(*first);
            }

            for pair in points.windows(2) {
                let (from, to) = (pair[0], pair[1]);

                if curve == Curve::Step {
                    let middle = (from.x + to.x) / 2.0;
                    builder.line_to(Point::new(middle, from.y));
                    builder.line_to(Point::new(middle, to.y));
                }

                builder.line_to(to);
            }
        }
    }
}

/// Starts a curve: a `move_to` when `move_first`, else a `line_to`. Returns
/// whether there is a segment to draw after `points[0]`.
fn begin(builder: &mut path::Builder, points: &[Point], move_first: bool) -> bool {
    let Some(first) = points.first() else {
        return false;
    };

    if move_first {
        builder.move_to(*first);
    } else {
        builder.line_to(*first);
    }

    if points.len() == 2 {
        builder.line_to(points[1]);
        return false;
    }

    points.len() > 2
}

/// Appends a d3 `curveNatural` to a path: a natural cubic spline through
/// `points`. See [`curve`].
pub fn natural(builder: &mut path::Builder, points: &[Point], move_first: bool) {
    if !begin(builder, points, move_first) {
        return;
    }

    let n = points.len();
    let xs: Vec<f32> = points.iter().map(|p| p.x).collect();
    let ys: Vec<f32> = points.iter().map(|p| p.y).collect();
    let (ax, bx) = control_points(&xs);
    let (ay, by) = control_points(&ys);

    for i in 0..n - 1 {
        builder.bezier_curve_to(
            Point::new(ax[i], ay[i]),
            Point::new(bx[i], by[i]),
            points[i + 1],
        );
    }
}

/// The control points of a natural cubic spline along one axis, as d3
/// computes them.
fn control_points(x: &[f32]) -> (Vec<f32>, Vec<f32>) {
    let n = x.len() - 1;
    let mut a = vec![0.0; n];
    let mut b = vec![0.0; n];
    let mut r = vec![0.0; n];

    a[0] = 0.0;
    b[0] = 2.0;
    r[0] = x[0] + 2.0 * x[1];

    for i in 1..n - 1 {
        a[i] = 1.0;
        b[i] = 4.0;
        r[i] = 4.0 * x[i] + 2.0 * x[i + 1];
    }

    a[n - 1] = 2.0;
    b[n - 1] = 7.0;
    r[n - 1] = 8.0 * x[n - 1] + x[n];

    for i in 1..n {
        let m = a[i] / b[i - 1];
        b[i] -= m;
        r[i] -= m * r[i - 1];
    }

    a[n - 1] = r[n - 1] / b[n - 1];

    for i in (0..n - 1).rev() {
        a[i] = (r[i] - a[i + 1]) / b[i];
    }

    b[n - 1] = (x[n] + a[n - 1]) / 2.0;

    for i in 0..n - 1 {
        b[i] = 2.0 * x[i + 1] - a[i + 1];
    }

    (a, b)
}

/// Appends a d3 `curveMonotoneX` to a path: a cubic Hermite spline whose
/// tangents keep it monotone between the points (Fritsch–Carlson).
fn monotone(builder: &mut path::Builder, points: &[Point], move_first: bool) {
    if !begin(builder, points, move_first) {
        return;
    }

    let n = points.len();
    let sign = |x: f32| if x < 0.0 { -1.0 } else { 1.0 };

    // The slope of a segment, 0 when it has no width.
    let slope = |a: Point, b: Point| {
        let h = b.x - a.x;
        if h == 0.0 { 0.0 } else { (b.y - a.y) / h }
    };

    // The tangents at the points, like d3's `slope3` and `slope2`.
    let mut tangents = vec![0.0; n];

    for i in 1..n - 1 {
        let (h0, h1) = (points[i].x - points[i - 1].x, points[i + 1].x - points[i].x);
        let (s0, s1) = (
            slope(points[i - 1], points[i]),
            slope(points[i], points[i + 1]),
        );
        let p = if h0 + h1 == 0.0 {
            0.0
        } else {
            (s0 * h1 + s1 * h0) / (h0 + h1)
        };
        let tangent = (sign(s0) + sign(s1)) * s0.abs().min(s1.abs()).min(0.5 * p.abs());

        tangents[i] = if tangent.is_finite() { tangent } else { 0.0 };
    }

    let end = |a: Point, b: Point, t: f32| {
        let h = b.x - a.x;
        if h == 0.0 {
            t
        } else {
            (3.0 * (b.y - a.y) / h - t) / 2.0
        }
    };

    tangents[0] = end(points[0], points[1], tangents[1]);
    tangents[n - 1] = end(points[n - 2], points[n - 1], tangents[n - 2]);

    for i in 0..n - 1 {
        let (a, b) = (points[i], points[i + 1]);
        let dx = (b.x - a.x) / 3.0;

        builder.bezier_curve_to(
            Point::new(a.x + dx, a.y + dx * tangents[i]),
            Point::new(b.x - dx, b.y - dx * tangents[i + 1]),
            b,
        );
    }
}

/// The point at `radius` from `center` in the direction of `angle` (in
/// degrees, counterclockwise from the right), like Recharts'
/// `polarToCartesian`.
pub fn polar(center: Point, radius: f32, angle: f32) -> Point {
    let radians = -angle.to_radians();

    Point::new(
        center.x + radius * radians.cos(),
        center.y + radius * radians.sin(),
    )
}

/// The direction of `point` seen from `center`: an angle in degrees,
/// counterclockwise from the right, from 0 to 360.
pub fn angle(center: Point, point: Point) -> f32 {
    let angle = (-(point.y - center.y))
        .atan2(point.x - center.x)
        .to_degrees();

    if angle < 0.0 { angle + 360.0 } else { angle }
}

/// The distance between two points.
pub fn distance(a: Point, b: Point) -> f32 {
    (a.x - b.x).hypot(a.y - b.y)
}

/// An annular sector: a slice of a pie, a ring, or a radial bar.
///
/// It goes from `start` to `end` (in degrees, counterclockwise from the
/// right; `end` may be smaller than `start` to go clockwise), between the
/// `inner` and `outer` radii.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sector {
    /// The center.
    pub center: Point,
    /// The inner radius, 0 for a slice of a pie.
    pub inner: f32,
    /// The outer radius.
    pub outer: f32,
    /// The angle where it starts.
    pub start: f32,
    /// The angle where it ends.
    pub end: f32,
}

impl Sector {
    /// The angle halfway through the sector.
    pub fn middle(&self) -> f32 {
        (self.start + self.end) / 2.0
    }

    /// Whether a point lies in the sector.
    pub fn contains(&self, point: Point) -> bool {
        let distance = distance(self.center, point);

        if distance > self.outer || distance < self.inner {
            return false;
        }

        let (low, high) = (self.start.min(self.end), self.start.max(self.end));

        if high - low >= 360.0 {
            return true;
        }

        // The angle of the point, brought into the turn starting at `low`.
        let angle = (angle(self.center, point) - low).rem_euclid(360.0) + low;

        angle <= high
    }

    /// Appends the outline of the sector to a path, with corners rounded by
    /// `corner` pixels at most (`cornerRadius`), like Recharts'
    /// `getSectorPath` and `getSectorWithCorner`.
    pub fn path(&self, builder: &mut path::Builder, corner: f32) {
        let Self {
            center,
            inner,
            outer,
            start,
            end,
        } = *self;
        let sweep = (end - start).clamp(-359.999, 359.999);
        let end = start + sweep;
        let sign = if sweep < 0.0 { -1.0 } else { 1.0 };
        let corner = corner.min((outer - inner) / 2.0).max(0.0);

        if corner > 0.0 && sweep.abs() > 0.0 {
            let start_outer = Corner::new(center, outer, start, sign, false, corner);
            let end_outer = Corner::new(center, outer, end, -sign, false, corner);
            let outer_sweep = sweep.abs() - start_outer.theta - end_outer.theta;

            if outer_sweep >= 0.0 {
                builder.move_to(start_outer.line);
                start_outer.arc(builder, true);
                arc(
                    builder,
                    center,
                    outer,
                    start_outer.angle,
                    end_outer.angle,
                    false,
                );
                end_outer.arc(builder, false);

                if inner > 0.0 {
                    let start_inner = Corner::new(center, inner, start, sign, true, corner);
                    let end_inner = Corner::new(center, inner, end, -sign, true, corner);

                    builder.line_to(end_inner.line);
                    end_inner.arc(builder, true);
                    arc(
                        builder,
                        center,
                        inner,
                        end_inner.angle,
                        start_inner.angle,
                        false,
                    );
                    start_inner.arc(builder, false);
                } else {
                    builder.line_to(center);
                }

                builder.close();
                return;
            }
        }

        builder.move_to(polar(center, outer, start));
        arc(builder, center, outer, start, end, false);

        if inner > 0.0 {
            builder.line_to(polar(center, inner, end));
            arc(builder, center, inner, end, start, false);
        } else {
            builder.line_to(center);
        }

        builder.close();
    }
}

/// Appends an arc of a circle around `center` from `from` to `to` (in
/// degrees) to a path, as line segments; starting with a `move_to` when
/// `move_first`.
pub fn arc(
    builder: &mut path::Builder,
    center: Point,
    radius: f32,
    from: f32,
    to: f32,
    move_first: bool,
) {
    let sweep = to - from;
    let steps = ((sweep.abs() / 360.0) * 180.0).ceil().max(1.0) as usize;

    if move_first {
        builder.move_to(polar(center, radius, from));
    }

    for step in 1..=steps {
        builder.line_to(polar(
            center,
            radius,
            from + sweep * step as f32 / steps as f32,
        ));
    }
}

/// A rounded corner of a [`Sector`]: a circle tangent to one of its arcs and
/// one of its radial edges, like Recharts' `getTangentCircle`.
struct Corner {
    /// The center of the circle.
    center: Point,
    /// Where the circle touches the arc.
    arc: Point,
    /// The angle of that point.
    angle: f32,
    /// Where the circle touches the radial edge.
    line: Point,
    /// How much of the arc the corner takes, in degrees.
    theta: f32,
    radius: f32,
}

impl Corner {
    fn new(center: Point, radius: f32, angle: f32, sign: f32, inner: bool, corner: f32) -> Self {
        let distance = if inner {
            radius + corner
        } else {
            radius - corner
        };
        let theta = (corner / distance).clamp(-1.0, 1.0).asin().to_degrees();
        let at = angle + sign * theta;

        Self {
            center: polar(center, distance, at),
            arc: polar(center, radius, at),
            angle: at,
            line: polar(center, distance * theta.to_radians().cos(), angle),
            theta,
            radius: corner,
        }
    }

    /// Appends the corner, from the radial edge to the arc when
    /// `to_arc`, else back.
    fn arc(&self, builder: &mut path::Builder, to_arc: bool) {
        let (from, to) = if to_arc {
            (self.line, self.arc)
        } else {
            (self.arc, self.line)
        };
        let from = angle(self.center, from);
        let mut to = angle(self.center, to);

        // The short way around.
        if to - from > 180.0 {
            to -= 360.0;
        } else if from - to > 180.0 {
            to += 360.0;
        }

        arc(builder, self.center, self.radius, from, to, false);
    }
}

/// A pie, or a donut, filling a plot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pie {
    /// The center of the pie.
    pub center: Point,
    /// The outer radius.
    pub radius: f32,
    /// The inner radius, 0 for a pie.
    pub inner: f32,
}

impl Pie {
    /// The largest pie that fits in `plot`, with a hole of `hole` times its
    /// radius (0 for a pie).
    pub fn new(plot: Rectangle, hole: f32) -> Self {
        let radius = plot.width.min(plot.height) / 2.0;

        Self {
            center: plot.center(),
            radius,
            inner: radius * hole,
        }
    }

    /// The sectors of `values` from `start` to `end` (in degrees, like
    /// `startAngle` and `endAngle`, which default to 0 and 360: from the
    /// right, counterclockwise). Empty when the values do not add up to a
    /// positive total.
    pub fn sectors(&self, values: &[f32], start: f32, end: f32) -> Vec<Sector> {
        let total: f32 = values.iter().filter(|value| **value > 0.0).sum();

        if total <= 0.0 {
            return Vec::new();
        }

        let sweep = (end - start).clamp(-360.0, 360.0);
        let mut angle = start;

        values
            .iter()
            .map(|value| {
                let from = angle;
                angle += value.max(0.0) / total * sweep;

                Sector {
                    center: self.center,
                    inner: self.inner,
                    outer: self.radius,
                    start: from,
                    end: angle,
                }
            })
            .collect()
    }

    /// The sector of `values` under a point. See [`Pie::sectors`].
    pub fn sector_at(&self, values: &[f32], start: f32, end: f32, point: Point) -> Option<usize> {
        self.sectors(values, start, end)
            .iter()
            .position(|sector| sector.end != sector.start && sector.contains(point))
    }
}

/// Places a box of `size` `offset` pixels past `anchor` along an axis, or
/// before it when it would overflow `start + length`, never before `start`;
/// like the tooltip of Recharts.
pub fn place(anchor: f32, size: f32, offset: f32, start: f32, length: f32) -> f32 {
    let positive = anchor + offset;

    if positive + size > start + length {
        (anchor - size - offset).max(start)
    } else {
        positive.max(start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nice_maxima() {
        assert_eq!(nice(305.0), 320.0);
        assert_eq!(nice(186.0), 200.0);
        assert_eq!(nice(505.0), 600.0);
        assert_eq!(nice(0.0), 1.0);
        assert_eq!(nice(10.0), 12.0);
    }

    #[test]
    fn nice_ticks() {
        assert_eq!(ticks(0.0, 505.0, 3), [0.0, 300.0, 600.0]);
        assert_eq!(ticks(-209.0, 214.0, 5), [-300.0, -150.0, 0.0, 150.0, 300.0]);
        assert_eq!(ticks(0.0, 1.0, 5), [0.0, 0.25, 0.5, 0.75, 1.0]);
        assert_eq!(ticks(0.0, 0.0, 5), [0.0, 0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn stacking() {
        let series: [&[f32]; 2] = [&[1.0, 2.0], &[3.0]];

        assert_eq!(
            stack(series, 2, Offset::None),
            vec![vec![[0.0, 1.0], [0.0, 2.0]], vec![[1.0, 4.0], [2.0, 2.0]]]
        );
        assert_eq!(
            stack(series, 2, Offset::Expand)[1],
            vec![[0.25, 1.0], [1.0, 1.0]]
        );
        assert_eq!(extent(series, 2, None), (0.0, 3.0));
        assert_eq!(extent(series, 2, Some(Offset::None)), (0.0, 4.0));

        let signs: [&[f32]; 2] = [&[2.0], &[-3.0]];
        assert_eq!(stack(signs, 1, Offset::Sign)[1], vec![[0.0, -3.0]]);
        assert_eq!(stack(signs, 1, Offset::None)[1], vec![[2.0, -1.0]]);
    }

    #[test]
    fn categories() {
        let plot = Rectangle::new(Point::ORIGIN, [100.0, 50.0].into());
        let band = Categories::band(plot, 4);
        let point = Categories::point(plot, 5);
        let vertical = Categories::band(plot, 5).vertical();

        assert_eq!(band.position(0), 12.5);
        assert_eq!(band.at(Point::new(30.0, 10.0)), Some(1));
        assert_eq!(point.position(4), 100.0);
        assert_eq!(point.at(Point::new(30.0, 10.0)), Some(1));
        assert_eq!(point.at(Point::new(30.0, 60.0)), None);
        assert_eq!(vertical.position(0), 5.0);
        assert_eq!(vertical.at(Point::new(90.0, 25.0)), Some(2));
    }

    #[test]
    fn linear_scales() {
        let plot = Rectangle::new(Point::ORIGIN, [100.0, 50.0].into());

        assert_eq!(Linear::vertical(plot, -10.0, 10.0).position(0.0), 25.0);
        assert_eq!(Linear::horizontal(plot, 0.0, 10.0).position(5.0), 50.0);
    }

    #[test]
    fn angles() {
        let center = Point::new(50.0, 50.0);
        let top = polar(center, 10.0, 90.0);

        assert!((top.x - 50.0).abs() < 1e-4 && (top.y - 40.0).abs() < 1e-4);
        assert!((angle(center, Point::new(50.0, 60.0)) - 270.0).abs() < 1e-4);
    }

    #[test]
    fn pie_sectors() {
        let plot = Rectangle::new(Point::ORIGIN, [100.0, 100.0].into());
        let pie = Pie::new(plot, 0.6);
        let values = [1.0, 1.0];

        assert_eq!(pie.sectors(&values, 0.0, 360.0).len(), 2);
        // Above the center: the first half, counterclockwise from the right.
        assert_eq!(
            pie.sector_at(&values, 0.0, 360.0, Point::new(50.0, 10.0)),
            Some(0)
        );
        assert_eq!(
            pie.sector_at(&values, 0.0, 360.0, Point::new(50.0, 90.0)),
            Some(1)
        );
        // In the hole.
        assert_eq!(
            pie.sector_at(&values, 0.0, 360.0, Point::new(55.0, 50.0)),
            None
        );
    }

    #[test]
    fn sectors_contain_points() {
        let sector = Sector {
            center: Point::ORIGIN,
            inner: 0.0,
            outer: 10.0,
            start: -90.0,
            end: 380.0,
        };

        assert!(sector.contains(Point::new(5.0, 0.0)));

        let quarter = Sector {
            end: 0.0,
            start: 90.0,
            ..sector
        };

        assert!(quarter.contains(Point::new(3.0, -3.0)));
        assert!(!quarter.contains(Point::new(-3.0, -3.0)));
    }

    #[test]
    fn tooltip_flips() {
        assert_eq!(place(10.0, 20.0, 10.0, 0.0, 100.0), 20.0);
        assert_eq!(place(90.0, 20.0, 10.0, 0.0, 100.0), 60.0);
    }
}
