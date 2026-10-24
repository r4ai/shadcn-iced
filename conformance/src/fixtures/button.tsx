import { ArrowUpRight, GitBranch, Send } from "lucide-react"

import { Button } from "@/components/ui/button"
import type { Fixtures } from "@/fixtures"

export default {
  default: () => <Button>Button</Button>,
  secondary: () => <Button variant="secondary">Secondary</Button>,
  destructive: () => <Button variant="destructive">Destructive</Button>,
  outline: () => <Button variant="outline">Outline</Button>,
  ghost: () => <Button variant="ghost">Ghost</Button>,
  link: () => <Button variant="link">Link</Button>,
  sizes: () => (
    <div className="flex items-center gap-2">
      <Button variant="outline" size="xs">
        Extra Small
      </Button>
      <Button variant="outline" size="sm">
        Small
      </Button>
      <Button variant="outline">Default</Button>
      <Button variant="outline" size="lg">
        Large
      </Button>
    </div>
  ),
  "icon-sizes": () => (
    <div className="flex items-center gap-2">
      <Button variant="outline" size="icon-xs">
        <ArrowUpRight />
      </Button>
      <Button variant="outline" size="icon-sm">
        <ArrowUpRight />
      </Button>
      <Button variant="outline" size="icon">
        <ArrowUpRight />
      </Button>
      <Button variant="outline" size="icon-lg">
        <ArrowUpRight />
      </Button>
    </div>
  ),
  "with-icon": () => (
    <div className="flex items-center gap-2">
      <Button variant="outline">
        <GitBranch data-icon="inline-start" />
        New Branch
      </Button>
      <Button>
        Send
        <Send data-icon="inline-end" />
      </Button>
    </div>
  ),
  disabled: () => (
    <div className="flex items-center gap-2">
      <Button disabled>Disabled</Button>
      <Button variant="outline" disabled>
        Outline
      </Button>
    </div>
  ),
  keyboard: () => (
    <div className="flex items-center gap-2">
      <Button>Button</Button>
      <Button variant="outline">Outline</Button>
      <Button variant="destructive">Destructive</Button>
    </div>
  ),
} satisfies Fixtures
