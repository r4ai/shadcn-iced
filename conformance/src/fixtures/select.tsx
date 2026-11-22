import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectSeparator,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import type { Fixtures } from "@/fixtures"

const items = [
  { label: "Select a fruit", value: null },
  { label: "Apple", value: "apple" },
  { label: "Banana", value: "banana" },
  { label: "Blueberry", value: "blueberry" },
  { label: "Grapes", value: "grapes" },
  { label: "Pineapple", value: "pineapple" },
]

const basic = () => (
  <Select items={items}>
    <SelectTrigger className="w-full max-w-48">
      <SelectValue placeholder="Select a fruit" />
    </SelectTrigger>
    <SelectContent>
      <SelectGroup>
        <SelectLabel>Fruits</SelectLabel>
        {items
          .filter((item) => item.value !== null)
          .map((item) => (
            <SelectItem key={item.value} value={item.value}>
              {item.label}
            </SelectItem>
          ))}
      </SelectGroup>
    </SelectContent>
  </Select>
)

const numbers = Array.from({ length: 30 }, (_, i) => ({
  label: `Item ${i + 1}`,
  value: `${i + 1}`,
}))

/** 30 items, which do not fit in the window, `top` pixels down the page. */
const long = (top: number, defaultValue: string) => () => (
  <div style={{ paddingTop: top }}>
    <Select items={numbers} defaultValue={defaultValue}>
      <SelectTrigger className="w-48">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectGroup>
          {numbers.map((item) => (
            <SelectItem key={item.value} value={item.value}>
              {item.label}
            </SelectItem>
          ))}
        </SelectGroup>
      </SelectContent>
    </Select>
  </div>
)

export default {
  basic,
  reopen: basic,
  keyboard: basic,
  "below-trigger": () => (
    <Select items={items}>
      <SelectTrigger className="w-full max-w-48">
        <SelectValue placeholder="Select a fruit" />
      </SelectTrigger>
      <SelectContent alignItemWithTrigger={false}>
        <SelectGroup>
          <SelectItem value="apple">Apple</SelectItem>
          <SelectItem value="banana">Banana</SelectItem>
          <SelectItem value="blueberry">Blueberry</SelectItem>
        </SelectGroup>
        <SelectSeparator />
        <SelectGroup>
          <SelectItem value="grapes">Grapes</SelectItem>
          <SelectItem value="pineapple" disabled>
            Pineapple
          </SelectItem>
        </SelectGroup>
      </SelectContent>
    </Select>
  ),
  "long-top": long(48, "12"),
  "long-middle": long(168, "2"),
  "long-bottom": long(304, "30"),
  "trigger-states": () => (
    <div className="flex flex-col items-start gap-2">
      <Select>
        <SelectTrigger className="w-48">
          <SelectValue placeholder="Default" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="apple">Apple</SelectItem>
        </SelectContent>
      </Select>
      <Select>
        <SelectTrigger size="sm" className="w-48">
          <SelectValue placeholder="Small" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="apple">Apple</SelectItem>
        </SelectContent>
      </Select>
      <Select
        items={[{ label: "Banana", value: "banana" }]}
        defaultValue="banana"
      >
        <SelectTrigger className="w-48">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="banana">Banana</SelectItem>
        </SelectContent>
      </Select>
      <Select disabled>
        <SelectTrigger className="w-48">
          <SelectValue placeholder="Disabled" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="apple">Apple</SelectItem>
        </SelectContent>
      </Select>
    </div>
  ),
} satisfies Fixtures
