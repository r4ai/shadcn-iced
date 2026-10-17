import { Link } from "@tanstack/solid-router";

import { buttonVariants } from "~/components/ui/button";

export function NotFound() {
  return (
    <main class="mx-auto flex max-w-xl flex-col items-center gap-4 px-4 py-24 text-center">
      <p class="font-mono text-sm text-muted-foreground">404</p>
      <h1 class="text-3xl font-semibold tracking-tight">Page not found</h1>
      <p class="text-muted-foreground">This page does not exist. The components are listed in the docs.</p>
      <Link to="/docs/components" class={buttonVariants()}>
        Browse Components
      </Link>
    </main>
  );
}
