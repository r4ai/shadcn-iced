// The showcase of the home page, where ui.shadcn.com shows its cards:
// examples that read well side by side.
import { Link } from "@tanstack/solid-router";
import { ArrowRight } from "lucide-solid";
import { For } from "solid-js";

import { PreviewFrame } from "~/components/preview-frame";
import { components } from "~/lib/components";

const PICKS = [
  { id: "card", title: "Demo", height: 420 },
  { id: "chart", title: "Area Chart - Stacked", height: 420 },
  { id: "calendar", title: "Demo", height: 420 },
  { id: "field", title: "Choice Card", height: 360 },
  { id: "tabs", title: "Demo", height: 360 },
  { id: "input-otp", title: "Demo", height: 360 },
];

const SHOWCASE = PICKS.flatMap((pick) => {
  const component = components.find((c) => c.id === pick.id);
  const example = component?.examples.find((e) => e.title === pick.title) ?? component?.examples[0];
  return component && example ? [{ ...pick, component, example }] : [];
});

export function Showcase() {
  return (
    <div class="container-wrapper flex-1 p-0">
      <div class="container overflow-hidden pb-12">
        <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
          <For each={SHOWCASE}>
            {(item) => (
              <figure class="flex flex-col overflow-hidden rounded-2xl border">
                <PreviewFrame
                  page={item.component.id}
                  example={item.example.index}
                  minHeight={item.height}
                  class="rounded-b-none"
                />
                <figcaption class="flex items-center justify-between gap-2 border-t px-4 py-2.5 text-sm">
                  <span class="font-medium">{item.component.title}</span>
                  <Link
                    to="/docs/components/$id"
                    params={{ id: item.component.id }}
                    class="inline-flex items-center gap-1 text-xs text-muted-foreground transition-colors hover:text-foreground"
                  >
                    View code <ArrowRight class="size-3" />
                  </Link>
                </figcaption>
              </figure>
            )}
          </For>
        </div>
      </div>
    </div>
  );
}
