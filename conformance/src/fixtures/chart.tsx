import {
  Area,
  AreaChart,
  Bar,
  BarChart,
  CartesianGrid,
  Line,
  LineChart,
  XAxis,
} from "recharts"

import {
  ChartContainer,
  ChartLegend,
  ChartLegendContent,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from "@/components/ui/chart"
import type { Fixtures } from "@/fixtures"

// The chart examples of shadcn/ui (apps/v4/registry/new-york-v4/charts),
// with a fixed size instead of their card.
import ChartAreaAxes from "./chart/area-axes"
import ChartAreaDefault from "./chart/area-default"
import ChartAreaGradient from "./chart/area-gradient"
import ChartAreaIcons from "./chart/area-icons"
import ChartAreaLegend from "./chart/area-legend"
import ChartAreaLinear from "./chart/area-linear"
import ChartAreaStackedExpand from "./chart/area-stacked-expand"
import ChartAreaStep from "./chart/area-step"
import ChartBarActive from "./chart/bar-active"
import ChartBarDefault from "./chart/bar-default"
import ChartBarHorizontal from "./chart/bar-horizontal"
import ChartBarLabel from "./chart/bar-label"
import ChartBarLabelCustom from "./chart/bar-label-custom"
import ChartBarMixed from "./chart/bar-mixed"
import ChartBarMultiple from "./chart/bar-multiple"
import ChartBarNegative from "./chart/bar-negative"
import ChartBarStacked from "./chart/bar-stacked"
import ChartLineDefault from "./chart/line-default"
import ChartLineDots from "./chart/line-dots"
import ChartLineDotsColors from "./chart/line-dots-colors"
import ChartLineLabel from "./chart/line-label"
import ChartLineLabelCustom from "./chart/line-label-custom"
import ChartLineLinear from "./chart/line-linear"
import ChartLineMultiple from "./chart/line-multiple"
import ChartLineStep from "./chart/line-step"
import ChartPieDonut from "./chart/pie-donut"
import ChartPieDonutActive from "./chart/pie-donut-active"
import ChartPieDonutText from "./chart/pie-donut-text"
import ChartPieLabel from "./chart/pie-label"
import ChartPieLabelCustom from "./chart/pie-label-custom"
import ChartPieLabelList from "./chart/pie-label-list"
import ChartPieLegend from "./chart/pie-legend"
import ChartPieSeparatorNone from "./chart/pie-separator-none"
import ChartPieSimple from "./chart/pie-simple"
import ChartPieStacked from "./chart/pie-stacked"
import ChartRadarDefault from "./chart/radar-default"
import ChartRadarDots from "./chart/radar-dots"
import ChartRadarGridCircle from "./chart/radar-grid-circle"
import ChartRadarGridCircleFill from "./chart/radar-grid-circle-fill"
import ChartRadarGridCircleNoLines from "./chart/radar-grid-circle-no-lines"
import ChartRadarGridCustom from "./chart/radar-grid-custom"
import ChartRadarGridFill from "./chart/radar-grid-fill"
import ChartRadarGridNone from "./chart/radar-grid-none"
import ChartRadarIcons from "./chart/radar-icons"
import ChartRadarLegend from "./chart/radar-legend"
import ChartRadarLinesOnly from "./chart/radar-lines-only"
import ChartRadarMultiple from "./chart/radar-multiple"
import ChartRadarRadius from "./chart/radar-radius"
import ChartRadialGrid from "./chart/radial-grid"
import ChartRadialLabel from "./chart/radial-label"
import ChartRadialShape from "./chart/radial-shape"
import ChartRadialSimple from "./chart/radial-simple"
import ChartRadialStacked from "./chart/radial-stacked"
import ChartRadialText from "./chart/radial-text"
import ChartTooltipAdvanced from "./chart/tooltip-advanced"
import ChartTooltipDefault from "./chart/tooltip-default"
import ChartTooltipFormatter from "./chart/tooltip-formatter"
import ChartTooltipIcons from "./chart/tooltip-icons"
import ChartTooltipIndicatorLine from "./chart/tooltip-indicator-line"
import ChartTooltipIndicatorNone from "./chart/tooltip-indicator-none"
import ChartTooltipLabelCustom from "./chart/tooltip-label-custom"
import ChartTooltipLabelFormatter from "./chart/tooltip-label-formatter"
import ChartTooltipLabelNone from "./chart/tooltip-label-none"

const chartData = [
  { month: "January", desktop: 186, mobile: 80 },
  { month: "February", desktop: 305, mobile: 200 },
  { month: "March", desktop: 237, mobile: 120 },
  { month: "April", desktop: 73, mobile: 190 },
  { month: "May", desktop: 209, mobile: 130 },
  { month: "June", desktop: 214, mobile: 140 },
]

const chartConfig = {
  desktop: {
    label: "Desktop",
    color: "var(--chart-1)",
  },
  mobile: {
    label: "Mobile",
    color: "var(--chart-2)",
  },
} satisfies ChartConfig

const month = (value: string) => value.slice(0, 3)

export default {
  "area-axes": () => <ChartAreaAxes />,
  "area-default": () => <ChartAreaDefault />,
  "area-gradient": () => <ChartAreaGradient />,
  "area-icons": () => <ChartAreaIcons />,
  "area-legend": () => <ChartAreaLegend />,
  "area-linear": () => <ChartAreaLinear />,
  "area-stacked-expand": () => <ChartAreaStackedExpand />,
  "area-step": () => <ChartAreaStep />,
  "bar-active": () => <ChartBarActive />,
  "bar-default": () => <ChartBarDefault />,
  "bar-horizontal": () => <ChartBarHorizontal />,
  "bar-label": () => <ChartBarLabel />,
  "bar-label-custom": () => <ChartBarLabelCustom />,
  "bar-mixed": () => <ChartBarMixed />,
  "bar-multiple": () => <ChartBarMultiple />,
  "bar-negative": () => <ChartBarNegative />,
  "bar-stacked": () => <ChartBarStacked />,
  "line-default": () => <ChartLineDefault />,
  "line-dots": () => <ChartLineDots />,
  "line-dots-colors": () => <ChartLineDotsColors />,
  "line-label": () => <ChartLineLabel />,
  "line-label-custom": () => <ChartLineLabelCustom />,
  "line-linear": () => <ChartLineLinear />,
  "line-multiple": () => <ChartLineMultiple />,
  "line-step": () => <ChartLineStep />,
  "pie-donut": () => <ChartPieDonut />,
  "pie-donut-active": () => <ChartPieDonutActive />,
  "pie-donut-text": () => <ChartPieDonutText />,
  "pie-label": () => <ChartPieLabel />,
  "pie-label-custom": () => <ChartPieLabelCustom />,
  "pie-label-list": () => <ChartPieLabelList />,
  "pie-legend": () => <ChartPieLegend />,
  "pie-separator-none": () => <ChartPieSeparatorNone />,
  "pie-simple": () => <ChartPieSimple />,
  "pie-stacked": () => <ChartPieStacked />,
  "radar-default": () => <ChartRadarDefault />,
  "radar-dots": () => <ChartRadarDots />,
  "radar-grid-circle": () => <ChartRadarGridCircle />,
  "radar-grid-circle-fill": () => <ChartRadarGridCircleFill />,
  "radar-grid-circle-no-lines": () => <ChartRadarGridCircleNoLines />,
  "radar-grid-custom": () => <ChartRadarGridCustom />,
  "radar-grid-fill": () => <ChartRadarGridFill />,
  "radar-grid-none": () => <ChartRadarGridNone />,
  "radar-icons": () => <ChartRadarIcons />,
  "radar-legend": () => <ChartRadarLegend />,
  "radar-lines-only": () => <ChartRadarLinesOnly />,
  "radar-multiple": () => <ChartRadarMultiple />,
  "radar-radius": () => <ChartRadarRadius />,
  "radial-grid": () => <ChartRadialGrid />,
  "radial-label": () => <ChartRadialLabel />,
  "radial-shape": () => <ChartRadialShape />,
  "radial-simple": () => <ChartRadialSimple />,
  "radial-stacked": () => <ChartRadialStacked />,
  "radial-text": () => <ChartRadialText />,
  "tooltip-advanced": () => <ChartTooltipAdvanced />,
  "tooltip-default": () => <ChartTooltipDefault />,
  "tooltip-formatter": () => <ChartTooltipFormatter />,
  "tooltip-icons": () => <ChartTooltipIcons />,
  "tooltip-indicator-line": () => <ChartTooltipIndicatorLine />,
  "tooltip-indicator-none": () => <ChartTooltipIndicatorNone />,
  "tooltip-label-custom": () => <ChartTooltipLabelCustom />,
  "tooltip-label-formatter": () => <ChartTooltipLabelFormatter />,
  "tooltip-label-none": () => <ChartTooltipLabelNone />,
  bar: () => (
    <ChartContainer config={chartConfig} className="h-[250px] w-full">
      <BarChart accessibilityLayer data={chartData}>
        <CartesianGrid vertical={false} />
        <XAxis
          dataKey="month"
          tickLine={false}
          tickMargin={10}
          axisLine={false}
          tickFormatter={month}
        />
        <ChartTooltip content={<ChartTooltipContent />} />
        <ChartLegend content={<ChartLegendContent />} />
        <Bar dataKey="desktop" fill="var(--color-desktop)" radius={4} />
        <Bar dataKey="mobile" fill="var(--color-mobile)" radius={4} />
      </BarChart>
    </ChartContainer>
  ),
  line: () => (
    <ChartContainer config={chartConfig} className="h-[250px] w-full">
      <LineChart
        accessibilityLayer
        data={chartData}
        margin={{ left: 12, right: 12 }}
      >
        <CartesianGrid vertical={false} />
        <XAxis
          dataKey="month"
          tickLine={false}
          axisLine={false}
          tickMargin={8}
          tickFormatter={month}
        />
        <ChartTooltip cursor={false} content={<ChartTooltipContent />} />
        <ChartLegend content={<ChartLegendContent />} />
        <Line
          dataKey="desktop"
          type="natural"
          stroke="var(--color-desktop)"
          strokeWidth={2}
          dot={{ fill: "var(--color-desktop)" }}
          activeDot={{ r: 6 }}
        />
        <Line
          dataKey="mobile"
          type="natural"
          stroke="var(--color-mobile)"
          strokeWidth={2}
          dot={{ fill: "var(--color-mobile)" }}
          activeDot={{ r: 6 }}
        />
      </LineChart>
    </ChartContainer>
  ),
  "area-stacked": () => (
    <ChartContainer config={chartConfig} className="h-[250px] w-full">
      <AreaChart
        accessibilityLayer
        data={chartData}
        margin={{ left: 12, right: 12 }}
      >
        <CartesianGrid vertical={false} />
        <XAxis
          dataKey="month"
          tickLine={false}
          axisLine={false}
          tickMargin={8}
          tickFormatter={month}
        />
        <ChartTooltip
          cursor={false}
          content={<ChartTooltipContent indicator="dot" />}
        />
        <ChartLegend content={<ChartLegendContent />} />
        <Area
          dataKey="desktop"
          type="natural"
          fill="var(--color-desktop)"
          fillOpacity={0.4}
          stroke="var(--color-desktop)"
          stackId="a"
        />
        <Area
          dataKey="mobile"
          type="natural"
          fill="var(--color-mobile)"
          fillOpacity={0.4}
          stroke="var(--color-mobile)"
          stackId="a"
        />
      </AreaChart>
    </ChartContainer>
  ),
} satisfies Fixtures
