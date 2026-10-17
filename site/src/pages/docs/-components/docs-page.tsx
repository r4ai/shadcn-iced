// A page of the docs, after app/(app)/docs/[[...slug]]/page.tsx of
// ui.shadcn.com: the title with the links to the previous and next pages,
// the description, the content in typeset, and the "On This Page" list.
import { Link, type LinkProps } from "@tanstack/solid-router";
import { ArrowLeft, ArrowRight } from "lucide-solid";
import { Show, type JSX } from "solid-js";

import { buttonVariants } from "~/components/ui/button";
import { cn } from "~/lib/utils";

import { DocsTableOfContents, type TocItem } from "./docs-toc";

export type { TocItem };

/** A link to a neighbour page: its label, and where it goes. */
export type PagerLink = { label: string } & Pick<LinkProps, "to" | "params">;

interface DocsPageProps {
  title: string;
  description: string;
  toc?: TocItem[];
  prev?: PagerLink;
  next?: PagerLink;
  children: JSX.Element;
}

export function DocsPage(props: DocsPageProps) {
  return (
    <div data-slot="docs" class="flex scroll-mt-24 items-stretch pb-8 text-[1.05rem] sm:text-[15px] xl:w-full">
      <div class="flex min-w-0 flex-1 flex-col">
        <div class="h-(--top-spacing) shrink-0" />
        <div class="mx-auto flex w-full max-w-160 min-w-0 flex-1 flex-col gap-6 px-4 py-6 text-foreground md:px-0 lg:py-8">
          <div class="flex flex-col gap-2">
            <div class="flex items-center justify-between md:items-start">
              <h1 class="scroll-m-24 text-3xl font-semibold tracking-tight sm:text-3xl">{props.title}</h1>
              <div class="docs-nav ml-auto flex gap-2">
                <Show when={props.prev}>
                  {(prev) => (
                    <Link
                      to={prev().to}
                      params={prev().params}
                      data-slot="button"
                      class={cn(
                        buttonVariants({ variant: "secondary", size: "icon" }),
                        "extend-touch-target size-8 shadow-none md:size-7",
                      )}
                    >
                      <ArrowLeft />
                      <span class="sr-only">Previous</span>
                    </Link>
                  )}
                </Show>
                <Show when={props.next}>
                  {(next) => (
                    <Link
                      to={next().to}
                      params={next().params}
                      data-slot="button"
                      class={cn(
                        buttonVariants({ variant: "secondary", size: "icon" }),
                        "extend-touch-target size-8 shadow-none md:size-7",
                      )}
                    >
                      <span class="sr-only">Next</span>
                      <ArrowRight />
                    </Link>
                  )}
                </Show>
              </div>
            </div>
            <p class="text-[1.05rem] text-muted-foreground sm:text-base sm:text-balance md:max-w-[80%]">
              {props.description}
            </p>
          </div>
          <div class="typeset w-full flex-1 pb-16 sm:pb-0">{props.children}</div>
          <div class="hidden h-16 w-full items-center gap-2 px-4 sm:flex sm:px-0">
            <Show when={props.prev}>
              {(prev) => (
                <Link
                  to={prev().to}
                  params={prev().params}
                  data-slot="button"
                  class={cn(buttonVariants({ variant: "secondary", size: "sm" }), "shadow-none")}
                >
                  <ArrowLeft /> {prev().label}
                </Link>
              )}
            </Show>
            <Show when={props.next}>
              {(next) => (
                <Link
                  to={next().to}
                  params={next().params}
                  data-slot="button"
                  class={cn(buttonVariants({ variant: "secondary", size: "sm" }), "ml-auto shadow-none")}
                >
                  {next().label} <ArrowRight />
                </Link>
              )}
            </Show>
          </div>
        </div>
      </div>
      <div class="sticky top-[calc(var(--header-height)+1px)] z-30 ml-auto hidden h-[90svh] w-(--sidebar-width) flex-col gap-4 overflow-hidden overscroll-none pb-8 xl:flex">
        <div class="h-(--top-spacing) shrink-0" />
        <Show when={props.toc?.length}>
          <div class="scrollbar-none flex scroll-fade flex-col gap-8 overflow-y-auto px-8">
            <DocsTableOfContents toc={props.toc!} />
          </div>
        </Show>
      </div>
    </div>
  );
}
