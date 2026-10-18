//! Beautiful charts.
//!
//! Port of <https://ui.shadcn.com/docs/components/chart>. shadcn/ui wraps
//! Recharts; this module draws the same kinds of charts with the same
//! geometry and look:
//!
//! - [`bar`] charts, vertical or [`horizontal`](Chart::horizontal), grouped
//!   or [`stacked`](Chart::stacked), with negative values,
//! - [`line`] and [`area`] charts with natural, monotone, linear or step
//!   [curves](Chart::curve), stacked areas (also
//!   [expanded](Chart::expanded) to 100%) and [gradients](Chart::gradient),
//! - [`pie`] and donut charts, with rings, labels and a text in the middle,
//! - [`radar`] charts, with polygon or circle grids,
//! - [`radial`] bar charts.
//!
//! Like `ChartContainer`, a chart draws a faint grid, muted axis labels, the
//! `--chart-*` colors (or [`Paint`]s of your own), a legend
//! (`ChartLegendContent`) and a tooltip on hover (`ChartTooltipContent`,
//! with its indicators, labels and formatters). The scales, the stacking, the
//! curves, the sectors and the hit testing live in
//! [`shadcn_iced_core::primitive::plot`]; this module configures and draws
//! them with the theme.
//!
//! ```no_run
//! # use shadcn_iced::chart::{self, Series};
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! chart::bar(["January", "February", "March"])
//!     .series(Series::new("Desktop", [186.0, 305.0, 237.0]))
//!     .series(Series::new("Mobile", [80.0, 200.0, 120.0]))
//!     .tick_formatter(|month| month[..3].to_owned())
//!     .into()
//! # }
//! ```
use shadcn_iced_core::primitive::plot::{
    self, Bars, Categories, Linear, Offset, Pie, Sector, Spread,
};
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, Icon, Mode, Tokens, font, style, tokens};

pub use shadcn_iced_core::primitive::plot::Curve;

use iced::advanced::graphics::geometry::Renderer as _;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::alignment;
use iced::border;
use iced::widget::canvas::{
    Fill, Frame, LineDash, Path, Stroke, Style as CanvasStyle, Text, gradient,
};
use iced::widget::text::{Alignment as TextAlignment, LineHeight};
use iced::widget::{Column, Row, Space, container, row, text};
use iced::{
    Alignment, Background, Border, Color, Event, Length, Padding, Pixels, Point, Rectangle, Shadow,
    Size, Theme, Vector, mouse,
};

/// A color of a chart: a fixed color, one of the `--chart-N` tokens, or a
/// color for each [`Mode`] (the `theme` of a `ChartConfig` entry).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Paint {
    /// A fixed color.
    Color(Color),
    /// The `--chart-N` token, from 1 to 5 (`var(--chart-1)`).
    Chart(usize),
    /// A color in light themes and another in dark themes.
    Theme {
        /// The color of light themes.
        light: Color,
        /// The color of dark themes.
        dark: Color,
    },
}

impl Paint {
    /// The `--chart-N` token, from 1 to 5.
    pub const fn chart(n: usize) -> Self {
        Self::Chart(n)
    }

    /// The color of the paint in a theme.
    pub fn resolve(self, tokens: &Tokens) -> Color {
        match self {
            Paint::Color(color) => color,
            Paint::Chart(n) => tokens.palette.chart[n.saturating_sub(1) % 5],
            Paint::Theme { light, dark } => match tokens.mode {
                Mode::Light => light,
                Mode::Dark => dark,
            },
        }
    }
}

impl From<Color> for Paint {
    fn from(color: Color) -> Self {
        Self::Color(color)
    }
}

/// A series of values, one per category.
#[derive(Debug, Clone, PartialEq)]
pub struct Series {
    /// The label shown in the legend and the tooltip.
    pub label: String,
    /// The values, one per category.
    pub values: Vec<f32>,
    /// The color. Defaults to the `--chart-N` token matching the position of
    /// the series (or of the category, for pies and radial charts of a
    /// single series).
    pub color: Option<Paint>,
    /// A color for each category (`<Cell fill />`), overriding `color`.
    pub cells: Vec<Paint>,
    /// The opacity of the fill of areas and radars (`fillOpacity`).
    pub fill_opacity: Option<f32>,
    /// An icon shown instead of the color in the legend and the tooltip
    /// (`icon` of a `ChartConfig` entry).
    pub icon: Option<Icon>,
    /// The inner and outer radii of the ring of a pie (`innerRadius`,
    /// `outerRadius`), to draw several series as nested rings.
    pub radii: Option<(f32, f32)>,
}

impl Series {
    /// Creates a series.
    pub fn new(label: impl Into<String>, values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            label: label.into(),
            values: values.into_iter().collect(),
            color: None,
            cells: Vec::new(),
            fill_opacity: None,
            icon: None,
            radii: None,
        }
    }

    /// Sets the color: a [`Color`] or a [`Paint`].
    pub fn color(mut self, color: impl Into<Paint>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Sets a color for each category (`<Cell fill />`), e.g. to color bars
    /// by browser or negative values differently.
    pub fn cells(mut self, cells: impl IntoIterator<Item = impl Into<Paint>>) -> Self {
        self.cells = cells.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the opacity of the fill of an area or a radar (`fillOpacity`).
    pub fn fill_opacity(mut self, opacity: f32) -> Self {
        self.fill_opacity = Some(opacity);
        self
    }

    /// Shows an icon instead of the color in the legend and the tooltip.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the inner and outer radii of the ring of the series in a pie.
    pub fn radii(mut self, inner: f32, outer: f32) -> Self {
        self.radii = Some((inner, outer));
        self
    }
}

/// The kind of a [`Chart`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Bars (`BarChart`).
    Bar,
    /// Lines (`LineChart`).
    Line,
    /// Lines with a translucent fill (`AreaChart`).
    Area,
    /// A pie or a donut, a slice per category (`PieChart`).
    Pie,
    /// A polygon per series around a center, a corner per category
    /// (`RadarChart`).
    Radar,
    /// Concentric bars, a ring per category (`RadialBarChart`).
    Radial,
}

/// How the tooltip shows the color of a series (`indicator`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Indicator {
    /// A small square.
    #[default]
    Dot,
    /// A thin vertical bar.
    Line,
    /// A dashed vertical line.
    Dashed,
    /// Nothing (`hideIndicator`).
    None,
}

/// Where the legend goes (`verticalAlign`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegendPosition {
    /// Above the chart.
    Top,
    /// Below the chart.
    #[default]
    Bottom,
}

/// The shape of the grid of a radar or radial chart (`gridType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GridShape {
    /// Polygons through the corners of the radar.
    #[default]
    Polygon,
    /// Circles.
    Circle,
}

type Formatter = Box<dyn Fn(&str) -> String>;
type ValueFormatter = Box<dyn Fn(f32) -> String>;

/// A text in the middle of a pie or a radial chart (`<Label />`): a large
/// value above a muted caption.
#[derive(Debug, Clone, PartialEq)]
struct CenterLabel {
    value: String,
    caption: String,
    size: f32,
    offset: f32,
}

/// A chart. See the [module documentation](self).
pub struct Chart {
    kind: Kind,
    categories: Vec<String>,
    series: Vec<Series>,
    height: f32,
    margin: Padding,

    // Cartesian charts.
    horizontal: bool,
    stack: Option<Offset>,
    curve: Curve,
    radius: f32,
    fill_opacity: Option<f32>,
    gradient: bool,
    stroke_width: Option<f32>,
    dots: bool,
    dot_radius: f32,
    active_dot: f32,
    highlight: Option<usize>,
    grid: bool,
    category_axis: bool,
    value_axis: bool,
    value_ticks: usize,
    tick_margin: f32,
    tick_formatter: Option<Formatter>,
    value_tick_formatter: Option<ValueFormatter>,
    value_labels: bool,
    category_labels: bool,
    label_lines: bool,

    // Polar charts.
    inner_radius: Option<f32>,
    outer_radius: Option<f32>,
    angles: (f32, f32),
    corner_radius: f32,
    background: bool,
    track: Option<(f32, f32)>,
    grid_shape: GridShape,
    radial_lines: bool,
    grid_fill: bool,
    grid_radii: Option<Vec<f32>>,
    radius_axis: Option<f32>,
    center_label: Option<CenterLabel>,

    // Legend and tooltip.
    legend: bool,
    legend_position: LegendPosition,
    tooltip: bool,
    cursor: bool,
    indicator: Indicator,
    tooltip_label: bool,
    tooltip_label_formatter: Option<Formatter>,
    tooltip_formatter: Option<ValueFormatter>,
    tooltip_unit: Option<String>,
    tooltip_total: bool,
    default_index: Option<usize>,
}

fn chart(kind: Kind, categories: impl IntoIterator<Item = impl Into<String>>) -> Chart {
    Chart {
        kind,
        categories: categories.into_iter().map(Into::into).collect(),
        series: Vec::new(),
        height: 250.0,
        margin: Padding::new(5.0),
        horizontal: false,
        stack: None,
        curve: Curve::Natural,
        radius: 4.0,
        fill_opacity: None,
        gradient: false,
        stroke_width: None,
        dots: false,
        dot_radius: 3.0,
        active_dot: 4.0,
        highlight: None,
        grid: kind != Kind::Pie && kind != Kind::Radial,
        category_axis: !matches!(kind, Kind::Pie | Kind::Radial),
        value_axis: false,
        value_ticks: 5,
        tick_margin: 8.0,
        tick_formatter: None,
        value_tick_formatter: None,
        value_labels: false,
        category_labels: false,
        label_lines: true,
        inner_radius: None,
        outer_radius: None,
        angles: if kind == Kind::Radar {
            (90.0, -270.0)
        } else {
            (0.0, 360.0)
        },
        corner_radius: 0.0,
        background: false,
        track: None,
        grid_shape: if kind == Kind::Radial {
            GridShape::Circle
        } else {
            GridShape::Polygon
        },
        radial_lines: true,
        grid_fill: false,
        grid_radii: None,
        radius_axis: None,
        center_label: None,
        legend: true,
        legend_position: LegendPosition::Bottom,
        tooltip: true,
        cursor: true,
        indicator: Indicator::Dot,
        tooltip_label: !matches!(kind, Kind::Pie | Kind::Radial),
        tooltip_label_formatter: None,
        tooltip_formatter: None,
        tooltip_unit: None,
        tooltip_total: false,
        default_index: None,
    }
}

/// Creates a bar chart.
pub fn bar(categories: impl IntoIterator<Item = impl Into<String>>) -> Chart {
    chart(Kind::Bar, categories)
}

/// Creates a line chart.
pub fn line(categories: impl IntoIterator<Item = impl Into<String>>) -> Chart {
    chart(Kind::Line, categories)
}

/// Creates an area chart.
pub fn area(categories: impl IntoIterator<Item = impl Into<String>>) -> Chart {
    chart(Kind::Area, categories)
}

/// Creates a pie chart of a single series: one slice per category, colored
/// with the `--chart-N` tokens in order. See [`rings`] for nested rings.
pub fn pie(
    categories: impl IntoIterator<Item = impl Into<String>>,
    values: impl IntoIterator<Item = f32>,
) -> Chart {
    chart(Kind::Pie, categories).series(Series::new("", values))
}

/// Creates a pie of nested rings, one per series, a slice per category:
/// add the series with their [`Series::radii`]. The tooltip names the
/// series of the hovered ring in its [label](Chart::tooltip_label).
pub fn rings(categories: impl IntoIterator<Item = impl Into<String>>) -> Chart {
    chart(Kind::Pie, categories)
}

/// Creates a radar chart: add a series per polygon.
pub fn radar(categories: impl IntoIterator<Item = impl Into<String>>) -> Chart {
    chart(Kind::Radar, categories)
}

/// Creates a radial bar chart: a ring per category, from the center out.
/// A single series colors each ring with the `--chart-N` tokens in order;
/// several series are [`stacked`](Chart::stacked) along the rings.
pub fn radial(categories: impl IntoIterator<Item = impl Into<String>>) -> Chart {
    chart(Kind::Radial, categories)
}

impl Chart {
    /// Appends a series.
    pub fn series(mut self, series: Series) -> Self {
        self.series.push(series);
        self
    }

    /// Sets the height. Defaults to 250 (`h-[250px]`).
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Sets the margin around the plot (`margin`). Defaults to 5 on every
    /// side, like Recharts. It may be negative, e.g. to pull a value axis
    /// closer to the edge.
    pub fn margin(mut self, margin: impl Into<Padding>) -> Self {
        self.margin = margin.into();
        self
    }

    /// Lays bars out horizontally: the categories go down the left side and
    /// the bars grow to the right (`layout="vertical"`).
    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }

    /// Stacks the series of bar, area and radial charts (`stackId`).
    pub fn stacked(mut self, stacked: bool) -> Self {
        self.stack = stacked.then_some(Offset::None);
        self
    }

    /// Stacks the series and scales every category to 100%
    /// (`stackOffset="expand"`).
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.stack = if expanded {
            Some(Offset::Expand)
        } else {
            self.stack.filter(|offset| *offset != Offset::Expand)
        };
        self
    }

    /// Sets the curve of lines and areas (`type`). Defaults to
    /// [`Curve::Natural`].
    pub fn curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }

    /// Sets the radius of the corners of bars (`radius`). Defaults to 4.
    /// Stacked bars only round the ends of their stacks.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Sets the opacity of the fill of areas and radars (`fillOpacity`),
    /// unless a series sets its own. Defaults to 0.4 for areas and 0.6 for
    /// radars.
    pub fn fill_opacity(mut self, opacity: f32) -> Self {
        self.fill_opacity = Some(opacity);
        self
    }

    /// Fills areas with a vertical gradient fading out downwards
    /// (`<linearGradient>` from 80% to 10% of the color).
    pub fn gradient(mut self, gradient: bool) -> Self {
        self.gradient = gradient;
        self
    }

    /// Sets the width of lines, of the edges of areas and of the outlines of
    /// radars (`strokeWidth`). Defaults to 2 for lines, 1 for areas, and
    /// none for radars.
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = Some(width);
        self
    }

    /// Draws a dot on each value of line and radar charts (`dot`).
    pub fn dots(mut self, dots: bool) -> Self {
        self.dots = dots;
        self
    }

    /// Sets the radius of the dots (`dot={{ r }}`). Defaults to 3.
    pub fn dot_radius(mut self, radius: f32) -> Self {
        self.dot_radius = radius;
        self
    }

    /// Sets the radius of the dots of line, area and radar charts on the
    /// hovered category (`activeDot={{ r }}`). Defaults to 4.
    pub fn active_dot(mut self, radius: f32) -> Self {
        self.active_dot = radius;
        self
    }

    /// Highlights a category: its bars get a dashed outline, its slice of a
    /// pie grows by 10 (`activeIndex`, `shape`).
    pub fn highlight(mut self, index: usize) -> Self {
        self.highlight = Some(index);
        self
    }

    /// Shows or hides the grid: the lines of the values of cartesian charts
    /// (`CartesianGrid`), the rings and spokes of radar and radial charts
    /// (`PolarGrid`).
    pub fn grid(mut self, grid: bool) -> Self {
        self.grid = grid;
        self
    }

    /// Shows or hides the axis of the categories (`XAxis`, or `YAxis` when
    /// [`horizontal`](Self::horizontal)), or the categories around a radar
    /// (`PolarAngleAxis`). Shown by default.
    pub fn category_axis(mut self, axis: bool) -> Self {
        self.category_axis = axis;
        self
    }

    /// Shows or hides the axis of the values (`YAxis`, or `XAxis` when
    /// [`horizontal`](Self::horizontal)). Hidden by default.
    pub fn value_axis(mut self, axis: bool) -> Self {
        self.value_axis = axis;
        self
    }

    /// Sets how many values the value axis and the grid show (`tickCount`).
    /// Defaults to 5.
    pub fn value_ticks(mut self, ticks: usize) -> Self {
        self.value_ticks = ticks.max(2);
        self
    }

    /// Sets the space between the plot and the labels of the axes
    /// (`tickMargin`). Defaults to 8.
    pub fn tick_margin(mut self, margin: f32) -> Self {
        self.tick_margin = margin;
        self
    }

    /// Formats the labels of the categories on the axis
    /// (`<XAxis tickFormatter />`), e.g. to abbreviate them. The tooltip
    /// keeps showing the whole category.
    pub fn tick_formatter(mut self, formatter: impl Fn(&str) -> String + 'static) -> Self {
        self.tick_formatter = Some(Box::new(formatter));
        self
    }

    /// Formats the labels of the value axis (`<YAxis tickFormatter />`).
    pub fn value_tick_formatter(mut self, formatter: impl Fn(f32) -> String + 'static) -> Self {
        self.value_tick_formatter = Some(Box::new(formatter));
        self
    }

    /// Writes the values next to the bars, points and slices
    /// (`<LabelList />`): above vertical bars and points, right of
    /// horizontal bars, outside of pies.
    pub fn value_labels(mut self, labels: bool) -> Self {
        self.value_labels = labels;
        self
    }

    /// Writes the categories next to the bars, points and slices
    /// (`<LabelList dataKey="month" />`): above vertical bars (below
    /// negative ones) and points, inside horizontal bars, slices and radial
    /// bars.
    pub fn category_labels(mut self, labels: bool) -> Self {
        self.category_labels = labels;
        self
    }

    /// Draws lines from the slices of a pie to their
    /// [value labels](Self::value_labels) (`labelLine`). Shown by default.
    pub fn label_lines(mut self, lines: bool) -> Self {
        self.label_lines = lines;
        self
    }

    /// Draws a donut instead of a pie (`innerRadius={60}`).
    pub fn donut(mut self, donut: bool) -> Self {
        self.inner_radius = donut.then_some(60.0);
        self
    }

    /// Sets the inner radius of pies and radial charts (`innerRadius`).
    /// Defaults to 0.
    pub fn inner_radius(mut self, radius: f32) -> Self {
        self.inner_radius = Some(radius);
        self
    }

    /// Sets the outer radius of pies, radars and radial charts
    /// (`outerRadius`). Defaults to 80% of the largest radius that fits.
    pub fn outer_radius(mut self, radius: f32) -> Self {
        self.outer_radius = Some(radius);
        self
    }

    /// Sets the angles where pies and radial bars start and end, in degrees
    /// counterclockwise from the right (`startAngle`, `endAngle`). Defaults
    /// to 0 and 360.
    pub fn angles(mut self, start: f32, end: f32) -> Self {
        self.angles = (start, end);
        self
    }

    /// Rounds the corners of radial bars (`cornerRadius`).
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Draws the whole track of each radial bar behind it (`background`).
    pub fn background(mut self, background: bool) -> Self {
        self.background = background;
        self
    }

    /// Draws a muted ring all around the center between two radii, behind
    /// radial bars (a `PolarGrid` filled with `fill-muted`).
    pub fn track(mut self, inner: f32, outer: f32) -> Self {
        self.track = Some((inner, outer));
        self
    }

    /// Sets the shape of the grid of radar and radial charts (`gridType`).
    pub fn grid_shape(mut self, shape: GridShape) -> Self {
        self.grid_shape = shape;
        self
    }

    /// Shows or hides the spokes of the grid of radar and radial charts
    /// (`radialLines`). Shown by default.
    pub fn radial_lines(mut self, lines: bool) -> Self {
        self.radial_lines = lines;
        self
    }

    /// Fills the grid of a radar with the color of its first series
    /// (`fill-(--color-desktop) opacity-20`).
    pub fn grid_fill(mut self, fill: bool) -> Self {
        self.grid_fill = fill;
        self
    }

    /// Draws the rings of the grid at these radii, in pixels
    /// (`polarRadius`), instead of at the values of the radius axis.
    pub fn grid_radii(mut self, radii: impl IntoIterator<Item = f32>) -> Self {
        self.grid_radii = Some(radii.into_iter().collect());
        self
    }

    /// Writes the values of a radar along a spoke at `angle` degrees
    /// counterclockwise from the right (`<PolarRadiusAxis angle />`).
    pub fn radius_axis(mut self, angle: f32) -> Self {
        self.radius_axis = Some(angle);
        self
    }

    /// Writes a large value above a muted caption in the middle of a pie or
    /// a radial chart (`<Label />`), e.g. a total.
    pub fn center_label(mut self, value: impl Into<String>, caption: impl Into<String>) -> Self {
        self.center_label = Some(CenterLabel {
            value: value.into(),
            caption: caption.into(),
            size: style::text::XL3,
            offset: 0.0,
        });
        self
    }

    /// Sets the font size of the value of the
    /// [center label](Self::center_label). Defaults to 30 (`text-3xl`).
    pub fn center_label_size(mut self, size: f32) -> Self {
        if let Some(label) = &mut self.center_label {
            label.size = size;
        }
        self
    }

    /// Moves the [center label](Self::center_label) up (negative) or down.
    pub fn center_label_offset(mut self, offset: f32) -> Self {
        if let Some(label) = &mut self.center_label {
            label.offset = offset;
        }
        self
    }

    /// Shows or hides the legend (`ChartLegend`).
    pub fn legend(mut self, legend: bool) -> Self {
        self.legend = legend;
        self
    }

    /// Puts the legend above or below the chart (`verticalAlign`).
    pub fn legend_position(mut self, position: LegendPosition) -> Self {
        self.legend_position = position;
        self
    }

    /// Shows or hides the tooltip (`ChartTooltip`).
    pub fn tooltip(mut self, tooltip: bool) -> Self {
        self.tooltip = tooltip;
        self
    }

    /// Shows or hides the cursor highlighting the hovered category while the
    /// tooltip shows (`<ChartTooltip cursor={false} />` hides it).
    pub fn cursor(mut self, cursor: bool) -> Self {
        self.cursor = cursor;
        self
    }

    /// Sets how the tooltip shows the color of each series (`indicator`,
    /// `hideIndicator`).
    pub fn tooltip_indicator(mut self, indicator: Indicator) -> Self {
        self.indicator = indicator;
        self
    }

    /// Shows or hides the label of the tooltip: the category, or the series
    /// of a pie (`hideLabel`). Shown by default, except for pies and radial
    /// charts.
    pub fn tooltip_label(mut self, label: bool) -> Self {
        self.tooltip_label = label;
        self
    }

    /// Formats the label of the tooltip (`labelFormatter`), e.g. to write a
    /// date in full.
    pub fn tooltip_label_formatter(mut self, formatter: impl Fn(&str) -> String + 'static) -> Self {
        self.tooltip_label = true;
        self.tooltip_label_formatter = Some(Box::new(formatter));
        self
    }

    /// Formats the values of the tooltip (`formatter`). Defaults to grouping
    /// the digits like `toLocaleString()`.
    pub fn tooltip_formatter(mut self, formatter: impl Fn(f32) -> String + 'static) -> Self {
        self.tooltip_formatter = Some(Box::new(formatter));
        self
    }

    /// Writes a muted unit after the values of the tooltip, like `kcal`.
    pub fn tooltip_unit(mut self, unit: impl Into<String>) -> Self {
        self.tooltip_unit = Some(unit.into());
        self
    }

    /// Adds the total of the values of the category to the tooltip.
    pub fn tooltip_total(mut self, total: bool) -> Self {
        self.tooltip_total = total;
        self
    }

    /// Shows the tooltip of a category while the pointer is away
    /// (`defaultIndex`).
    pub fn default_index(mut self, index: usize) -> Self {
        self.default_index = Some(index);
        self
    }
}

/// Formats a value like `toLocaleString()` does in English.
fn format_value(value: f32) -> String {
    let rounded = (value * 1000.0).round() / 1000.0;
    let formatted = format!("{rounded}");

    let (sign, formatted) = match formatted.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", formatted.as_str()),
    };
    let (integer, decimals) = formatted.split_at(formatted.find('.').unwrap_or(formatted.len()));

    let mut grouped = String::new();
    for (index, digit) in integer.chars().enumerate() {
        if index > 0 && (integer.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }

    format!("{sign}{grouped}{decimals}")
}

/// Formats a tick of a value axis like SVG text shows a number.
fn format_tick(value: f32) -> String {
    let rounded = (value * 1000.0).round() / 1000.0;
    format!("{rounded}")
}

/// The height of the category axis (`XAxis`).
const X_AXIS: f32 = 30.0;
/// The width of the value axis (`YAxis`).
const Y_AXIS: f32 = 60.0;
/// The size of the (hidden) tick lines, which still offsets the labels.
const TICK_SIZE: f32 = 6.0;
/// The distance between the tooltip and the hovered point.
const TOOLTIP_OFFSET: f32 = 10.0;
/// The distance between the ring of a radar and the labels of its corners.
const ANGLE_TICK: f32 = 8.0;

/// Where the baseline of a line of text goes relative to its anchor, like
/// the `verticalAnchor` of the `Text` of Recharts.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Baseline {
    /// On the anchor (`end`).
    End,
    /// Centered on the anchor (`middle`, `dy="0.355em"`).
    Middle,
    /// Below the anchor (`start`, `dy="0.71em"`).
    Start,
    /// Centered like `dominant-baseline="middle"`.
    Central,
}

impl Baseline {
    fn offset(self, size: f32) -> f32 {
        size * match self {
            Baseline::End => 0.0,
            Baseline::Middle => 0.355,
            Baseline::Start => 0.71,
            Baseline::Central => 0.3,
        }
    }
}

/// A line of text to draw on the plot.
struct Label {
    content: String,
    position: Point,
    color: Color,
    size: f32,
    font: iced::Font,
    align: TextAlignment,
    baseline: Baseline,
    rotation: f32,
}

impl Label {
    fn new(content: impl Into<String>, position: Point, color: Color) -> Self {
        Self {
            content: content.into(),
            position,
            color,
            size: style::text::XS,
            font: font::sans(),
            align: TextAlignment::Center,
            baseline: Baseline::End,
            rotation: 0.0,
        }
    }

    fn align(mut self, align: TextAlignment) -> Self {
        self.align = align;
        self
    }

    fn baseline(mut self, baseline: Baseline) -> Self {
        self.baseline = baseline;
        self
    }

    fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    fn font(mut self, font: iced::Font) -> Self {
        self.font = font;
        self
    }

    /// Rotates the text around its anchor, in degrees clockwise.
    fn rotate(mut self, degrees: f32) -> Self {
        self.rotation = degrees;
        self
    }

    fn draw(self, frame: &mut Frame) {
        let size = self.size;
        // The baseline of a Geist line of text lies 1.005em below its top
        // when the line is 1.3em high.
        let line_height = size * 1.3;
        let top = self.baseline.offset(size) - 1.005 * size;

        let text = Text {
            content: self.content,
            position: Point::new(0.0, top),
            color: self.color,
            size: Pixels(size),
            line_height: LineHeight::Absolute(Pixels(line_height)),
            font: self.font,
            align_x: self.align,
            align_y: alignment::Vertical::Top,
            ..Text::default()
        };

        if self.rotation == 0.0 {
            frame.fill_text(Text {
                position: self.position + Vector::new(0.0, top),
                ..text
            });
        } else {
            frame.with_save(|frame| {
                frame.translate(Vector::new(self.position.x, self.position.y));
                frame.rotate(self.rotation.to_radians());
                frame.fill_text(text);
            });
        }
    }
}

/// The parts of a chart that depend on its size.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Polar {
    center: Point,
    inner: f32,
    outer: f32,
}

impl Chart {
    fn count(&self) -> usize {
        self.categories.len()
    }

    fn is_cartesian(&self) -> bool {
        matches!(self.kind, Kind::Bar | Kind::Line | Kind::Area)
    }

    /// Whether each category has its own color: the slices of a pie and the
    /// rings of a radial chart of a single series.
    fn colors_categories(&self) -> bool {
        match self.kind {
            Kind::Pie => true,
            Kind::Radial => self.series.len() == 1,
            _ => false,
        }
    }

    /// The color of a series, or of a category of a series.
    fn color(&self, t: &Tokens, series: usize, category: Option<usize>) -> Color {
        let Some(data) = self.series.get(series) else {
            return t.palette.chart[series % 5];
        };

        if let Some(paint) = category.and_then(|category| data.cells.get(category)) {
            return paint.resolve(t);
        }

        if self.colors_categories() {
            return t.palette.chart[category.unwrap_or(0) % 5];
        }

        data.color
            .map(|paint| paint.resolve(t))
            .unwrap_or(t.palette.chart[series % 5])
    }

    fn opacity_of(&self, series: usize) -> f32 {
        self.series
            .get(series)
            .and_then(|series| series.fill_opacity)
            .or(self.fill_opacity)
            .unwrap_or(match self.kind {
                Kind::Radar => 0.6,
                _ => 0.4,
            })
    }

    fn stroke(&self) -> f32 {
        self.stroke_width.unwrap_or(match self.kind {
            Kind::Line => 2.0,
            Kind::Area => 1.0,
            _ => 0.0,
        })
    }

    fn tick(&self, category: &str) -> String {
        match &self.tick_formatter {
            Some(formatter) => formatter(category),
            None => category.to_owned(),
        }
    }

    fn value_tick(&self, value: f32) -> String {
        match &self.value_tick_formatter {
            Some(formatter) => formatter(value),
            None => format_tick(value),
        }
    }

    fn format(&self, value: f32) -> String {
        match &self.tooltip_formatter {
            Some(formatter) => formatter(value),
            None => format_value(value),
        }
    }

    fn values(&self) -> impl Iterator<Item = &[f32]> {
        self.series.iter().map(|series| series.values.as_slice())
    }

    /// Where each series starts and ends at each category: stacked, or from
    /// 0.
    fn ranges(&self) -> Vec<Vec<[f32; 2]>> {
        let count = self.count();
        let stack = self.stack.filter(|_| self.kind != Kind::Line);

        match stack {
            Some(offset) => plot::stack(self.values(), count, offset),
            None => self
                .series
                .iter()
                .map(|series| {
                    (0..count)
                        .map(|i| [0.0, plot::value(&series.values, i)])
                        .collect()
                })
                .collect(),
        }
    }

    /// The values of the ticks of the value axis, from the lowest.
    fn domain(&self) -> Vec<f32> {
        let stack = self.stack.filter(|_| self.kind != Kind::Line);

        if stack == Some(Offset::Expand) {
            return plot::ticks(0.0, 1.0, self.value_ticks);
        }

        let (min, max) = plot::extent(self.values(), self.count(), stack);

        plot::ticks(min, max, self.value_ticks)
    }

    /// The rectangle where the data goes in a chart of `size`, with a legend
    /// `legend` high.
    fn plot(&self, size: Size, legend: f32) -> Rectangle {
        let m = self.margin;
        let mut plot = Rectangle {
            x: m.left,
            y: m.top,
            width: size.width - m.left - m.right,
            height: size.height - m.top - m.bottom - legend,
        };

        if self.legend_position == LegendPosition::Top {
            plot.y += legend;
        }

        if self.is_cartesian() {
            let (left, bottom) = if self.horizontal {
                (self.category_axis, self.value_axis)
            } else {
                (self.value_axis, self.category_axis)
            };

            if left {
                plot.x += Y_AXIS;
                plot.width -= Y_AXIS;
            }

            if bottom {
                plot.height -= X_AXIS;
            }
        }

        plot.width = plot.width.max(0.0);
        plot.height = plot.height.max(0.0);
        plot
    }

    /// The scale of the categories: bands for bars, and spread edge to edge
    /// for lines and areas (a point scale).
    fn categories(&self, plot: Rectangle) -> Categories {
        let spread = match self.kind {
            Kind::Line | Kind::Area => Spread::Point,
            _ => Spread::Band,
        };
        let scale = Categories::new(plot, self.count(), spread);

        if self.horizontal {
            scale.vertical()
        } else {
            scale
        }
    }

    fn scale(&self, plot: Rectangle, domain: &[f32]) -> Linear {
        let min = domain.first().copied().unwrap_or(0.0);
        let max = domain.last().copied().unwrap_or(1.0);

        if self.horizontal {
            Linear::horizontal(plot, min, max)
        } else {
            Linear::vertical(plot, min, max)
        }
    }

    fn polar(&self, plot: Rectangle) -> Polar {
        let max = plot.width.min(plot.height) / 2.0;

        Polar {
            center: plot.center(),
            inner: self.inner_radius.unwrap_or(0.0),
            outer: self.outer_radius.unwrap_or(max * 0.8),
        }
    }

    /// The radii of the ring of a series of a pie.
    fn ring(&self, polar: Polar, series: usize) -> Pie {
        let (inner, outer) = self
            .series
            .get(series)
            .and_then(|series| series.radii)
            .unwrap_or((polar.inner, polar.outer));

        Pie {
            center: polar.center,
            radius: outer,
            inner,
        }
    }

    /// The angle of a corner of a radar.
    fn corner(&self, index: usize) -> f32 {
        let (start, end) = self.angles;
        start + (end - start) * index as f32 / self.count().max(1) as f32
    }

    /// The largest value of a radial chart, at the end of its angle. Like
    /// Recharts, it ignores stacking: stacks longer than it end at the end
    /// angle.
    fn radial_max(&self) -> f32 {
        let max = plot::extent(self.values(), self.count(), None).1;
        if max > 0.0 { max } else { 1.0 }
    }

    /// The radial bars of each series at each category.
    fn radial_bars(&self, polar: Polar) -> Vec<Vec<Sector>> {
        let count = self.count();
        let band = (polar.outer - polar.inner) / count.max(1) as f32;
        let groups = if self.stack.is_some() {
            1
        } else {
            self.series.len()
        };
        let bars = Bars::new(band, groups, 0.1, 4.0);
        let (start, end) = self.angles;
        let max = self.radial_max();
        let angle = |value: f32| start + (end - start) * value.min(max) / max;

        self.ranges()
            .into_iter()
            .enumerate()
            .map(|(series, ranges)| {
                ranges
                    .into_iter()
                    .enumerate()
                    .map(|(category, [low, high])| {
                        let slot = if self.stack.is_some() { 0 } else { series };
                        let inner = bars.x(polar.inner + band * category as f32, slot);

                        Sector {
                            center: polar.center,
                            inner,
                            outer: inner + bars.width,
                            start: angle(low),
                            end: angle(high),
                        }
                    })
                    .collect()
            })
            .collect()
    }

    /// What the tooltip shows when the pointer is over a point: a category,
    /// or a slice of a ring of a pie (`series * count + category`).
    fn hovered(&self, plot: Rectangle, point: Point) -> Option<usize> {
        let count = self.count();

        match self.kind {
            Kind::Bar | Kind::Line | Kind::Area => self.categories(plot).at(point),
            Kind::Pie => {
                let polar = self.polar(plot);
                let (start, end) = self.angles;

                self.series.iter().enumerate().find_map(|(index, series)| {
                    self.ring(polar, index)
                        .sector_at(&series.values, start, end, point)
                        .map(|category| index * count + category)
                })
            }
            Kind::Radar => {
                let polar = self.polar(plot);

                if count == 0
                    || plot::distance(polar.center, point) > polar.outer + ANGLE_TICK * 2.0
                {
                    return None;
                }

                let step = (self.angles.1 - self.angles.0) / count as f32;
                let turns = (plot::angle(polar.center, point) - self.angles.0) / step;

                Some((turns.round().rem_euclid(count as f32)) as usize % count)
            }
            Kind::Radial => {
                let polar = self.polar(plot);
                let distance = plot::distance(polar.center, point);

                if count == 0 || distance < polar.inner || distance > polar.outer {
                    return None;
                }

                let band = (polar.outer - polar.inner) / count as f32;

                Some((((distance - polar.inner) / band) as usize).min(count - 1))
            }
        }
    }

    /// The tooltips of the chart, by hovered target.
    fn targets(&self) -> usize {
        match self.kind {
            Kind::Pie => self.count() * self.series.len().max(1),
            _ => self.count(),
        }
    }

    /// Where the tooltip goes: past the hovered category (or the pointer)
    /// by 10, flipped to stay inside the plot, like Recharts.
    fn tooltip_position(
        &self,
        plot: Rectangle,
        hovered: usize,
        cursor: Option<Point>,
        size: Size,
    ) -> Point {
        let anchor = if self.is_cartesian() {
            let category = self.categories(plot).position(hovered);
            // Without a pointer (`defaultIndex`), halfway between the top of
            // the plot and the bottom of the chart, like Recharts'
            // `combineCoordinateForDefaultIndex`.
            let width = plot.x + plot.width + self.margin.right;
            let cursor = cursor.unwrap_or(Point::new(
                (plot.x + width) / 2.0,
                (plot.y + self.height) / 2.0,
            ));

            if self.horizontal {
                Point::new(cursor.x, category)
            } else {
                Point::new(category, cursor.y)
            }
        } else if self.kind == Kind::Pie {
            // The middle of the slice, like the `tooltipPosition` of the
            // sectors of Recharts.
            let polar = self.polar(plot);
            let count = self.count().max(1);
            let (series, category) = (hovered / count, hovered % count);
            let ring = self.ring(polar, series);
            let (start, end) = self.angles;

            self.series
                .get(series)
                .and_then(|series| {
                    ring.sectors(&series.values, start, end)
                        .get(category)
                        .copied()
                })
                .map(|sector| {
                    plot::polar(
                        polar.center,
                        (sector.inner + sector.outer) / 2.0,
                        sector.middle(),
                    )
                })
                .unwrap_or_else(|| plot.center())
        } else {
            // The pointer snapped to the hovered category: along the angle of
            // the corner of a radar, at the middle of the ring of a radial
            // chart, like the `activeCoordinate` of Recharts.
            let polar = self.polar(plot);

            match cursor {
                Some(cursor) if self.kind == Kind::Radar => plot::polar(
                    polar.center,
                    plot::distance(polar.center, cursor),
                    self.corner(hovered),
                ),
                Some(cursor) if self.kind == Kind::Radial => {
                    let band = (polar.outer - polar.inner) / self.count().max(1) as f32;

                    plot::polar(
                        polar.center,
                        polar.inner + band * (hovered as f32 + 0.5),
                        plot::angle(polar.center, cursor),
                    )
                }
                Some(cursor) => cursor,
                None => plot.center(),
            }
        };

        Point::new(
            plot::place(anchor.x, size.width, TOOLTIP_OFFSET, plot.x, plot.width),
            plot::place(anchor.y, size.height, TOOLTIP_OFFSET, plot.y, plot.height),
        )
    }

    fn draw_plot(
        &self,
        frame: &mut Frame,
        t: &Tokens,
        size: Size,
        legend: f32,
        hovered: Option<usize>,
    ) {
        let plot = self.plot(size, legend);

        match self.kind {
            Kind::Bar | Kind::Line | Kind::Area => self.draw_cartesian(frame, t, plot, hovered),
            Kind::Pie => self.draw_pie(frame, t, plot),
            Kind::Radar => self.draw_radar(frame, t, plot, hovered),
            Kind::Radial => self.draw_radial(frame, t, plot),
        }

        if let Some(label) = &self.center_label {
            let center = plot.center() + Vector::new(0.0, label.offset);
            let p = t.palette;

            // `fill-foreground text-3xl font-bold`, and a muted caption 24
            // below, both with `dominant-baseline="middle"`.
            Label::new(&label.value, center, p.foreground)
                .size(label.size)
                .font(font::bold())
                .baseline(Baseline::Central)
                .draw(frame);
            Label::new(
                &label.caption,
                center + Vector::new(0.0, 24.0),
                p.muted_foreground,
            )
            .baseline(Baseline::Central)
            .draw(frame);
        }
    }

    fn draw_cartesian(
        &self,
        frame: &mut Frame,
        t: &Tokens,
        plot: Rectangle,
        hovered: Option<usize>,
    ) {
        let p = t.palette;
        let count = self.count();

        if count == 0 {
            return;
        }

        let domain = self.domain();
        let scale = self.scale(plot, &domain);
        let categories = self.categories(plot);

        // `CartesianGrid vertical={false}` (or `horizontal={false}` for
        // horizontal bars): a line per value (`stroke-border/50`).
        if self.grid {
            for value in &domain {
                let at = scale.position(*value);
                let line = if self.horizontal {
                    Path::line(Point::new(at, plot.y), Point::new(at, plot.y + plot.height))
                } else {
                    Path::line(Point::new(plot.x, at), Point::new(plot.x + plot.width, at))
                };

                frame.stroke(
                    &line,
                    Stroke::default()
                        .with_color(alpha(p.border, 0.5))
                        .with_width(1.0),
                );
            }
        }

        if let Some(hovered) = hovered.filter(|_| self.cursor) {
            if self.kind == Kind::Bar {
                // `.recharts-tooltip-cursor`: `fill-muted`
                let (start, band) = (categories.band_start(hovered), categories.band_width());

                if self.horizontal {
                    frame.fill_rectangle(
                        Point::new(plot.x, start),
                        Size::new(plot.width, band),
                        p.muted,
                    );
                } else {
                    frame.fill_rectangle(
                        Point::new(start, plot.y),
                        Size::new(band, plot.height),
                        p.muted,
                    );
                }
            } else {
                let x = categories.position(hovered);
                frame.stroke(
                    &Path::line(Point::new(x, plot.y), Point::new(x, plot.y + plot.height)),
                    Stroke::default().with_color(p.border).with_width(1.0),
                );
            }
        }

        let mut labels = Vec::new();

        match self.kind {
            Kind::Bar => self.draw_bars(frame, t, &scale, &categories, &mut labels),
            _ => self.draw_lines(frame, t, &scale, &categories, hovered, &mut labels),
        }

        self.draw_axes(frame, t, plot, &domain, &scale, &categories);

        for label in labels {
            label.draw(frame);
        }
    }

    fn draw_bars(
        &self,
        frame: &mut Frame,
        t: &Tokens,
        scale: &Linear,
        categories: &Categories,
        labels: &mut Vec<Label>,
    ) {
        let p = t.palette;
        let stacked = self.stack.is_some();
        // `barCategoryGap="10%"`, `barGap={4}`, and whole widths.
        let groups = if stacked { 1 } else { self.series.len() };
        let bars = Bars::new(categories.band_width(), groups, 0.1, 4.0);
        let ranges = self.ranges();

        for category in 0..self.count() {
            // The segments of the category, to round the ends of stacks.
            let segments: Vec<(usize, [f32; 2])> = ranges
                .iter()
                .enumerate()
                .map(|(series, ranges)| (series, ranges[category]))
                .filter(|(_, [low, high])| low != high)
                .collect();

            for &(series, [low, high]) in &segments {
                let positive = high >= low;
                let (base, end) = if stacked {
                    let same: Vec<usize> = segments
                        .iter()
                        .filter(|(_, [l, h])| (h >= l) == positive)
                        .map(|(series, _)| *series)
                        .collect();
                    (same.first() == Some(&series), same.last() == Some(&series))
                } else {
                    (true, true)
                };

                let slot = if stacked { 0 } else { series };
                let along = bars.x(categories.band_start(category), slot);
                let (from, to) = (scale.position(low), scale.position(high));

                let (top_left, size) = if self.horizontal {
                    (
                        Point::new(from.min(to), along),
                        Size::new((to - from).abs(), bars.width),
                    )
                } else {
                    (
                        Point::new(along, from.min(to)),
                        Size::new(bars.width, (to - from).abs()),
                    )
                };

                let r = self
                    .radius
                    .min(size.width / 2.0)
                    .min(size.height / 2.0)
                    .max(0.0);
                let (r_base, r_end) = (if base { r } else { 0.0 }, if end { r } else { 0.0 });

                // The end of a bar is its top (or right), or its bottom (or
                // left) when it goes below 0.
                let radius = match (self.horizontal, positive) {
                    (false, true) => border::Radius {
                        top_left: r_end,
                        top_right: r_end,
                        bottom_right: r_base,
                        bottom_left: r_base,
                    },
                    (false, false) => border::Radius {
                        top_left: r_base,
                        top_right: r_base,
                        bottom_right: r_end,
                        bottom_left: r_end,
                    },
                    (true, true) => border::Radius {
                        top_left: r_base,
                        top_right: r_end,
                        bottom_right: r_end,
                        bottom_left: r_base,
                    },
                    (true, false) => border::Radius {
                        top_left: r_end,
                        top_right: r_base,
                        bottom_right: r_base,
                        bottom_left: r_end,
                    },
                };

                let color = self.color(t, series, Some(category));
                let path = Path::rounded_rectangle(top_left, size, radius);

                if self.highlight == Some(category) {
                    // `fillOpacity={0.8}`, `strokeDasharray={4}`.
                    frame.fill(&path, alpha(color, 0.8));
                    frame.stroke(
                        &path,
                        Stroke {
                            line_dash: LineDash {
                                segments: &[4.0, 4.0],
                                offset: 4,
                            },
                            ..Stroke::default().with_color(color).with_width(2.0)
                        },
                    );
                } else {
                    frame.fill(&path, color);
                }

                let rect = Rectangle::new(top_left, size);
                let value = plot::value(&self.series[series].values, category);
                let name = &self.categories[category];

                if self.horizontal {
                    // `position="insideLeft"` in `--color-label`, and
                    // `position="right"`, both 8 away.
                    let middle = rect.center_y();

                    if self.category_labels {
                        labels.push(
                            Label::new(
                                self.tick(name),
                                Point::new(rect.x + 8.0, middle),
                                p.background,
                            )
                            .align(TextAlignment::Left)
                            .baseline(Baseline::Middle),
                        );
                    }

                    if self.value_labels && end {
                        labels.push(
                            Label::new(
                                format_value(value),
                                Point::new(rect.x + rect.width + 8.0, middle),
                                p.foreground,
                            )
                            .align(TextAlignment::Left)
                            .baseline(Baseline::Middle),
                        );
                    }
                } else if end {
                    // `position="top"`: above the bar, or below it when it
                    // goes below 0.
                    let mut push = |content: String, color: Color, offset: f32| {
                        let label = if positive {
                            Label::new(content, Point::new(rect.center_x(), rect.y - offset), color)
                        } else {
                            Label::new(
                                content,
                                Point::new(rect.center_x(), rect.y + rect.height + offset),
                                color,
                            )
                            .baseline(Baseline::Start)
                        };

                        labels.push(label);
                    };

                    if self.value_labels {
                        push(format_value(value), p.foreground, 12.0);
                    }

                    if self.category_labels {
                        push(self.tick(name), color, 5.0);
                    }
                }
            }
        }
    }

    fn draw_lines(
        &self,
        frame: &mut Frame,
        t: &Tokens,
        scale: &Linear,
        categories: &Categories,
        hovered: Option<usize>,
        labels: &mut Vec<Label>,
    ) {
        let p = t.palette;
        let count = self.count();
        let ranges = self.ranges();
        let stroke = self.stroke();
        let mut active = Vec::new();

        for (index, ranges) in ranges.iter().enumerate() {
            let color = self.color(t, index, None);
            let position =
                |i: usize, value: f32| Point::new(categories.position(i), scale.position(value));

            let points: Vec<Point> = (0..count).map(|i| position(i, ranges[i][1])).collect();

            if self.kind == Kind::Area {
                let lower: Vec<Point> = (0..count)
                    .rev()
                    .map(|i| position(i, ranges[i][0]))
                    .collect();

                // `fillOpacity={0.4}`, down to the series below.
                let fill = Path::new(|builder| {
                    plot::curve(builder, &points, self.curve, true);
                    plot::curve(builder, &lower, self.curve, false);
                    builder.close();
                });
                let opacity = self.opacity_of(index);

                if self.gradient {
                    // `<linearGradient x1="0" y1="0" x2="0" y2="1">` over
                    // the bounds of the area, from 80% at 5% to 10% at 95%.
                    let (top, bottom) = points
                        .iter()
                        .chain(&lower)
                        .fold((f32::MAX, f32::MIN), |(top, bottom), point| {
                            (top.min(point.y), bottom.max(point.y))
                        });
                    let x = points.first().map_or(0.0, |point| point.x);
                    let gradient = gradient::Linear::new(Point::new(x, top), Point::new(x, bottom))
                        .add_stop(0.05, alpha(color, 0.8 * opacity))
                        .add_stop(0.95, alpha(color, 0.1 * opacity));

                    frame.fill(
                        &fill,
                        Fill {
                            style: CanvasStyle::Gradient(gradient.into()),
                            ..Fill::default()
                        },
                    );
                } else {
                    frame.fill(&fill, alpha(color, opacity));
                }
            }

            // `type="natural"`, `strokeWidth={2}` for lines.
            if stroke > 0.0 {
                frame.stroke(
                    &Path::new(|builder| plot::curve(builder, &points, self.curve, true)),
                    Stroke::default().with_color(color).with_width(stroke),
                );
            }

            for (i, point) in points.iter().enumerate() {
                let cell = self.color(t, index, Some(i));

                // `dot`: a circle of radius 3 stroked with the line.
                if self.dots {
                    frame.fill(&Path::circle(*point, self.dot_radius + stroke / 2.0), cell);
                }

                // `<LabelList position="top" offset={12} />`
                if self.value_labels {
                    let value = plot::value(&self.series[index].values, i);
                    labels.push(Label::new(
                        format_value(value),
                        *point - Vector::new(0.0, 12.0),
                        p.foreground,
                    ));
                }

                if self.category_labels {
                    labels.push(Label::new(
                        self.tick(&self.categories[i]),
                        *point - Vector::new(0.0, 12.0),
                        p.foreground,
                    ));
                }
            }

            if let Some(hovered) = hovered {
                active.push((points[hovered], self.color(t, index, Some(hovered))));
            }
        }

        // `activeDot`, over every series.
        for (point, color) in active {
            frame.fill(&Path::circle(point, self.active_dot), color);
        }
    }

    fn draw_axes(
        &self,
        frame: &mut Frame,
        t: &Tokens,
        plot: Rectangle,
        domain: &[f32],
        scale: &Linear,
        categories: &Categories,
    ) {
        let color = t.palette.muted_foreground;

        // Labels along the bottom (`XAxis`, `tickLine={false}
        // axisLine={false}`), with their baseline `0.71em` below the tick.
        let bottom = |frame: &mut Frame, x: f32, content: String| {
            Label::new(
                content,
                Point::new(x, plot.y + plot.height + TICK_SIZE + self.tick_margin),
                color,
            )
            .baseline(Baseline::Start)
            .draw(frame);
        };

        // Labels along the left side (`YAxis`), right aligned.
        let left = |frame: &mut Frame, y: f32, content: String| {
            Label::new(
                content,
                Point::new(plot.x - TICK_SIZE - self.tick_margin, y),
                color,
            )
            .align(TextAlignment::Right)
            .baseline(Baseline::Middle)
            .draw(frame);
        };

        if self.category_axis {
            let skip = if self.horizontal {
                categories.label_step(20.0)
            } else {
                categories.label_step(48.0)
            };

            for (index, category) in self.categories.iter().enumerate() {
                if index % skip != 0 {
                    continue;
                }

                let at = categories.position(index);

                if self.horizontal {
                    left(frame, at, self.tick(category));
                } else {
                    bottom(frame, at, self.tick(category));
                }
            }
        }

        if self.value_axis {
            for value in domain {
                let at = scale.position(*value);

                if self.horizontal {
                    bottom(frame, at, self.value_tick(*value));
                } else {
                    left(frame, at, self.value_tick(*value));
                }
            }
        }
    }

    fn draw_pie(&self, frame: &mut Frame, t: &Tokens, plot: Rectangle) {
        let p = t.palette;
        let polar = self.polar(plot);
        let (start, end) = self.angles;
        let mut labels = Vec::new();

        for (index, series) in self.series.iter().enumerate() {
            let ring = self.ring(polar, index);

            for (category, sector) in ring
                .sectors(&series.values, start, end)
                .into_iter()
                .enumerate()
            {
                if sector.start == sector.end {
                    continue;
                }

                let color = self.color(t, index, Some(category));
                let grow = if self.highlight == Some(category) {
                    10.0
                } else {
                    0.0
                };
                let sector = Sector {
                    outer: sector.outer + grow,
                    ..sector
                };

                frame.fill(&Path::new(|builder| sector.path(builder, 0.0)), color);

                let middle = sector.middle();
                let value = plot::value(&series.values, category);

                // `label`: the value 20 past the slice, joined to it by a
                // line of its color (`labelLine`).
                if self.value_labels {
                    let anchor = plot::polar(polar.center, sector.outer + 20.0, middle);

                    if self.label_lines {
                        frame.stroke(
                            &Path::line(plot::polar(polar.center, sector.outer, middle), anchor),
                            Stroke::default().with_color(color).with_width(1.0),
                        );
                    }

                    let align = if anchor.x > polar.center.x + 0.5 {
                        TextAlignment::Left
                    } else if anchor.x < polar.center.x - 0.5 {
                        TextAlignment::Right
                    } else {
                        TextAlignment::Center
                    };

                    labels.push(Label::new(format_value(value), anchor, p.foreground).align(align));
                }

                // `<LabelList dataKey="browser" />`: the category in the
                // middle of the slice, in `fill-background`.
                if self.category_labels {
                    let anchor =
                        plot::polar(polar.center, (sector.inner + sector.outer) / 2.0, middle);

                    labels.push(
                        Label::new(self.tick(&self.categories[category]), anchor, p.background)
                            .baseline(Baseline::Central),
                    );
                }
            }
        }

        for label in labels {
            label.draw(frame);
        }
    }

    fn draw_radar(&self, frame: &mut Frame, t: &Tokens, plot: Rectangle, hovered: Option<usize>) {
        let p = t.palette;
        let count = self.count();

        if count == 0 {
            return;
        }

        let polar = self.polar(plot);
        let domain = plot::ticks(0.0, plot::extent(self.values(), count, None).1, 5);
        let max = domain.last().copied().unwrap_or(1.0);
        let radius = |value: f32| polar.outer * value / max;
        let corners: Vec<f32> = (0..count).map(|i| self.corner(i)).collect();

        // `PolarGrid`: a ring per value of the radius axis (`stroke-border`),
        // and a spoke per corner.
        if self.grid {
            let rings: Vec<f32> = match &self.grid_radii {
                Some(radii) => radii.clone(),
                None => domain.iter().map(|value| radius(*value)).collect(),
            };
            let ring = |r: f32| match self.grid_shape {
                GridShape::Circle => Path::circle(polar.center, r),
                GridShape::Polygon => Path::new(|builder| {
                    for (i, angle) in corners.iter().enumerate() {
                        let point = plot::polar(polar.center, r, *angle);
                        if i == 0 {
                            builder.move_to(point);
                        } else {
                            builder.line_to(point);
                        }
                    }
                    builder.close();
                }),
            };
            // `fill-(--color-desktop) opacity-20`: the whole grid, filled
            // with the color of the first series, at 20%.
            let opacity = if self.grid_fill { 0.2 } else { 1.0 };
            let stroke = Stroke::default()
                .with_color(alpha(p.border, opacity))
                .with_width(1.0);

            if self.grid_fill {
                let color = self.color(t, 0, None);

                for r in rings.iter().rev() {
                    frame.fill(&ring(*r), alpha(color, opacity));
                }
            }

            for r in &rings {
                if *r > 0.0 {
                    frame.stroke(&ring(*r), stroke);
                }
            }

            if self.radial_lines {
                for angle in &corners {
                    frame.stroke(
                        &Path::line(polar.center, plot::polar(polar.center, polar.outer, *angle)),
                        stroke,
                    );
                }
            }
        }

        let stroke = self.stroke();
        let mut active = Vec::new();

        for (index, series) in self.series.iter().enumerate() {
            let color = self.color(t, index, None);
            let points: Vec<Point> = corners
                .iter()
                .enumerate()
                .map(|(i, angle)| {
                    plot::polar(polar.center, radius(plot::value(&series.values, i)), *angle)
                })
                .collect();
            let polygon = Path::new(|builder| {
                for (i, point) in points.iter().enumerate() {
                    if i == 0 {
                        builder.move_to(*point);
                    } else {
                        builder.line_to(*point);
                    }
                }
                builder.close();
            });

            frame.fill(&polygon, alpha(color, self.opacity_of(index)));

            if stroke > 0.0 {
                frame.stroke(
                    &polygon,
                    Stroke::default().with_color(color).with_width(stroke),
                );
            }

            if self.dots {
                for point in &points {
                    frame.fill(&Path::circle(*point, self.dot_radius.max(4.0)), color);
                }
            }

            if let Some(hovered) = hovered {
                active.push((points[hovered], color));
            }
        }

        for (point, color) in active {
            frame.fill(&Path::circle(point, self.active_dot), color);
        }

        // `PolarRadiusAxis`: the values along a spoke, turned with it.
        if let Some(angle) = self.radius_axis {
            for value in &domain {
                Label::new(
                    self.value_tick(*value),
                    plot::polar(polar.center, radius(*value), angle),
                    p.foreground,
                )
                .baseline(Baseline::Middle)
                .rotate(90.0 - angle)
                .draw(frame);
            }
        }

        // `PolarAngleAxis`: the categories 8 past the corners, in
        // `fill-muted-foreground`.
        if self.category_axis {
            for (index, angle) in corners.iter().enumerate() {
                let anchor = plot::polar(polar.center, polar.outer + ANGLE_TICK, *angle);
                let (sin, cos) = angle.to_radians().sin_cos();
                let align = if cos > 1e-5 {
                    TextAlignment::Left
                } else if cos < -1e-5 {
                    TextAlignment::Right
                } else {
                    TextAlignment::Center
                };
                // Above the corners at the top, below those at the bottom,
                // and centered on the others.
                let baseline = if cos.abs() > std::f32::consts::FRAC_1_SQRT_2 {
                    Baseline::Middle
                } else if sin > 0.0 {
                    Baseline::End
                } else {
                    Baseline::Start
                };

                Label::new(
                    self.tick(&self.categories[index]),
                    anchor,
                    p.muted_foreground,
                )
                .align(align)
                .baseline(baseline)
                .draw(frame);
            }
        }
    }

    fn draw_radial(&self, frame: &mut Frame, t: &Tokens, plot: Rectangle) {
        let p = t.palette;
        let polar = self.polar(plot);
        let corner = self.corner_radius;

        if let Some((inner, outer)) = self.track {
            let ring = Sector {
                center: polar.center,
                inner,
                outer,
                start: 0.0,
                end: 360.0,
            };

            frame.fill(&Path::new(|builder| ring.path(builder, 0.0)), p.muted);
        }

        let bars = self.radial_bars(polar);

        // `PolarGrid gridType="circle"`: a circle through each ring, and a
        // spoke per value.
        if self.grid {
            let stroke = Stroke::default().with_color(p.border).with_width(1.0);

            if let Some(first) = bars.first() {
                for sector in first {
                    let r = (sector.inner + sector.outer) / 2.0;
                    frame.stroke(&Path::circle(polar.center, r), stroke);
                }
            }

            if self.radial_lines {
                let (start, end) = self.angles;
                let max = self.radial_max();

                for value in plot::ticks(0.0, max, 5) {
                    if value > max {
                        continue;
                    }

                    let angle = start + (end - start) * value / max;
                    frame.stroke(
                        &Path::line(
                            plot::polar(polar.center, polar.inner, angle),
                            plot::polar(polar.center, polar.outer, angle),
                        ),
                        stroke,
                    );
                }
            }
        }

        if self.background {
            let (start, end) = self.angles;

            for sector in bars.first().into_iter().flatten() {
                let track = Sector {
                    start,
                    end,
                    ..*sector
                };
                frame.fill(&Path::new(|builder| track.path(builder, corner)), p.muted);
            }
        }

        let mut labels = Vec::new();

        for (series, sectors) in bars.iter().enumerate() {
            for (category, sector) in sectors.iter().enumerate() {
                if sector.start == sector.end {
                    continue;
                }

                let color = self.color(t, series, Some(category));
                frame.fill(&Path::new(|builder| sector.path(builder, corner)), color);

                // `position="insideStart"`: the category along the start of
                // the bar, in `fill-white`.
                if self.category_labels && series == 0 {
                    let radius = (sector.inner + sector.outer) / 2.0;
                    let sign = if sector.end >= sector.start {
                        1.0
                    } else {
                        -1.0
                    };
                    let content = self.tick(&self.categories[category]);
                    let width = content.chars().count() as f32 * 6.2;
                    let middle = sector.start + sign * (5.0 + (width / 2.0 / radius).to_degrees());
                    let anchor = plot::polar(polar.center, radius, middle);

                    labels.push(
                        Label::new(content, anchor, Color::WHITE)
                            .size(11.0)
                            .baseline(Baseline::Central)
                            .rotate(if sign > 0.0 {
                                -middle - 90.0
                            } else {
                                90.0 - middle
                            }),
                    );
                }
            }
        }

        for label in labels {
            label.draw(frame);
        }
    }

    /// `ChartLegendContent`: `flex items-center justify-center gap-4 pt-3`,
    /// with `h-2 w-2 rounded-[2px]` swatches or the icons of the series.
    fn legend_element<'a, Message: 'a>(&self) -> Element<'a, Message> {
        let entries: Vec<(String, Option<Icon>, usize, Option<usize>)> = if self.colors_categories()
        {
            self.categories
                .iter()
                .enumerate()
                .map(|(index, category)| (category.clone(), None, 0, Some(index)))
                .collect()
        } else {
            self.series
                .iter()
                .enumerate()
                .map(|(index, series)| (series.label.clone(), series.icon, index, None))
                .collect()
        };

        let items = entries.into_iter().map(|(label, icon, series, category)| {
            let indicator: Element<'a, Message> = match icon {
                // `[&>svg]:h-3 [&>svg]:w-3 [&>svg]:text-muted-foreground`
                Some(icon) => icon
                    .sized(12)
                    .style(|theme: &Theme| text::Style {
                        color: Some(tokens(theme).palette.muted_foreground),
                    })
                    .into(),
                None => {
                    let paint = self.paint(series, category);
                    swatch(8.0, move |t| paint(t))
                }
            };

            row![
                indicator,
                text(label)
                    .size(style::text::XS)
                    .line_height(style::text::leading(style::text::XS)),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
            .into()
        });

        let padding = match self.legend_position {
            LegendPosition::Top => Padding::ZERO.bottom(12),
            LegendPosition::Bottom => Padding::ZERO.top(12),
        };

        container(
            Row::with_children(items)
                .spacing(16)
                .align_y(Alignment::Center)
                .wrap()
                .vertical_spacing(8)
                .align_x(Alignment::Center),
        )
        .padding(padding)
        .into()
    }

    /// A function resolving the color of a series (or of a category of a
    /// series) in a theme, to style widgets with.
    fn paint(&self, series: usize, category: Option<usize>) -> impl Fn(&Tokens) -> Color + 'static {
        let data = self.series.get(series);
        let cell =
            category.and_then(|category| data.and_then(|data| data.cells.get(category)).copied());
        let color = data.and_then(|data| data.color);
        let by_category = self.colors_categories();

        move |t: &Tokens| {
            if let Some(cell) = cell {
                return cell.resolve(t);
            }

            if by_category {
                return t.palette.chart[category.unwrap_or(0) % 5];
            }

            color
                .map(|paint| paint.resolve(t))
                .unwrap_or(t.palette.chart[series % 5])
        }
    }

    /// `ChartTooltipContent` for a target (see [`Chart::hovered`]): `grid
    /// min-w-32 gap-1.5 rounded-lg border border-border/50 bg-background
    /// px-2.5 py-1.5 text-xs shadow-xl`.
    fn tooltip_element<'a, Message: 'a>(&self, target: usize) -> Element<'a, Message> {
        let count = self.count().max(1);
        let category = target % count;

        // The rows: a name, a value, and the series and category of the
        // color.
        let rows: Vec<(String, f32, usize, Option<usize>, Option<Icon>)> = match self.kind {
            Kind::Pie => {
                let series = target / count;

                vec![(
                    self.categories.get(category).cloned().unwrap_or_default(),
                    self.series
                        .get(series)
                        .map_or(0.0, |s| plot::value(&s.values, category)),
                    series,
                    Some(category),
                    None,
                )]
            }
            _ if self.colors_categories() => vec![(
                self.categories.get(category).cloned().unwrap_or_default(),
                self.series
                    .first()
                    .map_or(0.0, |s| plot::value(&s.values, category)),
                0,
                Some(category),
                None,
            )],
            _ => self
                .series
                .iter()
                .enumerate()
                .map(|(index, series)| {
                    (
                        series.label.clone(),
                        plot::value(&series.values, category),
                        index,
                        Some(category),
                        series.icon,
                    )
                })
                .collect(),
        };

        let label = self.tooltip_label.then(|| {
            let label = match self.kind {
                Kind::Pie => self
                    .series
                    .get(target / count)
                    .map(|series| series.label.clone())
                    .unwrap_or_default(),
                _ => self.categories.get(category).cloned().unwrap_or_default(),
            };

            match &self.tooltip_label_formatter {
                Some(formatter) => formatter(&label),
                None => label,
            }
        });

        // `leading-none`, or the 16px lines of `text-xs` when the values
        // have a unit (a custom `formatter`).
        let line = if self.tooltip_unit.is_some() {
            style::text::leading(style::text::XS)
        } else {
            LineHeight::Absolute(Pixels(style::text::XS))
        };
        let indicator = self.indicator;
        // A single row nests the label next to a line indicator.
        let nested = rows.len() == 1 && indicator != Indicator::Dot && label.is_some();

        // `font-medium`, with the 16px lines of `text-xs` above the rows, or
        // `leading-none` nested in a row.
        let label_text = |label: String, nested: bool| -> Element<'a, Message> {
            text(label)
                .size(style::text::XS)
                .line_height(if nested {
                    line
                } else {
                    style::text::leading(style::text::XS)
                })
                .font(font::medium())
                .into()
        };

        let value_text = |value: String| -> Element<'a, Message> {
            // `font-mono font-medium text-foreground tabular-nums`
            let value = text(value)
                .size(style::text::XS)
                .line_height(line)
                .font(font::weight(font::mono(), iced::font::Weight::Medium));

            match &self.tooltip_unit {
                // `items-baseline gap-0.5`, `font-normal text-muted-foreground`
                Some(unit) => row![
                    value,
                    text(unit.clone())
                        .size(style::text::XS)
                        .line_height(line)
                        .font(font::mono())
                        .style(|theme: &Theme| text::Style {
                            color: Some(tokens(theme).palette.muted_foreground),
                        }),
                ]
                .spacing(2)
                .into(),
                None => value.into(),
            }
        };

        let mut lines: Vec<RowLine<'a, Message>> = Vec::new();

        for (name, value, series, category, icon) in &rows {
            let paint = self.paint(*series, *category);
            let height = if nested {
                2.0 * style::text::XS + 6.0
            } else {
                style::text::XS
            };

            // `[&>svg]:h-2.5 [&>svg]:w-2.5 [&>svg]:text-muted-foreground`,
            // or `h-2.5 w-2.5` (dot), `w-1` (line), `w-0 border-[1.5px]
            // border-dashed` (dashed).
            let marker: Option<Element<'a, Message>> = match (icon, indicator) {
                (Some(icon), _) => Some(
                    icon.sized(10)
                        .style(|theme: &Theme| text::Style {
                            color: Some(tokens(theme).palette.muted_foreground),
                        })
                        .into(),
                ),
                (None, Indicator::Dot) => Some(swatch(10.0, paint)),
                (None, Indicator::Line) => Some(bar_indicator(4.0, height, paint)),
                (None, Indicator::Dashed) => Some(dashed_indicator(height, paint)),
                (None, Indicator::None) => None,
            };

            let name: Element<'a, Message> = text(name.clone())
                .size(style::text::XS)
                .line_height(line)
                .style(|theme: &Theme| text::Style {
                    color: Some(tokens(theme).palette.muted_foreground),
                })
                .into();

            let name = match (&label, nested) {
                (Some(label), true) => Column::new()
                    .push(label_text(label.clone(), true))
                    .push(name)
                    .spacing(6)
                    .into(),
                _ => name,
            };

            // `gap-2`
            let start = match marker {
                Some(marker) => row![marker, name]
                    .spacing(8)
                    .align_y(if nested {
                        Alignment::End
                    } else {
                        Alignment::Center
                    })
                    .into(),
                None => name,
            };

            lines.push(RowLine {
                start,
                end: value_text(self.format(*value)),
                rule: false,
                align_end: nested,
            });
        }

        if self.tooltip_total {
            let total: f32 = rows.iter().map(|(_, value, ..)| value).sum();

            // `mt-1.5 border-t pt-1.5 font-medium`
            lines.push(RowLine {
                start: label_text("Total".to_owned(), false),
                end: value_text(self.format(total)),
                rule: true,
                align_end: false,
            });
        }

        let mut content = Column::new().spacing(6);

        if let Some(label) = label.filter(|_| !nested) {
            content = content.push(label_text(label, false));
        }

        // `min-w-32` within a 1px border and `px-2.5`.
        content = content.push(Element::new(Rows {
            lines,
            spacing: 6.0,
            min_width: 128.0 - 2.0 * 11.0,
        }));

        container(content)
            .padding([7, 11])
            .style(|theme| {
                let t = tokens(theme);

                container::Style {
                    background: Some(Background::Color(t.palette.background)),
                    border: Border {
                        color: alpha(t.palette.border, 0.5),
                        width: 1.0,
                        radius: t.radius_lg().into(),
                    },
                    // `shadow-xl`, without its negative spread.
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
                        offset: Vector::new(0.0, 10.0),
                        blur_radius: 18.0,
                    },
                    ..container::Style::default()
                }
            })
            .into()
    }
}

/// A small colored square, like the indicators of the legend and tooltip.
fn swatch<'a, Message: 'a>(
    size: f32,
    color: impl Fn(&Tokens) -> Color + 'a,
) -> Element<'a, Message> {
    bar_indicator(size, size, color)
}

/// A colored rectangle with corners rounded by 2.
fn bar_indicator<'a, Message: 'a>(
    width: f32,
    height: f32,
    color: impl Fn(&Tokens) -> Color + 'a,
) -> Element<'a, Message> {
    container(Space::new().width(width).height(height))
        .style(move |theme| container::Style {
            background: Some(Background::Color(color(&tokens(theme)))),
            border: style::rounded(2.0),
            ..container::Style::default()
        })
        .into()
}

/// A dashed vertical line, 3 wide (`w-0 border-[1.5px] border-dashed`).
fn dashed_indicator<'a, Message: 'a>(
    height: f32,
    color: impl Fn(&Tokens) -> Color + 'a,
) -> Element<'a, Message> {
    const DASH: f32 = 3.0;

    let dashes = ((height + DASH) / (2.0 * DASH)).floor().max(1.0) as usize;
    let color = std::rc::Rc::new(color);

    Column::with_children((0..dashes).map(|_| {
        let color = color.clone();
        bar_indicator(3.0, DASH, move |t| color(t))
    }))
    .spacing(DASH)
    .height(height)
    .into()
}

impl<'a, Message: 'a> From<Chart> for Element<'a, Message> {
    fn from(chart: Chart) -> Self {
        let legend = chart.legend.then(|| chart.legend_element());
        let tooltips = if chart.tooltip {
            (0..chart.targets())
                .map(|index| chart.tooltip_element(index))
                .collect()
        } else {
            Vec::new()
        };

        Element::new(ChartWidget {
            chart,
            legend,
            tooltips,
        })
    }
}

/// The state of a chart: the hovered category (or slice), the pointer, and
/// the height of the legend.
#[derive(Debug, Default)]
pub struct State {
    hovered: Option<usize>,
    cursor: Option<Point>,
    legend: f32,
}

/// Draws the plot, and lays out the legend and the tooltip of the hovered
/// category as widgets.
struct ChartWidget<'a, Message> {
    chart: Chart,
    legend: Option<Element<'a, Message>>,
    tooltips: Vec<Element<'a, Message>>,
}

impl<'a, Message> ChartWidget<'a, Message> {
    fn children(&self) -> impl Iterator<Item = &Element<'a, Message>> {
        self.legend.iter().chain(&self.tooltips)
    }

    /// What the tooltip shows: the hovered target, or the default one.
    fn shown_target(&self, state: &State) -> Option<usize> {
        if state.cursor.is_some() {
            state.hovered
        } else {
            state.hovered.or(self.chart.default_index)
        }
    }

    /// The child index of the tooltip of the hovered category.
    fn tooltip_index(&self, state: &State) -> Option<usize> {
        let target = self.shown_target(state)?;

        (target < self.tooltips.len()).then(|| target + usize::from(self.legend.is_some()))
    }

    /// The children shown: the legend and the hovered tooltip.
    fn shown(&self, state: &State) -> impl Iterator<Item = usize> {
        let legend = self.legend.is_some().then_some(0);

        legend.into_iter().chain(self.tooltip_index(state))
    }

    /// The category to highlight in the plot.
    fn active_category(&self, state: &State) -> Option<usize> {
        let target = self.shown_target(state)?;

        Some(target % self.chart.count().max(1))
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for ChartWidget<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.children().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let children: Vec<&Element<'_, Message>> = self.children().collect();
        tree.diff_children(&children);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(self.chart.height))
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits
            .width(Length::Fill)
            .height(self.chart.height)
            .resolve(Length::Fill, self.chart.height, Size::ZERO);
        let margin = self.chart.margin;

        let mut nodes = Vec::new();
        let mut trees = tree.children.iter_mut();
        let mut legend_height = 0.0;

        if let Some(legend) = &mut self.legend {
            let available = (size.width - margin.left - margin.right).max(0.0);
            let node = legend.as_widget_mut().layout(
                trees.next().expect("legend tree"),
                renderer,
                &layout::Limits::new(Size::ZERO, Size::new(available, f32::INFINITY)),
            );
            let legend_size = node.size();
            legend_height = legend_size.height;

            let y = match self.chart.legend_position {
                LegendPosition::Top => margin.top,
                LegendPosition::Bottom => size.height - margin.bottom - legend_size.height,
            };

            nodes.push(node.move_to(Point::new(
                margin.left + (available - legend_size.width) / 2.0,
                y,
            )));
        }

        let plot = self.chart.plot(size, legend_height);
        let state = tree.state.downcast_mut::<State>();
        state.legend = legend_height;

        let cursor = state.cursor;
        let shown = self.shown_target(state);

        // Only the shown tooltip is laid out (and drawn): the layout is
        // invalidated whenever the hovered category changes. When only the
        // pointer moves, `draw` moves the tooltip after it.
        for (index, (tooltip, tree)) in self.tooltips.iter_mut().zip(trees).enumerate() {
            if shown != Some(index) {
                nodes.push(layout::Node::default());
                continue;
            }

            let node = tooltip.as_widget_mut().layout(
                tree,
                renderer,
                &layout::Limits::new(Size::ZERO, Size::INFINITE),
            );

            let target = if self.chart.kind == Kind::Pie {
                index
            } else {
                index % self.chart.count().max(1)
            };
            let position = self
                .chart
                .tooltip_position(plot, target, cursor, node.size());

            nodes.push(node.move_to(position));
        }

        layout::Node::with_children(size, nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        let shown: Vec<usize> = self.shown(tree.state.downcast_ref::<State>()).collect();
        let legend = usize::from(self.legend.is_some());
        let children: Vec<Layout<'_>> = layout.children().collect();

        for index in shown {
            let element = if index < legend {
                self.legend.as_mut()
            } else {
                self.tooltips.get_mut(index - legend)
            };

            if let Some(element) = element {
                element.as_widget_mut().operate(
                    &mut tree.children[index],
                    children[index],
                    renderer,
                    operation,
                );
            }
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        if !self.chart.tooltip {
            return;
        }

        let Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::CursorLeft) = event
        else {
            return;
        };

        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<State>();
        let position = cursor.position_in(bounds);
        let plot = self.chart.plot(bounds.size(), state.legend);

        let hovered = position.and_then(|point| self.chart.hovered(plot, point));
        let position = position.filter(|_| hovered.is_some());

        // A new category shows another tooltip, which needs a layout; the
        // pointer alone only moves it, which `draw` does.
        if hovered != state.hovered {
            state.hovered = hovered;
            state.cursor = position;

            shell.invalidate_layout();
            shell.request_redraw();
        } else if position != state.cursor {
            state.cursor = position;

            shell.request_redraw();
        }
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
        let bounds = layout.bounds();

        if bounds.width < 1.0 || bounds.height < 1.0 {
            return;
        }

        let state = tree.state.downcast_ref::<State>();
        let t = tokens(theme);
        let active = self.active_category(state).filter(|_| self.chart.tooltip);

        // Clipped to the bounds, like the `<svg>` of Recharts.
        renderer.with_layer(bounds, |renderer| {
            renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
                let mut frame = Frame::new(renderer, bounds.size());
                self.chart
                    .draw_plot(&mut frame, &t, bounds.size(), state.legend, active);
                renderer.draw_geometry(frame.into_geometry());
            });
        });

        let children: Vec<Layout<'_>> = layout.children().collect();

        if let Some(legend) = &self.legend {
            legend.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                children[0],
                cursor,
                viewport,
            );
        }

        if let Some(index) = self.tooltip_index(state) {
            let target = index - usize::from(self.legend.is_some());
            let tooltip = &self.tooltips[target];
            let laid = children[index].bounds();

            // The tooltip was laid out for the pointer of the last layout:
            // follow the pointer since.
            let category = if self.chart.kind == Kind::Pie {
                target
            } else {
                target % self.chart.count().max(1)
            };
            let position = self.chart.tooltip_position(
                self.chart.plot(bounds.size(), state.legend),
                category,
                state.cursor,
                laid.size(),
            );
            let offset = Vector::new(
                bounds.x + position.x - laid.x,
                bounds.y + position.y - laid.y,
            );

            let layer = Rectangle {
                x: bounds.x - 32.0,
                y: bounds.y - 32.0,
                width: bounds.width + 64.0,
                height: bounds.height + 64.0,
            };

            renderer.with_layer(layer, |renderer| {
                renderer.with_translation(offset, |renderer| {
                    tooltip.as_widget().draw(
                        &tree.children[index],
                        renderer,
                        theme,
                        style,
                        children[index],
                        cursor,
                        viewport,
                    );
                });
            });
        }
    }
}

/// A line of the tooltip: an element at each end.
struct RowLine<'a, Message> {
    start: Element<'a, Message>,
    end: Element<'a, Message>,
    /// Whether a rule (`border-t`) separates the line from the previous one.
    rule: bool,
    /// Whether both ends sit at the bottom of the line (`items-end`), else
    /// in the middle.
    align_end: bool,
}

/// The lines of a tooltip, with their ends at both sides of a column at
/// least `min_width` wide (`flex justify-between`).
struct Rows<'a, Message> {
    lines: Vec<RowLine<'a, Message>>,
    spacing: f32,
    min_width: f32,
}

impl<Message> Rows<'_, Message> {
    fn elements(&self) -> impl Iterator<Item = &Element<'_, Message>> {
        self.lines.iter().flat_map(|line| [&line.start, &line.end])
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Rows<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        self.elements().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let elements: Vec<_> = self.elements().collect();
        tree.diff_children(&elements);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let loose = limits.loose();
        let mut trees = tree.children.iter_mut();
        let mut laid = Vec::new();
        let mut width = self.min_width;

        for line in &mut self.lines {
            let start = line.start.as_widget_mut().layout(
                trees.next().expect("start tree"),
                renderer,
                &loose,
            );
            let end =
                line.end
                    .as_widget_mut()
                    .layout(trees.next().expect("end tree"), renderer, &loose);

            width = width.max(start.size().width + end.size().width);
            laid.push((start, end, line.rule, line.align_end));
        }

        let mut nodes = Vec::new();
        let mut y = 0.0;

        for (index, (start, end, rule, align_end)) in laid.into_iter().enumerate() {
            if index > 0 {
                y += self.spacing;
            }

            if rule {
                // `basis-full` in the wrapping row (`gap-2`), then `mt-1.5
                // border-t pt-1.5`.
                y += 8.0 + 1.0 + self.spacing;
            }

            let (a, b) = (start.size(), end.size());
            let height = a.height.max(b.height);
            let offset = |size: Size| {
                if align_end {
                    height - size.height
                } else {
                    (height - size.height) / 2.0
                }
            };

            nodes.push(start.move_to(Point::new(0.0, y + offset(a))));
            nodes.push(end.move_to(Point::new(width - b.width, y + offset(b))));

            y += height;
        }

        layout::Node::with_children(Size::new(width, y), nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        let elements = self
            .lines
            .iter_mut()
            .flat_map(|line| [&mut line.start, &mut line.end]);

        for ((child, tree), layout) in elements.zip(&mut tree.children).zip(layout.children()) {
            child
                .as_widget_mut()
                .operate(tree, layout, renderer, operation);
        }
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
        let bounds = layout.bounds();
        let children: Vec<Layout<'_>> = layout.children().collect();

        for (index, line) in self.lines.iter().enumerate() {
            if line.rule {
                let top = children[2 * index]
                    .bounds()
                    .y
                    .min(children[2 * index + 1].bounds().y);

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(bounds.x, top - self.spacing - 1.0),
                            Size::new(bounds.width, 1.0),
                        ),
                        ..renderer::Quad::default()
                    },
                    tokens(theme).palette.border,
                );
            }
        }

        for ((child, tree), layout) in self.elements().zip(&tree.children).zip(children) {
            child
                .as_widget()
                .draw(tree, renderer, theme, style, layout, cursor, viewport);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::format_value;

    #[test]
    fn values_are_grouped() {
        assert_eq!(format_value(305.0), "305");
        assert_eq!(format_value(1234.0), "1,234");
        assert_eq!(format_value(-1234567.5), "-1,234,567.5");
        assert_eq!(format_value(0.25), "0.25");
    }
}
