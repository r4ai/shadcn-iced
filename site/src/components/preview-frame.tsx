// A live preview: once the frame comes near the viewport, iced replaces its
// target element with the canvas of the example (see `lib/previews.ts`).
//
// Until the example measures itself, the frame takes the height measured by
// `site_data` at its width (`PREVIEW_HEIGHT_STYLES`), from the first paint:
// the page does not move when the preview starts.
import { LoaderCircle, TriangleAlert } from "lucide-solid";
import { createSignal, createUniqueId, onCleanup, onMount, Show, type JSX } from "solid-js";

import { previewWidths, type PreviewHeights } from "~/lib/components";
import { loadPreviews, onResized, previewsCrashed, scrollOver, wake } from "~/lib/previews";
import { cn } from "~/lib/utils";

/**
 * Picks the measured height of each frame by the width of its container: the
 * one at the widest measured width that fits, the height of a narrower
 * frame, so a preview does not grow when its example runs.
 */
export const PREVIEW_HEIGHT_STYLES = previewWidths
  .map((width, index) => {
    const rule = `[data-preview]{height:var(--preview-height-${index},var(--preview-min-height))}`;
    return index === 0 ? rule : `@container preview (min-width:${width}px){${rule}}`;
  })
  .join("");

interface PreviewFrameProps {
  /** The id of the component of the example. */
  page: string;
  /** The index of the example in its page. */
  example: number;
  minHeight: number;
  /** The heights of the example at `previewWidths`. */
  heights?: PreviewHeights;
  class?: string;
}

export function PreviewFrame(props: PreviewFrameProps) {
  const target = `preview-${createUniqueId().replace(/[^\w-]/g, "")}`;
  const [failed, setFailed] = createSignal(false);
  const [height, setHeight] = createSignal<number>();
  let frame!: HTMLDivElement;

  const style = (): JSX.CSSProperties => {
    const measured = height();
    if (measured !== undefined) return { height: `${measured}px` };

    const measuredAhead: JSX.CSSProperties = { "--preview-min-height": `${props.minHeight}px` };
    props.heights?.forEach((natural, index) => {
      measuredAhead[`--preview-height-${index}`] = `${Math.max(props.minHeight, natural ?? 0)}px`;
    });
    return measuredAhead;
  };

  onMount(() => {
    // Start downloading right away; mount the preview when it comes near.
    const ready = loadPreviews();
    let mounted = false;
    let disposed = false;

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (!entry?.isIntersecting) return;
        observer.disconnect();

        ready
          .then((previews) => {
            if (disposed) return;
            if (!previews.mount(target, props.page, props.example)) {
              throw new Error(`no page ${props.page}`);
            }
            mounted = true;
            return wake(frame);
          })
          .catch((error) => {
            console.error(error);
            setFailed(true);
          });
      },
      { rootMargin: "400px 0px" },
    );
    observer.observe(frame);

    const stopScrollOver = scrollOver(frame);
    const stopResized = onResized(target, (natural) => setHeight(Math.max(props.minHeight, Math.ceil(natural))));

    onCleanup(() => {
      disposed = true;
      observer.disconnect();
      stopScrollOver();
      stopResized();
      if (mounted) ready.then((previews) => previews.unmount(target));
    });
  });

  return (
    // The container of the frame, for `PREVIEW_HEIGHT_STYLES`.
    <div class="@container/preview w-full rounded-[inherit]">
      <div
        ref={(element) => (frame = element)}
        data-preview
        // winit cancels touch scrolling on its canvas; scrolling the page over a
        // preview matters more on a phone.
        on:touchstart={{ handleEvent: (event) => event.stopPropagation(), capture: true, passive: true }}
        class={cn("relative w-full overflow-hidden rounded-[inherit] bg-background", props.class)}
        style={style()}
      >
        {/* The canvas covers the spinner once it draws. */}
        <Show
          when={!failed() && !previewsCrashed()}
          fallback={
            <div class="absolute inset-0 z-10 flex flex-col items-center justify-center gap-2 bg-background p-6 text-center text-sm text-muted-foreground">
              <TriangleAlert class="size-4.5" />
              <p>The live preview could not start in this browser.</p>
            </div>
          }
        >
          <div
            class="absolute inset-0 flex flex-col items-center justify-center gap-2 text-xs text-muted-foreground"
            aria-hidden="true"
          >
            <LoaderCircle class="size-4 motion-safe:animate-spin" />
            Loading preview
          </div>
        </Show>
        <div id={target} />
      </div>
    </div>
  );
}
