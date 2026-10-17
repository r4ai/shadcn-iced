import { AspectRatio } from "@/components/ui/aspect-ratio"
import type { Fixtures } from "@/fixtures"

// A box with a label stands in for the image of the documentation, which is
// not deterministic.
const placeholder = (label: string) => (
  <div className="flex size-full items-center justify-center rounded-lg bg-muted text-sm text-muted-foreground">
    {label}
  </div>
)

export default {
  video: () => (
    <div className="w-full max-w-sm">
      <AspectRatio ratio={16 / 9}>{placeholder("16 : 9")}</AspectRatio>
    </div>
  ),
  square: () => (
    <div className="w-full max-w-48">
      <AspectRatio ratio={1}>{placeholder("1 : 1")}</AspectRatio>
    </div>
  ),
} satisfies Fixtures
