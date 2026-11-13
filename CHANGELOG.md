# Changelog

## fs-components 0.0.16

- Wiki page tree: a child is indented one step of the space scale; the arrow place has a fixed width, so the title of a parent does not move when it opens; a click on the free rest of a row opens the page, the arrow expands. Additive.
- New component `name-dialog` (`fs_components_widgets::NameDialog`): asks for one name (title, input, OK and Cancel, Enter and Escape, focus with the preset name selected) for "New page", "Sub-page", "Rename". Its texts are MessageFormat 2 files in German and English (`name_dialog/texts/*.mf2`, via `fs-components-mf2`), no fixed words in the dialog. `SearchField` gets `text`, `set_text`, `focus`. Additive.
- Notion frontend: the title and a first level-1 heading are one thing: while editing, the first heading block is the title (no second big title above it); the outline lists the title, so "No headings" shows only for a page without any. A first heading that differs from the title stays in the text.
- Limits, stated plainly: the name dialog is not wired into the wiki yet (no caller, no modal); Ctrl+V and Enter are not proven in a real window (only a broadcast test); in Notion edit mode the title is a raw `# ` block, not a title field of its own.

Deutsch:

- Seitenbaum im Wiki: ein Kind ist um eine Stufe der Abstandsskala eingerückt; der Pfeilplatz hat feste Breite, der Titel des Elternteils wandert beim Aufklappen nicht; ein Klick auf den freien Rest der Zeile öffnet die Seite, der Pfeil klappt auf. Additiv.
- Neue Komponente `name-dialog` (`fs_components_widgets::NameDialog`): fragt einen Namen ab (Titel, Eingabe, OK und Abbrechen, Enter und Esc, Fokus mit vorbelegtem, markiertem Namen) für „Neue Seite“, „Unterseite“, „Umbenennen“. Die Texte sind MessageFormat-2-Dateien auf Deutsch und Englisch (`name_dialog/texts/*.mf2`), keine festen Wörter im Dialog. `SearchField` bekommt `text`, `set_text`, `focus`. Additiv.
- Notion-Frontend: Titel und erste Überschrift der Ebene 1 sind dasselbe: beim Bearbeiten ist der erste Überschriftsblock der Titel; die Gliederung führt den Titel auf, „No headings“ erscheint nur noch bei Seiten ganz ohne Überschrift. Eine abweichende erste Überschrift bleibt im Text.
- Grenzen: der Namensdialog ist noch nicht im Wiki verdrahtet (kein Aufrufer, kein Modal); Strg+V und Enter sind nicht im echten Fenster belegt (nur Broadcast-Test); im Notion-Bearbeiten ist der Titel ein roher „# “-Block, kein eigenes Titelfeld.

## fs-components 0.0.15

- Shell: language setting under Program > Language, one shared setting for all frontends. The texts of the shell come from MessageFormat 2 files (`fs-components-mf2`, `fs-components-words`) in German and English.
- Menu: View > Frontends is a submenu; a submenu is as wide as its content.
- Gate `contract/texts-check` (every visible text in de and en, no hard coded UI text) runs in CI as a warning only; its self test is hard. The findings of the old tables are fixed in a later step.
- Limits, stated plainly: submenus do not open on hover; the Left/Right keys do not move between levels; no third menu level; MF2 conformance is 405 of 462 cases of the official suite (all 405 supported cases pass, see `crates/fs-components-mf2/CONFORMANCE.md`); date, time and currency formatting are not done yet.

## fs-components 0.0.14

- Wiki page tree: a page `a.md` and the folder `a/` of its sub-pages are one node (title opens the page, arrow expands) in the Confluence, Notion and Classic frontends. Additive, no API change.
