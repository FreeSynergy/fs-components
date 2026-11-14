# Editor-Vertrag (fs-components-editor)

Mehrere Editor-Bausteine stehen hinter einem Trait, `TextEditor` (Muster: Strategy).

- **Eingabe:** ein `Doc`, das Dokument im Text-Modell (Blöcke aus Markdown-Quelltext, eine Leerzeile dazwischen, ein Zeilenumbruch am Ende).
- **Ausgabe:** nur Änderungs-Operationen desselben Modells: `Op::InsertBlock`, `Op::RemoveBlock`, `Op::ReplaceBlock`.
- **Regel:** Ein Editor ändert Daten nur über diese Operationen und formatiert nie eigenmächtig um. Ein Dokument, das ohne Änderung geladen und gespeichert wird, bleibt Byte für Byte gleich, und jede Änderung ist als Operation sichtbar. `apply(geladen, take_ops())` ergibt genau `document()`.

## Bausteine

| Name | Was es ist | Stand |
|---|---|---|
| `SourceEditor` | reiner Text (Markdown, Code, Konfiguration) in einer Textfläche; erste Umsetzung: der Classic- und Confluence-Editor | gebaut |
| `BlockEditor` | Notion-Art: jeder Block ein eigenes Feld, Enter macht einen neuen Block, `/` öffnet das Blockmenü | gebaut (Menü: nur Blocktypen) |
| `RichTextEditor` | WYSIWYG | später |

Ein Baustein ist kein Editor-Programm. Öffnen, Speichern und die Wahl von Baustein und Frontend gehören dem Programm (später eine Facette von fs-text). Der Stil ist eine eigene Schicht (ADR 0074); jeder Baustein hat feste Namen für seine Stilteile.

## Warum ein eigenes Crate

Das Text-Modell des Wikis (`fs-text`, Repo fs-wiki) hängt von fs-components ab, also kann fs-components nicht davon abhängen. Der Vertrag hat kleine blockadressierte Operationstypen; das Wiki bildet sie auf sein Modell ab.

## Konformitätstests

Eine Sammlung, `fs_components_editor::conformance::check`, läuft für jede Umsetzung (`crates/fs-components-workbenches/tests/conformance.rs`): der Text des Nutzers vom 2026-11-14 und die GFM-Fälle (Tabellenausrichtung, Aufgabenlisten, `~durch~`, `~~durch~~`, `_kursiv_`, `*kursiv*`), Round-trip Byte für Byte, jede Änderung als Operation sichtbar. Ein neuer Baustein muss sie bestehen.

## Erweiterungspunkte (nur deklariert, nicht gebaut)

Module wie Hot-Plugins (ADR 0045), ein Trait je Baustein: `SourceModule` (Sprachen, Language Server, Vorschau), `BlockModule` (Blocktypen, Einträge des `/`-Menüs), `RichModule` (Marken, Leistenknöpfe), gehalten in einer `Registry`. Die Traits sind vorerst leer.

## Grenzen

Text, der nicht in der kanonischen Form steht (mehrere Leerzeilen, kein Zeilenumbruch am Ende), wird vom Block-Editor vereinheitlicht; die Fixtures sind kanonisch.
