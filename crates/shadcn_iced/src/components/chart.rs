//! Beautiful charts.
//!
//! Port of <https://ui.shadcn.com/docs/components/chart>. shadcn/ui wraps
//! Recharts; this module draws the same kinds of charts (bar, line, area and
//! pie) with the same geometry and look: a faint horizontal grid, muted axis
//! labels, the `--chart-*` colors, a legend and a tooltip on hover
//! (`ChartTooltipContent`, `ChartLegendContent`). The scales, the stacking,
//! the curves, the pie and the hit testing live in
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
use shadcn_iced_core::primitive::plot::{self, Bars, Categories, Linear, Pie, Spread};
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, Tokens, font, style, tokens};

use iced::advanced::graphics::geometry::Renderer as _;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::alignment;
use iced::widget::canvas::{Frame, Path, Stroke, Text};
use iced::widget::text::{Alignment as TextAlignment, LineHeight};
use iced::widget::{Column, Row, Space, container, row, text};
use iced::{
    Alignment, Background, Border, Color, Event, Length, Padding, Pixels, Point, Rectangle, Shadow,
    Size, Theme, Vector, mouse,
};

/// A series of values, one per category.
#[derive(Debug, Clone, PartialEq)]
pub struct Series {
    /// The label shown in the legend and the tooltip.
    pub label: String,
    /// The values, one per category.
    pub values: Vec<f32>,
    /// The color. Defaults to the `--chart-N` token matching the position of
    /// the series.
    pub color: Option<Color>,
}

impl Series {
    /// Creates a series.
    pub fn new(label: impl Into<String>, values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            label: label.into(),
            values: values.into_iter().collect(),
            color: None,
        }
    }

    /// Sets the color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

/// The kind of a [`Chart`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Vertical bars.
    Bar,
    /// Smooth lines.
    Line,
    /// Smooth lines with a translucent fill.
    Area,
    /// A pie (or donut) of the first series.
    Pie,
}

/// A chart. See the [module documentation](self).
pub struct Chart {
    kind: Kind,
    categories: Vec<String>,
    series: Vec<Series>,
    height: f32,
    legend: bool,
    tooltip: bool,
    cursor: bool,
    grid: bool,
    stacked: bool,
    dots: bool,
    active_dot: f32,
    donut: bool,
    margin: Padding,
    tick_margin: f32,
    tick_formatter: Option<Box<dyn Fn(&str) -> String>>,
}

fn chart(kind: Kind, categories: impl IntoIterator<Item = impl Into<String>>) -> Chart {
    Chart {
        kind,
        categories: categories.into_iter().map(Into::into).collect(),
        series: Vec::new(),
        height: 250.0,
        legend: true,
        tooltip: true,
        cursor: true,
        grid: true,
        stacked: false,
        dots: false,
        active_dot: 4.0,
        donut: false,
        margin: Padding::new(5.0),
        tick_margin: 8.0,
        tick_formatter: None,
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

/// Creates a pie chart of a single series: one slice per category.
pub fn pie(
    categories: impl IntoIterator<Item = impl Into<String>>,
    values: impl IntoIterator<Item = f32>,
) -> Chart {
    let mut chart = chart(Kind::Pie, categories);
    chart.series.push(Series::new("", values));
    chart.grid = false;
    chart
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

    /// Shows or hides the legend (`ChartLegend`).
    pub fn legend(mut self, legend: bool) -> Self {
        self.legend = legend;
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

    /// Shows or hides the grid (`CartesianGrid vertical={false}`).
    pub fn grid(mut self, grid: bool) -> Self {
        self.grid = grid;
        self
    }

    /// Stacks the series of bar and area charts (`stackId`).
    pub fn stacked(mut self, stacked: bool) -> Self {
        self.stacked = stacked;
        self
    }

    /// Draws a dot on each value of line charts (`dot`).
    pub fn dots(mut self, dots: bool) -> Self {
        self.dots = dots;
        self
    }

    /// Sets the radius of the dots of line and area charts on the hovered
    /// category (`activeDot={{ r }}`). Defaults to 4.
    pub fn active_dot(mut self, radius: f32) -> Self {
        self.active_dot = radius;
        self
    }

    /// Draws a donut instead of a pie.
    pub fn donut(mut self, donut: bool) -> Self {
        self.donut = donut;
        self
    }

    /// Sets the margin around the plot (`margin`). Defaults to 5 on every
    /// side, like Recharts.
    pub fn margin(mut self, margin: impl Into<Padding>) -> Self {
        self.margin = margin.into();
        self
    }

    /// Sets the space between the plot and the labels of the categories
    /// (`<XAxis tickMargin />`). Defaults to 8.
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

    fn color(&self, tokens: &Tokens, index: usize) -> Color {
        color(&self.colors(), tokens, index)
    }

    fn colors(&self) -> Vec<Option<Color>> {
        match self.kind {
            Kind::Pie => vec![None; self.categories.len()],
            _ => self.series.iter().map(|series| series.color).collect(),
        }
    }

    fn max(&self) -> f32 {
        plot::nice(plot::max(
            self.series.iter().map(|s| s.values.as_slice()),
            self.categories.len(),
            self.stacked,
        ))
    }

    fn tick(&self, category: &str) -> String {
        match &self.tick_formatter {
            Some(formatter) => formatter(category),
            None => category.to_owned(),
        }
    }
}

fn color(colors: &[Option<Color>], tokens: &Tokens, index: usize) -> Color {
    colors
        .get(index)
        .copied()
        .flatten()
        .unwrap_or(tokens.palette.chart[index % 5])
}

/// Formats a value like `toLocaleString()` does in English.
fn format_value(value: f32) -> String {
    let formatted = if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    };

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

/// The height of the category axis (`XAxis`).
const AXIS: f32 = 30.0;
/// The size of the (hidden) tick lines, which still offsets the labels.
const TICK_SIZE: f32 = 6.0;
/// The height of the legend (`pt-3` and a `text-xs` line).
const LEGEND: f32 = 28.0;
/// The distance between the tooltip and the hovered point.
const TOOLTIP_OFFSET: f32 = 10.0;

impl Chart {
    fn plot(&self, bounds: Size) -> Rectangle {
        let m = self.margin;
        let bottom = if self.kind == Kind::Pie { 0.0 } else { AXIS };
        let legend = if self.legend { LEGEND } else { 0.0 };

        Rectangle {
            x: m.left,
            y: m.top,
            width: (bounds.width - m.left - m.right).max(0.0),
            height: (bounds.height - m.top - m.bottom - bottom - legend).max(0.0),
        }
    }

    /// The scale of the categories: bands for bars, and spread edge to edge
    /// for lines and areas (a point scale).
    fn categories(&self, plot: Rectangle) -> Categories {
        let spread = match self.kind {
            Kind::Line | Kind::Area => Spread::Point,
            _ => Spread::Band,
        };

        Categories::new(plot, self.categories.len(), spread)
    }

    fn pie_geometry(&self, plot: Rectangle) -> Pie {
        Pie::new(plot, if self.donut { 0.6 } else { 0.0 })
    }

    /// The hovered category, or slice of a pie.
    fn hovered(&self, plot: Rectangle, point: Point) -> Option<usize> {
        match self.kind {
            Kind::Pie => self
                .pie_geometry(plot)
                .slice_at(&self.series.first()?.values, point),
            _ => self.categories(plot).at(point),
        }
    }

    /// Where the tooltip goes: past the hovered category (or the pointer)
    /// by 10, flipped to stay inside the plot, like Recharts.
    fn tooltip_position(
        &self,
        plot: Rectangle,
        hovered: usize,
        cursor: Point,
        size: Size,
    ) -> Point {
        let anchor = match self.kind {
            Kind::Pie => cursor,
            _ => Point::new(self.categories(plot).position(hovered), cursor.y),
        };

        Point::new(
            plot::place(anchor.x, size.width, TOOLTIP_OFFSET, plot.x, plot.width),
            plot::place(anchor.y, size.height, TOOLTIP_OFFSET, plot.y, plot.height),
        )
    }

    fn draw_plot(&self, frame: &mut Frame, t: &Tokens, size: Size, hovered: Option<usize>) {
        let p = t.palette;
        let plot = self.plot(size);
        let count = self.categories.len();

        if self.kind == Kind::Pie {
            self.draw_pie(frame, t, plot, hovered);
            return;
        }

        if count == 0 {
            return;
        }

        let scale = Linear::new(plot, self.max());
        let y = |value: f32| scale.position(value);
        let x = self.categories(plot);
        let band = x.band_width();

        // `CartesianGrid vertical={false}`: a line per tick
        // (`stroke-border/50`).
        if self.grid {
            for line_y in scale.ticks(4) {
                frame.stroke(
                    &Path::line(
                        Point::new(plot.x, line_y),
                        Point::new(plot.x + plot.width, line_y),
                    ),
                    Stroke::default()
                        .with_color(alpha(p.border, 0.5))
                        .with_width(1.0),
                );
            }
        }

        if let Some(hovered) = hovered.filter(|_| self.cursor) {
            match self.kind {
                // `.recharts-tooltip-cursor`: `fill-muted`
                Kind::Bar => frame.fill_rectangle(
                    Point::new(x.band_start(hovered), plot.y),
                    Size::new(band, plot.height),
                    p.muted,
                ),
                _ => {
                    let x = x.position(hovered);
                    frame.stroke(
                        &Path::line(Point::new(x, plot.y), Point::new(x, plot.y + plot.height)),
                        Stroke::default().with_color(p.border).with_width(1.0),
                    );
                }
            }
        }

        match self.kind {
            Kind::Bar => {
                // `barCategoryGap="10%"`, `barGap={4}`, and whole widths.
                let groups = if self.stacked { 1 } else { self.series.len() };
                let bars = Bars::new(band, groups, 0.1, 4.0);
                let bases = plot::baselines(
                    self.series.iter().map(|s| s.values.as_slice()),
                    count,
                    self.stacked,
                );

                for category in 0..count {
                    for ((index, series), bases) in self.series.iter().enumerate().zip(&bases) {
                        let value = plot::value(&series.values, category);
                        let base = bases[category];
                        let slot = if self.stacked { 0 } else { index };
                        let top = y(base + value);
                        let height = y(base) - top;

                        // `radius={4}`
                        let radius = 4.0_f32.min(bars.width / 2.0).min(height / 2.0).max(0.0);

                        frame.fill(
                            &Path::rounded_rectangle(
                                Point::new(bars.x(x.band_start(category), slot), top),
                                Size::new(bars.width, height),
                                radius.into(),
                            ),
                            self.color(t, index),
                        );
                    }
                }
            }
            Kind::Line | Kind::Area => {
                let stacked = self.stacked && self.kind == Kind::Area;
                let bases = plot::baselines(
                    self.series.iter().map(|s| s.values.as_slice()),
                    count,
                    stacked,
                );
                let mut active = Vec::new();

                for ((index, series), bases) in self.series.iter().enumerate().zip(&bases) {
                    let color = self.color(t, index);
                    let value = |i: usize| plot::value(&series.values, i);

                    let points: Vec<Point> = (0..count)
                        .map(|i| Point::new(x.position(i), y(bases[i] + value(i))))
                        .collect();

                    if self.kind == Kind::Area {
                        let lower: Vec<Point> = (0..count)
                            .rev()
                            .map(|i| Point::new(x.position(i), y(bases[i])))
                            .collect();

                        // `fillOpacity={0.4}`, down to the series below.
                        let fill = Path::new(|builder| {
                            plot::natural(builder, &points, true);
                            plot::natural(builder, &lower, false);
                            builder.close();
                        });

                        frame.fill(&fill, alpha(color, 0.4));
                    }

                    // `type="natural"`, `strokeWidth={2}` for lines.
                    frame.stroke(
                        &Path::new(|builder| plot::natural(builder, &points, true)),
                        Stroke::default()
                            .with_color(color)
                            .with_width(if self.kind == Kind::Line { 2.0 } else { 1.0 }),
                    );

                    // `dot`: a circle of radius 3 stroked with the line.
                    if self.dots {
                        for point in &points {
                            frame.fill(&Path::circle(*point, 4.0), color);
                        }
                    }

                    if let Some(hovered) = hovered {
                        active.push((points[hovered], color));
                    }
                }

                // `activeDot`, over every series.
                for (point, color) in active {
                    frame.fill(&Path::circle(point, self.active_dot), color);
                }
            }
            Kind::Pie => {}
        }

        // The labels of the categories (`tickLine={false} axisLine={false}`,
        // `fill-muted-foreground`), with their baseline `0.71em` below the
        // tick.
        let skip = x.label_step(48.0);

        for (index, category) in self.categories.iter().enumerate() {
            if index % skip != 0 {
                continue;
            }

            frame.fill_text(Text {
                content: self.tick(category),
                position: Point::new(
                    x.position(index),
                    plot.y + plot.height + TICK_SIZE + self.tick_margin + TICK_BASELINE,
                ),
                color: p.muted_foreground,
                size: Pixels(style::text::XS),
                line_height: LineHeight::Absolute(Pixels(16.0)),
                font: font::sans(),
                align_x: TextAlignment::Center,
                align_y: alignment::Vertical::Top,
                ..Text::default()
            });
        }
    }

    fn draw_pie(&self, frame: &mut Frame, t: &Tokens, plot: Rectangle, hovered: Option<usize>) {
        let Some(series) = self.series.first() else {
            return;
        };
        let pie = self.pie_geometry(plot);

        for (index, slice) in Pie::slices(&series.values).into_iter().enumerate() {
            let grow = if hovered == Some(index) { 4.0 } else { 0.0 };
            let slice = Path::new(|builder| pie.slice(builder, slice, grow));

            frame.fill(&slice, self.color(t, index));
            frame.stroke(
                &slice,
                Stroke::default().with_color(t.palette.card).with_width(2.0),
            );
        }
    }

    /// `ChartLegendContent`: `flex items-center justify-center gap-4 pt-3`,
    /// with `h-2 w-2 rounded-[2px]` swatches.
    fn legend_element<'a, Message: 'a>(&self) -> Element<'a, Message> {
        let colors = self.colors();
        let labels: Vec<String> = match self.kind {
            Kind::Pie => self.categories.clone(),
            _ => self.series.iter().map(|s| s.label.clone()).collect(),
        };

        let items = labels.into_iter().enumerate().map(|(index, label)| {
            let colors = colors.clone();

            row![
                swatch(8.0, move |t| color(&colors, t, index)),
                text(label)
                    .size(style::text::XS)
                    .line_height(style::text::leading(style::text::XS)),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
            .into()
        });

        container(
            Row::with_children(items)
                .spacing(16)
                .align_y(Alignment::Center),
        )
        .padding(Padding::ZERO.top(12))
        .into()
    }

    /// `ChartTooltipContent` for a category: `grid min-w-32 gap-1.5
    /// rounded-lg border border-border/50 bg-background px-2.5 py-1.5 text-xs
    /// shadow-xl`.
    fn tooltip_element<'a, Message: 'a>(&self, index: usize) -> Element<'a, Message> {
        let colors = self.colors();

        let rows: Vec<(String, String, usize)> = match self.kind {
            Kind::Pie => vec![(
                self.categories.get(index).cloned().unwrap_or_default(),
                format_value(
                    self.series
                        .first()
                        .and_then(|s| s.values.get(index))
                        .copied()
                        .unwrap_or(0.0),
                ),
                index,
            )],
            _ => self
                .series
                .iter()
                .enumerate()
                .map(|(i, series)| {
                    (
                        series.label.clone(),
                        format_value(series.values.get(index).copied().unwrap_or(0.0)),
                        i,
                    )
                })
                .collect(),
        };

        // `leading-none`
        let line = LineHeight::Absolute(Pixels(style::text::XS));

        let labels = Column::with_children(rows.iter().map(|(label, _, i)| {
            let colors = colors.clone();
            let i = *i;

            // `h-2.5 w-2.5 rounded-[2px]`, `gap-2`
            row![
                swatch(10.0, move |t| color(&colors, t, i)),
                text(label.clone())
                    .size(style::text::XS)
                    .line_height(line)
                    .style(|theme: &Theme| text::Style {
                        color: Some(tokens(theme).palette.muted_foreground),
                    }),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .into()
        }))
        .spacing(6);

        // `font-mono font-medium text-foreground tabular-nums`
        let values = Column::with_children(rows.iter().map(|(_, value, _)| {
            text(value.clone())
                .size(style::text::XS)
                .line_height(line)
                .font(font::weight(font::mono(), iced::font::Weight::Medium))
                .into()
        }))
        .spacing(6)
        .align_x(Alignment::End);

        let mut content = Column::new().spacing(6);

        if self.kind != Kind::Pie {
            // `font-medium`
            content = content.push(
                text(self.categories.get(index).cloned().unwrap_or_default())
                    .size(style::text::XS)
                    .line_height(style::text::leading(style::text::XS))
                    .font(font::medium()),
            );
        }

        // `min-w-32` within a 1px border and `px-2.5`.
        content = content.push(Element::new(Between {
            start: labels.into(),
            end: values.into(),
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

/// Where the top of a 12px Geist line (16px high) goes relative to the tick,
/// so that its baseline lies `0.71em` below it like SVG text with
/// `dy="0.71em"`.
const TICK_BASELINE: f32 = -3.7;

/// A small colored square, like the indicators of the legend and tooltip.
fn swatch<'a, Message: 'a>(
    size: f32,
    color: impl Fn(&Tokens) -> Color + 'a,
) -> Element<'a, Message> {
    container(Space::new().width(size).height(size))
        .style(move |theme| container::Style {
            background: Some(Background::Color(color(&tokens(theme)))),
            border: style::rounded(2.0),
            ..container::Style::default()
        })
        .into()
}

impl<'a, Message: 'a> From<Chart> for Element<'a, Message> {
    fn from(chart: Chart) -> Self {
        let legend = chart.legend.then(|| chart.legend_element());
        let tooltips = if chart.tooltip {
            (0..chart.categories.len())
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

/// The state of a chart: the hovered category (or slice) and the pointer.
#[derive(Debug, Default)]
pub struct State {
    hovered: Option<usize>,
    cursor: Option<Point>,
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

    /// The child index of the tooltip of the hovered category.
    fn tooltip_index(&self, state: &State) -> Option<usize> {
        let hovered = state.hovered?;

        (hovered < self.tooltips.len()).then(|| hovered + usize::from(self.legend.is_some()))
    }

    /// The children shown: the legend and the hovered tooltip.
    fn shown(&self, state: &State) -> impl Iterator<Item = usize> {
        let legend = self.legend.is_some().then_some(0);

        legend.into_iter().chain(self.tooltip_index(state))
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
        let plot = self.chart.plot(size);
        let margin = self.chart.margin;
        let state = tree.state.downcast_ref::<State>();
        let (hovered, cursor) = (state.hovered, state.cursor);

        let mut nodes = Vec::new();
        let mut trees = tree.children.iter_mut();

        if let Some(legend) = &mut self.legend {
            let available = (size.width - margin.left - margin.right).max(0.0);
            let node = legend.as_widget_mut().layout(
                trees.next().expect("legend tree"),
                renderer,
                &layout::Limits::new(Size::ZERO, Size::new(available, f32::INFINITY)),
            );
            let legend_size = node.size();

            nodes.push(node.move_to(Point::new(
                margin.left + (available - legend_size.width) / 2.0,
                size.height - margin.bottom - legend_size.height,
            )));
        }

        for (index, (tooltip, tree)) in self.tooltips.iter_mut().zip(trees).enumerate() {
            let node = tooltip.as_widget_mut().layout(
                tree,
                renderer,
                &layout::Limits::new(Size::ZERO, Size::INFINITE),
            );

            let position = match (hovered, cursor) {
                (Some(hovered), Some(cursor)) if hovered == index => {
                    self.chart
                        .tooltip_position(plot, hovered, cursor, node.size())
                }
                _ => Point::ORIGIN,
            };

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
        let position = cursor.position_in(bounds);
        let plot = self.chart.plot(bounds.size());

        let hovered = position.and_then(|point| self.chart.hovered(plot, point));

        let state = tree.state.downcast_mut::<State>();

        if hovered != state.hovered || (hovered.is_some() && position != state.cursor) {
            state.hovered = hovered;
            state.cursor = position;

            shell.invalidate_layout();
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

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            let mut frame = Frame::new(renderer, bounds.size());
            self.chart
                .draw_plot(&mut frame, &t, bounds.size(), state.hovered);
            renderer.draw_geometry(frame.into_geometry());
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
            let tooltip = &self.tooltips[index - usize::from(self.legend.is_some())];
            let layer = Rectangle {
                x: bounds.x - 32.0,
                y: bounds.y - 32.0,
                width: bounds.width + 64.0,
                height: bounds.height + 64.0,
            };

            renderer.with_layer(layer, |renderer| {
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
        }
    }
}

/// Places `start` and `end` at both ends of a row at least `min_width` wide
/// (`flex justify-between`).
struct Between<'a, Message> {
    start: Element<'a, Message>,
    end: Element<'a, Message>,
    min_width: f32,
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Between<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.start), Tree::new(&self.end)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.start, &self.end]);
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
        let start = self
            .start
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &loose);
        let end = self
            .end
            .as_widget_mut()
            .layout(&mut tree.children[1], renderer, &loose);

        let (a, b) = (start.size(), end.size());
        let width = (a.width + b.width).max(self.min_width);
        let height = a.height.max(b.height);

        layout::Node::with_children(
            Size::new(width, height),
            vec![start, end.move_to(Point::new(width - b.width, 0.0))],
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        for ((child, tree), layout) in [&mut self.start, &mut self.end]
            .into_iter()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
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
        for ((child, tree), layout) in [&self.start, &self.end]
            .into_iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
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
    }
}
