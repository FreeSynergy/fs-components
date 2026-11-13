# Changelog

## fs-components 0.0.15

- Shell: language setting under Program > Language, one shared setting for all frontends. The texts of the shell come from MessageFormat 2 files (`fs-components-mf2`, `fs-components-words`) in German and English.
- Menu: View > Frontends is a submenu; a submenu is as wide as its content.
- Gate `contract/texts-check` (every visible text in de and en, no hard coded UI text) runs in CI as a warning only; its self test is hard. The findings of the old tables are fixed in a later step.
- Limits, stated plainly: submenus do not open on hover; the Left/Right keys do not move between levels; no third menu level; MF2 conformance is 405 of 462 cases of the official suite (all 405 supported cases pass, see `crates/fs-components-mf2/CONFORMANCE.md`); date, time and currency formatting are not done yet.

## fs-components 0.0.14

- Wiki page tree: a page `a.md` and the folder `a/` of its sub-pages are one node (title opens the page, arrow expands) in the Confluence, Notion and Classic frontends. Additive, no API change.
