import { useState } from "react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import type { Fixtures } from "@/fixtures"

function Typing() {
  const [value, setValue] = useState("")

  return (
    <div className="flex flex-col gap-3">
      <Input
        type="email"
        placeholder="Email"
        value={value}
        onChange={(event) => setValue(event.target.value)}
      />
      <p className="text-sm text-muted-foreground">
        {value ? `You typed: ${value}` : "Nothing typed"}
      </p>
    </div>
  )
}

const fixtures = {
  default: () => <Input type="email" placeholder="Email" />,
  typing: () => <Typing />,
  states: () => (
    <div className="flex flex-col gap-3">
      <Input placeholder="Name" defaultValue="Evil Rabbit" />
      <Input placeholder="Email" disabled />
      <Input placeholder="Email" defaultValue="invalid@" aria-invalid />
      <Input type="password" placeholder="Password" defaultValue="secret" />
    </div>
  ),
  "with-button": () => (
    <div className="flex w-full max-w-sm items-center gap-2">
      <Input type="email" placeholder="Email" />
      <Button type="submit" variant="outline">
        Subscribe
      </Button>
    </div>
  ),
} satisfies Fixtures

// The page is 768px wide so that `md:text-sm` applies, like on iced; the
// fixtures are `w-80` wide.
const narrow = new Set(["default", "typing", "states", "with-button"])

export default Object.fromEntries(
  Object.entries(fixtures).map(([id, render]) => [
    id,
    narrow.has(id) ? () => <div className="w-80">{render()}</div> : render,
  ])
) satisfies Fixtures
