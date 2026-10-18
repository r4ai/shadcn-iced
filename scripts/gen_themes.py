#!/usr/bin/env python3
"""Generate `crates/shadcn_iced_core/src/theme/presets.rs` from shadcn/ui's
`apps/v4/registry/themes.ts`.

Usage: uv run --project scripts scripts/gen_themes.py <path-to-shadcn-ui-checkout>
"""
import math
import re
import sys
from pathlib import Path

KEYS = [
    "background", "foreground", "card", "card-foreground", "popover",
    "popover-foreground", "primary", "primary-foreground", "secondary",
    "secondary-foreground", "muted", "muted-foreground", "accent",
    "accent-foreground", "destructive", "border", "input", "ring",
    "chart-1", "chart-2", "chart-3", "chart-4", "chart-5", "sidebar",
    "sidebar-foreground", "sidebar-primary", "sidebar-primary-foreground",
    "sidebar-accent", "sidebar-accent-foreground", "sidebar-border",
    "sidebar-ring",
]
BASES = ["neutral", "stone", "zinc", "mauve", "olive", "mist", "taupe"]


def oklch_to_rgba(s):
    m = re.match(r"oklch\(([\d.]+)\s+([\d.]+)\s+([\d.]+)(?:\s*/\s*([\d.]+)%)?\)", s)
    if not m:
        raise ValueError(s)
    L, C, H = float(m[1]), float(m[2]), float(m[3])
    alpha = float(m[4]) / 100 if m[4] else 1.0
    a = C * math.cos(math.radians(H))
    b = C * math.sin(math.radians(H))
    l_ = L + 0.3963377774 * a + 0.2158037573 * b
    m_ = L - 0.1055613458 * a - 0.0638541728 * b
    s_ = L - 0.0894841775 * a - 1.2914855480 * b
    l, mm, ss = l_ ** 3, m_ ** 3, s_ ** 3
    r = 4.0767416621 * l - 3.3077115913 * mm + 0.2309699292 * ss
    g = -1.2684380046 * l + 2.6097574011 * mm - 0.3413193965 * ss
    bb = -0.0041960863 * l - 0.7034186147 * mm + 1.7076147010 * ss

    def enc(x):
        x = max(0.0, min(1.0, x))
        v = 12.92 * x if x <= 0.0031308 else 1.055 * x ** (1 / 2.4) - 0.055
        return round(max(0.0, min(1.0, v)) * 255)

    return enc(r), enc(g), enc(bb), alpha


def rust_color(s):
    r, g, b, a = oklch_to_rgba(s)
    if a >= 1.0:
        return f"rgb(0x{r:02x}, 0x{g:02x}, 0x{b:02x})"
    return f"rgba(0x{r:02x}, 0x{g:02x}, 0x{b:02x}, {a:.2f})"


def parse(text):
    themes = {}
    for block in re.split(r"\n  \{\n", text)[1:]:
        name = re.search(r'name: "([\w-]+)"', block)[1]
        modes = {}
        for mode in ("light", "dark"):
            mm = re.search(mode + r": \{(.*?)\n      \}", block, re.S)
            vars_ = {}
            for k, v in re.findall(r'"?([\w-]+)"?: "(oklch\([^"]+\))"', mm[1]):
                vars_[k] = v
            modes[mode] = vars_
        themes[name] = modes
    return themes


def ident(key):
    return key.replace("-", "_")


def main():
    src = Path(sys.argv[1]) / "apps/v4/registry/themes.ts"
    themes = parse(src.read_text())
    out = [
        "//! Color presets generated from shadcn/ui's `registry/themes.ts`.",
        "//!",
        "//! Do not edit by hand: run `uv run --project scripts scripts/gen_themes.py <shadcn-ui>`.",
        "#![allow(clippy::unreadable_literal)]",
        "",
        "use super::{Palette, PaletteOverrides, rgb, rgba};",
        "",
    ]
    for name in BASES:
        for mode in ("light", "dark"):
            vars_ = themes[name][mode]
            const = f"{name.upper()}_{mode.upper()}"
            out.append(f"/// The `{name}` base color ({mode}).")
            out.append(f"pub const {const}: Palette = Palette {{")
            for k in KEYS:
                if k.startswith("chart-"):
                    continue
                out.append(f"    {ident(k)}: {rust_color(vars_[k])},")
            charts = ", ".join(rust_color(vars_[f"chart-{i}"]) for i in range(1, 6))
            out.append(f"    chart: [{charts}],")
            out.append("};")
            out.append("")
    accents = [n for n in themes if n not in BASES]
    for name in accents:
        for mode in ("light", "dark"):
            vars_ = themes[name][mode]
            const = f"{name.upper()}_{mode.upper()}"
            out.append(f"/// The `{name}` accent color overrides ({mode}).")
            out.append(f"pub const {const}: PaletteOverrides = PaletteOverrides {{")
            for k in KEYS:
                if k.startswith("chart-"):
                    continue
                v = f"Some({rust_color(vars_[k])})" if k in vars_ else "None"
                out.append(f"    {ident(k)}: {v},")
            if all(f"chart-{i}" in vars_ for i in range(1, 6)):
                charts = ", ".join(rust_color(vars_[f"chart-{i}"]) for i in range(1, 6))
                out.append(f"    chart: Some([{charts}]),")
            else:
                out.append("    chart: None,")
            out.append("};")
            out.append("")
    out.append("/// Every base color as `(name, light, dark)`.")
    out.append("pub const BASE_COLORS: &[(&str, Palette, Palette)] = &[")
    for name in BASES:
        out.append(f'    ("{name}", {name.upper()}_LIGHT, {name.upper()}_DARK),')
    out.append("];")
    out.append("")
    out.append("/// Every accent color as `(name, light, dark)`.")
    out.append("pub const ACCENT_COLORS: &[(&str, PaletteOverrides, PaletteOverrides)] = &[")
    for name in accents:
        out.append(f'    ("{name}", {name.upper()}_LIGHT, {name.upper()}_DARK),')
    out.append("];")
    dest = Path(__file__).resolve().parent.parent / "crates/shadcn_iced_core/src/theme/presets.rs"
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text("\n".join(out) + "\n")
    print(f"wrote {dest} ({len(BASES)} bases, {len(accents)} accents)")


if __name__ == "__main__":
    main()
