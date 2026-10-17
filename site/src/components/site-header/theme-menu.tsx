// The theme of the live previews, where ui.shadcn.com has its "New" button.
import { Palette } from "lucide-solid";

import { PreviewThemeFields } from "~/components/preview-theme-fields";
import { useTheme } from "~/components/theme-provider";
import { Button } from "~/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverDescription,
  PopoverHeader,
  PopoverTitle,
  PopoverTrigger,
} from "~/components/ui/popover";
import { DEFAULT_THEME } from "~/lib/theme";

// The "New" button it replaces is a radix-nova button (style-nova.css), size
// `sm`, with `h-[31px] rounded-lg`.
const NOVA_BUTTON_SM =
  "h-[31px] gap-1 rounded-lg border border-transparent bg-clip-padding px-2.5 text-[0.8rem] hover:bg-primary/80 focus-visible:ring-3 active:not-aria-[haspopup]:translate-y-px has-[>svg]:px-2.5 [&_svg:not([class*='size-'])]:size-3.5";

export function ThemeMenu() {
  const { setTheme } = useTheme();

  return (
    <Popover placement="bottom-end">
      <PopoverTrigger as={Button} size="sm" class={NOVA_BUTTON_SM}>
        <Palette />
        Theme
      </PopoverTrigger>
      <PopoverContent class="w-64 gap-4 p-4">
        <PopoverHeader>
          <PopoverTitle>Preview theme</PopoverTitle>
          <PopoverDescription>Applies to every live preview.</PopoverDescription>
        </PopoverHeader>
        <PreviewThemeFields size="sm" />
        <Button variant="outline" size="sm" onClick={() => setTheme(DEFAULT_THEME)}>
          Reset
        </Button>
      </PopoverContent>
    </Popover>
  );
}
