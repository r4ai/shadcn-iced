// Port of components/docs-sidebar.tsx of ui.shadcn.com, with the classes of
// the sidebar of registry/new-york-v4/ui/sidebar.tsx (collapsible="none").
import { Link, useLocation } from "@tanstack/solid-router";
import { createEffect, For, on, onCleanup, onMount, type JSX } from "solid-js";

import { components } from "~/lib/components";
import { restoreScroll, saveScrollState } from "~/lib/docs-sidebar-scroll";
import { DOCS_SECTIONS, isSectionActive } from "~/lib/navigation";

export function DocsSidebar() {
  const location = useLocation();
  const path = () => location().pathname.replace(/\/$/, "");
  let content!: HTMLDivElement;

  // On each page: come back to its scroll, or show its item.
  createEffect(
    on(path, () => {
      restoreScroll(content);
      saveScrollState(content);
    }),
  );

  onMount(() => {
    const save = () => saveScrollState(content);
    content.addEventListener("scroll", save, { passive: true });
    onCleanup(() => content.removeEventListener("scroll", save));
  });

  return (
    <div
      data-slot="sidebar"
      class="sticky top-[calc(var(--header-height)+0.6rem)] z-30 hidden h-[calc(100svh-10rem)] w-(--sidebar-width) flex-col overflow-hidden overscroll-none bg-sidebar bg-transparent text-sidebar-foreground [--sidebar-menu-width:--spacing(56)] lg:flex"
    >
      <div class="absolute top-12 right-2 bottom-0 hidden h-full w-px bg-[linear-gradient(to_bottom,transparent_0%,var(--border)_10%,var(--border)_90%,transparent_100%)] lg:flex" />
      <div
        ref={(element) => (content = element)}
        data-slot="sidebar-content"
        data-docs-sidebar-content
        class="scrollbar-none flex min-h-0 w-(--sidebar-menu-width) flex-1 scroll-fade flex-col gap-2 overflow-auto overflow-x-hidden pl-2.5"
      >
        <SidebarGroup label="Sections" class="pt-12">
          <ul data-slot="sidebar-menu" class="flex w-full min-w-0 flex-col gap-1">
            <For each={DOCS_SECTIONS}>
              {(section) => (
                <SidebarItem>
                  <Link to={section.to} class={MENU_BUTTON} data-active={isSectionActive(section.to, path())}>
                    <span class="absolute inset-0 flex w-(--sidebar-menu-width) bg-transparent" />
                    {section.label}
                  </Link>
                </SidebarItem>
              )}
            </For>
          </ul>
        </SidebarGroup>
        <SidebarGroup label="Components">
          <ul data-slot="sidebar-menu" class="flex w-full min-w-0 flex-col gap-0.5">
            <For each={components}>
              {(component) => {
                const active = () => path() === `/docs/components/${component.id}`;
                return (
                  <SidebarItem>
                    <Link
                      to="/docs/components/$id"
                      params={{ id: component.id }}
                      class={MENU_BUTTON}
                      data-active={active()}
                    >
                      <span class="absolute inset-0 flex w-(--sidebar-menu-width) bg-transparent" />
                      {component.title}
                    </Link>
                  </SidebarItem>
                );
              }}
            </For>
          </ul>
        </SidebarGroup>
      </div>
    </div>
  );
}

// `sidebarMenuButtonVariants` (default variant and size), with the classes of
// the docs sidebar.
const MENU_BUTTON =
  "peer/menu-button ring-sidebar-ring hover:bg-sidebar-accent hover:text-sidebar-accent-foreground active:bg-sidebar-accent active:text-sidebar-accent-foreground data-[active=true]:bg-sidebar-accent data-[active=true]:text-sidebar-accent-foreground data-[active=true]:border-accent data-[active=true]:bg-accent relative flex h-[30px] w-fit items-center gap-2 overflow-visible rounded-md border border-transparent p-2 text-left text-[0.8rem] font-medium outline-hidden transition-[width,height,padding] after:absolute after:inset-x-0 after:-inset-y-1 after:z-0 after:rounded-md focus-visible:ring-2 [&>span:last-child]:truncate";

function SidebarGroup(props: { label: string; class?: string; children: JSX.Element }) {
  return (
    <div data-slot="sidebar-group" class={`relative flex w-full min-w-0 flex-col p-2 ${props.class ?? ""}`}>
      <div
        data-slot="sidebar-group-label"
        class="flex h-8 shrink-0 items-center rounded-md px-2 text-xs font-medium text-muted-foreground"
      >
        {props.label}
      </div>
      <div data-slot="sidebar-group-content" class="w-full text-sm">
        {props.children}
      </div>
    </div>
  );
}

function SidebarItem(props: { children: JSX.Element }) {
  return (
    <li data-slot="sidebar-menu-item" class="group/menu-item relative">
      {props.children}
    </li>
  );
}
