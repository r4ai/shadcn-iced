//! Beautiful charts, drawn on an iced canvas.
//!
//! Port of <https://ui.shadcn.com/docs/components/chart>. shadcn/ui wraps
//! Recharts; this module draws the same kinds of charts (bar, line, area and
//! pie) with the same look: dashed horizontal grid, muted axis labels, the
//! `--chart-*` colors, a legend and a tooltip on hover.
//!
//! ```no_run
//! # use iced_ui::chart::{self, Series};
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! chart::bar(["January", "February", "March"])
//!     .series(Series::new("Desktop", [186.0, 305.0, 237.0]))
//!     .series(Series::new("Mobile", [80.0, 200.0, 120.0]))
//!     .into()
//! # }
//! ```
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, Tokens, font, tokens};

use iced::alignment;
use iced::widget::canvas::{self, Canvas, Frame, Path, Stroke, Text, path};
use iced::widget::text::Alignment as TextAlignment;
use iced::{Color, Event, Length, Pixels, Point, Rectangle, Renderer, Size, Theme, Vector, mouse};

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
    grid: bool,
    stacked: bool,
    dots: bool,
    donut: bool,
}

fn chart(kind: Kind, categories: impl IntoIterator<Item = impl Into<String>>) -> Chart {
    Chart {
        kind,
        categories: categories.into_iter().map(Into::into).collect(),
        series: Vec::new(),
        height: 250.0,
        legend: true,
        tooltip: true,
        grid: true,
        stacked: false,
        dots: false,
        donut: false,
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

    /// Sets the height. Defaults to 250.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Shows or hides the legend.
    pub fn legend(mut self, legend: bool) -> Self {
        self.legend = legend;
        self
    }

    /// Shows or hides the tooltip.
    pub fn tooltip(mut self, tooltip: bool) -> Self {
        self.tooltip = tooltip;
        self
    }

    /// Shows or hides the grid.
    pub fn grid(mut self, grid: bool) -> Self {
        self.grid = grid;
        self
    }

    /// Stacks the series of bar and area charts.
    pub fn stacked(mut self, stacked: bool) -> Self {
        self.stacked = stacked;
        self
    }

    /// Draws a dot on each value of line charts.
    pub fn dots(mut self, dots: bool) -> Self {
        self.dots = dots;
        self
    }

    /// Draws a donut instead of a pie.
    pub fn donut(mut self, donut: bool) -> Self {
        self.donut = donut;
        self
    }

    fn color(&self, tokens: &Tokens, index: usize) -> Color {
        if self.kind == Kind::Pie {
            return tokens.palette.chart[index % 5];
        }

        self.series[index]
            .color
            .unwrap_or(tokens.palette.chart[index % 5])
    }

    fn max(&self) -> f32 {
        let max = if self.stacked {
            (0..self.categories.len())
                .map(|i| {
                    self.series
                        .iter()
                        .map(|s| s.values.get(i).copied().unwrap_or(0.0))
                        .sum::<f32>()
                })
                .fold(0.0, f32::max)
        } else {
            self.series
                .iter()
                .flat_map(|s| s.values.iter().copied())
                .fold(0.0, f32::max)
        };

        nice(max)
    }
}

/// Rounds a maximum up to a "nice" number for the axis.
fn nice(max: f32) -> f32 {
    if max <= 0.0 {
        return 1.0;
    }

    let magnitude = 10f32.powf(max.log10().floor());
    let normalized = max / magnitude;

    let nice = [1.0, 1.2, 1.5, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0]
        .into_iter()
        .find(|step| normalized <= *step)
        .unwrap_or(10.0);

    nice * magnitude
}

fn format_value(value: f32) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}

/// The state of a chart.
#[derive(Debug, Default)]
pub struct State {
    hovered: Option<usize>,
    cursor: Option<Point>,
}

const AXIS: f32 = 24.0;
const LEGEND: f32 = 28.0;

impl Chart {
    fn plot(&self, bounds: Size) -> Rectangle {
        let bottom = if self.kind == Kind::Pie { 0.0 } else { AXIS };
        let legend = if self.legend { LEGEND } else { 0.0 };

        Rectangle {
            x: 12.0,
            y: 8.0,
            width: (bounds.width - 24.0).max(0.0),
            height: (bounds.height - 8.0 - bottom - legend).max(0.0),
        }
    }

    fn category_at(&self, plot: Rectangle, point: Point) -> Option<usize> {
        let count = self.categories.len();

        if count == 0 || !plot.contains(point) {
            return None;
        }

        let band = plot.width / count as f32;
        Some((((point.x - plot.x) / band) as usize).min(count - 1))
    }

    fn slice_at(&self, plot: Rectangle, point: Point) -> Option<usize> {
        let values = &self.series.first()?.values;
        let total: f32 = values.iter().sum();
        let center = plot.center();
        let radius = plot.width.min(plot.height) / 2.0;
        let offset = point - center;
        let distance = (offset.x * offset.x + offset.y * offset.y).sqrt();

        if distance > radius || (self.donut && distance < radius * 0.6) || total <= 0.0 {
            return None;
        }

        let mut angle = offset.y.atan2(offset.x) + std::f32::consts::FRAC_PI_2;
        if angle < 0.0 {
            angle += std::f32::consts::TAU;
        }

        let mut start = 0.0;
        for (index, value) in values.iter().enumerate() {
            let sweep = value / total * std::f32::consts::TAU;
            if angle >= start && angle < start + sweep {
                return Some(index);
            }
            start += sweep;
        }

        None
    }
}

impl<Message> canvas::Program<Message> for Chart {
    type State = State;

    fn update(
        &self,
        state: &mut State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        if !self.tooltip {
            return None;
        }

        if let Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::CursorLeft) = event {
            let position = cursor.position_in(bounds);
            let plot = self.plot(bounds.size());

            let hovered = position.and_then(|point| match self.kind {
                Kind::Pie => self.slice_at(plot, point),
                _ => self.category_at(plot, point),
            });

            if hovered != state.hovered || (hovered.is_some() && position != state.cursor) {
                state.hovered = hovered;
                state.cursor = position;
                return Some(canvas::Action::request_redraw());
            }
        }

        None
    }

    fn draw(
        &self,
        state: &State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let t = tokens(theme);
        let p = t.palette;
        let mut frame = Frame::new(renderer, bounds.size());
        let plot = self.plot(bounds.size());
        let count = self.categories.len();

        if self.kind == Kind::Pie {
            self.draw_pie(&mut frame, &t, plot, state.hovered);
        } else if count > 0 {
            let max = self.max();
            let y = |value: f32| plot.y + plot.height - value / max * plot.height;
            let band = plot.width / count as f32;

            if self.grid {
                for step in 0..=4 {
                    let line_y = plot.y + plot.height * step as f32 / 4.0;
                    frame.stroke(
                        &Path::line(
                            Point::new(plot.x, line_y),
                            Point::new(plot.x + plot.width, line_y),
                        ),
                        Stroke {
                            line_dash: if step == 4 {
                                canvas::LineDash::default()
                            } else {
                                canvas::LineDash {
                                    segments: &[3.0, 3.0],
                                    offset: 0,
                                }
                            },
                            ..Stroke::default()
                                .with_color(alpha(p.border, if step == 4 { 1.0 } else { 0.8 }))
                                .with_width(1.0)
                        },
                    );
                }
            }

            if let Some(hovered) = state.hovered {
                match self.kind {
                    Kind::Bar => frame.fill_rectangle(
                        Point::new(plot.x + band * hovered as f32, plot.y),
                        Size::new(band, plot.height),
                        p.muted,
                    ),
                    _ => {
                        let x = plot.x + band * (hovered as f32 + 0.5);
                        frame.stroke(
                            &Path::line(Point::new(x, plot.y), Point::new(x, plot.y + plot.height)),
                            Stroke::default().with_color(p.border).with_width(1.0),
                        );
                    }
                }
            }

            match self.kind {
                Kind::Bar => {
                    let groups = if self.stacked {
                        1
                    } else {
                        self.series.len().max(1)
                    };
                    let gap = 4.0;
                    let inner = band * 0.8;
                    let bar_width =
                        ((inner - gap * (groups as f32 - 1.0)) / groups as f32).max(1.0);

                    for category in 0..count {
                        let mut base = 0.0;

                        for (index, series) in self.series.iter().enumerate() {
                            let value = series.values.get(category).copied().unwrap_or(0.0);
                            let slot = if self.stacked { 0 } else { index };
                            let x = plot.x
                                + band * category as f32
                                + (band - inner) / 2.0
                                + slot as f32 * (bar_width + gap);
                            let top = y(base + value);
                            let bottom = y(base);
                            let radius = if self.stacked && index + 1 < self.series.len() {
                                0.0
                            } else {
                                4.0
                            };

                            let rect = Path::new(|builder| {
                                rounded_top(
                                    builder,
                                    Rectangle::new(
                                        Point::new(x, top),
                                        Size::new(bar_width, bottom - top),
                                    ),
                                    radius,
                                );
                            });
                            frame.fill(&rect, self.color(&t, index));

                            if self.stacked {
                                base += value;
                            }
                        }
                    }
                }
                Kind::Line | Kind::Area => {
                    let mut bases = vec![0.0; count];

                    for (index, series) in self.series.iter().enumerate() {
                        let color = self.color(&t, index);
                        let points: Vec<Point> = (0..count)
                            .map(|i| {
                                let value = series.values.get(i).copied().unwrap_or(0.0);
                                let base = if self.stacked && self.kind == Kind::Area {
                                    bases[i]
                                } else {
                                    0.0
                                };
                                Point::new(plot.x + band * (i as f32 + 0.5), y(base + value))
                            })
                            .collect();

                        if self.kind == Kind::Area {
                            let lower: Vec<Point> = (0..count)
                                .map(|i| {
                                    Point::new(
                                        plot.x + band * (i as f32 + 0.5),
                                        y(if self.stacked { bases[i] } else { 0.0 }),
                                    )
                                })
                                .collect();

                            let fill = Path::new(|builder| {
                                smooth(builder, &points, true);
                                for point in lower.iter().rev() {
                                    builder.line_to(*point);
                                }
                                builder.close();
                            });

                            frame.fill(&fill, alpha(color, 0.4));
                        }

                        frame.stroke(
                            &Path::new(|builder| smooth(builder, &points, true)),
                            Stroke::default().with_color(color).with_width(2.0),
                        );

                        if self.dots || state.hovered.is_some() {
                            for (i, point) in points.iter().enumerate() {
                                if self.dots || state.hovered == Some(i) {
                                    frame.fill(&Path::circle(*point, 4.0), color);
                                    frame.stroke(
                                        &Path::circle(*point, 4.0),
                                        Stroke::default().with_color(p.background).with_width(1.5),
                                    );
                                }
                            }
                        }

                        if self.stacked {
                            for (i, base) in bases.iter_mut().enumerate() {
                                *base += series.values.get(i).copied().unwrap_or(0.0);
                            }
                        }
                    }
                }
                Kind::Pie => {}
            }

            // X axis labels (`tickLine={false} axisLine={false}`).
            let skip = ((count as f32 * 48.0) / plot.width.max(1.0))
                .ceil()
                .max(1.0) as usize;

            for (index, category) in self.categories.iter().enumerate() {
                if index % skip != 0 {
                    continue;
                }

                frame.fill_text(Text {
                    content: category.chars().take(3).collect(),
                    position: Point::new(
                        plot.x + band * (index as f32 + 0.5),
                        plot.y + plot.height + 8.0,
                    ),
                    color: p.muted_foreground,
                    size: Pixels(12.0),
                    font: font::sans(),
                    align_x: TextAlignment::Center,
                    align_y: alignment::Vertical::Top,
                    ..Text::default()
                });
            }
        }

        if self.legend {
            self.draw_legend(&mut frame, &t, bounds.size());
        }

        if let (Some(hovered), Some(cursor)) = (state.hovered, state.cursor) {
            self.draw_tooltip(&mut frame, &t, hovered, cursor, bounds.size());
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}

impl Chart {
    fn legend_entries(&self) -> Vec<String> {
        match self.kind {
            Kind::Pie => self.categories.clone(),
            _ => self
                .series
                .iter()
                .map(|series| series.label.clone())
                .collect(),
        }
    }

    fn draw_legend(&self, frame: &mut Frame, t: &Tokens, size: Size) {
        let entries = self.legend_entries();
        let widths: Vec<f32> = entries
            .iter()
            .map(|label| 8.0 + 6.0 + text_width(label, 12.0))
            .collect();
        let total: f32 =
            widths.iter().sum::<f32>() + 16.0 * (widths.len().saturating_sub(1)) as f32;
        let mut x = (size.width - total) / 2.0;
        let y = size.height - LEGEND / 2.0;

        for (index, (label, width)) in entries.iter().zip(widths).enumerate() {
            frame.fill(
                &Path::rounded_rectangle(Point::new(x, y - 4.0), Size::new(8.0, 8.0), 2.0.into()),
                self.color(t, index),
            );
            frame.fill_text(Text {
                content: label.clone(),
                position: Point::new(x + 14.0, y),
                color: t.palette.foreground,
                size: Pixels(12.0),
                font: font::sans(),
                align_y: alignment::Vertical::Center,
                ..Text::default()
            });
            x += width + 16.0;
        }
    }

    fn draw_tooltip(
        &self,
        frame: &mut Frame,
        t: &Tokens,
        hovered: usize,
        cursor: Point,
        size: Size,
    ) {
        let p = t.palette;
        let title = self.categories.get(hovered).cloned().unwrap_or_default();

        let rows: Vec<(String, String, Color)> = match self.kind {
            Kind::Pie => vec![(
                title.clone(),
                format_value(self.series[0].values.get(hovered).copied().unwrap_or(0.0)),
                self.color(t, hovered),
            )],
            _ => self
                .series
                .iter()
                .enumerate()
                .map(|(index, series)| {
                    (
                        series.label.clone(),
                        format_value(series.values.get(hovered).copied().unwrap_or(0.0)),
                        self.color(t, index),
                    )
                })
                .collect(),
        };

        let show_title = self.kind != Kind::Pie;
        let label_width = rows
            .iter()
            .map(|(label, _, _)| text_width(label, 12.0))
            .fold(0.0, f32::max);
        let value_width = rows
            .iter()
            .map(|(_, value, _)| text_width(value, 12.0))
            .fold(0.0, f32::max);
        // `cn-chart-tooltip`: `min-w-32 px-2.5 py-1.5 gap-1.5 text-xs`, rows
        // of a `h-2.5 w-2.5` indicator, `gap-2`, a label and a value.
        let width = (10.0 + 10.0 + 8.0 + label_width + 16.0 + value_width + 10.0)
            .max(text_width(&title, 12.0) + 20.0)
            .max(128.0);
        let row_height = 16.0;
        let gap = 6.0;
        let lines = rows.len() as f32 + if show_title { 1.0 } else { 0.0 };
        let height = 12.0 + lines * row_height + (lines - 1.0).max(0.0) * gap;
        let radius = t.radius_lg();

        let mut origin = cursor + Vector::new(12.0, 12.0);
        if origin.x + width > size.width {
            origin.x = cursor.x - width - 12.0;
        }
        if origin.y + height > size.height {
            origin.y = (size.height - height).max(0.0);
        }

        let panel = Path::rounded_rectangle(origin, Size::new(width, height), radius.into());

        // `shadow-xl`, approximated with stacked translucent layers since
        // canvas paths cannot be blurred.
        for (offset, spread, opacity) in [(8.0, 2.0, 0.03), (12.0, 5.0, 0.03), (16.0, 8.0, 0.02)] {
            frame.fill(
                &Path::rounded_rectangle(
                    origin + Vector::new(-spread / 2.0, offset - spread),
                    Size::new(width + spread, height + spread / 2.0),
                    (radius + spread / 2.0).into(),
                ),
                Color::from_rgba(0.0, 0.0, 0.0, opacity),
            );
        }

        frame.fill(&panel, p.background);
        frame.stroke(
            &panel,
            Stroke::default()
                .with_color(alpha(p.border, 0.5))
                .with_width(1.0),
        );

        let mut y = origin.y + 6.0;

        if show_title {
            frame.fill_text(Text {
                content: title,
                position: Point::new(origin.x + 10.0, y + row_height / 2.0),
                color: p.foreground,
                size: Pixels(12.0),
                font: font::medium(),
                align_y: alignment::Vertical::Center,
                ..Text::default()
            });
            y += row_height + gap;
        }

        for (label, value, color) in rows {
            let center = y + row_height / 2.0;

            frame.fill(
                &Path::rounded_rectangle(
                    Point::new(origin.x + 10.0, center - 5.0),
                    Size::new(10.0, 10.0),
                    2.0.into(),
                ),
                color,
            );
            frame.fill_text(Text {
                content: label,
                position: Point::new(origin.x + 28.0, center),
                color: p.muted_foreground,
                size: Pixels(12.0),
                font: font::sans(),
                align_y: alignment::Vertical::Center,
                ..Text::default()
            });
            frame.fill_text(Text {
                content: value,
                position: Point::new(origin.x + width - 10.0, center),
                color: p.foreground,
                size: Pixels(12.0),
                font: iced::Font {
                    weight: iced::font::Weight::Medium,
                    ..font::mono()
                },
                align_x: TextAlignment::Right,
                align_y: alignment::Vertical::Center,
                ..Text::default()
            });
            y += row_height + gap;
        }
    }

    fn draw_pie(&self, frame: &mut Frame, t: &Tokens, plot: Rectangle, hovered: Option<usize>) {
        let Some(series) = self.series.first() else {
            return;
        };
        let total: f32 = series.values.iter().sum();

        if total <= 0.0 {
            return;
        }

        let center = plot.center();
        let radius = plot.width.min(plot.height) / 2.0;
        let inner = if self.donut { radius * 0.6 } else { 0.0 };
        let mut start = -std::f32::consts::FRAC_PI_2;

        for (index, value) in series.values.iter().enumerate() {
            let sweep = value / total * std::f32::consts::TAU;
            let outer = if hovered == Some(index) {
                radius + 4.0
            } else {
                radius
            };

            let slice = Path::new(|builder| {
                let steps = ((sweep / std::f32::consts::TAU) * 128.0).ceil().max(2.0) as usize;
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

                if inner > 0.0 {
                    for step in (0..=steps).rev() {
                        builder.line_to(point(inner, start + sweep * step as f32 / steps as f32));
                    }
                } else {
                    builder.line_to(center);
                }

                builder.close();
            });

            frame.fill(&slice, self.color(t, index));
            frame.stroke(
                &slice,
                Stroke::default().with_color(t.palette.card).with_width(2.0),
            );
            start += sweep;
        }
    }
}

/// Approximates the width of some text in Geist.
fn text_width(content: &str, size: f32) -> f32 {
    content.chars().count() as f32 * size * 0.56
}

fn rounded_top(builder: &mut path::Builder, rect: Rectangle, radius: f32) {
    let radius = radius.min(rect.width / 2.0).min(rect.height.max(0.0));

    builder.move_to(Point::new(rect.x, rect.y + rect.height));
    builder.line_to(Point::new(rect.x, rect.y + radius));
    builder.arc_to(
        Point::new(rect.x, rect.y),
        Point::new(rect.x + radius, rect.y),
        radius,
    );
    builder.line_to(Point::new(rect.x + rect.width - radius, rect.y));
    builder.arc_to(
        Point::new(rect.x + rect.width, rect.y),
        Point::new(rect.x + rect.width, rect.y + radius),
        radius,
    );
    builder.line_to(Point::new(rect.x + rect.width, rect.y + rect.height));
    builder.close();
}

/// Draws a smooth curve through the points (monotone-like Catmull-Rom).
fn smooth(builder: &mut path::Builder, points: &[Point], move_first: bool) {
    let Some(first) = points.first() else { return };

    if move_first {
        builder.move_to(*first);
    } else {
        builder.line_to(*first);
    }

    for i in 0..points.len().saturating_sub(1) {
        let p0 = if i == 0 { points[0] } else { points[i - 1] };
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = points.get(i + 2).copied().unwrap_or(p2);

        let c1 = Point::new(p1.x + (p2.x - p0.x) / 6.0, p1.y + (p2.y - p0.y) / 6.0);
        let c2 = Point::new(p2.x - (p3.x - p1.x) / 6.0, p2.y - (p3.y - p1.y) / 6.0);

        builder.bezier_curve_to(c1, c2, p2);
    }
}

impl<'a, Message: 'a> From<Chart> for Element<'a, Message> {
    fn from(chart: Chart) -> Self {
        let height = chart.height;

        Canvas::new(chart).width(Length::Fill).height(height).into()
    }
}

#[cfg(test)]
mod tests {
    use super::nice;

    #[test]
    fn nice_maxima() {
        assert_eq!(nice(305.0), 400.0);
        assert_eq!(nice(186.0), 200.0);
        assert_eq!(nice(0.0), 1.0);
        assert_eq!(nice(10.0), 10.0);
    }
}
