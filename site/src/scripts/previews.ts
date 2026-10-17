// Mounts the live previews (`[data-preview]`) of a page: the examples of the
// gallery, compiled to WebAssembly (`examples/gallery-web`), each drawn by
// iced into a canvas of its own.
import { currentTheme, isDark, onThemeChange } from "./theme";

interface Previews {
  mount(target: string, page: string, example: number): boolean;
  setTheme(dark: boolean, base: string, accent: string, radius: string): void;
  themeCode(dark: boolean, base: string, accent: string, radius: string): string;
}

declare global {
  interface Window {
    shadcnIcedPreviewResized?: (target: string, height: number) => void;
    shadcnIcedPreviewScroll?: (dx: number, dy: number) => void;
  }
}

let previews: Promise<Previews> | null = null;

/** Loads the previews (once). */
export function load(): Promise<Previews> {
  previews ??= (async () => {
    const base = import.meta.env.BASE_URL.replace(/\/$/, "");
    const url = new URL(`${base}/wasm/gallery_web.js`, location.origin).href;
    const module = await import(/* @vite-ignore */ url);
    await module.default();

    const api = module as Previews;
    const apply = () => {
      const theme = currentTheme();
      api.setTheme(isDark(), theme.base, theme.accent, theme.radius);
    };
    apply();
    onThemeChange(apply);

    return api;
  })();

  return previews;
}

const frames = new Map<string, HTMLElement>();

window.shadcnIcedPreviewResized = (target, height) => {
  const frame = frames.get(target);
  if (!frame) return;

  const min = Number(frame.dataset.minHeight ?? 0);
  frame.style.height = `${Math.max(min, Math.ceil(height))}px`;
};

window.shadcnIcedPreviewScroll = (dx, dy) => window.scrollBy(dx, dy);

const nextFrame = () => new Promise((resolve) => requestAnimationFrame(resolve));

/**
 * Draws the first frame of a preview: iced misses the first redraw of a
 * window opened after the start of the event loop, until the window resizes.
 */
async function wake(frame: HTMLElement) {
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

function fail(frame: HTMLElement, error: unknown) {
  console.error(error);
  frame.dataset.state = "error";
}

function start() {
  const pending = [...document.querySelectorAll<HTMLElement>("[data-preview]")];
  if (pending.length === 0) return;

  // Start downloading right away; mount each preview as it comes near.
  const ready = load();

  const observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;

        const frame = entry.target as HTMLElement;
        observer.unobserve(frame);

        const target = frame.dataset.target!;
        frames.set(target, frame);

        ready
          .then((api) => {
            if (!api.mount(target, frame.dataset.page!, Number(frame.dataset.example))) {
              throw new Error(`no page ${frame.dataset.page}`);
            }
            frame.dataset.state = "mounted";
            return wake(frame);
          })
          .catch((error) => fail(frame, error));
      }
    },
    { rootMargin: "400px 0px" },
  );

  for (const frame of pending) {
    // winit cancels touch scrolling on its canvas; scrolling the page over a
    // preview matters more on a phone.
    frame.addEventListener("touchstart", (event) => event.stopPropagation(), {
      capture: true,
      passive: true,
    });
    observer.observe(frame);
  }
}

start();
