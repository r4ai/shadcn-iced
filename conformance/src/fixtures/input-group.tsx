import { SearchIcon } from "lucide-react"

import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
  InputGroupText,
} from "@/components/ui/input-group"
import { Kbd } from "@/components/ui/kbd"
import type { Fixtures } from "@/fixtures"

const fixtures = {
  icon: () => (
    <InputGroup>
      <InputGroupInput placeholder="Search..." />
      <InputGroupAddon>
        <SearchIcon />
      </InputGroupAddon>
      <InputGroupAddon align="inline-end">12 results</InputGroupAddon>
    </InputGroup>
  ),
  text: () => (
    <InputGroup>
      <InputGroupAddon>
        <InputGroupText>https://</InputGroupText>
      </InputGroupAddon>
      <InputGroupInput placeholder="example.com" />
      <InputGroupAddon align="inline-end">
        <InputGroupText>.com</InputGroupText>
      </InputGroupAddon>
    </InputGroup>
  ),
  button: () => (
    <InputGroup>
      <InputGroupInput placeholder="Type to search..." />
      <InputGroupAddon align="inline-end">
        <InputGroupButton variant="secondary">Search</InputGroupButton>
      </InputGroupAddon>
    </InputGroup>
  ),
  kbd: () => (
    <InputGroup>
      <InputGroupInput placeholder="Search..." />
      <InputGroupAddon>
        <SearchIcon />
      </InputGroupAddon>
      <InputGroupAddon align="inline-end">
        <Kbd>⌘K</Kbd>
      </InputGroupAddon>
    </InputGroup>
  ),
  states: () => (
    <div className="flex flex-col gap-3">
      <InputGroup>
        <InputGroupInput placeholder="Search..." aria-invalid />
        <InputGroupAddon>
          <SearchIcon />
        </InputGroupAddon>
        <InputGroupAddon align="inline-end">12 results</InputGroupAddon>
      </InputGroup>
      <InputGroup data-disabled>
        <InputGroupInput placeholder="Search..." disabled />
        <InputGroupAddon>
          <SearchIcon />
        </InputGroupAddon>
        <InputGroupAddon align="inline-end">12 results</InputGroupAddon>
      </InputGroup>
    </div>
  ),
} satisfies Fixtures

// The page is 768px wide so that `md:text-sm` applies, like on iced; the
// fixtures are `w-80` wide.
const narrow = new Set(["icon", "text", "button", "kbd", "states"])

export default Object.fromEntries(
  Object.entries(fixtures).map(([id, render]) => [
    id,
    narrow.has(id) ? () => <div className="w-80">{render()}</div> : render,
  ])
) satisfies Fixtures
