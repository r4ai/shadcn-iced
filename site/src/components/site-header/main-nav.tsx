// Port of components/main-nav.tsx of ui.shadcn.com.
import { Link, useLocation } from "@tanstack/solid-router";
import { For } from "solid-js";

import { buttonVariants } from "~/components/ui/button";
import { isSectionActive, NAV_ITEMS } from "~/lib/navigation";
import { cn } from "~/lib/utils";

export function MainNav(props: { class?: string }) {
  const location = useLocation();

  return (
    <nav class={cn("items-center gap-0", props.class)}>
      <For each={NAV_ITEMS}>
        {(item) => (
          <Link
            to={item.to}
            data-slot="button"
            data-active={isSectionActive(item.to, location().pathname)}
            class={cn(buttonVariants({ variant: "ghost", size: "sm" }), "relative items-center px-2.5")}
          >
            {item.label}
          </Link>
        )}
      </For>
    </nav>
  );
}
