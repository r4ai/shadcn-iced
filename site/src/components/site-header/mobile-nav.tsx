// Port of components/mobile-nav.tsx of ui.shadcn.com: a full-screen menu
// under the header.
import { Link } from "@tanstack/solid-router";
import { createSignal, For, type JSX } from "solid-js";

import { Button } from "~/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "~/components/ui/popover";
import { components } from "~/lib/components";
import { DOCS_SECTIONS, NAV_ITEMS } from "~/lib/navigation";
import { cn } from "~/lib/utils";

export function MobileNav(props: { class?: string }) {
  const [open, setOpen] = createSignal(false);
  const close = () => setOpen(false);

  return (
    // No padding to the viewport (the collisionPadding of Radix): the menu fills
    // the space under the header.
    <Popover open={open()} onOpenChange={setOpen} placement="bottom-start" gutter={14} shift={-16} overflowPadding={0}>
      <PopoverTrigger
        as={Button}
        variant="ghost"
        class={cn(
          "extend-touch-target h-8 touch-manipulation items-center justify-start gap-2.5 p-0! hover:bg-transparent focus-visible:bg-transparent focus-visible:ring-0 active:bg-transparent dark:hover:bg-transparent",
          props.class,
        )}
      >
        <div class="relative flex h-8 w-4 items-center justify-center">
          <div class="relative size-4">
            <span
              class={cn(
                "absolute left-0 block h-0.5 w-4 bg-foreground transition-all duration-100",
                open() ? "top-[0.4rem] -rotate-45" : "top-1",
              )}
            />
            <span
              class={cn(
                "absolute left-0 block h-0.5 w-4 bg-foreground transition-all duration-100",
                open() ? "top-[0.4rem] rotate-45" : "top-2.5",
              )}
            />
          </div>
          <span class="sr-only">Toggle Menu</span>
        </div>
        <span class="flex h-8 items-center text-lg leading-none font-medium">Menu</span>
      </PopoverTrigger>
      <PopoverContent class="no-scrollbar block h-(--kb-popper-content-available-height) w-(--kb-popper-content-available-width) overflow-y-auto rounded-none border-none bg-background/90 p-0 shadow-none ring-0 backdrop-blur duration-100 data-expanded:animate-none!">
        <div class="flex flex-col gap-12 overflow-auto px-6 py-6">
          <MobileGroup title="Menu">
            <For each={NAV_ITEMS}>
              {(item) => (
                <MobileLink to={item.to} onClick={close}>
                  {item.label}
                </MobileLink>
              )}
            </For>
          </MobileGroup>
          <MobileGroup title="Sections">
            <For each={DOCS_SECTIONS}>
              {(section) => (
                <MobileLink to={section.to} onClick={close}>
                  {section.label}
                </MobileLink>
              )}
            </For>
          </MobileGroup>
          <MobileGroup title="Components">
            <For each={components}>
              {(component) => (
                <Link
                  to="/docs/components/$id"
                  params={{ id: component.id }}
                  onClick={close}
                  class="flex items-center gap-2 text-2xl font-medium"
                >
                  {component.title}
                </Link>
              )}
            </For>
          </MobileGroup>
        </div>
      </PopoverContent>
    </Popover>
  );
}

function MobileGroup(props: { title: string; children: JSX.Element }) {
  return (
    <div class="flex flex-col gap-4">
      <div class="text-sm font-medium text-muted-foreground">{props.title}</div>
      <div class="flex flex-col gap-3">{props.children}</div>
    </div>
  );
}

function MobileLink(props: {
  to: (typeof NAV_ITEMS)[number]["to"] | (typeof DOCS_SECTIONS)[number]["to"];
  onClick: () => void;
  children: string;
}) {
  return (
    <Link to={props.to} onClick={props.onClick} class="flex items-center gap-2 text-2xl font-medium">
      {props.children}
    </Link>
  );
}
