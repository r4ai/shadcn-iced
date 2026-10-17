import { Button } from "@/components/ui/button"
import { Kbd, KbdGroup } from "@/components/ui/kbd"
import type { Fixtures } from "@/fixtures"

export default {
  demo: () => (
    <div className="flex flex-col items-center gap-4">
      <KbdGroup>
        <Kbd>⌘</Kbd>
        <Kbd>⇧</Kbd>
        <Kbd>⌥</Kbd>
        <Kbd>⌃</Kbd>
      </KbdGroup>
      <KbdGroup>
        <Kbd>Ctrl</Kbd>
        <span>+</span>
        <Kbd>B</Kbd>
      </KbdGroup>
    </div>
  ),
  group: () => (
    <p className="text-sm text-muted-foreground">
      Use{" "}
      <KbdGroup>
        <Kbd>Ctrl + B</Kbd>
        <Kbd>Ctrl + K</Kbd>
      </KbdGroup>{" "}
      to open the command palette
    </p>
  ),
  button: () => (
    <Button variant="outline">
      Accept
      <Kbd data-icon="inline-end" className="translate-x-0.5">
        ⏎
      </Kbd>
    </Button>
  ),
} satisfies Fixtures
