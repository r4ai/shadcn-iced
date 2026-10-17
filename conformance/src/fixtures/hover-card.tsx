import { Button } from "@/components/ui/button"
import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger,
} from "@/components/ui/hover-card"
import type { Fixtures } from "@/fixtures"

// The delays are Base UI's defaults (600ms to open, 300ms to close), which
// are the only ones `shadcn_iced` offers.
const card = () => (
  <HoverCard>
    <HoverCardTrigger render={<Button variant="link" />}>
      @nextjs
    </HoverCardTrigger>
    <HoverCardContent className="flex w-64 flex-col gap-0.5">
      <div className="font-semibold">@nextjs</div>
      <div>The React Framework – created and maintained by @vercel.</div>
      <div className="mt-1 text-xs text-muted-foreground">
        Joined December 2021
      </div>
    </HoverCardContent>
  </HoverCard>
)

export default {
  default: card,
  escape: card,
  "stays-open": card,
} satisfies Fixtures
