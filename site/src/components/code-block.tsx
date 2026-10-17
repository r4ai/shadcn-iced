// The code blocks of ui.shadcn.com: the figure of rehype-pretty-code with a
// copy button (mdx-components.tsx, components/copy-button.tsx), the terminal
// of components/code-block-command.tsx for shell commands, and the
// expandable block of components/code-collapsible-wrapper.tsx. The code is
// highlighted at build time (see `plugins/site-data.ts`).
import { Check, Copy, File, SquareTerminal } from "lucide-solid";
import { createSignal, onCleanup, Show } from "solid-js";

import { Button } from "~/components/ui/button";
import { Separator } from "~/components/ui/separator";
import { cn } from "~/lib/utils";

interface CodeBlockProps {
  snippet: Highlighted;
  /** A file name or label shown above the code. */
  title?: string;
  /** Collapses the code to a few lines, with a button to expand it. */
  expandable?: boolean;
  class?: string;
}

export function CodeBlock(props: CodeBlockProps) {
  return (
    <Show
      when={props.expandable}
      fallback={
        <Show when={props.snippet.lang === "shellscript" && !props.title} fallback={<CodeFigure {...props} />}>
          <CommandFigure snippet={props.snippet} class={props.class} />
        </Show>
      }
    >
      <CollapsibleCode {...props} />
    </Show>
  );
}

function CodeFigure(props: CodeBlockProps) {
  return (
    <figure data-rehype-pretty-code-figure class={props.class}>
      <Show when={props.title}>
        <figcaption
          data-rehype-pretty-code-title
          data-language={props.snippet.lang}
          class="flex items-center gap-2 text-code-foreground [&_svg]:size-4 [&_svg]:text-code-foreground [&_svg]:opacity-70"
        >
          {/* `getIconForLanguageExtension` gives this icon for Rust and TOML. */}
          <File />
          {props.title}
        </figcaption>
      </Show>
      <CopyButton value={props.snippet.code} />
      <div class="contents" innerHTML={props.snippet.html} />
    </figure>
  );
}

/** A shell command, in the terminal of `CodeBlockCommand`. */
function CommandFigure(props: { snippet: Highlighted; class?: string }) {
  const tool = () => props.snippet.code.split(/\s/, 1)[0] || "shell";

  return (
    <figure data-rehype-pretty-code-figure class={props.class}>
      <div class="flex items-center gap-2 border-b border-border/50 px-3 py-1">
        <div class="flex size-4 items-center justify-center rounded-[1px] bg-foreground opacity-70">
          <SquareTerminal class="size-3 text-code" />
        </div>
        <span class="inline-flex h-7 items-center rounded-md border border-input bg-background px-2 pt-0.5 text-sm font-medium text-foreground">
          {tool()}
        </span>
      </div>
      <CopyButton value={props.snippet.code} class="top-2 opacity-70 hover:opacity-100 focus-visible:opacity-100" />
      <div class="no-scrollbar overflow-x-auto [&_pre]:leading-none" innerHTML={props.snippet.html} />
    </figure>
  );
}

function CollapsibleCode(props: CodeBlockProps) {
  const [open, setOpen] = createSignal(false);
  const label = () => (open() ? "Collapse" : "Expand");

  return (
    <div data-state={open() ? "open" : "closed"} class={cn("group/collapsible relative md:-mx-1", props.class)}>
      <div class="absolute top-1.5 right-9 z-10 flex items-center">
        <Button
          variant="ghost"
          size="sm"
          class="h-7 rounded-md px-2 text-muted-foreground"
          aria-expanded={open()}
          onClick={() => setOpen(!open())}
        >
          {label()}
        </Button>
        <Separator orientation="vertical" class="mx-1.5 h-4!" />
      </div>
      <div
        data-slot="code-collapsible-content"
        class="relative mt-6 overflow-hidden group-data-[state=closed]/collapsible:max-h-64 [&>figure]:mt-0 [&>figure]:md:mx-0!"
      >
        <CodeFigure snippet={props.snippet} title={props.title} />
      </div>
      <button
        type="button"
        class="absolute inset-x-0 -bottom-2 flex h-20 items-center justify-center rounded-b-lg bg-gradient-to-b from-code/70 to-code text-sm text-muted-foreground group-data-[state=open]/collapsible:hidden"
        onClick={() => setOpen(true)}
      >
        {label()}
      </button>
    </div>
  );
}

/**
 * Copies `value`, like `copyToClipboardWithMeta` of components/copy-button.tsx:
 * with the Clipboard API, or a selected textarea where it is missing (outside
 * of secure contexts) or refused.
 */
async function copyToClipboard(value: string) {
  if (!value) return false;
  try {
    await navigator.clipboard.writeText(value);
    return true;
  } catch {
    return legacyCopyToClipboard(value);
  }
}

function legacyCopyToClipboard(value: string) {
  const textArea = document.createElement("textarea");
  textArea.value = value;
  textArea.setAttribute("readonly", "");
  Object.assign(textArea.style, { position: "fixed", opacity: "0", pointerEvents: "none" });
  document.body.append(textArea);
  textArea.focus();
  textArea.select();
  textArea.setSelectionRange(0, value.length);

  let copied = false;
  try {
    // oxlint-disable-next-line typescript/no-deprecated -- the fallback of the Clipboard API
    copied = document.execCommand("copy");
  } catch {
    copied = false;
  }
  textArea.remove();
  return copied;
}

/** Copies `value`, and shows a check for a moment. */
export function CopyButton(props: { value: string; class?: string }) {
  const [copied, setCopied] = createSignal(false);
  let timer: ReturnType<typeof setTimeout> | undefined;
  onCleanup(() => clearTimeout(timer));

  const copy = async () => {
    if (!(await copyToClipboard(props.value))) return;
    setCopied(true);
    clearTimeout(timer);
    timer = setTimeout(() => setCopied(false), 2000);
  };

  return (
    <Button
      data-slot="copy-button"
      data-copied={copied()}
      size="icon"
      variant="ghost"
      class={cn("absolute top-3 right-2 z-10 size-7 bg-code hover:opacity-100 focus-visible:opacity-100", props.class)}
      onClick={copy}
    >
      <span class="sr-only">Copy</span>
      <Show when={copied()} fallback={<Copy />}>
        <Check />
      </Show>
    </Button>
  );
}
