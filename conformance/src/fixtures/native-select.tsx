import { NativeSelect, NativeSelectOption } from "@/components/ui/native-select"
import type { Fixtures } from "@/fixtures"

function Status(props: React.ComponentProps<typeof NativeSelect>) {
  return (
    <NativeSelect {...props}>
      <NativeSelectOption value="">Select status</NativeSelectOption>
      <NativeSelectOption value="todo">Todo</NativeSelectOption>
      <NativeSelectOption value="in-progress">In Progress</NativeSelectOption>
      <NativeSelectOption value="done">Done</NativeSelectOption>
      <NativeSelectOption value="cancelled">Cancelled</NativeSelectOption>
    </NativeSelect>
  )
}

export default {
  default: () => <Status />,
  selected: () => <Status defaultValue="in-progress" />,
  sizes: () => (
    <div className="flex flex-col gap-3">
      <Status defaultValue="todo" />
      <Status size="sm" defaultValue="todo" />
    </div>
  ),
  keyboard: () => <Status />,
  states: () => (
    <div className="flex flex-col gap-3">
      <Status disabled defaultValue="todo" />
      <Status aria-invalid defaultValue="todo" />
    </div>
  ),
} satisfies Fixtures
