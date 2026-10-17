// The headings of the docs, with the anchor of `HeadingAnchor` of
// mdx-components.tsx of ui.shadcn.com.
import { Dynamic } from "solid-js/web";

export function Heading(props: { level: 2 | 3; id: string; children: string }) {
  return (
    <Dynamic component={`h${props.level}`} id={props.id}>
      <a class="group no-underline" href={`#${props.id}`}>
        <span class="underline-offset-4 group-hover:underline">{props.children}</span>
        <span aria-hidden="true" class="ml-2 text-muted-foreground opacity-0 group-hover:opacity-100">
          #
        </span>
      </a>
    </Dynamic>
  );
}
