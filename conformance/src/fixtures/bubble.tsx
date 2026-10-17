import { useState } from "react"

import {
  Bubble,
  BubbleContent,
  BubbleGroup,
  BubbleReactions,
} from "@/components/ui/bubble"
import type { Fixtures } from "@/fixtures"

function Picker() {
  const [picked, setPicked] = useState(false)

  return (
    <div className="flex w-full max-w-sm flex-col gap-4">
      <BubbleGroup>
        <Bubble variant="tinted" align="end">
          <BubbleContent
            render={<button type="button" onClick={() => setPicked(!picked)} />}
          >
            I forgot my password
          </BubbleContent>
        </Bubble>
        <Bubble variant="tinted" align="end">
          <BubbleContent render={<button type="button" />}>
            Something else. Talk to a human.
          </BubbleContent>
        </Bubble>
      </BubbleGroup>
      <p className="text-sm">{picked ? "Picked" : "Nothing picked"}</p>
    </div>
  )
}

export default {
  variants: () => (
    <div className="flex w-full max-w-sm flex-col gap-4">
      <Bubble>
        <BubbleContent>This is the default variant.</BubbleContent>
      </Bubble>
      <Bubble variant="secondary" align="end">
        <BubbleContent>This is the secondary variant.</BubbleContent>
      </Bubble>
      <Bubble variant="muted">
        <BubbleContent>This is the muted variant.</BubbleContent>
      </Bubble>
      <Bubble variant="tinted" align="end">
        <BubbleContent>This is the tinted variant.</BubbleContent>
      </Bubble>
      <Bubble variant="outline">
        <BubbleContent>This is the outline variant.</BubbleContent>
      </Bubble>
      <Bubble variant="destructive" align="end">
        <BubbleContent>This is the destructive variant.</BubbleContent>
      </Bubble>
      <Bubble variant="ghost">
        <BubbleContent>This is the ghost variant.</BubbleContent>
      </Bubble>
    </div>
  ),
  "group-and-reactions": () => (
    <div className="flex w-full max-w-sm flex-col gap-8">
      <BubbleGroup>
        <Bubble variant="muted">
          <BubbleContent>Hey! Want to see chat bubbles?</BubbleContent>
        </Bubble>
        <Bubble variant="muted">
          <BubbleContent>
            I can group messages and keep them easy to scan.
          </BubbleContent>
          <BubbleReactions>👍</BubbleReactions>
        </Bubble>
      </BubbleGroup>
      <Bubble align="end">
        <BubbleContent>Tests passed on the first try.</BubbleContent>
        <BubbleReactions side="top" align="start">
          🎉 👏
        </BubbleReactions>
      </Bubble>
      <Bubble align="end">
        <BubbleContent>Find the bug and fix it.</BubbleContent>
        <BubbleReactions align="start">👀</BubbleReactions>
      </Bubble>
    </div>
  ),
  button: () => <Picker />,
} satisfies Fixtures
