# Changelog

## fs-components 0.0.17

- Notion frontend, editing: the page title is a big input field above the content (placeholder "Untitled" / "Ohne Titel" from MessageFormat 2 files, `page_title/texts/*.mf2`), no raw `# ` block any more. The blocks start after the title; a first level-1 heading that differs from the title stays a block. Saving puts title and blocks together again (round-trip tested: Markdown to editor to Markdown is identical); a title that was not in the text is written as `# ` heading only when the person changes it. Enter in the title moves the keyboard to the first block.
- New component `page-title` (`fs_components_widgets::PageTitle`, `TitleLabels`). `SearchField` gets size `Title` (big, no frame), `on_submit`, `is_focused`, `set_placeholder`, `set_text_at_end`. `WikiLabels` gets `title_hint`. `Reader::page_title`. Additive.
- Notion page tree: the icon of a page (a sheet of paper) had a very pale edge; the edge is now the kind colour darkened (`sheet_edge`), at least contrast 3.0 (`icon::MIN_ICON_CONTRAST`, WCAG 1.4.11) on white, sidebar and selected row; tested by colours and by pixels.
- Reading mode: the title shown is the `title` of the document the source gives, not the file name. That is the job of the source (fs-wiki); the test source of this crate now gives the first heading as title.
- Limits, stated plainly: the title field is not bold (Blinc `text_input` has no font weight), so it differs from the bold title in reading mode; Enter in the title is proven by an in-process key test, not in a real window; the jump to the first block is wired but not covered by its own test; the name dialog is still not wired into the wiki; the contrast threshold 3.0 is our choice (TOKENS.md names none); the Confluence frontend is unchanged.

Deutsch:

- Notion-Frontend, Bearbeiten: der Seitentitel ist ein großes Eingabefeld über dem Inhalt (Platzhalter „Untitled“ / „Ohne Titel“ aus MessageFormat-2-Dateien, `page_title/texts/*.mf2`), kein roher „# “-Block mehr. Die Blöcke beginnen nach dem Titel; eine erste Überschrift der Ebene 1, die vom Titel abweicht, bleibt ein Block. Speichern setzt Titel und Blöcke wieder zusammen (Round-trip getestet: Markdown zu Editor zu Markdown identisch); ein Titel, der nicht im Text stand, wird nur dann als „# “-Überschrift geschrieben, wenn die Person ihn ändert. Enter im Titel setzt den Fokus in den ersten Block.
- Neue Komponente `page-title` (`fs_components_widgets::PageTitle`, `TitleLabels`). `SearchField` bekommt die Größe `Title` (groß, ohne Rahmen), `on_submit`, `is_focused`, `set_placeholder`, `set_text_at_end`. `WikiLabels` bekommt `title_hint`. `Reader::page_title`. Additiv.
- Notion-Seitenbaum: das Icon einer Seite (ein Blatt Papier) hatte einen sehr blassen Rand; der Rand ist jetzt die Artfarbe, abgedunkelt (`sheet_edge`), mindestens Kontrast 3,0 (`icon::MIN_ICON_CONTRAST`, WCAG 1.4.11) auf Weiß, Seitenleiste und gewählter Zeile; getestet über Farben und über Pixel.
- Lesemodus: angezeigt wird der `title` des Dokuments, den die Quelle liefert, nicht der Dateiname. Das ist Sache der Quelle (fs-wiki); die Testquelle dieses Crates liefert jetzt die erste Überschrift als Titel.
- Grenzen: das Titelfeld ist nicht fett (Blincs `text_input` kennt kein Schriftgewicht), also anders als der fette Titel im Lesemodus; Enter im Titel ist durch einen Tastentest im Prozess belegt, nicht im echten Fenster; der Sprung in den ersten Block ist verdrahtet, hat aber keinen eigenen Test; der Namensdialog ist noch nicht im Wiki verdrahtet; die Kontrastschwelle 3,0 ist unsere Wahl (TOKENS.md nennt keine); das Confluence-Frontend ist unverändert.

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
