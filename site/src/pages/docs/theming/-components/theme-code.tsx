// The Rust code that builds the theme picked in the customizer, from the
// previews (`themeCode`), so it is always the code shadcn-iced runs.
import { createSignal, onMount } from "solid-js";

import { CopyButton } from "~/components/code-block";
import { useTheme } from "~/components/theme-provider";
import { loadPreviews, previews } from "~/lib/previews";
import { cn } from "~/lib/utils";

export function ThemeCode() {
  const { dark, theme } = useTheme();
  const [failed, setFailed] = createSignal(false);

  onMount(() => loadPreviews().catch(() => setFailed(true)));

  const code = () => previews()?.themeCode(dark(), theme.base, theme.accent, theme.radius);

  return (
    <figure data-rehype-pretty-code-figure class="m-0! md:mx-0!">
      <CopyButton value={code() ?? ""} />
      <pre
        class={cn(
          "no-scrollbar min-h-28 overflow-x-auto px-4 py-3.5 font-mono text-sm leading-relaxed",
          !code() && "text-muted-foreground",
        )}
      >
        {code() ?? (failed() ? "The theme code needs WebAssembly." : "Loading…")}
      </pre>
    </figure>
  );
}
