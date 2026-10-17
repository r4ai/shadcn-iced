import { useState } from "react"
import { BoldIcon, ItalicIcon } from "lucide-react"

import { Toggle } from "@/components/ui/toggle"
import type { Fixtures } from "@/fixtures"

function Controlled() {
  const [pressed, setPressed] = useState(false)

  return (
    <div className="flex flex-col gap-3">
      <Toggle
        className="self-start"
        aria-label="Toggle italic"
        pressed={pressed}
        onPressedChange={setPressed}
      >
        <ItalicIcon />
        Italic
      </Toggle>
      <p className="text-sm text-muted-foreground">
        {pressed ? "Italic is on" : "Italic is off"}
      </p>
    </div>
  )
}

export default {
  default: () => (
    <Toggle aria-label="Toggle italic">
      <ItalicIcon />
      Italic
    </Toggle>
  ),
  outline: () => (
    <Toggle variant="outline" aria-label="Toggle italic">
      <ItalicIcon />
      Italic
    </Toggle>
  ),
  icon: () => (
    <Toggle aria-label="Toggle bold">
      <BoldIcon />
    </Toggle>
  ),
  sizes: () => (
    <div className="flex items-center gap-2">
      <Toggle variant="outline" size="sm" aria-label="Toggle small">
        Small
      </Toggle>
      <Toggle variant="outline" aria-label="Toggle default">
        Default
      </Toggle>
      <Toggle variant="outline" size="lg" aria-label="Toggle large">
        Large
      </Toggle>
    </div>
  ),
  disabled: () => (
    <div className="flex items-center gap-2">
      <Toggle aria-label="Toggle disabled" disabled>
        Disabled
      </Toggle>
      <Toggle
        variant="outline"
        aria-label="Toggle disabled pressed"
        defaultPressed
        disabled
      >
        Pressed
      </Toggle>
    </div>
  ),
  keyboard: () => <Controlled />,
} satisfies Fixtures
