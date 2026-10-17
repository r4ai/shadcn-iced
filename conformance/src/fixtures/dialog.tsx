import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import type { Fixtures } from "@/fixtures"

const profile = () => (
  <Dialog>
    <DialogTrigger render={<Button variant="outline" />}>
      Open Dialog
    </DialogTrigger>
    <DialogContent className="sm:max-w-sm">
      <DialogHeader>
        <DialogTitle>Edit profile</DialogTitle>
        <DialogDescription>
          Make changes to your profile here. Click save when you&apos;re done.
        </DialogDescription>
      </DialogHeader>
      <DialogFooter>
        <DialogClose render={<Button variant="outline" />}>Cancel</DialogClose>
        <DialogClose render={<Button />}>Save changes</DialogClose>
      </DialogFooter>
    </DialogContent>
  </Dialog>
)

export default {
  default: profile,
  "close-button": profile,
  footer: profile,
  "outside-click": profile,
  narrow: profile,
} satisfies Fixtures
