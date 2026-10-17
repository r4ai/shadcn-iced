import { createFileRoute, Link } from "@tanstack/solid-router";
import { For } from "solid-js";

import { components } from "~/lib/components";
import { seo } from "~/lib/seo";

import { DocsPage } from "../-components/docs-page";

const TITLE = "Components";
const DESCRIPTION = `${components.length} components ported from shadcn/ui, each with live previews running in your browser.`;

export const Route = createFileRoute("/docs/components/")({
  head: () => seo(`${TITLE} - shadcn-iced`, DESCRIPTION),
  component: ComponentList,
});

// The grid of components/components-list.tsx of ui.shadcn.com.
function ComponentList() {
  return (
    <DocsPage
      title={TITLE}
      description={DESCRIPTION}
      prev={{ to: "/docs/theming", label: "Theming" }}
      next={
        components[0] && { to: "/docs/components/$id", params: { id: components[0].id }, label: components[0].title }
      }
    >
      <div
        data-not-typeset
        class="mt-8 grid grid-cols-2 gap-4 md:grid-cols-3 md:gap-x-8 lg:gap-x-16 lg:gap-y-6 xl:gap-x-20"
      >
        <For each={components}>
          {(component) => (
            <Link
              to="/docs/components/$id"
              params={{ id: component.id }}
              class="inline-flex items-center gap-2 text-lg font-medium underline-offset-4 hover:underline md:text-base"
            >
              {component.title}
            </Link>
          )}
        </For>
      </div>
    </DocsPage>
  );
}
