#!/usr/bin/env python3
"""Generate the bundled symbol and emoji fonts from google/fonts.

- `crates/shadcn_iced_core/assets/fonts/ShadcnIcedSymbols-Regular.ttf`
  ("Shadcn Iced Symbols"): the keyboard symbols (⌘ ⌥ ⌃ ⏎ ⌫ ...) and arrows
  that Geist lacks, from the Arrows, Miscellaneous Technical, Control Pictures
  and Supplemental Arrows blocks. Noto Sans Symbols 2 has ⌘ ⌥ ⏎ but not ⌃ or
  ⇥: its subset is merged with the glyphs that it lacks from Noto Sans Symbols,
  then Noto Sans Math, so that a single font with a single set of metrics has
  them all.
- `conformance/fonts/NotoEmoji-Regular.ttf`: monochrome emoji, so that the
  conformance snapshots do not depend on the emoji font of the system.

Variable fonts are instantiated at the regular weight: cosmic-text renders
the default instance of a variable font only.

Usage: uv run --project scripts scripts/gen_fonts.py
"""
import io
import sys
import tempfile
import urllib.request
from pathlib import Path

from fontTools import subset
from fontTools.merge import Merger
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer

# google/fonts, pinned so that the fonts can be regenerated identically.
COMMIT = "47d54ca1ebb14fd864695f1d39dc734f39017c33"
BASE = f"https://raw.githubusercontent.com/google/fonts/{COMMIT}/ofl"

ROOT = Path(__file__).resolve().parent.parent
CORE = ROOT / "crates/shadcn_iced_core/assets/fonts"
CONFORMANCE = ROOT / "conformance/fonts"

SYMBOLS = [
    (0x2190, 0x21FF),  # Arrows
    (0x2300, 0x23FF),  # Miscellaneous Technical
    (0x2400, 0x243F),  # Control Pictures (␣)
    (0x27F0, 0x27FF),  # Supplemental Arrows-A
    (0x2900, 0x297F),  # Supplemental Arrows-B
]


def fetch(path: str) -> TTFont:
    with urllib.request.urlopen(f"{BASE}/{path}") as response:
        return TTFont(io.BytesIO(response.read()))


def regular(font: TTFont) -> TTFont:
    if "fvar" in font:
        font = instancer.instantiateVariableFont(font, {"wght": 400})
    return font


def cut(font: TTFont, unicodes) -> TTFont:
    options = subset.Options()
    options.layout_features = ["*"]
    options.hinting = False
    options.name_IDs = ["*"]
    options.notdef_outline = True
    # Unused by iced and browsers, and not merged by fontTools.
    options.drop_tables += ["vhea", "vmtx", "STAT", "GDEF", "MATH"]
    subsetter = subset.Subsetter(options)
    subsetter.populate(unicodes=unicodes)
    subsetter.subset(font)
    return font


def save(font: TTFont, out: Path):
    font.save(out)
    print(f"{out.relative_to(ROOT)}: {out.stat().st_size} bytes", file=sys.stderr)


def rename(font: TTFont, family: str):
    postscript = family.replace(" ", "") + "-Regular"
    names = {
        1: family,
        2: "Regular",
        3: f"{postscript};{font['head'].fontRevision:.3f}",
        4: f"{family} Regular",
        6: postscript,
        16: family,
        17: "Regular",
    }
    name = font["name"]
    name.names = [record for record in name.names if record.nameID not in names]
    for id, value in names.items():
        name.setName(value, id, 3, 1, 0x409)


def main():
    symbols = [c for start, end in SYMBOLS for c in range(start, end + 1)]

    symbols_2 = cut(
        regular(fetch("notosanssymbols2/NotoSansSymbols2-Regular.ttf")), symbols
    )
    fonts = [symbols_2]
    for path in [
        "notosanssymbols/NotoSansSymbols%5Bwght%5D.ttf",
        "notosansmath/NotoSansMath-Regular.ttf",
    ]:
        covered = {c for font in fonts for c in font.getBestCmap()}
        fonts.append(
            cut(regular(fetch(path)), [c for c in symbols if c not in covered])
        )

    with tempfile.TemporaryDirectory() as directory:
        paths = []
        for index, font in enumerate(fonts):
            paths.append(Path(directory) / f"{index}.ttf")
            font.save(paths[-1])
        # The first font gives its metrics and names.
        merged = Merger().merge(paths)
    rename(merged, "Shadcn Iced Symbols")
    save(merged, CORE / "ShadcnIcedSymbols-Regular.ttf")

    emoji = regular(fetch("notoemoji/NotoEmoji%5Bwght%5D.ttf"))
    CONFORMANCE.mkdir(exist_ok=True)
    save(
        cut(emoji, emoji.getBestCmap().keys()),
        CONFORMANCE / "NotoEmoji-Regular.ttf",
    )


if __name__ == "__main__":
    main()
