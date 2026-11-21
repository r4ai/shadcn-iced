// Port of components/command-menu.tsx of ui.shadcn.com: a button in the
// header that opens a dialog to search the site (also with ⌘K, Ctrl+K or /).
// Kobalte has no command (cmdk), so the menu is a combobox of its own: the
// input keeps the focus, and the arrows move the selected item.
//
// The items are the options of the ARIA combobox pattern (with
// `aria-activedescendant`): the keys of the input choose them, not their own
// focus, and no element of HTML has their roles.
// oxlint-disable jsx-a11y/prefer-tag-over-role, jsx-a11y/click-events-have-key-events, jsx-a11y/interactive-supports-focus
import * as DialogPrimitive from "@kobalte/core/dialog";
import { useNavigate } from "@tanstack/solid-router";
import { CornerDownLeft, Search } from "lucide-solid";
import { createMemo, createSignal, For, onCleanup, onMount, Show, type JSX } from "solid-js";

import { Button } from "~/components/ui/button";
import { Separator } from "~/components/ui/separator";
import { search, type SearchItem } from "~/lib/search";
import { cn } from "~/lib/utils";

/** Whether a key is the shortcut of the menu (outside of a text field). */
function isShortcut(event: KeyboardEvent) {
  if (!((event.key === "k" && (event.metaKey || event.ctrlKey)) || event.key === "/")) return false;
  const target = event.target;
  return !(
    (target instanceof HTMLElement && target.isContentEditable) ||
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement
  );
}

const itemId = (index: number) => `command-item-${index}`;

export function CommandMenu() {
  const [open, setOpen] = createSignal(false);

  const down = (event: KeyboardEvent) => {
    if (!isShortcut(event)) return;
    event.preventDefault();
    setOpen((wasOpen) => !wasOpen);
  };

  onMount(() => {
    document.addEventListener("keydown", down);
    onCleanup(() => document.removeEventListener("keydown", down));
  });

  return (
    <DialogPrimitive.Root open={open()} onOpenChange={setOpen}>
      <DialogPrimitive.Trigger
        as={Button}
        variant="outline"
        data-slot="command-menu-trigger"
        class="relative h-8 w-full justify-start rounded-lg border-none bg-muted pl-3 text-foreground shadow-none transition-colors hover:bg-muted/50 md:w-48 lg:w-40 xl:w-64 dark:bg-card"
      >
        <span class="hidden xl:inline-flex">Search documentation...</span>
        <span class="inline-flex xl:hidden">Search...</span>
      </DialogPrimitive.Trigger>
      <DialogPrimitive.Portal>
        {/* No overlay, as on ui.shadcn.com. */}
        <DialogPrimitive.Content
          data-slot="dialog-content"
          class="fixed top-[15%] left-[50%] z-50 grid w-full max-w-[calc(100%-2rem)] translate-x-[-50%] gap-4 rounded-xl border-none bg-background bg-clip-padding p-2 pb-11 shadow-2xl ring-4 ring-neutral-200/80 duration-200 outline-none data-expanded:animate-in data-expanded:fade-in-0 data-expanded:zoom-in-95 sm:max-w-lg dark:bg-neutral-900 dark:ring-neutral-800 data-closed:animate-out data-closed:fade-out-0 data-closed:zoom-out-95"
        >
          <div class="sr-only">
            <DialogPrimitive.Title>Search documentation...</DialogPrimitive.Title>
            <DialogPrimitive.Description>Search for a command to run...</DialogPrimitive.Description>
          </div>
          <Command onClose={() => setOpen(false)} />
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  );
}

/** Whether the shortcuts are shown with ⌘ (on Apple platforms) or Ctrl. */
const isMac = () => typeof navigator !== "undefined" && /Mac|iPhone|iPad|iPod/.test(navigator.platform);

/**
 * The search: an input, the items that match it in groups, and a footer with
 * what Enter and ⌘C do with the selected item. Mounted with the dialog: each
 * opening starts over.
 */
function Command(props: { onClose: () => void }) {
  const navigate = useNavigate();
  const [query, setQuery] = createSignal("");
  // The groups, with the index of each item among all of them.
  const groups = createMemo(() => {
    let index = 0;
    return search(query()).map((group) => ({
      heading: group.heading,
      items: group.items.map((item) => ({ item, index: index++ })),
    }));
  });
  const items = createMemo(() => groups().flatMap((group) => group.items.map(({ item }) => item)));
  const [selected, setSelected] = createSignal(0);
  const current = () => items()[selected()];
  const [copied, setCopied] = createSignal(false);
  let list: HTMLDivElement | undefined;

  const select = (index: number, scroll = true) => {
    setSelected(index);
    setCopied(false);
    if (scroll) {
      // The first item shows the heading of its group too.
      const element = index === 0 ? list?.firstElementChild : document.getElementById(itemId(index));
      element?.scrollIntoView({ block: "nearest" });
    }
  };

  const run = (item: SearchItem) => {
    props.onClose();
    const [to, hash] = item.href.split("#");
    void navigate({ to, hash } as never);
  };

  const onInput = (value: string) => {
    setQuery(value);
    select(0, false);
    if (list) list.scrollTop = 0;
  };

  const onKeyDown = (event: KeyboardEvent) => {
    const count = items().length;

    switch (event.key) {
      case "ArrowDown":
      case "ArrowUp": {
        event.preventDefault();
        if (!count) return;
        const step = event.key === "ArrowDown" ? 1 : -1;
        select((selected() + step + count) % count);
        break;
      }
      case "Home":
      case "End":
        if (!event.ctrlKey) return;
        event.preventDefault();
        if (count) select(event.key === "Home" ? 0 : count - 1);
        break;
      case "Enter": {
        if (event.isComposing) return;
        event.preventDefault();
        const item = current();
        if (item) run(item);
        break;
      }
      case "c": {
        // ⌘C copies the command of the selected item, unless text is selected.
        const input = event.currentTarget as HTMLInputElement;
        const copy = current()?.copy;
        if (!(event.metaKey || event.ctrlKey) || !copy || input.selectionStart !== input.selectionEnd) return;
        event.preventDefault();
        void navigator.clipboard?.writeText(copy).then(() => setCopied(true));
        break;
      }
    }
  };

  return (
    <>
      <div
        data-slot="command"
        class="flex h-full w-full flex-col overflow-hidden rounded-none bg-transparent text-popover-foreground"
      >
        <div
          data-slot="command-input-wrapper"
          class="flex h-9 items-center gap-2 rounded-md border border-input bg-input/50 px-3"
        >
          <Search class="size-4 shrink-0 opacity-50" />
          <input
            data-slot="command-input"
            role="combobox"
            aria-expanded="true"
            aria-controls="command-list"
            aria-autocomplete="list"
            aria-activedescendant={current() ? itemId(selected()) : undefined}
            aria-label="Search documentation..."
            autocomplete="off"
            autocorrect="off"
            spellcheck={false}
            placeholder="Search documentation..."
            class="flex h-9 w-full rounded-md bg-transparent py-0 text-sm outline-hidden placeholder:text-muted-foreground"
            value={query()}
            onInput={(event) => onInput(event.currentTarget.value)}
            onKeyDown={onKeyDown}
          />
        </div>
        <div
          ref={(element) => (list = element)}
          id="command-list"
          role="listbox"
          aria-label="Suggestions"
          data-slot="command-list"
          class="no-scrollbar max-h-[300px] min-h-80 scroll-pt-2 scroll-pb-1.5 overflow-x-hidden overflow-y-auto"
        >
          <Show
            when={groups().length}
            fallback={
              <div data-slot="command-empty" class="py-12 text-center text-sm text-muted-foreground">
                No results found.
              </div>
            }
          >
            <For each={groups()}>
              {(group) => (
                <div
                  role="group"
                  aria-labelledby={`command-group-${group.heading}`}
                  data-slot="command-group"
                  class="overflow-hidden p-0 text-foreground"
                >
                  <div
                    id={`command-group-${group.heading}`}
                    aria-hidden="true"
                    class="scroll-mt-16 p-3 pb-1 text-xs font-medium text-muted-foreground"
                  >
                    {group.heading}
                  </div>
                  <For each={group.items}>
                    {({ item, index }) => (
                      <CommandItem
                        id={itemId(index)}
                        selected={selected() === index}
                        result={item.kind === "result"}
                        onPointerMove={() => selected() !== index && select(index, false)}
                        onClick={() => run(item)}
                      >
                        <Show when={item.kind === "page"}>
                          <ArrowRightIcon />
                        </Show>
                        <Show when={item.kind === "component"}>
                          <div class="aspect-square size-4 rounded-full border border-dashed border-muted-foreground" />
                        </Show>
                        <span class={cn(item.kind === "result" && "line-clamp-1")}>{item.label}</span>
                        <Show when={item.detail}>
                          <span class="ml-auto shrink-0 text-xs font-normal text-muted-foreground">{item.detail}</span>
                        </Show>
                      </CommandItem>
                    )}
                  </For>
                </div>
              )}
            </For>
          </Show>
        </div>
      </div>
      <div class="absolute inset-x-0 bottom-0 z-20 flex h-10 items-center gap-2 rounded-b-xl border-t border-t-neutral-100 bg-neutral-50 px-4 text-xs font-medium text-muted-foreground dark:border-t-neutral-700 dark:bg-neutral-800">
        <div class="flex items-center gap-2">
          <Kbd>
            <CornerDownLeft />
          </Kbd>{" "}
          <Show when={current()}>Go to Page</Show>
        </div>
        <Show when={current()?.copy}>
          {(copy) => (
            <>
              <Separator orientation="vertical" class="h-4!" />
              <div class="flex min-w-0 items-center gap-1">
                <Kbd>{isMac() ? "⌘" : "Ctrl"}</Kbd>
                <Kbd>C</Kbd>
                <span class="truncate">{copied() ? "Copied" : copy()}</span>
              </div>
            </>
          )}
        </Show>
      </div>
    </>
  );
}

function CommandItem(props: {
  id: string;
  selected: boolean;
  result: boolean;
  onPointerMove: () => void;
  onClick: () => void;
  children: JSX.Element;
}) {
  return (
    <div
      id={props.id}
      role="option"
      aria-selected={props.selected}
      data-selected={props.selected}
      data-slot="command-item"
      class={cn(
        "relative flex h-9 cursor-default items-center gap-2 rounded-md border border-transparent px-3 text-sm outline-hidden select-none data-[selected=true]:border-input data-[selected=true]:bg-input/50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 [&_svg:not([class*='text-'])]:text-muted-foreground",
        props.result ? "font-normal" : "font-medium",
      )}
      onPointerMove={props.onPointerMove}
      onClick={props.onClick}
    >
      {props.children}
    </div>
  );
}

function Kbd(props: { children: JSX.Element }) {
  return (
    <kbd class="pointer-events-none flex h-5 items-center justify-center gap-1 rounded border bg-background px-1 font-sans text-[0.7rem] font-medium text-muted-foreground select-none [&_svg:not([class*='size-'])]:size-3">
      {props.children}
    </kbd>
  );
}

/** The arrow-right icon of Tabler, as on ui.shadcn.com. */
function ArrowRightIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <path stroke="none" d="M0 0h24v24H0z" fill="none" />
      <path d="M5 12l14 0" />
      <path d="M13 18l6 -6" />
      <path d="M13 6l6 6" />
    </svg>
  );
}
