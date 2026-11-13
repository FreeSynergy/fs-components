"""Second half of TOKENS.md: the exceptions proposal and the open points. Pattern: Template method (static text over the data)."""
from __future__ import annotations


def build(data):
    nc = data["new_count"]["scope"]
    return ["## 6. Proposal for `contract/exceptions.toml` (NOT applied; a diff)", "",
            "The file has two tables today: `[not-a-class]` and `[unmapped]`. Proposed new table (what the counter and the check read must learn it first: **[offen]**, `contractlib/model.py` ignores unknown tables, `hard.py` has its own skip list):", "",
            "```diff", "+# Fixed values that are not violations (ADR 0074, section 2: the content belongs to the content; strategy 3 and 4).",
            "+[fixed-values]",
            '+"crates/fs-components-widgets/src/icon_kind.rs" = "drawn icon of a kind of thing: its colours (7), line (2) and shadow (2) are the picture, content, not style"',
            '+"crates/fs-components-widgets/src/icon_export.rs" = "drawn icon: its colour (1) is the picture, content"',
            '+"crates/fs-components-styles/src/styles/craft_sheet.rs" = "style definition of Craft (strategy 4): the sheet IS the style; its 54 values move onto tokens only if the owner wants the sheet token based"',
            "+",
            "+# Colours chosen by the person in the data of a drawing are content. The standard series colours are style:",
            "+# charts have 0 fixed colours today (charts/src/view/look.rs ACCENTS reads 7 theme colour tokens), so no row is needed;",
            "+# proposal [offen]: name the 7 tokens `color.accent.tile.0..6` also `chart.series.0..6` (alias) so that a style can set them apart.",
            '+[data-colours]',
            '+"chart series colour set by the user in the data" = "content: lives in the chart data, never in a block or a style"',
            "```", "",
            "## 7. Open points", "",
            "1. **Fit rule guard (rule 4)** is my addition; the strategy says only 2 px. Without it new tokens would be fewer but hairlines would change by up to 100 percent.",
            "2. **Tie rule** (smaller step) is a choice; it decides `space.xs` for 6 px (2 px off) and `space.s` for 10 px (last wave).",
            "3. **Which style were the literals written for?** Everything is compared with Craft (strategy 1). If a block was designed against another style, its deviation to Craft is not its deviation to that style **[offen]**.",
            f"4. **Radius vs Craft.** Craft is square (0 on all three roles); the {nc['radius']} new step `radius.small` (5) gives Craft a rounded corner in `badge` and `search_field` (2 places in scope; 23 in all). Alternative: use the three role tokens (0 new tokens; deviation of 5 to 6 px in Craft: a deliberate look change). The owner decides.",
            "5. **`border.width.strong` is 3 px** (covers 2, 3, 4 px): 8 places of 2 px and 1 of 4 px move by 1 px. Alternative: strong = 2 and 4 px stays one more token.",
            "6. **Colours:** no tolerance was given, so the two file manager colours are NEW (see section 3).",
            "7. **Zero values** (66 places: `size 0` 64, `border 0` 2) and `opacity 1.0` (1) are counted by `count` but need no token; the counter should skip them, or the recount target is not 0.",
            "8. **Names** of NEW tokens are proposals from the place of use; their values in the other four styles are unknown. DTCG types: px as `dimension`, opacity as `number`.",
            "9. **Charts:** `chart.series.N` is not needed as a new token (see section 6); whether an alias helps a style is [offen].",
            "10. **Sizes** that are single layout measures (560, 120, 96, 72, 64, 60, 56) are the weakest tokens: kept as tokens only because the strategy forbids a literal.",
            "11. The 54 places of `craft_sheet.rs` (6, 10, 14, 39, 22, 31, 33, 9 px ...) do not lie on the scale either; they stay as the Craft definition. If the sheet is to use tokens, the same fit applies (rows 'style definition' in section 4).",
            "12. Line numbers of the sources are those of the commit this file was generated from; `tokens --check contract/umbau` finds a stale file.",
            ""]
