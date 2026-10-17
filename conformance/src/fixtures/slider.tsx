import { useState } from "react"

import { Slider } from "@/components/ui/slider"
import type { Fixtures } from "@/fixtures"

function Controlled() {
  const [value, setValue] = useState(50)

  return (
    <div className="flex flex-col gap-3">
      <Slider
        value={[value]}
        max={100}
        step={1}
        onValueChange={(next) => setValue(Array.isArray(next) ? next[0] : next)}
      />
      <p className="text-sm text-muted-foreground">Value: {value}</p>
    </div>
  )
}

export default {
  default: () => <Slider defaultValue={[50]} max={100} step={1} />,
  controlled: () => <Controlled />,
  range: () => <Slider defaultValue={[25, 75]} max={100} step={1} />,
  vertical: () => (
    <div className="h-40">
      <Slider defaultValue={[50]} max={100} step={1} orientation="vertical" />
    </div>
  ),
  disabled: () => <Slider defaultValue={[50]} max={100} step={1} disabled />,
} satisfies Fixtures
