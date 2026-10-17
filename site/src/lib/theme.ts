// The theme options of the previews: the options of shadcn/ui's `/create`
// page that shadcn-iced supports, with the names `shadcn_iced::theme` uses.

export const BASES = ["neutral", "stone", "zinc", "mauve", "olive", "mist", "taupe"] as const;

export const ACCENTS = [
  "amber",
  "blue",
  "cyan",
  "emerald",
  "fuchsia",
  "green",
  "indigo",
  "lime",
  "orange",
  "pink",
  "purple",
  "red",
  "rose",
  "sky",
  "teal",
  "violet",
  "yellow",
] as const;

export const RADII = ["default", "none", "small", "medium", "large"] as const;

export interface ThemeOptions {
  base: string;
  /** An accent, or `""` for the base color. */
  accent: string;
  radius: string;
}

export const DEFAULT_THEME: ThemeOptions = { base: "neutral", accent: "", radius: "default" };

export const capitalize = (name: string) => name.charAt(0).toUpperCase() + name.slice(1);
