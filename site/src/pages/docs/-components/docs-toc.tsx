// Port of components/docs-toc.tsx of ui.shadcn.com (the list variant): the
// heading in view is marked active.
import { createSignal, For, onCleanup, onMount } from "solid-js";

export interface TocItem {
  id: string;
  label: string;
  depth?: 2 | 3;
}

export function DocsTableOfContents(props: { toc: TocItem[] }) {
  const [active, setActive] = createSignal<string>();

  onMount(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) setActive(entry.target.id);
        }
      },
      { rootMargin: "0% 0% -80% 0%" },
    );

    for (const item of props.toc) {
      const element = document.getElementById(item.id);
      if (element) observer.observe(element);
    }
    onCleanup(() => observer.disconnect());
  });

  return (
    <div class="flex flex-col gap-2 p-4 pt-0 text-sm">
      <p class="h-6 bg-background text-xs font-medium text-muted-foreground">On This Page</p>
      <For each={props.toc}>
        {(item) => (
          <a
            href={`#${item.id}`}
            class="text-[0.8rem] text-muted-foreground no-underline transition-colors hover:text-foreground data-[active=true]:font-medium data-[active=true]:text-foreground data-[depth=3]:pl-4"
            data-active={active() === item.id}
            data-depth={item.depth ?? 2}
          >
            {item.label}
          </a>
        )}
      </For>
    </div>
  );
}
