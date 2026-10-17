import { ArrowUpRightIcon, BadgeCheckIcon, BookmarkIcon } from "lucide-react"

import { Badge } from "@/components/ui/badge"
import type { Fixtures } from "@/fixtures"

export default {
  variants: () => (
    <div className="flex gap-2">
      <Badge>Default</Badge>
      <Badge variant="secondary">Secondary</Badge>
      <Badge variant="destructive">Destructive</Badge>
      <Badge variant="outline">Outline</Badge>
      <Badge variant="ghost">Ghost</Badge>
      <Badge variant="link">Link</Badge>
    </div>
  ),
  "with-icon": () => (
    <div className="flex gap-2">
      <Badge variant="secondary">
        <BadgeCheckIcon data-icon="inline-start" />
        Verified
      </Badge>
      <Badge variant="outline">
        <BookmarkIcon data-icon="inline-start" />
        Bookmark
      </Badge>
    </div>
  ),
  // In a flex container like the other fixtures: alone, the inline badge
  // would sit on the baseline of a 24px line box, 3px lower.
  link: () => (
    <div className="flex">
      <Badge render={<a href="#link" />}>
        Open Link
        <ArrowUpRightIcon data-icon="inline-end" />
      </Badge>
    </div>
  ),
} satisfies Fixtures
