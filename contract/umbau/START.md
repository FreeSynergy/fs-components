# Starting state of the conversion (ADR 0074)

Generated. Do not edit. Reproduce with `python3 -I contract/umbau/count --write contract/umbau` (Python 3, no dependencies; reads `crates/` and `contract/`). Production code only: tests, `#[cfg(test)]` parts and the style definitions (`styles/craft.rs`, `windows.rs`, `apple.rs`, `admin.rs`, `simple.rs`, which ARE the tokens) are not counted as violations. One group = one crate (`crates/<name>`); the module table below splits it by the first folder or file under `src/`. Workflow input `group`: a crate name.

Contract version 0.1. `contract/hardcoded` finds hex, `Color::rgba(..)`/`Rgba::hex(..)` with literals, and `px` values in CSS strings and numbers in Rust calls (`.p(8.0)`, `.w(..)`). `umbau` adds CSS `rgb()/hsl()`, named CSS colours in colour properties and `Color::NAME` constants (none found), and leaves out the one place `contract/hardcoded` counts wrongly (`Live::new().gap(250)` in `message_list/handle.rs`: milliseconds, no spacing). So spacing is 173 here and 174 in `HARDCODED-VALUES.md`.

## Totals

| Kind | Places in blocks | Style definition values (tokens, not violations) |
|---|---|---|
| color | 18 | 164 |
| spacing | 154 | 25 |
| size | 60 | 25 |
| radius | 23 | 15 |
| font | 43 | 30 |
| border | 89 | 16 |
| shadow | 0 | 60 |
| opacity | 6 | 0 |

Names (class and id names of production code, distinct): **630** = exception (not a class) 14, mapped 303, provisional 267, remove 46. No old name is itself a contract name: every `mapped`, `provisional` and `remove` name has to change or go; `exception` names are no class (thread names, handle ids; `contract/exceptions.toml`). Names not in the contract: 616.

Colours in test code (not counted above), per crate: fs-components-fields 9, fs-components-render 1, fs-components-styles 13, fs-components-widgets 1 = 24.

Compared with the figures named in ADR 0074 (205 fixed colours, 474 class and id names): **not reproduced** [open]. Colours: 18 in blocks + 164 in the style definitions (tokens) + 24 in tests; no combination of these is 205 without a counting rule that is not written down. Names: the distinct names of production code are 630; 474 matches no status group.

## Per crate (the groups)

| Group (crate) | Colours | Spacing | Size | Other fixed (radius, font, border, shadow, opacity) | Names not in contract |
|---|---|---|---|---|---|
| `fs-components-render` | 0 | 0 | 0 | 0 | 0 |
| `fs-components-styles` | 0 | 0 | 0 | 0 | 42 |
| `fs-components-fields` | 0 | 0 | 0 | 1 | 5 |
| `fs-components-kanban` | 0 | 0 | 0 | 1 | 3 |
| `fs-components-timeline` | 0 | 0 | 1 | 0 | 10 |
| `fs-admin-frontend` | 0 | 0 | 0 | 3 | 11 |
| `fs-components-calendar-grid` | 0 | 0 | 0 | 3 | 7 |
| `fs-components-map` | 0 | 2 | 0 | 1 | 3 |
| `fs-components-primitives` | 0 | 0 | 0 | 3 | 2 |
| `fs-components-app` | 0 | 3 | 0 | 3 | 23 |
| `fs-components-workbenches` | 10 | 66 | 35 | 58 | 214 |
| `fs-components-widgets` | 8 | 83 | 24 | 88 | 327 |

A name used in two crates is counted in both rows, so the rows add up to more than the distinct total.

## Per module

| Group | Module | Colours | Spacing | Size | Other | Names not in contract |
|---|---|---|---|---|---|---|
| `fs-admin-frontend` | `view` | 0 | 0 | 0 | 3 | 11 |
| `fs-components-app` | `menu_overlay` | 0 | 0 | 0 | 0 | 0 |
| `fs-components-app` | `menu_view` | 0 | 2 | 0 | 0 | 7 |
| `fs-components-app` | `status_panel` | 0 | 1 | 0 | 0 | 5 |
| `fs-components-app` | `styles` | 0 | 0 | 0 | 3 | 15 |
| `fs-components-app` | `view` | 0 | 0 | 0 | 0 | 12 |
| `fs-components-calendar-grid` | `handle` | 0 | 0 | 0 | 0 | 1 |
| `fs-components-calendar-grid` | `view` | 0 | 0 | 0 | 3 | 6 |
| `fs-components-fields` | `view` | 0 | 0 | 0 | 1 | 5 |
| `fs-components-kanban` | `handle` | 0 | 0 | 0 | 0 | 1 |
| `fs-components-kanban` | `view` | 0 | 0 | 0 | 1 | 2 |
| `fs-components-map` | `(root)` | 0 | 2 | 0 | 1 | 3 |
| `fs-components-map` | `loader` | 0 | 0 | 0 | 0 | 0 |
| `fs-components-map` | `widget` | 0 | 0 | 0 | 0 | 3 |
| `fs-components-primitives` | `search_field` | 0 | 0 | 0 | 3 | 2 |
| `fs-components-render` | `offscreen` | 0 | 0 | 0 | 0 | 0 |
| `fs-components-render` | `setup` | 0 | 0 | 0 | 0 | 0 |
| `fs-components-styles` | `styles` | 0 | 0 | 0 | 0 | 42 |
| `fs-components-timeline` | `handle` | 0 | 0 | 0 | 0 | 1 |
| `fs-components-timeline` | `view` | 0 | 0 | 1 | 0 | 9 |
| `fs-components-widgets` | `agent_tree` | 0 | 0 | 2 | 3 | 15 |
| `fs-components-widgets` | `avatar_group` | 0 | 0 | 0 | 0 | 1 |
| `fs-components-widgets` | `badge` | 0 | 0 | 2 | 2 | 2 |
| `fs-components-widgets` | `breadcrumb` | 0 | 0 | 1 | 0 | 4 |
| `fs-components-widgets` | `calendar_view` | 0 | 8 | 3 | 11 | 29 |
| `fs-components-widgets` | `craft` | 0 | 6 | 2 | 0 | 30 |
| `fs-components-widgets` | `file_preview` | 0 | 0 | 1 | 0 | 11 |
| `fs-components-widgets` | `file_tile` | 0 | 0 | 0 | 1 | 11 |
| `fs-components-widgets` | `lightbox` | 0 | 0 | 0 | 0 | 8 |
| `fs-components-widgets` | `list_view` | 0 | 1 | 0 | 1 | 10 |
| `fs-components-widgets` | `message_composer` | 0 | 0 | 1 | 3 | 9 |
| `fs-components-widgets` | `message_list` | 0 | 0 | 1 | 2 | 23 |
| `fs-components-widgets` | `participant` | 0 | 0 | 0 | 1 | 6 |
| `fs-components-widgets` | `plan` | 8 | 66 | 5 | 48 | 78 |
| `fs-components-widgets` | `share_dialog` | 0 | 0 | 3 | 3 | 17 |
| `fs-components-widgets` | `sortable_list` | 0 | 0 | 0 | 2 | 9 |
| `fs-components-widgets` | `task_card` | 0 | 0 | 0 | 0 | 7 |
| `fs-components-widgets` | `text_view` | 0 | 2 | 2 | 3 | 15 |
| `fs-components-widgets` | `tool_call` | 0 | 0 | 1 | 7 | 31 |
| `fs-components-widgets` | `transfer_list` | 0 | 0 | 0 | 1 | 10 |
| `fs-components-widgets` | `tree_view` | 0 | 0 | 0 | 0 | 1 |
| `fs-components-workbenches` | `calendar` | 0 | 0 | 2 | 4 | 24 |
| `fs-components-workbenches` | `file_manager` | 2 | 10 | 4 | 12 | 52 |
| `fs-components-workbenches` | `llm_chat` | 0 | 0 | 0 | 3 | 13 |
| `fs-components-workbenches` | `people` | 0 | 0 | 2 | 3 | 26 |
| `fs-components-workbenches` | `tasks` | 0 | 0 | 2 | 4 | 31 |
| `fs-components-workbenches` | `view` | 8 | 56 | 25 | 32 | 68 |
