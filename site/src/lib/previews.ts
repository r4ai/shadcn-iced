// The live previews: the examples of the gallery, compiled to WebAssembly
// (`examples/gallery-web`), each drawn by iced into a canvas of its own.
import { createSignal } from "solid-js";

export interface Previews {
  mount(target: string, page: string, example: number): boolean;
  unmount(target: string): void;
  setTheme(dark: boolean, base: string, accent: string, radius: string): void;
  themeCode(dark: boolean, base: string, accent: string, radius: string): string;
}

declare global {
  interface Window {
    shadcnIcedPreviewResized?: (target: string, height: number) => void;
    shadcnIcedPreviewWheel?: (captured: boolean) => void;
  }
}

const [api, setApi] = createSignal<Previews>();
const [crashed, setCrashed] = createSignal(false);

/** The previews, once they are loaded. */
export const previews = api;

/** Whether the previews stopped: a panic of iced ends all of them. */
export const previewsCrashed = crashed;

let loading: Promise<Previews> | undefined;

/**
 * Lets iced fall back to its software renderer when WebGPU has no adapter.
 * iced picks its renderer by trying them on the canvas, and wgpu takes the
 * canvas with a `webgpu` context before it finds no adapter, so the 2D
 * context of the software renderer then fails. Without an adapter, the
 * canvases have no `webgpu` context: wgpu fails right away instead.
 */
async function hideWebGpuWithoutAdapter() {
  const gpu = (navigator as Navigator & { gpu?: { requestAdapter(): Promise<unknown> } }).gpu;
  const adapter = await gpu?.requestAdapter().catch(() => null);
  if (adapter) return;

  const getContext = HTMLCanvasElement.prototype.getContext;
  HTMLCanvasElement.prototype.getContext = function (this: HTMLCanvasElement, type: string, ...args: unknown[]) {
    return type === "webgpu" ? null : getContext.call(this, type, ...(args as []));
  } as typeof getContext;
}

/** Loads the previews (once). */
export function loadPreviews(): Promise<Previews> {
  loading ??= (async () => {
    await hideWebGpuWithoutAdapter();
    window.addEventListener("error", (event) => {
      if (event.error instanceof WebAssembly.RuntimeError) setCrashed(true);
    });

    const base = import.meta.env.BASE_URL.replace(/\/$/, "");
    const url = new URL(`${base}/wasm/gallery_web.js`, location.origin).href;
    const module = await import(/* @vite-ignore */ url);
    await module.default();

    const loaded = module as Previews;
    setApi(() => loaded);
    return loaded;
  })();

  return loading;
}

const resized = new Map<string, (height: number) => void>();

/** Listens to the natural height of the preview mounted on `target`. */
export function onResized(target: string, listener: (height: number) => void) {
  resized.set(target, listener);
  return () => resized.delete(target);
}

if (typeof window !== "undefined") {
  window.shadcnIcedPreviewResized = (target, height) => resized.get(target)?.(height);
  window.shadcnIcedPreviewWheel = (captured) => (wheelCaptured = captured);
}

/** Whether the preview under the wheel event being dispatched used it. */
let wheelCaptured: boolean | undefined;

/** Holds back the cancel of winit, before it handles the event. */
function holdWheel(event: WheelEvent) {
  if (!(event.target instanceof HTMLCanvasElement)) return;
  wheelCaptured = undefined;
  event.preventDefault = () => {};
}

/** Cancels the event unless iced said it did not use it. */
function settleWheel(event: WheelEvent) {
  if (!(event.target instanceof HTMLCanvasElement)) return;
  if (wheelCaptured !== false) Event.prototype.preventDefault.call(event);
  wheelCaptured = undefined;
}

/**
 * Lets the browser scroll the page over the preview in `frame`, as it does
 * elsewhere, unless the preview uses the wheel event (e.g. a scroll area).
 * winit cancels every wheel event over its canvas, so its cancel is held back
 * until iced, which handles the event within the listener of winit, tells
 * whether it used it. Without an answer (an overlay of the preview used it),
 * the event stays cancelled.
 */
export function scrollOver(frame: HTMLElement) {
  frame.addEventListener("wheel", holdWheel, { capture: true, passive: true });
  frame.addEventListener("wheel", settleWheel, { passive: false });

  return () => {
    frame.removeEventListener("wheel", holdWheel, { capture: true });
    frame.removeEventListener("wheel", settleWheel);
  };
}

const nextFrame = () => new Promise((resolve) => requestAnimationFrame(resolve));

/**
 * Draws the first frame of a preview: iced misses the first redraw of a
 * window opened after the start of the event loop, until the window resizes.
 */
export async function wake(frame: HTMLElement) {
  // One frame after the other.
  // oxlint-disable no-await-in-loop
  for (let i = 0; i < 600; i++) {
    const canvas = frame.querySelector("canvas");
    if (canvas) {
      await nextFrame();
      canvas.style.height = "calc(100% - 1px)";
      await nextFrame();
      await nextFrame();
      canvas.style.height = "100%";
      return;
    }
    await nextFrame();
  }
}
