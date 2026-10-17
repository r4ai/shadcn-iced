// The chart showcase, after app/(app)/charts of ui.shadcn.com: a header, a
// link per kind of chart, and the charts of the selected kind in a grid.
import { Link } from "@tanstack/solid-router";
import { For, Show } from "solid-js";
import { charts } from "virtual:site/charts";

import { cn } from "~/lib/utils";

import { ComponentPreview } from "../../docs/components/$id/-components/component-preview";

/** The kind of a chart page (`charts-area` → `area`). */
export const kindOf = (id: string) => id.replace(/^charts-/, "");

export const DESCRIPTION =
  "A collection of ready-to-use charts built with the chart component of shadcn-iced. Copy and paste into your apps.";

export function ChartsPage(props: { kind: string }) {
  const page = () => charts.find((chart) => kindOf(chart.id) === props.kind) ?? charts[0];

  return (
    <div class="flex flex-1 flex-col">
      <section class="border-grid">
        <div class="container-wrapper">
          <div class="container flex flex-col items-center gap-2 px-6 py-8 text-center md:py-16 md:pb-8 lg:py-20 lg:pb-12 xl:gap-4">
            <h1 class="leading-tighter max-w-4xl text-3xl font-semibold tracking-tight text-balance text-primary lg:leading-[1.1] xl:text-5xl xl:tracking-tighter">
              Beautiful Charts
            </h1>
            <p class="max-w-3xl text-base text-balance text-foreground sm:text-lg">{DESCRIPTION}</p>
          </div>
        </div>
      </section>

      <div class="container-wrapper">
        <div class="container px-6">
          <nav class="no-scrollbar flex items-center gap-1 overflow-x-auto pb-6">
            <For each={charts}>
              {(chart) => (
                <Link
                  to="/charts/$kind"
                  params={{ kind: kindOf(chart.id) }}
                  class={cn(
                    "flex h-7 shrink-0 items-center justify-center rounded-full px-4 text-center text-sm font-medium text-muted-foreground transition-colors hover:text-primary",
                    page()?.id === chart.id && "bg-muted text-primary",
                  )}
                >
                  {chart.title.replace(/ Charts$/, "")}
                </Link>
              )}
            </For>
          </nav>

          <Show when={page()} keyed>
            {(current) => (
              <section class="pb-12">
                <h2 class="sr-only">{current.title}</h2>
                <div class="grid gap-4 md:grid-cols-2 lg:gap-x-6">
                  <For each={current.examples}>
                    {(example) => (
                      <div class="min-w-0 [&_[data-slot=component-preview]]:mt-0 [&_[data-slot=component-preview]]:mb-4">
                        <ComponentPreview page={current.id} example={example} />
                      </div>
                    )}
                  </For>
                </div>
              </section>
            )}
          </Show>
        </div>
      </div>
    </div>
  );
}
