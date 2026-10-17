// Port of the popover of shadcn/ui in the base-nova style
// (registry/bases/base/ui/popover.tsx with registry/styles/style-nova.css) on
// Kobalte, which marks an open popover with `data-expanded`, and gives no
// `data-side`: the root tracks the side the content opens on.
import type { PolymorphicProps } from "@kobalte/core/polymorphic";
import * as PopoverPrimitive from "@kobalte/core/popover";
import { createContext, createSignal, splitProps, useContext, type JSX, type ValidComponent } from "solid-js";

import { cn } from "~/lib/utils";

const SideContext = createContext<() => string>(() => "bottom");

export function Popover(props: PopoverPrimitive.PopoverRootProps) {
  const [side, setSide] = createSignal("bottom");
  // The root passes it on to its popper, though its type leaves it out.
  const tracking = {
    onCurrentPlacementChange: (placement: string) => setSide(placement.split("-")[0]!),
  } as Partial<PopoverPrimitive.PopoverRootProps>;

  return (
    <SideContext.Provider value={side}>
      <PopoverPrimitive.Root gutter={4} {...props} {...tracking} />
    </SideContext.Provider>
  );
}

export const PopoverTrigger = PopoverPrimitive.Trigger;

type PopoverContentProps<T extends ValidComponent = "div"> = PopoverPrimitive.PopoverContentProps<T> & {
  class?: string;
};

export function PopoverContent<T extends ValidComponent = "div">(props: PolymorphicProps<T, PopoverContentProps<T>>) {
  const [local, others] = splitProps(props as PopoverContentProps, ["class"]);
  const side = useContext(SideContext);

  return (
    <PopoverPrimitive.Portal>
      <PopoverPrimitive.Content
        data-slot="popover-content"
        data-side={side()}
        class={cn(
          "z-50 flex w-72 origin-(--kb-popover-content-transform-origin) flex-col gap-2.5 rounded-lg bg-popover p-2.5 text-sm text-popover-foreground shadow-md ring-1 ring-foreground/10 outline-hidden duration-100 data-expanded:animate-in data-expanded:fade-in-0 data-expanded:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-closed:animate-out data-closed:fade-out-0 data-closed:zoom-out-95",
          local.class,
        )}
        {...others}
      />
    </PopoverPrimitive.Portal>
  );
}

export const PopoverHeader = (props: { class?: string; children: JSX.Element }) => (
  <div data-slot="popover-header" class={cn("flex flex-col gap-0.5 text-sm", props.class)}>
    {props.children}
  </div>
);

export const PopoverTitle = (props: { class?: string; children: JSX.Element }) => (
  <PopoverPrimitive.Title data-slot="popover-title" class={cn("font-medium", props.class)}>
    {props.children}
  </PopoverPrimitive.Title>
);

export const PopoverDescription = (props: { class?: string; children: JSX.Element }) => (
  <PopoverPrimitive.Description data-slot="popover-description" class={cn("text-muted-foreground", props.class)}>
    {props.children}
  </PopoverPrimitive.Description>
);
