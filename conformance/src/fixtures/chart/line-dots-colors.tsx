import { TrendingUp } from "lucide-react"
import { CartesianGrid, Dot, Line, LineChart } from "recharts"

import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from "@/components/ui/chart"

const chartData = [
  { browser: "chrome", visitors: 275, fill: "var(--color-chrome)" },
  { browser: "safari", visitors: 200, fill: "var(--color-safari)" },
  { browser: "firefox", visitors: 187, fill: "var(--color-firefox)" },
  { browser: "edge", visitors: 173, fill: "var(--color-edge)" },
  { browser: "other", visitors: 90, fill: "var(--color-other)" },
]

const chartConfig = {
  visitors: {
    label: "Visitors",
    color: "var(--chart-2)",
  },
  chrome: {
    label: "Chrome",
    color: "var(--chart-1)",
  },
  safari: {
    label: "Safari",
    color: "var(--chart-2)",
  },
  firefox: {
    label: "Firefox",
    color: "var(--chart-3)",
  },
  edge: {
    label: "Edge",
    color: "var(--chart-4)",
  },
  other: {
    label: "Other",
    color: "var(--chart-5)",
  },
} satisfies ChartConfig

export default function Fixture() {
  return (
    <ChartContainer config={chartConfig} className="h-[250px] w-full">
      <LineChart
        accessibilityLayer
        data={chartData}
        margin={{
          top: 24,
          left: 24,
          right: 24,
        }}
      >
        <CartesianGrid vertical={false} />
        <ChartTooltip
          cursor={false}
          content={
            <ChartTooltipContent
              indicator="line"
              nameKey="visitors"
              hideLabel
            />
          }
        />
        <Line
          dataKey="visitors"
          type="natural"
          stroke="var(--color-visitors)"
          strokeWidth={2}
          dot={({ payload, ...props }) => {
            return (
              <Dot
                key={payload.browser}
                r={5}
                cx={props.cx}
                cy={props.cy}
                fill={payload.fill}
                stroke={payload.fill}
              />
            )
          }}
        />
      </LineChart>
    </ChartContainer>
  )
}
