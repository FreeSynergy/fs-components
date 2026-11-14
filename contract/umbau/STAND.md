# State of the fixed values (measured, not generated)

- **Date:** 2026-11-14
- **Head measured:** `7a47bdc` plus the counter changes of this commit series, on main v0.0.17 (`55e0eda`); the counter reads `crates/` and `contract/`, so the numbers are those of the working tree.
- **Command:** `python3 -I contract/umbau/count` (table), `--group CRATE[/MODULE]` (JSON), `--write contract/umbau` (writes `START.md`, `start.json`, `ZUORDNUNG.md`); then `python3 -I contract/umbau/tokens --write contract/umbau` (`TOKENS.md`, `tokens.json`).
- **Checks:** `contract/umbau/selftest` and `contract/umbau/tokens_selftest` (probes move the counter as expected: both green), `tokens --check contract/umbau` (current).

Fixed values in production code per crate (colours + spacings + sizes + radius, font, border, shadow, opacity). Style definitions (`styles/*.rs`), drawn icons and values that set nothing (zero spacing, `opacity: 1`, `Color::TRANSPARENT`) are not counted.

| Group (crate) | Colours | Spacings | Sizes | Other | Fixed in total | Names not in contract |
|---|---|---|---|---|---|---|
| `fs-components-render` (group 1) | 0 | 0 | 0 | 0 | **0** | 0 |
| `fs-components-styles` | 0 | 0 | 0 | 0 | 0 | 42 |
| `fs-components-charts` | 0 | 0 | 0 | 0 | 0 | 0 |
| `fs-components-fields` | 0 | 0 | 0 | 1 | 1 | 5 |
| `fs-components-kanban` | 0 | 0 | 0 | 1 | 1 | 3 |
| `fs-components-timeline` | 0 | 0 | 1 | 0 | 1 | 10 |
| `fs-admin-frontend` | 0 | 0 | 0 | 3 | 3 | 11 |
| `fs-components-calendar-grid` | 0 | 0 | 0 | 3 | 3 | 7 |
| `fs-components-map` | 0 | 2 | 0 | 1 | 3 | 3 |
| `fs-components-primitives` | 0 | 0 | 0 | 3 | 3 | 3 |
| `fs-components-app` | 0 | 3 | 0 | 3 | 6 | 24 |
| `fs-components-workbenches` | 10 | 64 | 34 | 57 | 165 | 217 |
| `fs-components-widgets` | 8 | 83 | 25 | 89 | 205 | 331 |
| **all** | 18 | 152 | 60 | 162 | **392** | 625 (distinct) |

Since the branch was cut (c97eca0d, counted 393 places): the new code of v0.0.14-v0.0.17 (page tree, name dialog, Notion title, shell) added a few places (`name_dialog` 420 px, borders); the two drawn-icon files are no longer counted; `Color::TRANSPARENT` no longer counts (probe added to `selftest`). New tokens needed now: **18** (was 17; the 420 px width of the name dialog became `size.dialog-narrow`).

Limits: the counter reads source text; values built at run time or passed through a variable are not seen. The figures of ADR 0074 (205 colours, 474 names) are still not reproduced (see `START.md`).

The plan of the groups 2-16 (order, crate, number of values, effort) is in `PLAN.md`, section 2.
