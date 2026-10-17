import { CircleAlertIcon, CircleCheckIcon, TerminalIcon } from "lucide-react"

import {
  Trace,
  TraceCollapsible,
  TraceCollapsibleContent,
  TraceCollapsibleTrigger,
  TraceDescription,
  TraceIcon,
  TraceLine,
  TraceTitle,
} from "@/components/ui/trace"
import type { Fixtures } from "@/fixtures"

export default {
  lines: () => (
    <div className="w-full max-w-sm">
      <Trace>
        <TraceLine>
          <TraceIcon>
            <CircleCheckIcon />
          </TraceIcon>
          <TraceTitle>Ran pnpm install</TraceTitle>
          <TraceDescription>2.1s</TraceDescription>
        </TraceLine>
        <TraceLine status="error">
          <TraceIcon>
            <CircleAlertIcon />
          </TraceIcon>
          <TraceTitle>Failed to run pnpm lint</TraceTitle>
        </TraceLine>
        <TraceLine status="pending">
          <TraceIcon>
            <TerminalIcon />
          </TraceIcon>
          <TraceTitle>Run pnpm build</TraceTitle>
        </TraceLine>
      </Trace>
    </div>
  ),
  collapsible: () => (
    <div className="w-full max-w-sm">
      <Trace>
        <TraceCollapsible>
          <TraceCollapsibleTrigger>
            <TraceLine>
              <TraceIcon>
                <CircleCheckIcon />
              </TraceIcon>
              <TraceTitle>Thought for 5s</TraceTitle>
            </TraceLine>
          </TraceCollapsibleTrigger>
          <TraceCollapsibleContent>
            I should check what the last commit changed.
          </TraceCollapsibleContent>
        </TraceCollapsible>
        <TraceCollapsible>
          <TraceCollapsibleTrigger>
            <TraceLine>
              <TraceIcon>
                <TerminalIcon />
              </TraceIcon>
              <TraceTitle>Ran pnpm --version</TraceTitle>
            </TraceLine>
          </TraceCollapsibleTrigger>
          <TraceCollapsibleContent>
            Exit code 0, no output.
          </TraceCollapsibleContent>
        </TraceCollapsible>
      </Trace>
    </div>
  ),
} satisfies Fixtures
