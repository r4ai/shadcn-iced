// The layout of the docs, after app/(app)/docs/layout.tsx of ui.shadcn.com:
// the sidebar and the page. Small screens use the menu of the header.
import { createFileRoute, Outlet } from "@tanstack/solid-router";

import { DocsSidebar } from "./-components/docs-sidebar";

export const Route = createFileRoute("/docs")({
  component: DocsLayout,
});

function DocsLayout() {
  return (
    <div class="container-wrapper flex flex-1 flex-col px-2">
      <div
        data-slot="sidebar-wrapper"
        class="group/sidebar-wrapper flex min-h-min w-full flex-1 items-start px-0 [--top-spacing:0] lg:grid lg:grid-cols-[var(--sidebar-width)_minmax(0,1fr)] lg:[--top-spacing:calc(var(--spacing)*4)]"
        style={{ "--sidebar-width": "calc(var(--spacing) * 72)" }}
      >
        <DocsSidebar />
        <div class="h-full w-full">
          <Outlet />
        </div>
      </div>
    </div>
  );
}
