import { Button } from "@/components/ui/button"
import {
  Popover,
  PopoverContent,
  PopoverDescription,
  PopoverHeader,
  PopoverTitle,
  PopoverTrigger,
} from "@/components/ui/popover"
import type { Fixtures } from "@/fixtures"

export default {
  default: () => (
    <Popover>
      <PopoverTrigger render={<Button variant="outline" />}>
        Open Popover
      </PopoverTrigger>
      <PopoverContent align="start">
        <PopoverHeader>
          <PopoverTitle>Dimensions</PopoverTitle>
          <PopoverDescription>
            Set the dimensions for the layer.
          </PopoverDescription>
        </PopoverHeader>
      </PopoverContent>
    </Popover>
  ),
  align: () => (
    <div className="flex gap-2">
      {(
        [
          ["Start", "start"],
          ["Center", "center"],
          ["End", "end"],
        ] as const
      ).map(([label, align]) => (
        <Popover key={align}>
          <PopoverTrigger render={<Button variant="outline" size="sm" />}>
            {label}
          </PopoverTrigger>
          <PopoverContent align={align} className="w-40">
            Aligned to {align}
          </PopoverContent>
        </Popover>
      ))}
    </div>
  ),
} satisfies Fixtures
