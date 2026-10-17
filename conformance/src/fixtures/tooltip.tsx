import { Button } from "@/components/ui/button"
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip"
import type { Fixtures } from "@/fixtures"

export default {
  default: () => (
    <TooltipProvider>
      <div className="pt-12">
        <Tooltip>
          <TooltipTrigger render={<Button variant="outline" />}>
            Hover
          </TooltipTrigger>
          <TooltipContent>
            <p>Add to library</p>
          </TooltipContent>
        </Tooltip>
      </div>
    </TooltipProvider>
  ),
  sides: () => (
    <TooltipProvider>
      <div className="flex gap-2 pt-16 pl-32">
        {(
          [
            ["Left", "left"],
            ["Top", "top"],
            ["Bottom", "bottom"],
            ["Right", "right"],
          ] as const
        ).map(([label, side]) => (
          <Tooltip key={side}>
            <TooltipTrigger render={<Button variant="outline" />}>
              {label}
            </TooltipTrigger>
            <TooltipContent side={side}>
              <p>Add to library</p>
            </TooltipContent>
          </Tooltip>
        ))}
      </div>
    </TooltipProvider>
  ),
} satisfies Fixtures
