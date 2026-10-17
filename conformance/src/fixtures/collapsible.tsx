import { ChevronsUpDown } from "lucide-react"

import type { Fixtures } from "@/fixtures"
import { Button } from "@/components/ui/button"
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible"

const repositories = (props: { defaultOpen?: boolean }) => (
  <Collapsible
    defaultOpen={props.defaultOpen}
    className="flex w-[350px] flex-col gap-2"
  >
    <div className="flex items-center justify-between gap-4 px-4">
      <h4 className="text-sm font-semibold">
        @peduarte starred 3 repositories
      </h4>
      <CollapsibleTrigger
        render={<Button variant="ghost" size="icon" className="size-8" />}
      >
        <ChevronsUpDown />
        <span className="sr-only">Toggle</span>
      </CollapsibleTrigger>
    </div>
    <CollapsibleContent className="flex flex-col gap-2">
      <div className="rounded-md border px-4 py-2 font-mono text-sm">
        @radix-ui/colors
      </div>
      <div className="rounded-md border px-4 py-2 font-mono text-sm">
        @stitches/react
      </div>
    </CollapsibleContent>
  </Collapsible>
)

export default {
  default: () => repositories({}),
  keyboard: () => repositories({}),
  "default-open": () => repositories({ defaultOpen: true }),
} satisfies Fixtures
