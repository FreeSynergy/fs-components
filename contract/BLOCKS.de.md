# Theme-Vertrag: Bausteine

Erzeugt aus `contract/blocks/*.toml` mit `contract/render`. Nicht von Hand ändern.

Vertragsversion: **0.1**

## Allgemeine Bausteine (34)

| Baustein | Name nach | Teile | Zustände | Varianten | Kurzbeschreibung |
|---|---|---|---|---|---|
| `alert` | aria `alert` | `root`, `icon`, `text`, `action` | `default`, `hover`, `focus`, `actionable` | `error`, `warning`, `info` | Eine Meldung, die man sehen muss: Fehler, Warnung, Hinweis. Die Schwere ist die Variante. |
| `app-window` | eigener Name | `root`, `bar`, `nav`, `main`, `row`, `title`, `title-box`, `empty`, `status` | `focus`, `busy` | keine | Die Standard-Fensterhülle eines Programms: Menüleiste, Navigation, Hauptfläche, Statuszeile. |
| `avatar` | open-ui `avatar` | `root`, `image`, `initials` | `default`, `online`, `away`, `offline` | `small`, `large`, `color-0`, `color-1`, `color-2`, `color-3`, `color-4`, `color-5`, `color-6` | Ein Bild oder die Initialen einer Person oder eines Programms. |
| `badge` | open-ui `badge` | `root`, `label`, `dot` | `default` | `info`, `success`, `warning`, `error`, `accent` | Ein kurzer Status oder eine Zahl an einem anderen Baustein (installiert, Update, Punkt). |
| `breadcrumb` | open-ui `breadcrumb` | `root`, `item`, `separator`, `current` | `hover`, `focus`, `current` | keine | Der Weg bis hierher, jeder Schritt ein Link, der letzte ist der Ort selbst. |
| `button` | aria `button` | `icon`, `label` | `hover`, `focus`, `cursor`, `pressed`, `disabled`, `busy` | `primary`, `secondary`, `outline`, `ghost`, `link`, `destructive`, `small`, `icon`, `fill`, `line` | Ein drückbares Bedienelement. Varianten tragen das Aussehen (primary, outline, icon ...), Zustände die Lage. |
| `card` | open-ui `card` | `root`, `header`, `title`, `body`, `footer`, `badge`, `actions` | `hover`, `selected`, `busy`, `disabled` | `panel`, `shared-lines` | Eine umrahmte Fläche für eine Sache (Programm, Aufgabe, Panel). Reiche Karten sind eigene Bausteine. |
| `checkbox` | aria `checkbox` | `box`, `mark`, `label` | `checked`, `hover`, `focus`, `disabled` | keine | Eine Ja-Nein-Wahl mit Beschriftung; auch der runde Erledigt-Haken einer Aufgabe. |
| `confirm-bar` | eigener Name | `root`, `text`, `confirm`, `cancel` | `default`, `busy` | keine | Die Leiste, die noch einmal fragt (wirklich löschen?), mit dem bestätigenden Knopf. |
| `detail-page` | eigener Name | `root`, `head`, `title`, `title-text`, `face`, `actions`, `body`, `section`, `bar`, `pinned` | `editing`, `busy`, `default` | keine | Die Seite eines Objekts: Kopf mit Titel und Aktionen, Abschnitte, Felder. |
| `dialog` | aria `dialog` | `root`, `backdrop`, `title`, `body`, `footer`, `close` | `open`, `busy` | keine | Ein Fenster über dem Fenster, das eines fragt oder zeigt und wieder geschlossen wird. |
| `empty-state` | eigener Name | `root`, `icon`, `title`, `text`, `action` | `default` | keine | Was eine Liste oder Seite zeigt, wenn nichts da ist: Text, optional Symbol und Aktion. |
| `form-field` | eigener Name | `root`, `label`, `value`, `input`, `choices`, `hint`, `cell` | `required`, `invalid`, `disabled`, `focus`, `checked` | keine | Eine Zeile eines Formulars oder einer Detailansicht: Beschriftung, Wert oder Eingabe, Auswahl, Hinweis. |
| `grid` | aria `grid` | `root`, `head`, `row`, `cell`, `mark`, `edge`, `empty`, `failed` | `selected`, `hover`, `focus`, `busy` | `sortable` | Zeilen und Spalten, durch die man mit Tasten geht: die Listenansicht mit Kopf, Zeilen und Zellen. |
| `link` | aria `link` | `label`, `arrow` | `hover`, `focus`, `visited`, `disabled` | keine | Text, der irgendwohin führt (Seite, Projekt), mit optionalem Pfeil. |
| `load-more` | eigener Name | `root`, `button`, `label` | `default`, `busy`, `disabled` | keine | Die Zeile am Ende einer Liste, die mehr nachlädt. |
| `menu` | aria `menu` | `root`, `item`, `separator` | `open`, `hover`, `focus`, `current`, `disabled` | keine | Eine Liste von Befehlen, die aus einer Menüleiste oder einem Knopf aufgeht; die Einträge sind Menüeinträge. |
| `menubar` | aria `menubar` | `root`, `trigger`, `panel`, `item` | `open`, `hover`, `focus` | keine | Die Menüleiste eines Fensters (Datei, Bearbeiten ...); jeder Auslöser öffnet ein Panel. |
| `note` | aria `note` | `root`, `text` | `default` | `info`, `ok`, `warning`, `error`, `quiet`, `dim` | Leiser Erklärtext neben dem Inhalt (Hinweis, Fußnote); der Ton ist die Variante. |
| `progressbar` | aria `progressbar` | `track`, `bar`, `label` | `busy`, `complete`, `failed` | keine | Wie weit etwas ist: Spur und Balken, optional Beschriftung. |
| `search-hit` | eigener Name | `root`, `title`, `path`, `snippet` | `selected`, `hover`, `focus` | keine | Ein Treffer einer Suche: Titel, Pfad, Textauszug. |
| `searchbox` | aria `searchbox` | `root`, `input`, `icon`, `clear` | `focus`, `busy`, `disabled` | keine | Ein Textfeld, das filtert oder sucht; das eine Suchfeld jedes Programms. |
| `section` | eigener Name | `root`, `head`, `title`, `body`, `actions`, `footer` | `expanded`, `collapsed` | keine | Eine Gruppe mit Titel innerhalb einer Seite (Titel, Inhalt, Aktionen am Ende). |
| `sidebar` | eigener Name | `root`, `inner`, `body`, `head`, `heading`, `section`, `item`, `count`, `title` | `expanded`, `collapsed`, `selected`, `hover`, `disabled` | keine | Der Streifen neben der Hauptfläche: Überschriften, Ziele zum Anspringen, ein Baum oder eine Liste. |
| `status` | aria `status` | `root`, `line`, `more`, `panel`, `text`, `problem` | `busy`, `open`, `invalid` | keine | Die Statuszeile eines Fensters mit Detail-Panel: was das Programm gerade tut und was schiefging. |
| `tab` | aria `tab` | `root`, `label`, `close` | `selected`, `hover`, `focus`, `disabled` | keine | Ein Reiter einer Reiterleiste (Datei-Reiter, Ansichtswechsel, Filter, der umschaltet). |
| `table` | aria `table` | `root`, `head`, `row`, `cell` | `hover`, `selected` | keine | Feste Zeilen und Spalten zum Lesen (Tabelle im Text, Tabelle von Paketen). |
| `tablist` | aria `tablist` | `root`, `row`, `panel` | `default`, `busy` | keine | Die Reihe der Reiter samt Zubehör (das Panel zeigt das Frontend). |
| `tag` | open-ui `tag` | `root`, `label`, `close` | `default`, `selected`, `hover`, `disabled` | `neutral`, `accent`, `success`, `warning`, `danger` | Eine kleine beschriftete Pille: Filter, Aufgaben-Label, Markierung. Der Ton ist die Variante. |
| `text` | open-ui `text` | `run` | `default` | `caps`, `caps-dim`, `mono`, `sub`, `quiet`, `ok`, `warning`, `info`, `error`, `title`, `strong` | Schriftstile für Fließtext und Beschriftungen (Versalien, Mono, gedimmt); die Variante ist der Stil. |
| `textbox` | aria `textbox` | `root`, `label`, `input`, `display`, `hint`, `error`, `suggestion` | `focus`, `disabled`, `readonly`, `invalid`, `required` | `compact`, `multiline` | Eine Zeile oder Fläche für Text mit Beschriftung, Hinweis und Vorschlägen (Open UI: Text Input). |
| `toolbar` | aria `toolbar` | `root`, `row`, `group`, `cell`, `search`, `action` | `default`, `busy` | keine | Eine Leiste mit Bedienelementen über einer Fläche: Suche, Aktionen, Platz dazwischen. |
| `tree` | aria `tree` | `root`, `item`, `toggle`, `icon`, `label` | `expanded`, `collapsed`, `selected`, `focus`, `busy` | keine | Eine Hierarchie aus Einträgen, die auf- und zuklappen (Ordner, Seiten); die Einträge sind Baumeinträge. |
| `workbench` | eigener Name | `root`, `toolbar`, `side`, `work`, `main`, `body`, `status`, `preview`, `tree`, `panel`, `band` | `busy`, `empty`, `invalid` | keine | Die Arbeitsfläche eines Programms: Werkzeugleiste, Seite, Hauptfläche mit Inhalt, Status. |

## Baustein-spezifische Bausteine (ein Widget oder Crate) (21)

| Baustein | Name nach | Teile | Zustände | Varianten | Kurzbeschreibung |
|---|---|---|---|---|---|
| `agent-tree` | eigener Name | `root`, `row`, `head`, `head-text`, `main`, `name`, `state`, `meta`, `summary`, `toggle`, `below`, `empty` | `expanded`, `collapsed`, `question`, `failed`, `delivered` | keine | Der Baum der Agenten und ihrer Schritte, jede Zeile mit Name, Zustand und dem, was sie von der Person braucht. |
| `avatar-group` | eigener Name | `root`, `avatar`, `overflow` | `online`, `away`, `offline` | keine | Eine Reihe von Avataren mit einem +N-Kreis für den Rest. |
| `board` | eigener Name | `root`, `bar`, `notice`, `columns` | `busy`, `empty` | keine | Das Brett: Leiste, Hinweiszeile und die Spalten mit Karten. |
| `board-column` | eigener Name | `root`, `head`, `name`, `count`, `cards`, `add`, `add-text`, `entry` | `hover`, `selected` | `drop-target` | Eine Spalte eines Bretts: Kopf mit Name und Zahl, die Karten, eine Zeile zum Hinzufügen. |
| `calendar-grid` | eigener Name | `root`, `toolbar`, `month`, `timed`, `allday`, `slot`, `now` | `default`, `selected` | keine | Das Zeitraster des Kalenders: Monatszellen, Zeitfenster, Ganztagszeile, die Jetzt-Linie. |
| `calendar-view` | eigener Name | `root`, `column`, `day`, `head`, `head-cell`, `gutter`, `hour`, `item`, `all-day`, `agenda`, `agenda-day`, `agenda-row`, `agenda-time`, `mark`, `more`, `now`, `number`, `empty` | `today`, `selected`, `outside` | `color-0`, `color-1`, `color-2`, `color-3`, `color-4`, `color-5` | Monats-, Wochen-, Tages- und Agenda-Ansicht von Terminen. |
| `file-preview` | eigener Name | `root`, `head`, `title`, `fact`, `label`, `value`, `mark`, `marks`, `note`, `text`, `refused` | `default`, `busy` | keine | Die Vorschau einer Datei: Titel, Angaben, Text oder der Grund, warum sie abgelehnt wird. |
| `file-tile` | eigener Name | `root`, `icon`, `name`, `line`, `badge`, `mark`, `marks`, `plain`, `grid`, `empty` | `selected`, `hover`, `focus`, `cursor` | keine | Eine Datei oder ein Ordner als Kachel mit Symbol, Name und einer Zeile Angaben. |
| `lightbox` | eigener Name | `root`, `bar`, `body`, `stage`, `caption`, `count`, `note` | `open`, `busy` | keine | Bilder im ganzen Fenster mit Leiste, Bildunterschrift und Zähler. |
| `map` | eigener Name | `attribution`, `controls`, `control` | `default`, `busy`, `invalid` | keine | Die Karte: Quellenzeile und die Zoom-Bedienung (Kacheln werden gezeichnet, nicht gestaltet). |
| `message-composer` | eigener Name | `root`, `area`, `buttons`, `files`, `file`, `file-name`, `notes`, `note`, `failed` | `busy`, `disabled`, `invalid` | keine | Das Feld, in dem eine Nachricht geschrieben wird, mit Anhängen und Senden-Knöpfen. |
| `message-list` | eigener Name | `root`, `msg`, `head`, `name`, `meta`, `body`, `col`, `delivery`, `failure`, `failure-text`, `edge`, `empty`, `note`, `notice`, `reasoning`, `refs`, `tools` | `mine`, `failed` | `person`, `software`, `ai`, `unknown` | Die Nachrichten einer Unterhaltung; wer sie schrieb, ist die Variante. |
| `participant` | eigener Name | `root`, `text` | `default` | `person`, `software`, `ai`, `unknown` | Ein Chip, der einen Teilnehmenden nennt: Person, Software, KI oder unbekannt. |
| `share-dialog` | eigener Name | `root`, `title`, `who`, `list`, `row`, `mark`, `name`, `meta`, `note`, `perms`, `pick`, `buttons`, `foot`, `unknown` | `cursor`, `selected`, `checked` | keine | Der Dialog, der zeigt, wer etwas sehen darf, und die Auswahl erlaubt. |
| `sortable-list` | eigener Name | `root`, `lane`, `row`, `title`, `note` | `cursor`, `grabbed`, `locked` | keine | Eine Liste, deren Zeilen per Tasten oder Ziehen verschoben werden. |
| `task-card` | eigener Name | `root`, `title`, `line`, `mark`, `tags`, `tag` | `open`, `selected`, `hover` | keine | Die Karte einer Aufgabe: Titel, Zeile mit Angaben, Markierungen und Labels. |
| `text-view` | eigener Name | `root`, `block`, `heading`, `quote`, `code`, `codespan`, `item`, `marker`, `rule`, `run`, `codebox`, `codelines` | `default` | keine | Ein formatiertes Dokument: Überschriften, Absätze, Listen, Zitate, Code. |
| `tile-grid` | eigener Name | `root`, `tile`, `head`, `body`, `number`, `footer` | `hover`, `selected` | keine | Reihen nummerierter Kacheln, die ihre Linien teilen, mit Kopf und Körper je Kachel. |
| `timeline` | eigener Name | `root`, `toolbar`, `axis`, `body`, `scroll`, `arrows`, `today`, `today-arrow`, `today-edge`, `today-row` | `default`, `busy` | keine | Einträge auf einer Zeitachse mit Heute-Linie und Abhängigkeitspfeilen. |
| `tool-call` | eigener Name | `list`, `root`, `head`, `name`, `kind`, `state`, `risk`, `summary`, `options`, `details`, `caption`, `button`, `chip-text`, `code`, `refused` | `question`, `failed` | `kind-reads`, `kind-writes`, `kind-runs`, `kind-connected`, `kind-unknown`, `risk-ok`, `risk-changes`, `risk-high`, `risk-unknown` | Was ein Agent tun will, mit seinem Risiko, und die Knöpfe zum Erlauben oder Ablehnen. |
| `transfer-list` | eigener Name | `root`, `row`, `main`, `name`, `meta`, `reason`, `actions`, `more`, `empty` | `queued`, `running`, `done`, `failed` | keine | Hoch- und Herunterladen mit Fortschritt und den Knöpfen je Zustand. |

## Programm-spezifische Bausteine (reiche Bausteine eines Programms) (12)

| Baustein | Name nach | Teile | Zustände | Varianten | Kurzbeschreibung |
|---|---|---|---|---|---|
| `chat` | eigener Name | `root`, `main`, `side`, `sidehead`, `head`, `headtext`, `title`, `meta`, `extras`, `dock`, `stop`, `stopbox`, `attention` | `busy`, `attention` | keine | Das Unterhaltungsfenster: Verlauf an der Seite, der Text, das Dock mit der Eingabe, der Stopp-Knopf. |
| `command-band` | eigener Name | `root`, `inner`, `row`, `input`, `status`, `notice`, `problem`, `outside` | `busy`, `invalid` | keine | Das Band des Dateimanagers: die Frage, der getippte Name, die Knöpfe. |
| `install-job` | eigener Name | `root`, `phases`, `phase`, `track` | `busy`, `failed`, `done`, `current` | keine | Die laufende Installation oder das Update: die Phasen und die aktuelle. |
| `page-block` | eigener Name | `root`, `list`, `grip`, `band` | `editing`, `hover` | keine | Ein bearbeitbarer Block einer Wiki-Seite mit Griff. |
| `page-tree-row` | eigener Name | `root`, `grip`, `icon`, `label`, `dot`, `ticks`, `tick`, `folder`, `state` | `current`, `quiet`, `selected`, `hover`, `expanded`, `collapsed` | `dot-ok`, `dot-old`, `dot-other` | Eine Seite im Wiki-Baum: Griff, Symbol, Titel, Zustandspunkt. |
| `place` | eigener Name | `root`, `icon`, `label` | `current`, `disabled`, `hover`, `selected` | `top`, `trash`, `settings`, `network`, `pinned` | Ein Ort in der Seitenleiste des Dateimanagers. |
| `program-card` | eigener Name | `root`, `icon`, `title`, `summary`, `badge`, `action`, `head`, `version` | `installed`, `update`, `offered`, `busy`, `selected`, `hover` | keine | Die Karte eines Programms im Store: Symbol, Titel, Kurzbeschreibung, Marke, Aktion. |
| `program-detail` | eigener Name | `root`, `head`, `heading`, `sub`, `track`, `preview`, `facts` | `busy`, `invalid` | keine | Die Seite eines Programms: Kopf, Beschreibung, Installationsfortschritt, Wissenswertes. |
| `program-row` | eigener Name | `root`, `list`, `icon`, `name`, `summary`, `version`, `badge`, `action`, `cell`, `count`, `search`, `filters` | `installed`, `update`, `busy`, `selected`, `hover` | keine | Ein Programm als Zeile: Name, Version, Marke, Aktion. |
| `quick-add` | eigener Name | `root`, `plus`, `field` | `default`, `focus`, `busy` | keine | Die immer vorhandene Zeile zum Hinzufügen des nächsten Eintrags. |
| `revision-row` | eigener Name | `root` | `current`, `hover` | keine | Eine Zeile des Seitenverlaufs. |
| `task-row` | eigener Name | `root`, `head`, `main`, `title`, `sub`, `reason`, `more`, `facts`, `code` | `hover`, `selected`, `checked`, `muted` | keine | Eine Aufgabe als Zeile: Erledigt-Haken, Titel, Unterzeile, Angaben und Labels. |
