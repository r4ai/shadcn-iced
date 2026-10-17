import { useState } from "react"

import { Textarea } from "@/components/ui/textarea"
import type { Fixtures } from "@/fixtures"

function Typing() {
  const [value, setValue] = useState("")

  return (
    <div className="flex flex-col gap-3">
      <Textarea
        placeholder="Type your message here."
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
  default: () => <Textarea placeholder="Type your message here." />,
  typing: () => <Typing />,
  states: () => (
    <div className="flex flex-col gap-3">
      <Textarea
        placeholder="Type your message here."
        defaultValue="Hello, world!"
      />
      <Textarea placeholder="Type your message here." disabled />
      <Textarea
        placeholder="Type your message here."
        defaultValue="This message is invalid."
        aria-invalid
      />
      <Textarea
        placeholder="Type your message here."
        defaultValue={"First line\nSecond line\nThird line\nFourth line"}
      />
    </div>
  ),
} satisfies Fixtures

// The page is 768px wide so that `md:text-sm` applies, like on iced; the
// fixtures are `w-80` wide.
const narrow = new Set(["default", "typing", "states"])

export default Object.fromEntries(
  Object.entries(fixtures).map(([id, render]) => [
    id,
    narrow.has(id) ? () => <div className="w-80">{render()}</div> : render,
  ])
) satisfies Fixtures
