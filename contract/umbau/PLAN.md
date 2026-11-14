# Plan: order of the groups for the conversion (ADR 0074)

Written by hand from `START.md` (state 2026-11-14, on main v0.0.17, see `STAND.md`) and `ZUORDNUNG.md` (numbers from `start.json`, command in `START.md`). Nothing here is converted yet. What is not proven is marked **[open]**.

## 1. What decides the order

- **Smallest and least risky first.** Size of a group = fixed values in blocks (colours, spacings, sizes, other) plus class and id names that are not contract names (`START.md`, column "Names not in contract").
- **A token that does not exist cannot be used.** `ZUORDNUNG.md` section 2 lists 105 distinct values (357 places) without a token. Before any group that needs one is converted, the owner decides the proposals, and the tokens are added to the style definitions and to the DTCG files (ADR 0074, "Folgen", step 2). Group 0 below does that.
- **Whole-program shells before programs is wrong here**: `fs-components-app` (window shell, menus, status) is small in numbers (8 fixed, 23 names) but every program draws it, so one mistake shows everywhere. It comes after the small crates, not first.
- **Values equal to a token keep the picture only in the style the literal was written for.** The same token has different values in the five styles (`ZUORDNUNG.md`, caution). Which style a block's literals belong to, and which style is "the default theme" for the pixel comparison of the workflow, is **[open]**: the code has `StyleId::Craft` and four more (`crates/fs-components-styles/src/style.rs`), and no document says which one the comparison uses.

## 2. Order

Numbers: C colours, Sp spacing, Sz size, O other fixed (radius, font, border, shadow, opacity), N names not in the contract.

| # | Group (crate / module) | C | Sp | Sz | O | N | Risk | Notes |
|---|---|---|---|---|---|---|---|---|
| 0 | token files and `fs-components-styles` | 0 | 0 | 0 | 0 | 42 | high (every block reads it) | The values here ARE the styles (`styles/craft.rs` ... excluded from the count; `craft_sheet.rs` is listed in `[fixed-values]`, so it counts 0). Work: write the DTCG files from the definitions, add the 18 new tokens (`TOKENS.md`). |
| 1 | `fs-components-render` | 0 | 0 | 0 | 0 | 0 | none | Nothing to convert. Used as the picture tool, see section 4. **Group 1 = 0 (measured 2026-11-14).** |
| 2 | `fs-components-charts` | 0 | 0 | 0 | 0 | 0 | none | No fixed value left in the count (the earlier `opacity 1.0` sets nothing). Colours of a drawing: content, **[open]**. |
| 3 | `fs-components-kanban` | 0 | 0 | 0 | 1 | 3 | low | `board`, `board-column`. |
| 4 | `fs-components-primitives` | 0 | 0 | 0 | 3 | 3 | low | `search_field`; `radius.control` instead of 6 (deliberate picture change). |
| 5 | `fs-components-map` | 0 | 2 | 0 | 1 | 3 | low | Tiles are drawn, not styled. |
| 6 | `fs-components-fields` | 0 | 0 | 0 | 1 | 5 | low | `form-field`, `textbox`. Caret depends on a class with the font (`DESIGN.md`): `font_class.rs` and the typing test cover it. |
| 7 | `fs-components-calendar-grid` | 0 | 0 | 0 | 3 | 7 | low | `calendar-grid`. |
| 8 | `fs-components-timeline` | 0 | 0 | 1 | 0 | 10 | low | `timeline`. |
| 9 | `fs-admin-frontend` | 0 | 0 | 0 | 3 | 11 | low to medium | A Frontend (admin, ADR 0066); only borders in `src/view`. |
| 10 | `fs-components-app` | 0 | 3 | 0 | 3 | 24 | medium to high | `app-window`, `menubar`, `menu`, `status`; all programs. Tests: `menu_picture.rs`, `picture.rs`, `own_bar.rs`. |
| 11 | `fs-components-widgets`, wave A: 20 modules (all except `craft`, `calendar_view`, `plan`) | 0 | 3 | 15 | 30 | 194 | low to medium | One module per run (numbers per module in `START.md`). Includes the new `name_dialog` (420 px, `size.dialog-narrow`) and `page_title`. 48 fixed values; the names are the bulk. |
| 12 | `fs-components-widgets`: `craft`, `calendar_view` | 0 | 14 | 5 | 11 | 59 | medium | `craft` (6 spacing, 2 size, 30 names), `calendar_view` (8 spacing, 3 size, 11 other, 29 names). The drawn file icons (`icon_kind`, `icon_export`) are in `[fixed-values]` and no longer counted. |
| 13 | `fs-components-widgets/plan` | 8 | 66 | 5 | 48 | 78 | high | The plan widgets (reference frontend): 127 fixed values, most of the values without a token. Split by file, `styles.rs` first. |
| 14 | `fs-components-workbenches`, wave A: `llm_chat`, `calendar`, `people`, `tasks` | 0 | 0 | 6 | 14 | 94 | medium | 20 fixed values; names are the bulk. |
| 15 | `fs-components-workbenches/file_manager` | 2 | 10 | 4 | 12 | 52 | medium | Colours `#e3e3e3` and `#f7f7f8` become role tokens (deliberate picture change). 28 fixed values. |
| 16 | `fs-components-workbenches/view` (wiki frontends) | 8 | 54 | 24 | 31 | 71 | high | 1:1 rebuilds of references. 117 fixed values. Whether the look survives as an own style is **[open]**. |

Effort (fixed values + names, state 2026-11-14, see `STAND.md`): groups 1-10 are small (each at most 6 fixed values and 24 names, together 21 fixed values and 66 names); group 11 is the first large one (48 + 194); group 13 and 16 are the two big ones (127 and 117 fixed values).

Totals to reach (2026-11-14, after v0.0.17): 18 colours, 152 spacings, 60 sizes, 162 other fixed values in blocks (392 together); 625 names not in the contract, 46 of them `remove` (`START.md`).

## 3. Per group: what to do, what is checked

For every group (the same as the workflow `frontend-umbau`):

1. `contract/umbau/count --group <crate>[/<module>]` gives the numbers of the group as JSON; `contract/umbau/count --write DIR` writes the whole state. Recount must report 0 colours, 0 spacings, 0 sizes for the group (the workflow target: "0 fixed colors and spacings"; sizes and the other kinds are counted too and the target for them is **[open]**: radius, border, font and opacity have no token yet).
2. The names: `contract/check` must stay green. Renaming a class means changing the mapping (`contract/mapping/*.toml`) and the code together; the check lists names used in source but not in the mapping and mapping entries no source uses.
3. Tests of the crate: `test --workspace --all-features --locked` through `build-slot.sh`. Many tests assert colours or class names (for example the "reddish pixels" probe of `crates/fs-components-widgets/tests/transfer_list.rs:88`); they change with the code and are part of the group's work.
4. Pictures before and after (section 4).

Images needed per group (existing drawing tests that already produce a picture from the real view; **[open]** whether they cover every block of the group):

| Group | Existing drawing tests (files in `tests/`) |
|---|---|
| render | `fs-components-render/tests/picture.rs` (the tool itself) |
| charts | `draw_edge.rs` (the `golden/*.json` files are chart data, not pictures) |
| kanban | `view_moves.rs`, `view_withdrawn.rs`, `view_locked.rs` |
| primitives | none of its own; the search field is drawn in `fs-components-workbenches/tests/search_field_size.rs` |
| map | `ui.rs` (`common/mod.rs` keeps pictures when `FS_COMP_SHOTS` is set) |
| fields | `fields.rs`, `rebuild.rs`, `blinc_limits.rs`; example `form` |
| calendar-grid | `view.rs`, `view_keys.rs` |
| timeline | `view.rs`, `view_labels.rs`, `view_scroll.rs`, `view_today.rs` |
| admin-frontend | `view.rs`, `eighth_view.rs` (title and buttons; keeps a picture when `FS_COMP_SHOTS` is set) |
| app | `menu_picture.rs`, `picture.rs`, `own_bar.rs`; example `shell -- /tmp/shell.png` |
| widgets | 19 test files draw (`agent_tree`, `lightbox`, `message_list`, `message_composer`, `share_dialog`, `tool_call`, `transfer_list`, `sortable_list`, `craft`, `avatar_group`, `calendar_view_ui_more` ...); `tests/common/mod.rs` saves every drawn picture under its `name` when `FS_COMP_SHOTS` is set. No drawing test for `plan`, `badge`, `breadcrumb`, `file_tile`, `file_preview`, `text_view` by name was found by file name [open: check by content]. |
| workbenches | 9 test files draw (`pixels.rs`, `pixels_more.rs`, `card_picture.rs`, `focus_picture.rs`, `snapshot.rs`, `people_narrow.rs` ...); only `people_narrow.rs` saves its picture under `FS_COMP_SHOTS`. Examples `reader -- --snapshot out.png`, `llm_chat`. |

Missing views (a block with no drawing test) need a new test or example that draws it; that is **new code outside this analysis** and is a first step of the group.

## 4. How "same pictures" can be checked

What the repository has today (read in the code, not run):

- **Offscreen drawing of the real view, no window.** `fs_components_render::draw_to_png(canvas, build, after_layout, path)` (`crates/fs-components-render`, `DESIGN.md` lines on `fs-components-render`); `AppWidget::picture`; `snapshot` of the workbenches. Tests skip with a message when there is no graphics adapter (`menu_picture.rs`, `render/tests/picture.rs`).
- **Pictures kept by an environment variable.** `FS_COMP_SHOTS=<dir>` makes `tests/common/mod.rs` (widgets, map), `menu_picture.rs`, `people_narrow.rs` and `eighth_view.rs` save the PNG of each drawn test under the test's picture name. That is the hook for "before" and "after" runs: run the tests once with `FS_COMP_SHOTS=before/`, convert, run again with `FS_COMP_SHOTS=after/`.
- **Pixel assertions inside tests**: `ink(img, rect, background)` and `background(img)` in `tests/common/mod.rs`, colour probes, per test. They prove "something is drawn here", not "the whole picture is the same".
- **Real window, real keys**: `tools/live-probe` (nested `kwin_wayland --virtual`, `spectacle` pictures, never the user's session) and the typing test of `fs-release` (`scripts/typing-test*`). Heavy; for the final look at a Frontend, not for each group.

What the repository does **not** have:

- **A pixel-by-pixel comparison of two pictures.** `workflows-pflege-2/BERICHT.md` says it too: `pixel-compare` and `count-fixed-values` are catalogue proposals, "im Code gibt es sie noch nicht". The counter now exists (`contract/umbau/count`); the comparer does not. A small script or Rust test that reads two directories of PNGs and reports the number of differing pixels per file is needed (Rust: the `image` crate is already a dependency of the render crate; Python without dependencies would have to decode PNG with `zlib` itself). **[open]** which of the two, and how it is wired into the workflow step `same-pictures`.
- **A stored reference set.** There is no `golden` directory of pictures (the charts `golden/` are JSON).

How the comparison itself has to be proven (same rule as the counter's counter-test):

1. Draw twice with unchanged code: the comparer must report 0 differing pixels. If it does not (fonts, animation frames: Blinc fades pictures in over the first frames, `DESIGN.md`; the picture helpers wait 3 frames), the comparison is useless until that is fixed. **[open]: not measured.**
2. Change one colour on purpose: the comparer must report more than 0.
3. Fonts and adapter: the tests say they give the same verdict with and without system fonts, but pixel tests of text only run when `has_fonts()` is true (`DESIGN.md`). Before and after must run on the same machine, adapter and fonts. **[open]**.

What a clean comparison can and cannot say: equal pictures prove the token has the same value as the literal in the style used for the drawing. They say nothing about the other four styles (where tokens differ and pictures are supposed to differ). Pictures per style are therefore a second set **[open]**: ADR 0074 section 3 wants five or six styles, and pictures of "Vorbild" frontends are never published (section 4).

## 5. Open questions (collected)

1. Which style is "the default theme" of the pixel comparison, per block? (Section 1.)
2. Values without a token: which of the proposals in `ZUORDNUNG.md` section 2 does the owner accept? (Exact values keep the picture; rounding to the existing steps changes it.)
3. Are the colours of drawn icons (`icon_kind`, `icon_export`) and charts content (stay) or style (token)?
4. Does `styles/craft_sheet.rs` count as a style definition (no violation) or as a block (convert)?
5. The wiki frontends in `workbenches/view` and the plan widgets: converted to tokens of the existing styles, or kept as own styles?
6. The comparer: Rust test or script, and where the before/after pictures live.
7. The figures of ADR 0074 (205 colours, 474 names) are not reproduced by the counter (`START.md`); which rule produced them?
8. The target of the recount for radius, border, font and opacity (the workflow names only colours and spacings).
