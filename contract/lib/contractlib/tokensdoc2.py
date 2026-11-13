"""Second half of TOKENS.md: the exceptions (now applied) and the open points. Pattern: Template method (static text over the data)."""
from __future__ import annotations


def build(data):
    return ["## 6. `contract/exceptions.toml` (applied)", "",
            "New tables, read by `contractlib/hard.py` (`exemptions`), so `count`, `contract/hardcoded` and the token report agree. The reason starts with `style:` or `content:`.", "",
            "```toml", "[fixed-values]  # a whole file is no violation",
            '"crates/fs-components-widgets/src/icon_kind.rs" = "content: drawn icon ..."    # 7 colours, 2 lines, 2 shadows',
            '"crates/fs-components-widgets/src/icon_export.rs" = "content: drawn icon ..."  # 1 colour',
            '"crates/fs-components-styles/src/styles/craft_sheet.rs" = "style: ..."          # the sheet IS the style (54 values)',
            "[data-colours]  # the COLOURS of a file are content; empty today (charts read 7 theme tokens: no literal colour)", "```", "",
            "`chart.series.N` is not needed as a new token; a style may alias `color.accent.tile.0..6` [offen].", "",
            "## 7. Open points", "",
            "1. **Tie rule** (smaller step): decides `3 px -> 2 px` (`--space-0_5`) and `1.5 px -> 1 px`; larger would give `4` and `2`. **[offen]**",
            "2. **Which style were the literals written for?** Everything is compared with Craft; a block designed against another style deviates differently **[offen]**.",
            "3. **Wide single sizes** (560, 120, 96, 72, 64, 60, 56) are weak tokens; they could be properties of a contract block **[offen]**.",
            "4. **Names** of NEW tokens are proposals; their values in the four other styles are vorerst the Craft value **[offen per style]**. DTCG types: px `dimension`, opacity `number`.",
            "5. **Opacity unit** 0.05 is mine (the strategy gives px only) **[offen]**.",
            "6. The last wave (Wiki, plan) is placed against these scales only for information; its values move into styles.",
            "7. The 54 values of `craft_sheet.rs` stay as the Craft definition; to make the sheet token based the same fit applies (rows `style definition` in section 4).",
            "8. Source line numbers are those of the generating commit; `tokens --check contract/umbau` finds a stale file.", ""]
