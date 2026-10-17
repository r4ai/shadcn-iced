import {
  ArrowLeftIcon,
  MinusIcon,
  MoreHorizontalIcon,
  PlusIcon,
} from "lucide-react"

import type { Fixtures } from "@/fixtures"
import { Button } from "@/components/ui/button"
import {
  ButtonGroup,
  ButtonGroupSeparator,
  ButtonGroupText,
} from "@/components/ui/button-group"

export default {
  default: () => (
    <ButtonGroup>
      <ButtonGroup>
        <Button variant="outline" size="icon">
          <ArrowLeftIcon />
        </Button>
      </ButtonGroup>
      <ButtonGroup>
        <Button variant="outline">Archive</Button>
        <Button variant="outline">Report</Button>
      </ButtonGroup>
      <ButtonGroup>
        <Button variant="outline">Snooze</Button>
        <Button variant="outline" size="icon">
          <MoreHorizontalIcon />
        </Button>
      </ButtonGroup>
    </ButtonGroup>
  ),
  orientation: () => (
    <ButtonGroup orientation="vertical">
      <Button variant="outline" size="icon">
        <PlusIcon />
      </Button>
      <Button variant="outline" size="icon">
        <MinusIcon />
      </Button>
    </ButtonGroup>
  ),
  size: () => (
    <div className="flex flex-col items-start gap-8">
      <ButtonGroup>
        <Button variant="outline" size="sm">
          Small
        </Button>
        <Button variant="outline" size="sm">
          Button
        </Button>
        <Button variant="outline" size="sm">
          Group
        </Button>
        <Button variant="outline" size="icon-sm">
          <PlusIcon />
        </Button>
      </ButtonGroup>
      <ButtonGroup>
        <Button variant="outline">Default</Button>
        <Button variant="outline">Button</Button>
        <Button variant="outline">Group</Button>
        <Button variant="outline" size="icon">
          <PlusIcon />
        </Button>
      </ButtonGroup>
      <ButtonGroup>
        <Button variant="outline" size="lg">
          Large
        </Button>
        <Button variant="outline" size="lg">
          Button
        </Button>
        <Button variant="outline" size="lg">
          Group
        </Button>
        <Button variant="outline" size="icon-lg">
          <PlusIcon />
        </Button>
      </ButtonGroup>
    </div>
  ),
  separator: () => (
    <ButtonGroup>
      <Button variant="secondary" size="sm">
        Copy
      </Button>
      <ButtonGroupSeparator />
      <Button variant="secondary" size="sm">
        Paste
      </Button>
    </ButtonGroup>
  ),
  split: () => (
    <ButtonGroup>
      <Button variant="secondary">Button</Button>
      <ButtonGroupSeparator />
      <Button variant="secondary" size="icon">
        <PlusIcon />
      </Button>
    </ButtonGroup>
  ),
  text: () => (
    <ButtonGroup>
      <Button variant="outline">Button</Button>
      <ButtonGroupText>Text</ButtonGroupText>
      <Button variant="outline">Button</Button>
    </ButtonGroup>
  ),
} satisfies Fixtures
