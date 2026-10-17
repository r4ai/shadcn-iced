import { useState } from "react"
import { BoldIcon, ItalicIcon, UnderlineIcon } from "lucide-react"

import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group"
import type { Fixtures } from "@/fixtures"

function Single() {
  const [value, setValue] = useState<string[]>([])

  return (
    <div className="flex flex-col gap-3">
      <ToggleGroup
        variant="outline"
        spacing={0}
        value={value}
        onValueChange={setValue}
      >
        <ToggleGroupItem value="left">Left</ToggleGroupItem>
        <ToggleGroupItem value="center">Center</ToggleGroupItem>
        <ToggleGroupItem value="right">Right</ToggleGroupItem>
      </ToggleGroup>
      <p className="text-sm text-muted-foreground">
        Selected: {value[0] ?? "none"}
      </p>
    </div>
  )
}

function Formatting(props: React.ComponentProps<typeof ToggleGroup>) {
  return (
    <ToggleGroup {...props}>
      <ToggleGroupItem value="bold" aria-label="Toggle bold">
        <BoldIcon />
      </ToggleGroupItem>
      <ToggleGroupItem value="italic" aria-label="Toggle italic">
        <ItalicIcon />
      </ToggleGroupItem>
      <ToggleGroupItem value="underline" aria-label="Toggle underline">
        <UnderlineIcon />
      </ToggleGroupItem>
    </ToggleGroup>
  )
}

export default {
  multiple: () => <Formatting multiple variant="outline" spacing={0} />,
  single: () => <Single />,
  sizes: () => (
    <div className="flex items-center gap-4">
      <Formatting multiple variant="outline" size="sm" spacing={0} />
      <Formatting multiple variant="outline" spacing={0} />
      <Formatting multiple variant="outline" size="lg" spacing={0} />
    </div>
  ),
  spacing: () => (
    <Formatting multiple variant="outline" defaultValue={["bold"]} />
  ),
  vertical: () => (
    <Formatting
      multiple
      variant="outline"
      spacing={0}
      orientation="vertical"
      defaultValue={["italic"]}
    />
  ),
  disabled: () => <Formatting disabled defaultValue={["bold"]} />,
} satisfies Fixtures
