import { useState } from "react"

import { Checkbox } from "@/components/ui/checkbox"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import type { Fixtures } from "@/fixtures"

function WithInput() {
  const [value, setValue] = useState("")

  return (
    <div className="grid w-full max-w-sm items-center gap-3">
      <Label htmlFor="email">Email</Label>
      <Input
        type="email"
        id="email"
        placeholder="Email"
        value={value}
        onChange={(event) => setValue(event.target.value)}
      />
      {value && <p className="text-sm text-muted-foreground">Typed: {value}</p>}
    </div>
  )
}

const fixtures = {
  default: () => (
    <div className="flex items-center gap-3">
      <Checkbox id="terms" />
      <Label htmlFor="terms">Accept terms and conditions</Label>
    </div>
  ),
  "with-input": () => <WithInput />,
  disabled: () => (
    <div className="flex items-center gap-3">
      <Checkbox id="terms" disabled />
      <Label htmlFor="terms">Accept terms and conditions</Label>
    </div>
  ),
} satisfies Fixtures

// The page is 768px wide so that `md:text-sm` applies, like on iced; the
// fixtures are `w-80` wide.
const narrow = new Set(["with-input"])

export default Object.fromEntries(
  Object.entries(fixtures).map(([id, render]) => [
    id,
    narrow.has(id) ? () => <div className="w-80">{render()}</div> : render,
  ])
) satisfies Fixtures
