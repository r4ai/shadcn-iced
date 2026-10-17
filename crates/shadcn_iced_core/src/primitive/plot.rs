//! The geometry of charts, like the parts of [d3](https://d3js.org) that
//! [Recharts](https://recharts.org) builds on.
//!
//! These are plain functions and types, not widgets: a chart lays out its
//! plot with a [`Categories`] scale along x and a [`Linear`] scale along y,
//! stacks its series with [`baselines`], draws its lines with [`natural`]
//! curves and its pies with [`Pie`], and finds what the pointer is over with
//! [`Categories::at`] and [`Pie::slice_at`]. The look (colors, strokes,
//! labels, tooltips) stays in the chart.
//!
//! ```
//! use shadcn_iced_core::primitive::plot::{self, Categories, Linear};
//! use iced::Rectangle;
//!
//! let plot = Rectangle::new([0.0, 0.0].into(), [300.0, 200.0].into());
//! let values = [186.0, 305.0, 237.0];
//!
//! let x = Categories::band(plot, values.len());
//! let y = Linear::new(plot, plot::nice(305.0));
//!
//! assert_eq!(x.position(0), 50.0);
//! assert_eq!(y.position(0.0), 200.0);
//! assert_eq!(x.at([120.0, 10.0].into()), Some(1));
//! ```
use iced::widget::canvas::path;
use iced::{Point, Rectangle};

use std::f32::consts::{FRAC_PI_2, TAU};

/// The top of a value axis starting at 0: the last of 5 "nice" ticks, like
/// Recharts' `getNiceTickValues`. Returns 1 when `max` is not positive.
pub fn nice(max: f32) -> f32 {
    const TICKS: f64 = 5.0;

    if max <= 0.0 {
        return 1.0;
    }

    let max = f64::from(max);
    let rough = max / (TICKS - 1.0);
    let digits = rough.log10().floor() + 1.0;
    let magnitude = 10f64.powf(digits);
    let scale = if digits == 1.0 { 0.1 } else { 0.05 };

    let mut correction = 0.0;

    loop {
        let step = (((rough / magnitude) / scale).ceil() + correction) * scale * magnitude;

        if (max / step).ceil() + 1.0 <= TICKS {
            return (step * (TICKS - 1.0)) as f32;
        }

        correction += 1.0;
    }
}

/// The value of a series at a category, 0 when it is missing.
pub fn value(values: &[f32], index: usize) -> f32 {
    values.get(index).copied().unwrap_or(0.0)
}

/// The largest value of `series` over `count` categories: the largest sum
/// of a category when `stacked`, else the largest value.
pub fn max<'a>(series: impl IntoIterator<Item = &'a [f32]>, count: usize, stacked: bool) -> f32 {
    let series: Vec<&[f32]> = series.into_iter().collect();

    if stacked {
        (0..count)
            .map(|i| series.iter().map(|values| value(values, i)).sum::<f32>())
            .fold(0.0, f32::max)
    } else {
        series
            .iter()
            .flat_map(|values| values.iter().copied())
            .fold(0.0, f32::max)
    }
}

/// Where each series starts at each category (`stackId`): the sum of the
/// series before it when `stacked`, else 0.
pub fn baselines<'a>(
    series: impl IntoIterator<Item = &'a [f32]>,
    count: usize,
    stacked: bool,
) -> Vec<Vec<f32>> {
    let mut bases = vec![0.0; count];

    series
        .into_iter()
        .map(|values| {
            let current = if stacked {
                bases.clone()
            } else {
                vec![0.0; count]
            };

            if stacked {
                for (i, base) in bases.iter_mut().enumerate() {
                    *base += value(values, i);
                }
            }

            current
        })
        .collect()
}

/// How categories are spread along the x axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Spread {
    /// Each category is a band, its position is the center of the band
    /// (`scaleBand`, for bars and pies).
    Band,
    /// The categories are spread edge to edge (`scalePoint`, for lines and
    /// areas).
    Point,
}

/// A scale mapping categories to horizontal positions in a plot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Categories {
    plot: Rectangle,
    count: usize,
    spread: Spread,
}

impl Categories {
    /// Creates a scale of `count` categories over `plot`.
    pub fn new(plot: Rectangle, count: usize, spread: Spread) -> Self {
        Self {
            plot,
            count,
            spread,
        }
    }

    /// A [`Spread::Band`] scale.
    pub fn band(plot: Rectangle, count: usize) -> Self {
        Self::new(plot, count, Spread::Band)
    }

    /// A [`Spread::Point`] scale.
    pub fn point(plot: Rectangle, count: usize) -> Self {
        Self::new(plot, count, Spread::Point)
    }

    /// The width of a band.
    pub fn band_width(&self) -> f32 {
        self.plot.width / self.count.max(1) as f32
    }

    /// The left edge of the band of a category.
    pub fn band_start(&self, index: usize) -> f32 {
        self.plot.x + self.band_width() * index as f32
    }

    /// The horizontal position of a category.
    pub fn position(&self, index: usize) -> f32 {
        let count = self.count.max(1);

        match self.spread {
            Spread::Point if count == 1 => self.plot.center_x(),
            Spread::Point => self.plot.x + self.plot.width * index as f32 / (count - 1) as f32,
            Spread::Band => self.plot.x + self.plot.width / count as f32 * (index as f32 + 0.5),
        }
    }

    /// The category under a point of the plot: the band it is in, or the
    /// closest point.
    pub fn at(&self, point: Point) -> Option<usize> {
        let count = self.count;

        if count == 0 || !self.plot.contains(point) {
            return None;
        }

        let index = match self.spread {
            Spread::Point if count > 1 => {
                let step = self.plot.width / (count - 1) as f32;
                ((point.x - self.plot.x) / step + 0.5).floor()
            }
            _ => ((point.x - self.plot.x) / (self.plot.width / count as f32)).floor(),
        };

        Some((index.max(0.0) as usize).min(count - 1))
    }

    /// Every how many categories a label fits, when labels need `spacing`
    /// pixels each.
    pub fn label_step(&self, spacing: f32) -> usize {
        ((self.count as f32 * spacing) / self.plot.width.max(1.0))
            .ceil()
            .max(1.0) as usize
    }
}

/// A scale mapping values from 0 to `max` to vertical positions in a plot,
/// 0 at the bottom.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Linear {
    plot: Rectangle,
    max: f32,
}

impl Linear {
    /// Creates a scale from 0 to `max` over `plot`.
    pub fn new(plot: Rectangle, max: f32) -> Self {
        Self { plot, max }
    }

    /// The vertical position of a value.
    pub fn position(&self, value: f32) -> f32 {
        self.plot.y + self.plot.height - value / self.max * self.plot.height
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
    /// wider than a pixel get whole widths.
    pub fn new(band: f32, groups: usize, category_gap: f32, gap: f32) -> Self {
        let groups = groups.max(1) as f32;
        let offset = band * category_gap;
        let mut gap = gap;

        if band - 2.0 * offset - (groups - 1.0) * gap <= 0.0 {
            gap = 0.0;
        }

        let mut width = (band - 2.0 * offset - (groups - 1.0) * gap) / groups;
        if width > 1.0 {
            width = width.floor();
        }

        Self { offset, width, gap }
    }

    /// The left edge of the bar in `slot`, in a band starting at `start`.
    pub fn x(&self, start: f32, slot: usize) -> f32 {
        start + self.offset + slot as f32 * (self.width + self.gap)
    }
}

/// Appends a d3 `curveNatural` to a path: a natural cubic spline through
/// `points`. Starts with a `move_to` when `move_first`, else continues the
/// current subpath with a `line_to`, which joins the upper and lower edges
/// of an area.
pub fn natural(builder: &mut path::Builder, points: &[Point], move_first: bool) {
    let Some(first) = points.first() else { return };

    if move_first {
        builder.move_to(*first);
    } else {
        builder.line_to(*first);
    }

    let n = points.len();

    if n == 2 {
        builder.line_to(points[1]);
        return;
    }

    if n < 2 {
        return;
    }

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

/// A slice of a [`Pie`]: angles in radians, clockwise from the right.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slice {
    /// Where the slice starts.
    pub start: f32,
    /// How far it goes.
    pub sweep: f32,
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

    /// The slices of `values`, starting at the top and going clockwise.
    /// Empty when the values do not add up to a positive total.
    pub fn slices(values: &[f32]) -> Vec<Slice> {
        let total: f32 = values.iter().sum();

        if total <= 0.0 {
            return Vec::new();
        }

        let mut start = -FRAC_PI_2;

        values
            .iter()
            .map(|value| {
                let sweep = value / total * TAU;
                let slice = Slice { start, sweep };
                start += sweep;
                slice
            })
            .collect()
    }

    /// The slice of `values` under a point.
    pub fn slice_at(&self, values: &[f32], point: Point) -> Option<usize> {
        let total: f32 = values.iter().sum();
        let offset = point - self.center;
        let distance = (offset.x * offset.x + offset.y * offset.y).sqrt();

        if distance > self.radius || distance < self.inner || total <= 0.0 {
            return None;
        }

        let mut angle = offset.y.atan2(offset.x) + FRAC_PI_2;
        if angle < 0.0 {
            angle += TAU;
        }

        let mut start = 0.0;
        for (index, value) in values.iter().enumerate() {
            let sweep = value / total * TAU;
            if angle >= start && angle < start + sweep {
                return Some(index);
            }
            start += sweep;
        }

        None
    }

    /// Appends the outline of a slice to a path, `grow` pixels further than
    /// the outer radius (to raise a hovered slice).
    pub fn slice(&self, builder: &mut path::Builder, slice: Slice, grow: f32) {
        let Slice { start, sweep } = slice;
        let center = self.center;
        let outer = self.radius + grow;
        let steps = ((sweep / TAU) * 128.0).ceil().max(2.0) as usize;
        let point = |radius: f32, angle: f32| {
            Point::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            )
        };

        builder.move_to(point(outer, start));

        for step in 1..=steps {
            builder.line_to(point(outer, start + sweep * step as f32 / steps as f32));
        }

        if self.inner > 0.0 {
            for step in (0..=steps).rev() {
                builder.line_to(point(
                    self.inner,
                    start + sweep * step as f32 / steps as f32,
                ));
            }
        } else {
            builder.line_to(center);
        }

        builder.close();
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
    fn stacking() {
        let series: [&[f32]; 2] = [&[1.0, 2.0], &[3.0]];

        assert_eq!(max(series, 2, false), 3.0);
        assert_eq!(max(series, 2, true), 4.0);
        assert_eq!(
            baselines(series, 2, true),
            vec![vec![0.0, 0.0], vec![1.0, 2.0]]
        );
        assert_eq!(
            baselines(series, 2, false),
            vec![vec![0.0, 0.0], vec![0.0, 0.0]]
        );
    }

    #[test]
    fn categories() {
        let plot = Rectangle::new(Point::ORIGIN, [100.0, 50.0].into());
        let band = Categories::band(plot, 4);
        let point = Categories::point(plot, 5);

        assert_eq!(band.position(0), 12.5);
        assert_eq!(band.at(Point::new(30.0, 10.0)), Some(1));
        assert_eq!(point.position(4), 100.0);
        assert_eq!(point.at(Point::new(30.0, 10.0)), Some(1));
        assert_eq!(point.at(Point::new(30.0, 60.0)), None);
    }

    #[test]
    fn pie_slices() {
        let plot = Rectangle::new(Point::ORIGIN, [100.0, 100.0].into());
        let pie = Pie::new(plot, 0.6);
        let values = [1.0, 1.0];

        assert_eq!(Pie::slices(&values).len(), 2);
        // Right of the center: the first half, clockwise from the top.
        assert_eq!(pie.slice_at(&values, Point::new(90.0, 50.0)), Some(0));
        assert_eq!(pie.slice_at(&values, Point::new(10.0, 50.0)), Some(1));
        // In the hole.
        assert_eq!(pie.slice_at(&values, Point::new(55.0, 50.0)), None);
    }

    #[test]
    fn tooltip_flips() {
        assert_eq!(place(10.0, 20.0, 10.0, 0.0, 100.0), 20.0);
        assert_eq!(place(90.0, 20.0, 10.0, 0.0, 100.0), 60.0);
    }
}
