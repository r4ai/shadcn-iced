// Port of the tabs of shadcn/ui (registry/new-york-v4/ui/tabs.tsx) on Kobalte,
// which marks the selected tab with `data-selected` (Radix: `data-state=active`).
import type { PolymorphicProps } from "@kobalte/core/polymorphic";
import * as TabsPrimitive from "@kobalte/core/tabs";
import { splitProps, type ValidComponent } from "solid-js";
import { tv, type VariantProps } from "tailwind-variants";

import { cn } from "~/lib/utils";

type WithClass<T> = T & { class?: string };

export function Tabs<T extends ValidComponent = "div">(
  props: PolymorphicProps<T, WithClass<TabsPrimitive.TabsRootProps<T>>>,
) {
  const [local, others] = splitProps(props as WithClass<TabsPrimitive.TabsRootProps>, ["class"]);

  return (
    <TabsPrimitive.Root
      data-slot="tabs"
      class={cn("group/tabs flex gap-2 data-[orientation=horizontal]:flex-col", local.class)}
      {...others}
    />
  );
}

export const tabsListVariants = tv({
  base: "group/tabs-list inline-flex w-fit items-center justify-center rounded-lg p-[3px] text-muted-foreground group-data-[orientation=horizontal]/tabs:h-9 group-data-[orientation=vertical]/tabs:h-fit group-data-[orientation=vertical]/tabs:flex-col data-[variant=line]:rounded-none",
  variants: {
    variant: {
      default: "bg-muted",
      line: "gap-1 bg-transparent",
    },
  },
  defaultVariants: { variant: "default" },
});

type TabsListProps<T extends ValidComponent = "div"> = WithClass<TabsPrimitive.TabsListProps<T>> &
  VariantProps<typeof tabsListVariants>;

export function TabsList<T extends ValidComponent = "div">(props: PolymorphicProps<T, TabsListProps<T>>) {
  const [local, others] = splitProps(props as TabsListProps, ["class", "variant"]);

  return (
    <TabsPrimitive.List
      data-slot="tabs-list"
      data-variant={local.variant ?? "default"}
      class={cn(tabsListVariants({ variant: local.variant }), local.class)}
      {...others}
    />
  );
}

export function TabsTrigger<T extends ValidComponent = "button">(
  props: PolymorphicProps<T, WithClass<TabsPrimitive.TabsTriggerProps<T>>>,
) {
  const [local, others] = splitProps(props as WithClass<TabsPrimitive.TabsTriggerProps>, ["class"]);

  return (
    <TabsPrimitive.Trigger
      data-slot="tabs-trigger"
      class={cn(
        "relative inline-flex h-[calc(100%-1px)] flex-1 items-center justify-center gap-1.5 rounded-md border border-transparent px-2 py-1 text-sm font-medium whitespace-nowrap text-foreground/60 transition-all group-data-[orientation=vertical]/tabs:w-full group-data-[orientation=vertical]/tabs:justify-start hover:text-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-1 focus-visible:outline-ring disabled:pointer-events-none disabled:opacity-50 group-data-[variant=default]/tabs-list:data-[selected]:shadow-sm group-data-[variant=line]/tabs-list:data-[selected]:shadow-none dark:text-muted-foreground dark:hover:text-foreground [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
        "group-data-[variant=line]/tabs-list:bg-transparent group-data-[variant=line]/tabs-list:data-[selected]:bg-transparent dark:group-data-[variant=line]/tabs-list:data-[selected]:border-transparent dark:group-data-[variant=line]/tabs-list:data-[selected]:bg-transparent",
        "data-[selected]:bg-background data-[selected]:text-foreground dark:data-[selected]:border-input dark:data-[selected]:bg-input/30 dark:data-[selected]:text-foreground",
        "after:absolute after:bg-foreground after:opacity-0 after:transition-opacity group-data-[orientation=horizontal]/tabs:after:inset-x-0 group-data-[orientation=horizontal]/tabs:after:bottom-[-5px] group-data-[orientation=horizontal]/tabs:after:h-0.5 group-data-[orientation=vertical]/tabs:after:inset-y-0 group-data-[orientation=vertical]/tabs:after:-right-1 group-data-[orientation=vertical]/tabs:after:w-0.5 group-data-[variant=line]/tabs-list:data-[selected]:after:opacity-100",
        local.class,
      )}
      {...others}
    />
  );
}

export function TabsContent<T extends ValidComponent = "div">(
  props: PolymorphicProps<T, WithClass<TabsPrimitive.TabsContentProps<T>>>,
) {
  const [local, others] = splitProps(props as WithClass<TabsPrimitive.TabsContentProps>, ["class"]);

  return <TabsPrimitive.Content data-slot="tabs-content" class={cn("flex-1 outline-none", local.class)} {...others} />;
}
