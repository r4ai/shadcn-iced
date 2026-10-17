// Port of the select of shadcn/ui in the base-nova style
// (registry/bases/base/ui/select.tsx with registry/styles/style-nova.css) on
// Kobalte, for a list of string options with a label.
//
// Like the `alignItemWithTrigger` of Base UI (on by default in shadcn/ui), the
// list opens over the trigger, with the selected item in place of its value.
import * as SelectPrimitive from "@kobalte/core/select";
import { Check, ChevronDown } from "lucide-solid";

import { cn } from "~/lib/utils";

export interface SelectOption {
  value: string;
  label: string;
}

interface SelectProps {
  label: string;
  options: SelectOption[];
  value: string;
  onChange: (value: string) => void;
  size?: "sm" | "default";
  class?: string;
}

/** The space kept between the list and the edges of the viewport. */
const VIEWPORT_PADDING = 8;

/**
 * How long after the list opens the release of the press that opened it is
 * ignored, on the selected item and on the others (Base UI): the list opens
 * under the pointer, so the release would pick an item.
 */
const SELECTED_MOUSE_UP_DELAY = 400;
const UNSELECTED_MOUSE_UP_DELAY = 200;

/** Below this height, the list opens under the trigger instead. */
const MIN_ALIGNED_HEIGHT = 96;

type AnchorRect = { x: number; y: number; width: number; height: number };

export function Select(props: SelectProps) {
  const selected = () => props.options.find((option) => option.value === props.value) ?? props.options[0];
  let content: HTMLElement | undefined;
  let openedAt = 0;
  // Whether a press started in the list since it opened.
  let pressedInList = false;
  // The position of the list, computed once per opening.
  let placement: AnchorRect | undefined;

  const ignoreOpeningRelease = (event: PointerEvent) => {
    const item = (event.target as Element).closest("[data-slot=select-item]");
    const delay = item?.hasAttribute("data-selected") ? SELECTED_MOUSE_UP_DELAY : UNSELECTED_MOUSE_UP_DELAY;
    if (item && !pressedInList && performance.now() - openedAt < delay) event.stopPropagation();
  };

  // The anchor of the list: a line where its top goes, so that the selected
  // item covers the trigger. Like Base UI, the list is shortened, or scrolled,
  // to keep the item there inside the viewport.
  const alignItemWithTrigger = (trigger?: HTMLElement): AnchorRect | undefined => {
    if (placement) return placement;

    const listbox = content?.querySelector<HTMLElement>("[role=listbox]");
    const item = listbox?.querySelector<HTMLElement>("[data-slot=select-item][data-selected]");
    if (!trigger || !listbox || !item) return undefined;

    listbox.style.maxHeight = "none";
    const anchor = trigger.getBoundingClientRect();
    const height = listbox.offsetHeight;
    const offset = item.offsetTop;
    const natural = anchor.top + (anchor.height - item.offsetHeight) / 2 - offset;

    // The part of the list above the viewport scrolls out of view.
    const scroll = Math.max(0, VIEWPORT_PADDING - natural);
    const top = natural + scroll;
    const visible = Math.min(height - scroll, window.innerHeight - VIEWPORT_PADDING - top);

    if (visible < MIN_ALIGNED_HEIGHT) {
      listbox.style.maxHeight = "";
      placement = { x: anchor.left, y: anchor.bottom + 4, width: anchor.width, height: 0 };
    } else {
      listbox.style.maxHeight = `${visible}px`;
      listbox.scrollTop = scroll;
      placement = { x: anchor.left, y: top, width: anchor.width, height: 0 };
    }

    return placement;
  };

  return (
    <SelectPrimitive.Root<SelectOption>
      class={cn("grid gap-2", props.class)}
      options={props.options}
      optionValue="value"
      optionTextValue="label"
      value={selected()}
      onChange={(option) => option && props.onChange(option.value)}
      onOpenChange={(open) => {
        if (!open) return;
        openedAt = performance.now();
        pressedInList = false;
        placement = undefined;
      }}
      disallowEmptySelection
      placement="bottom-start"
      gutter={0}
      flip={false}
      getAnchorRect={alignItemWithTrigger}
      sameWidth
      itemComponent={(item) => (
        <SelectPrimitive.Item
          item={item.item}
          data-slot="select-item"
          class="relative flex w-full cursor-default items-center gap-1.5 rounded-md py-1 pr-8 pl-1.5 text-sm outline-hidden select-none data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4"
        >
          <SelectPrimitive.ItemLabel class="flex flex-1 shrink-0 gap-2 whitespace-nowrap">
            {item.item.rawValue.label}
          </SelectPrimitive.ItemLabel>
          <SelectPrimitive.ItemIndicator class="pointer-events-none absolute right-2 flex size-4 items-center justify-center">
            <Check class="pointer-events-none" />
          </SelectPrimitive.ItemIndicator>
        </SelectPrimitive.Item>
      )}
    >
      <SelectPrimitive.Label data-slot="label" class="flex items-center gap-2 text-sm leading-none font-medium">
        {props.label}
      </SelectPrimitive.Label>
      <SelectPrimitive.Trigger
        data-slot="select-trigger"
        data-size={props.size ?? "default"}
        class="flex w-full items-center justify-between gap-1.5 rounded-lg border border-input bg-transparent py-2 pr-2 pl-2.5 text-sm whitespace-nowrap transition-colors outline-none select-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 data-[size=default]:h-8 data-[size=sm]:h-7 data-[size=sm]:rounded-[min(var(--radius-md),10px)] dark:bg-input/30 dark:hover:bg-input/50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4"
      >
        <SelectPrimitive.Value<SelectOption>
          data-slot="select-value"
          class="line-clamp-1 flex flex-1 items-center text-left"
        >
          {(state) => state.selectedOption()?.label}
        </SelectPrimitive.Value>
        <SelectPrimitive.Icon>
          <ChevronDown class="pointer-events-none size-4 text-muted-foreground" />
        </SelectPrimitive.Icon>
      </SelectPrimitive.Trigger>
      <SelectPrimitive.Portal>
        {/* No animation, as `data-[align-trigger=true]:animate-none`. */}
        <SelectPrimitive.Content
          ref={(element) => {
            content = element;
            element.addEventListener("pointerdown", () => (pressedInList = true), true);
            element.addEventListener("pointerup", ignoreOpeningRelease, true);
          }}
          data-slot="select-content"
          data-align-trigger
          class="relative isolate z-50 min-w-36 overflow-x-hidden overflow-y-auto rounded-lg bg-popover text-popover-foreground shadow-md ring-1 ring-foreground/10"
        >
          <SelectPrimitive.Listbox class="max-h-[calc(100svh-1rem)] scroll-my-1 overflow-y-auto p-1 outline-none" />
        </SelectPrimitive.Content>
      </SelectPrimitive.Portal>
    </SelectPrimitive.Root>
  );
}
