import { Button } from "@/components/ui/button"
import { Toaster, toast } from "@/components/ui/toast"
import type { Fixtures } from "@/fixtures"

export default {
  default: () => (
    <Toaster>
      <Button
        variant="outline"
        onClick={() => {
          const id = toast.add({
            title: "Event created",
            description: "Sunday, December 3 at 9:00 AM",
            actionProps: {
              children: "Undo",
              onClick: () => toast.close(id),
            },
          })
        }}
      >
        Show Toast
      </Button>
    </Toaster>
  ),
  "auto-dismiss": () => (
    <Toaster>
      <Button
        variant="outline"
        onClick={() => toast.add({ title: "Event created" })}
      >
        Show Toast
      </Button>
    </Toaster>
  ),
  types: () => (
    <Toaster>
      <div className="flex gap-2">
        <Button
          variant="outline"
          onClick={() => toast.add({ title: "Event created", type: "success" })}
        >
          Success
        </Button>
        <Button
          variant="outline"
          onClick={() => toast.add({ title: "Event failed", type: "error" })}
        >
          Error
        </Button>
      </div>
    </Toaster>
  ),
} satisfies Fixtures
