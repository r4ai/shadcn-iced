import type { Fixtures } from "@/fixtures"
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable"

const handle = () => (
  <ResizablePanelGroup
    orientation="horizontal"
    className="max-w-sm rounded-lg border"
  >
    <ResizablePanel defaultSize="25%">
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
  handle,
  keyboard: handle,
} satisfies Fixtures
