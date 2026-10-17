// The example apps, after app/(app)/examples of ui.shadcn.com: a header, a
// link per app, and the selected app as a single live preview, with the
// command that runs it on its own and its code.
import { Link } from "@tanstack/solid-router";
import { For, Show } from "solid-js";
import { apps } from "virtual:site/apps";

import { CodeBlock } from "~/components/code-block";
import { cn } from "~/lib/utils";

import { ComponentPreview } from "../../docs/components/$id/-components/component-preview";

export const DESCRIPTION =
  "Example apps built with shadcn-iced: mock data, but every control works. Run them on your desktop, or try them right here.";

export function ExamplesPage(props: { name: string }) {
  const app = () => apps.find((candidate) => candidate.id === props.name) ?? apps[0];

  return (
    <div class="flex flex-1 flex-col">
      <section class="border-grid">
        <div class="container-wrapper">
          <div class="container flex flex-col items-center gap-2 px-6 py-8 text-center md:py-16 md:pb-8 lg:py-20 lg:pb-12 xl:gap-4">
            <h1 class="leading-tighter max-w-4xl text-3xl font-semibold tracking-tight text-balance text-primary lg:leading-[1.1] xl:text-5xl xl:tracking-tighter">
              Example Apps
            </h1>
            <p class="max-w-3xl text-base text-balance text-foreground sm:text-lg">{DESCRIPTION}</p>
          </div>
        </div>
      </section>

      <div class="container-wrapper">
        <div class="container px-6">
          <nav class="no-scrollbar flex items-center gap-1 overflow-x-auto pb-6">
            <For each={apps}>
              {(item) => (
                <Link
                  to="/examples/$name"
                  params={{ name: item.id }}
                  class={cn(
                    "flex h-7 shrink-0 items-center justify-center rounded-full px-4 text-center text-sm font-medium text-muted-foreground transition-colors hover:text-primary",
                    app()?.id === item.id && "bg-muted text-primary",
                  )}
                >
                  {item.title}
                </Link>
              )}
            </For>
          </nav>

          <Show when={app()} keyed>
            {(current) => (
              <section class="pb-12">
                <h2 class="sr-only">{current.title}</h2>
                <p class="mb-4 text-sm text-muted-foreground">{current.description}</p>
                <CodeBlock snippet={current.command} />
                <div class="[&_[data-slot=component-preview]]:mt-4 [&_[data-slot=component-preview]]:mb-0">
                  <ComponentPreview page={current.id} example={current.examples[0]!} />
                </div>
              </section>
            )}
          </Show>
        </div>
      </div>
    </div>
  );
}
