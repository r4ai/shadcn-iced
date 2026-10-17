// The theme of the live previews: a base color, a theme (accent) and a radius,
// the options of shadcn/ui's `/create` page that shadcn-iced supports.
import { useTheme } from "~/components/theme-provider";
import { Select } from "~/components/ui/select";
import { ACCENTS, BASES, capitalize, RADII } from "~/lib/theme";
import { cn } from "~/lib/utils";

const options = (names: readonly string[]) => names.map((name) => ({ value: name, label: capitalize(name) }));

const BASE_OPTIONS = options(BASES);
// The base color as the theme is `""` in the theme options, which Kobalte
// takes for no selection.
const SAME_AS_BASE = "base";
const ACCENT_OPTIONS = [{ value: SAME_AS_BASE, label: "Same as base" }, ...options(ACCENTS)];
const RADIUS_OPTIONS = options(RADII);

export function PreviewThemeFields(props: { size?: "sm" | "default"; class?: string }) {
  const { theme, setTheme } = useTheme();

  return (
    <div class={cn("grid gap-3", props.class)}>
      <Select
        label="Base color"
        size={props.size}
        options={BASE_OPTIONS}
        value={theme.base}
        onChange={(base) => setTheme({ base })}
      />
      <Select
        label="Theme"
        size={props.size}
        options={ACCENT_OPTIONS}
        value={theme.accent || SAME_AS_BASE}
        onChange={(accent) => setTheme({ accent: accent === SAME_AS_BASE ? "" : accent })}
      />
      <Select
        label="Radius"
        size={props.size}
        options={RADIUS_OPTIONS}
        value={theme.radius}
        onChange={(radius) => setTheme({ radius })}
      />
    </div>
  );
}
