import { useState } from "react"

import type { Fixtures } from "@/fixtures"
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable"

const handle = ({
  onResize,
}: {
  onResize?: (percentage: number) => void
} = {}) => (
  <ResizablePanelGroup
    orientation="horizontal"
    className="max-w-sm rounded-lg border"
  >
    <ResizablePanel
      defaultSize="25%"
      onResize={(size) => onResize?.(size.asPercentage)}
    >
      <div className="flex h-[200px] items-center justify-center p-6">
        <span className="font-semibold">Sidebar</span>
      </div>
    </ResizablePanel>
    <ResizableHandle withHandle />
    <ResizablePanel defaultSize="75%">
      <div className="flex h-full items-center justify-center p-6">
        <span className="font-semibold">Content</span>
      </div>
    </ResizablePanel>
  </ResizablePanelGroup>
)

function Keyboard() {
  const [size, setSize] = useState(25)

  return (
    <div className="flex flex-col gap-3">
      {handle({ onResize: setSize })}
      <p className="text-sm text-muted-foreground">
        Sidebar: {Math.round(size)}%
      </p>
    </div>
  )
}

export default {
  default: () => (
    <ResizablePanelGroup
      orientation="horizontal"
      className="max-w-sm rounded-lg border"
    >
      <ResizablePanel defaultSize="50%">
        <div className="flex h-[200px] items-center justify-center p-6">
          <span className="font-semibold">One</span>
        </div>
      </ResizablePanel>
      <ResizableHandle />
      <ResizablePanel defaultSize="50%">
        <ResizablePanelGroup orientation="vertical">
          <ResizablePanel defaultSize="25%">
            <div className="flex h-full items-center justify-center p-6">
              <span className="font-semibold">Two</span>
            </div>
          </ResizablePanel>
          <ResizableHandle />
          <ResizablePanel defaultSize="75%">
            <div className="flex h-full items-center justify-center p-6">
              <span className="font-semibold">Three</span>
            </div>
          </ResizablePanel>
        </ResizablePanelGroup>
      </ResizablePanel>
    </ResizablePanelGroup>
  ),
  vertical: () => (
    <ResizablePanelGroup
      orientation="vertical"
      className="min-h-[202px] max-w-sm rounded-lg border"
    >
      <ResizablePanel defaultSize="25%">
        <div className="flex h-full items-center justify-center p-6">
          <span className="font-semibold">Header</span>
        </div>
      </ResizablePanel>
      <ResizableHandle />
      <ResizablePanel defaultSize="75%">
        <div className="flex h-full items-center justify-center p-6">
          <span className="font-semibold">Content</span>
        </div>
      </ResizablePanel>
    </ResizablePanelGroup>
  ),
  handle: () => handle(),
  keyboard: () => <Keyboard />,
} satisfies Fixtures
