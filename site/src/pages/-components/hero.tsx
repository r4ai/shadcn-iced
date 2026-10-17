// The header of the home page, after components/page-header.tsx and
// components/announcement.tsx of ui.shadcn.com.
import { Link } from "@tanstack/solid-router";
import { ArrowRight } from "lucide-solid";

import { badgeVariants } from "~/components/ui/badge";
import { components } from "~/lib/components";
import { cn } from "~/lib/utils";

// The buttons of the page use the radix-luma style (styles/radix-luma/ui/button
// with registry/styles/style-luma.css), not the new-york one of the docs.
const LUMA_BUTTON =
  "inline-flex h-9 shrink-0 items-center justify-center gap-1.5 rounded-4xl border border-transparent bg-clip-padding px-3 text-sm font-medium whitespace-nowrap transition-all outline-none select-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/30 active:not-aria-[haspopup]:translate-y-px disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4";
const LUMA_VARIANTS = {
  default: "bg-primary text-primary-foreground hover:bg-primary/80",
  secondary:
    "bg-secondary text-secondary-foreground hover:bg-[color-mix(in_oklch,var(--secondary),var(--foreground)_5%)]",
};

export function Hero() {
  return (
    <section class="border-grid">
      <div class="container-wrapper">
        <div class="container flex flex-col items-center gap-2 px-6 py-8 text-center md:py-16 md:pb-8 lg:py-20 lg:pb-12 xl:gap-4">
          <Link to="/docs/components" data-slot="badge" class={cn(badgeVariants({ variant: "secondary" }), "bg-muted")}>
            {components.length} components, the whole shadcn/ui registry <ArrowRight />
          </Link>
          <h1 class="leading-tighter max-w-4xl text-3xl font-semibold tracking-tight text-balance text-primary lg:leading-[1.1] lg:font-semibold xl:text-5xl xl:tracking-tighter">
            Beautifully designed components for iced
          </h1>
          <p class="max-w-4xl text-base text-balance text-foreground sm:text-lg">
            The components of shadcn/ui, ported to Rust. Use them as a library, or copy them into your project and own
            the code. Every preview below is iced running in your browser.
          </p>
          <div class="flex w-full items-center justify-center gap-2 pt-2 **:data-[slot=button]:shadow-none">
            <Link to="/docs/installation" data-slot="button" class={cn(LUMA_BUTTON, LUMA_VARIANTS.default, "h-[35px]")}>
              Get Started
            </Link>
            <Link to="/docs/components" data-slot="button" class={cn(LUMA_BUTTON, LUMA_VARIANTS.secondary)}>
              View Components
            </Link>
          </div>
        </div>
      </div>
    </section>
  );
}
