// Port of the separator of shadcn/ui (registry/new-york-v4/ui/separator.tsx).
import * as SeparatorPrimitive from "@kobalte/core/separator";

import { cn } from "~/lib/utils";

export function Separator(props: { orientation?: "horizontal" | "vertical"; class?: string }) {
  return (
    <SeparatorPrimitive.Root
      as="div"
      data-slot="separator"
      orientation={props.orientation ?? "horizontal"}
      class={cn(
        "shrink-0 bg-border data-[orientation=horizontal]:h-px data-[orientation=horizontal]:w-full data-[orientation=vertical]:h-full data-[orientation=vertical]:w-px",
        props.class,
      )}
    />
  );
}
