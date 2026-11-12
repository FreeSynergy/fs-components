# Theme-Vertrag von fs-components

**Version 0.1 (Entwurf, abgeleitet aus fs-components v0.0.12).** ADR 0074, Schritt 1. Englische Fassung: [THEME-CONTRACT.md](THEME-CONTRACT.md).

Der Vertrag ist die Liste der **Bausteine** (der Stücke, aus denen ein Fenster besteht), jeder mit festen **Teilen** und **Zuständen** (und, wo sinnvoll, **Varianten**). Ein Stil (Theme) gestaltet nur diese Namen. So funktioniert er auf jedem Programm, auch auf künftigen. Programme und Frontends enthalten keine Farben, Abstände oder Schriften (ADR 0074, Abschnitt 2). Das ist der erste Entwurf: Er wurde **aus dem abgeleitet, was der Code heute tut**, nicht erfunden. Was bleibt, entscheidet die Prüfung.

## Dateien

| Datei | Inhalt |
|---|---|
| `contract.toml` | Version, wo die anderen Dateien liegen, welche Namenspräfixe als Klassennamen zählen, die Standard-Zustände. |
| `blocks/*.toml` | Die Bausteine: Name, Herkunft des Namens, Geltungsbereich, Teile, Zustände, Varianten, Einzeiler in EN und DE. |
| `mapping/*.toml` | Jede Klasse oder Id von heute (fs-components v0.0.12, der Store und die zwei laufenden Zweige) und der Vertragsname, der daraus wird. |
| `exceptions.toml` | Namen im Quelltext, die keine Klasse sind (Thread-Namen, Zustands-Schlüssel, Handle-Ids), je mit Grund. |
| `BLOCKS.md`, `BLOCKS.de.md`, `MAPPING.md` | Erzeugte Tabellen (`contract/render`). Nicht von Hand ändern. |
| `HARDCODED-VALUES.md` | Erzeugte Liste der festen Werte im Code (`contract/hardcoded`), Grundlage für Schritt 2. |
| `check`, `selftest`, `render`, `hardcoded` | Python-3-Skripte ohne Abhängigkeiten und ohne Netz (der Code liegt in `lib/contractlib/`). |

Ein Format: TOML. Die Dateien sind nur aufgeteilt, damit sie kurz bleiben; `contract.toml` listet sie.

## Regeln

1. **Baustein-Namen.** Ein Baustein heißt nach der WAI-ARIA-1.2-Rolle, wo eine passt (`button`, `menu`, `tab`, `tree`, `grid`, `dialog`, `alert` ...). Wo Open UI einen gebräuchlichen Komponentennamen hat, nach Open UI (`card`, `tag`, `badge`, `avatar`, `breadcrumb`, `text`). Sonst ein eigener Name **mit schriftlichem Grund** (`naming = "own"`, `reason`). Bei `aria` und `open-ui` ist der Bausteinname gleich dem Rollen- oder Komponentennamen; `check` prüft beides gegen die Listen in `lib/contractlib/vocab.py`.
2. **Namen nur `[a-z0-9-]`**, keine Großbuchstaben, keine Unterstriche, keine Doppelungen (Bausteine, Teile, Zustände, Varianten).
3. **Jeder Baustein hat Teile und Zustände** und eine Kurzbeschreibung in beiden Sprachen. Hat ein Baustein nichts zu melden, steht `default` (ausdrücklich, nicht stillschweigend).
4. **Teile** sind die benannten Stellen im Baustein (`program-card`: `icon`, `title`, `summary`, `badge`, `action`). **Zustände** sind Lagen des Bausteins oder eines Teils (`installed`, `update`, `busy`, dazu ARIA-Zustände wie `selected`, `expanded`, `disabled`). **Varianten** sind Aussehen, die das Programm wählt und die sich nicht ändern (ein Knopf ist `primary` oder `outline`; ein Tag hat einen Ton). Ein Zustand ändert sich, während das Programm läuft.
5. **Geltungsbereich** sagt, wem der Baustein gehört: `general` (jedes Programm), `block` (ein Widget oder Crate von fs-components), `program` (reicher Baustein eines Programms: Programmkarte, Aufgabenzeile, Seitenbaum-Zeile ...). Reiche Bausteine sind gewollt: Ein Stil kann eine Store-Karte, eine Board-Spalte oder eine Aufgabenzeile nur gezielt gestalten, wenn jede einen eigenen Namen hat.
6. **Ziel-Schreibweise** im Mapping: `baustein[.teil][/variante][:zustand]`, zum Beispiel `button/primary:disabled`, `program-card:installed`. Das Ziel `-` ist nur mit Status `remove` erlaubt.
7. Wie die Vertragsnamen in CSS- oder Blinc-Selektoren aussehen, wird **hier nicht entschieden** (Schritt 3 mit den Stilen). Das Mapping sagt nur, welcher alte Name zu welchem Vertragsnamen wird.

## Status eines abgebildeten Namens

| Status | Bedeutung |
|---|---|
| `mapped` | Der Name hat seinen Platz; beim Umstellen (Schritt 4) ändert sich nur der Name. |
| `provisional` | Der Platz existiert, aber das Programm oder die Workbench ist noch vorläufig (README: alle Workbenches außer dem Reader), oder das Ziel ist eine Vermutung. Braucht eine Entscheidung. |
| `remove` | Kein Vertragsname. Der Name ist Anordnung (Abstandshalter, Breite) oder das Aussehen eines Stils (alle `craft-*`-Klassen): Tokens oder das Frontend übernehmen ihn. |

Namen in `mapping/other-places.toml` stammen aus dem Store (`external`) und den Zweigen `tasks-visual` (`fs-plan-*`) und `wiki-frontends-1` (`fs-wk-*`, nur gelesen) (`coming`). Die Prüfung sucht sie nicht in diesem Repository.

## Stand bei Version 0.1

Aktuelle Zahlen: `contract/check --stats`. Zum Zeitpunkt des Schreibens: 67 Bausteine (34 allgemein, 21 baustein-spezifisch, 12 programm-spezifisch). Von den 474 Namen aus `.class("..")`/`.id("..")` in fs-components (Produktion und Tests): 261 abgebildet, 127 vorläufig, 27 zu entfernen, 58 nur in Tests (außerhalb des Vertrags), 1 Ausnahme (`tabindex=0`, das ist ein Fehler, keine Klasse: `.class("tabindex=0")` in `file_manager/view/actions.rs`). Dazu benutzt der Quelltext 82 weitere Namen, die kein `.class(..)`-Aufruf ausschreibt (CSS-Selektoren, Konstanten, Match-Arme); 13 davon sind keine Klassen (Thread-Namen, Schlüssel, Handle-Ids, ein Crate-Name, ein Cargo-Feature) und stehen auf der Ausnahmeliste, die übrigen sind abgebildet. Nichts bleibt unabgebildet.

## Was die Skripte beweisen

- `contract/check`: Jede Klasse oder Id in `crates/*/src` (`.class("..")`, `.id("..")`, jedes Wort einer Mehrklassen-Zeichenkette und jeder Name mit Präfix `fs-`, `craft-` oder `cn-` in einer Codezeile, das fängt CSS-Texte, Konstanten und Match-Arme) steht im Mapping oder auf der Ausnahmeliste; kein Mapping und keine Ausnahme ist veraltet; jeder Baustein hat Teile, Zustände, Kurzbeschreibung (EN, DE), einen gültigen Geltungsbereich und einen begründeten Namen; Namen sind `[a-z0-9-]`; Mapping-Ziele zeigen auf echte Bausteine, Teile, Zustände und Varianten; nirgends Doppelungen; die erzeugten Tabellen sind aktuell.
- `contract/selftest`: Beschädigt eine Kopie des Repositories auf 25 Arten (neue Klasse, entfernte Mapping-Zeile, Baustein ohne Teile, Großbuchstabe, Doppelung, veraltete Zeile, von Hand geänderte Tabelle ...) und verlangt, dass `check` jedes Mal mit der richtigen Meldung scheitert. Es beweist auch, dass `hardcoded` einen neuen festen Wert bemerkt.
- `contract/hardcoded --check`: `HARDCODED-VALUES.md` ist aktuell. Das gehört bewusst nicht zu `check`, weil jede Änderung eines Pixelwerts im Code es sonst brechen würde.

Grenzen, damit niemand der Prüfung zu viel traut: Die Suche ist textuell. Eine Klasse, die zur Laufzeit gebaut wird (`format!`, Variable), wird nicht gesehen; mit `format!` gebaute Ids (`fs-app-menu-{index}`, `{prefix}-card-{id}`, `cal-item-{key}`) sind Element-Kennungen, keine Stil-Haken, und liegen außerhalb des Vertrags. Ein Klassenname ohne `fs-`, `craft-` oder `cn-` wird nur in `.class("..")`/`.id("..")` gefunden. Ein Name, den nur Testdateien benutzen, wird nicht geprüft (`mapping/test-only.toml` listet die 58, die es gibt). Die Prüfung schaut weder in den Store noch in die Zweige. Ein Mapping kann dem Sinn nach falsch sein und trotzdem bestehen: Die Prüfung muss es lesen.

## Nicht übernommen aus den Berichten

Die mit `[nicht belegt]` markierten Zahlen der Berichte werden nicht benutzt. Die ARIA-Rollenliste in `vocab.py` ist die Liste von 1.2 einschließlich `code`, 82 Namen; der Bericht zählte 81 und seine Tabelle hat kein `code`, also irrt einer von beiden, und die Prüfung sollte es am W3C-Text nachsehen. WAI-ARIA 1.3 wurde nicht angesehen. Die Open-UI-Liste sind die 33 Research-Komponenten von open-ui.org, wie der Bericht sie nennt; die Komponenten-Matrix war nicht lesbar und wird nicht benutzt.

## Offene Punkte (für die Prüfung)

- Ist `provisional` das richtige Wort für eine Workbench-Rolle, die sauber auf einen allgemeinen Baustein passt (zum Beispiel `fs-tasks-failed` auf `alert/error`)? Sie ist nur vorläufig, weil die Workbench es ist.
- Mehrere Store-Bausteine (`program-row`, `program-detail`) sind aus CSS-Namen des Stores rekonstruiert, nicht aus einem Entwurf. Ebenso `page-tree-row`, `page-block`, `quick-add`, `task-row`, `board-column`, die aus unveröffentlichten Zweigen stammen.
- Bausteine, die der Code noch nicht hat, die ADR 0074 aber erwartet (tooltip, switch, select, accordion, toast, slider ...), stehen nicht drin: Der Vertrag wurde aus dem Bestand abgeleitet. Open UI listet sie; sie kommen, wenn ein Programm sie braucht.
- Teile- und Zustandslisten enthalten Namen, die noch keine alte Klasse erreicht (349 von 433 Teilen werden erreicht). Das sind die Lücken, die ein Stil-Autor gefüllt braucht; manche können fallen.
- Das Mapping von `craft-*` auf `remove` nimmt an, dass Craft nach ADR 0074, Schritt 3 ein Theme wird und seine Klassen verschwinden.
- Farben, Größen, Schriften: `HARDCODED-VALUES.md`, Teil B, ist eine Heuristik (der Baustein eines Werts ist der nächste Klassenname). Schritt 2 muss es lesen, nicht darauf vertrauen.
