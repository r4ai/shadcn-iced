import { toast } from "sonner"

import { Button } from "@/components/ui/button"
import { Toaster } from "@/components/ui/sonner"
import type { Fixtures } from "@/fixtures"

export default {
  default: () => (
    <>
      <Button
        variant="outline"
        onClick={() =>
          toast("Event has been created", {
            description: "Sunday, December 03, 2023 at 9:00 AM",
            action: {
              label: "Undo",
              onClick: () => {},
            },
          })
        }
      >
        Show Toast
      </Button>
      <Toaster />
    </>
  ),
  "auto-dismiss": () => (
    <>
      <Button variant="outline" onClick={() => toast("Event has been created")}>
        Show Toast
      </Button>
      <Toaster />
    </>
  ),
  types: () => (
    <>
      <div className="flex gap-2">
        <Button
          variant="outline"
          onClick={() => toast.success("Event has been created")}
        >
          Success
        </Button>
        <Button
          variant="outline"
          onClick={() => toast.error("Event has not been created")}
        >
          Error
        </Button>
      </div>
      <Toaster />
    </>
  ),
} satisfies Fixtures
