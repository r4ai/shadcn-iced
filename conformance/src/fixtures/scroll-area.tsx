import { Fragment } from "react"

import type { Fixtures } from "@/fixtures"
import { ScrollArea, ScrollBar } from "@/components/ui/scroll-area"
import { Separator } from "@/components/ui/separator"

const tags = Array.from({ length: 50 }).map(
  (_, i, a) => `v1.2.0-beta.${a.length - i}`
)

const works = ["Ornella Binni", "Tom Byrom", "Vladimir Malyavko"]

function Tags() {
  return (
    <ScrollArea className="h-72 w-48 rounded-md border">
      <div className="p-4">
        <h4 className="mb-4 text-sm leading-none font-medium">Tags</h4>
        {tags.map((tag) => (
          <Fragment key={tag}>
            <div className="text-sm">{tag}</div>
            <Separator className="my-2" />
          </Fragment>
        ))}
      </div>
    </ScrollArea>
  )
}

export default {
  default: Tags,
  wheel: Tags,
  horizontal: () => (
    <ScrollArea className="w-96 rounded-md border whitespace-nowrap">
      <div className="flex w-max space-x-4 p-4">
        {works.map((artist) => (
          <figure key={artist} className="shrink-0">
            <div className="h-[200px] w-[150px] rounded-md bg-muted" />
            <figcaption className="pt-2 text-xs text-muted-foreground">
              Photo by{" "}
              <span className="font-semibold text-foreground">{artist}</span>
            </figcaption>
          </figure>
        ))}
      </div>
      <ScrollBar orientation="horizontal" />
    </ScrollArea>
  ),
} satisfies Fixtures
