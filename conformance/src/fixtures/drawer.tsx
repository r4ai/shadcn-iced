import { Button } from "@/components/ui/button"
import {
  Drawer,
  DrawerClose,
  DrawerContent,
  DrawerDescription,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle,
  DrawerTrigger,
} from "@/components/ui/drawer"
import type { Fixtures } from "@/fixtures"

type Direction = "up" | "right" | "down" | "left"

const confirm = (
  direction: Direction = "down",
  label = "Open Drawer",
  showSwipeHandle = false
) => (
  <Drawer swipeDirection={direction} showSwipeHandle={showSwipeHandle}>
    <DrawerTrigger render={<Button variant="outline" />}>{label}</DrawerTrigger>
    <DrawerContent>
      <DrawerHeader>
        <DrawerTitle>Are you absolutely sure?</DrawerTitle>
        <DrawerDescription>This action cannot be undone.</DrawerDescription>
      </DrawerHeader>
      <DrawerFooter>
        <Button>Submit</Button>
        <DrawerClose render={<Button variant="outline" />}>Cancel</DrawerClose>
      </DrawerFooter>
    </DrawerContent>
  </Drawer>
)

export default {
  default: () => confirm(),
  footer: () => confirm(),
  "outside-click": () => confirm(),
  "swipe-handle": () => confirm("down", "Open Drawer", true),
  directions: () => (
    <div className="flex gap-2">
      {(["up", "right", "down", "left"] as const).map((direction) =>
        confirm(direction, direction)
      )}
    </div>
  ),
} satisfies Fixtures
