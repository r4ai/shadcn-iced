import { Button } from "@/components/ui/button"
import {
  Sheet,
  SheetClose,
  SheetContent,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet"
import type { Fixtures } from "@/fixtures"

const SIDES = ["top", "right", "bottom", "left"] as const

const profile = (side?: (typeof SIDES)[number], label = "Open") => (
  <Sheet>
    <SheetTrigger render={<Button variant="outline" />}>{label}</SheetTrigger>
    <SheetContent side={side}>
      <SheetHeader>
        <SheetTitle>Edit profile</SheetTitle>
        <SheetDescription>
          Make changes to your profile here. Click save when you&apos;re done.
        </SheetDescription>
      </SheetHeader>
      <SheetFooter>
        <SheetClose render={<Button />}>Save changes</SheetClose>
        <SheetClose render={<Button variant="outline" />}>Close</SheetClose>
      </SheetFooter>
    </SheetContent>
  </Sheet>
)

export default {
  default: () => profile(),
  "close-button": () => profile(),
  footer: () => profile(),
  "outside-click": () => profile(),
  sides: () => (
    <div className="flex gap-2">{SIDES.map((side) => profile(side, side))}</div>
  ),
} satisfies Fixtures
