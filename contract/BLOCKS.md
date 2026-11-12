# Theme contract: blocks

Generated from `contract/blocks/*.toml` by `contract/render`. Do not edit.

Contract version: **0.1**

## General blocks (34)

| Block | Name from | Parts | States | Variants | Summary |
|---|---|---|---|---|---|
| `alert` | aria `alert` | `root`, `icon`, `text`, `action` | `default`, `hover`, `focus`, `actionable` | `error`, `warning`, `info` | A message the person must see: failed, warning, notice. Severity is the variant. |
| `app-window` | own name | `root`, `bar`, `nav`, `main`, `row`, `title`, `title-box`, `empty`, `status` | `focus`, `busy` | none | The standard window shell of a program: menu bar, navigation, main area, status line. |
| `avatar` | open-ui `avatar` | `root`, `image`, `initials` | `default`, `online`, `away`, `offline` | `small`, `large`, `color-0`, `color-1`, `color-2`, `color-3`, `color-4`, `color-5`, `color-6` | A picture or the initials of a person or program. |
| `badge` | open-ui `badge` | `root`, `label`, `dot` | `default` | `info`, `success`, `warning`, `error`, `accent` | A short status or count on another block (installed, update, a dot). |
| `breadcrumb` | open-ui `breadcrumb` | `root`, `item`, `separator`, `current` | `hover`, `focus`, `current` | none | The path to where you are, each step a link, the last one is the place itself. |
| `button` | aria `button` | `icon`, `label` | `hover`, `focus`, `cursor`, `pressed`, `disabled`, `busy` | `primary`, `secondary`, `outline`, `ghost`, `link`, `destructive`, `small`, `icon`, `fill`, `line` | A pressable control. Variants carry the look (primary, outline, icon ...), states carry the situation. |
| `card` | open-ui `card` | `root`, `header`, `title`, `body`, `footer`, `badge`, `actions` | `hover`, `selected`, `busy`, `disabled` | `panel`, `shared-lines` | A framed area that holds one thing (a program, a task, a panel). Rich cards are their own blocks. |
| `checkbox` | aria `checkbox` | `box`, `mark`, `label` | `checked`, `hover`, `focus`, `disabled` | none | A two-state choice (on, off) with a label; also the round done mark of a task. |
| `confirm-bar` | own name | `root`, `text`, `confirm`, `cancel` | `default`, `busy` | none | The bar that asks once more (really delete?) with the confirming button. |
| `detail-page` | own name | `root`, `head`, `title`, `title-text`, `face`, `actions`, `body`, `section`, `bar`, `pinned` | `editing`, `busy`, `default` | none | The page of one object: head with title and actions, sections, fields. |
| `dialog` | aria `dialog` | `root`, `backdrop`, `title`, `body`, `footer`, `close` | `open`, `busy` | none | A window above the window that asks or shows one thing and is closed again. |
| `empty-state` | own name | `root`, `icon`, `title`, `text`, `action` | `default` | none | What a list or page shows when there is nothing: text, optional icon and action. |
| `form-field` | own name | `root`, `label`, `value`, `input`, `choices`, `hint`, `cell` | `required`, `invalid`, `disabled`, `focus`, `checked` | none | One row of a form or detail view: label, value or input, choices, a hint. |
| `grid` | aria `grid` | `root`, `head`, `row`, `cell`, `mark`, `edge`, `empty`, `failed` | `selected`, `hover`, `focus`, `busy` | `sortable` | Rows and columns the person moves through with keys: the list view with head, rows and cells. |
| `link` | aria `link` | `label`, `arrow` | `hover`, `focus`, `visited`, `disabled` | none | Text that goes somewhere (a page, a project), with an optional arrow. |
| `load-more` | own name | `root`, `button`, `label` | `default`, `busy`, `disabled` | none | The row at the end of a list that fetches more. |
| `menu` | aria `menu` | `root`, `item`, `separator` | `open`, `hover`, `focus`, `current`, `disabled` | none | A list of commands that opens from a menubar or a button; the entries are menu items. |
| `menubar` | aria `menubar` | `root`, `trigger`, `panel`, `item` | `open`, `hover`, `focus` | none | The bar of menus of a window (File, Edit ...); each trigger opens a panel. |
| `note` | aria `note` | `root`, `text` | `default` | `info`, `ok`, `warning`, `error`, `quiet`, `dim` | Quiet explanatory text beside content (a hint, a footnote); tone is the variant. |
| `progressbar` | aria `progressbar` | `track`, `bar`, `label` | `busy`, `complete`, `failed` | none | How far something is: a track and a bar, optional label. |
| `search-hit` | own name | `root`, `title`, `path`, `snippet` | `selected`, `hover`, `focus` | none | One hit of a search: title, path, snippet. |
| `searchbox` | aria `searchbox` | `root`, `input`, `icon`, `clear` | `focus`, `busy`, `disabled` | none | A text box that filters or finds; the one search field of every program. |
| `section` | own name | `root`, `head`, `title`, `body`, `actions`, `footer` | `expanded`, `collapsed` | none | A titled group inside a page (title, content, actions at the end). |
| `sidebar` | own name | `root`, `inner`, `body`, `head`, `heading`, `section`, `item`, `count`, `title` | `expanded`, `collapsed`, `selected`, `hover`, `disabled` | none | The strip beside the main area: headings, items to go to, a tree or a list. |
| `status` | aria `status` | `root`, `line`, `more`, `panel`, `text`, `problem` | `busy`, `open`, `invalid` | none | The status line of a window with its more-panel: what the program does now and what went wrong. |
| `tab` | aria `tab` | `root`, `label`, `close` | `selected`, `hover`, `focus`, `disabled` | none | One tab of a tab list (a file tab, a view switch, a filter chip that switches). |
| `table` | aria `table` | `root`, `head`, `row`, `cell` | `hover`, `selected` | none | Static rows and columns for reading (a table in a text, a table of packages). |
| `tablist` | aria `tablist` | `root`, `row`, `panel` | `default`, `busy` | none | The row of tabs and what belongs to it (the panel is shown by the frontend). |
| `tag` | open-ui `tag` | `root`, `label`, `close` | `default`, `selected`, `hover`, `disabled` | `neutral`, `accent`, `success`, `warning`, `danger` | A small labelled pill: a filter, a label of a task, a mark. Tone is the variant. |
| `text` | open-ui `text` | `run` | `default` | `caps`, `caps-dim`, `mono`, `sub`, `quiet`, `ok`, `warning`, `info`, `error`, `title`, `strong` | Type styles of running text and labels (capitals, mono, dim); the variant is the style. |
| `textbox` | aria `textbox` | `root`, `label`, `input`, `display`, `hint`, `error`, `suggestion` | `focus`, `disabled`, `readonly`, `invalid`, `required` | `compact`, `multiline` | A line or area of editable text with its label, hint and suggestions (Open UI: Text Input). |
| `toolbar` | aria `toolbar` | `root`, `row`, `group`, `cell`, `search`, `action` | `default`, `busy` | none | A bar of controls above a surface: search, actions, the room between them. |
| `tree` | aria `tree` | `root`, `item`, `toggle`, `icon`, `label` | `expanded`, `collapsed`, `selected`, `focus`, `busy` | none | A hierarchy of items that open and close (folders, pages); the items are tree items. |
| `workbench` | own name | `root`, `toolbar`, `side`, `work`, `main`, `body`, `status`, `preview`, `tree`, `panel`, `band` | `busy`, `empty`, `invalid` | none | The working surface of a program: toolbar, side, main area with its body, status. |

## Block-specific blocks (one widget or crate) (21)

| Block | Name from | Parts | States | Variants | Summary |
|---|---|---|---|---|---|
| `agent-tree` | own name | `root`, `row`, `head`, `head-text`, `main`, `name`, `state`, `meta`, `summary`, `toggle`, `below`, `empty` | `expanded`, `collapsed`, `question`, `failed`, `delivered` | none | The tree of agents and their steps, each row with name, state and what it needs from the person. |
| `avatar-group` | own name | `root`, `avatar`, `overflow` | `online`, `away`, `offline` | none | A row of avatars with a +N circle for the rest. |
| `board` | own name | `root`, `bar`, `notice`, `columns` | `busy`, `empty` | none | The board: a bar, the notice line and the columns of cards. |
| `board-column` | own name | `root`, `head`, `name`, `count`, `cards`, `add`, `add-text`, `entry` | `hover`, `selected` | `drop-target` | One column of a board: head with name and count, the cards, an add row. |
| `calendar-grid` | own name | `root`, `toolbar`, `month`, `timed`, `allday`, `slot`, `now` | `default`, `selected` | none | The time grid of the calendar: month cells, timed slots, all-day row, the now line. |
| `calendar-view` | own name | `root`, `column`, `day`, `head`, `head-cell`, `gutter`, `hour`, `item`, `all-day`, `agenda`, `agenda-day`, `agenda-row`, `agenda-time`, `mark`, `more`, `now`, `number`, `empty` | `today`, `selected`, `outside` | `color-0`, `color-1`, `color-2`, `color-3`, `color-4`, `color-5` | Month, week, day and agenda views of events. |
| `file-preview` | own name | `root`, `head`, `title`, `fact`, `label`, `value`, `mark`, `marks`, `note`, `text`, `refused` | `default`, `busy` | none | The preview of a file: title, facts, text, or the reason it is refused. |
| `file-tile` | own name | `root`, `icon`, `name`, `line`, `badge`, `mark`, `marks`, `plain`, `grid`, `empty` | `selected`, `hover`, `focus`, `cursor` | none | One file or folder as a tile with icon, name and a line of facts. |
| `lightbox` | own name | `root`, `bar`, `body`, `stage`, `caption`, `count`, `note` | `open`, `busy` | none | Pictures full-window with a bar, caption and count. |
| `map` | own name | `attribution`, `controls`, `control` | `default`, `busy`, `invalid` | none | The map: attribution line and the zoom controls (tiles are drawn, not styled). |
| `message-composer` | own name | `root`, `area`, `buttons`, `files`, `file`, `file-name`, `notes`, `note`, `failed` | `busy`, `disabled`, `invalid` | none | The field where a message is written, with attached files and the send buttons. |
| `message-list` | own name | `root`, `msg`, `head`, `name`, `meta`, `body`, `col`, `delivery`, `failure`, `failure-text`, `edge`, `empty`, `note`, `notice`, `reasoning`, `refs`, `tools` | `mine`, `failed` | `person`, `software`, `ai`, `unknown` | The messages of a conversation; who wrote it is the variant. |
| `participant` | own name | `root`, `text` | `default` | `person`, `software`, `ai`, `unknown` | A chip that names a participant: person, software, AI or unknown. |
| `share-dialog` | own name | `root`, `title`, `who`, `list`, `row`, `mark`, `name`, `meta`, `note`, `perms`, `pick`, `buttons`, `foot`, `unknown` | `cursor`, `selected`, `checked` | none | The dialog that shows who may see something and lets the person choose. |
| `sortable-list` | own name | `root`, `lane`, `row`, `title`, `note` | `cursor`, `grabbed`, `locked` | none | A list whose rows are moved by keys or drag. |
| `task-card` | own name | `root`, `title`, `line`, `mark`, `tags`, `tag` | `open`, `selected`, `hover` | none | The card of one task: title, a line of facts, marks and tags. |
| `text-view` | own name | `root`, `block`, `heading`, `quote`, `code`, `codespan`, `item`, `marker`, `rule`, `run`, `codelines` | `default` | none | A formatted document: headings, paragraphs, lists, quotes, code. |
| `tile-grid` | own name | `root`, `tile`, `head`, `body`, `number`, `footer` | `hover`, `selected` | none | Rows of numbered tiles that share their lines, with head and body per tile. |
| `timeline` | own name | `root`, `toolbar`, `axis`, `body`, `scroll`, `arrows`, `today`, `today-arrow`, `today-edge`, `today-row` | `default`, `busy` | none | Items on a time axis with the today line and dependency arrows. |
| `tool-call` | own name | `list`, `root`, `head`, `name`, `kind`, `state`, `risk`, `summary`, `options`, `details`, `caption`, `button`, `chip-text`, `code`, `refused` | `question`, `failed` | `kind-reads`, `kind-writes`, `kind-runs`, `kind-connected`, `kind-unknown`, `risk-ok`, `risk-changes`, `risk-high`, `risk-unknown` | What an agent wants to do, with its risk, and the buttons to allow or refuse. |
| `transfer-list` | own name | `root`, `row`, `main`, `name`, `meta`, `reason`, `actions`, `more`, `empty` | `queued`, `running`, `done`, `failed` | none | Uploads and downloads with progress and the buttons by state. |

## Program-specific blocks (rich blocks of one program) (12)

| Block | Name from | Parts | States | Variants | Summary |
|---|---|---|---|---|---|
| `chat` | own name | `root`, `main`, `side`, `sidehead`, `head`, `headtext`, `title`, `meta`, `extras`, `dock`, `stop`, `stopbox`, `attention` | `busy`, `attention` | none | The conversation window: history at the side, the text, the dock with the input, the stop button. |
| `command-band` | own name | `root`, `inner`, `row`, `input`, `status`, `notice`, `problem`, `outside` | `busy`, `invalid` | none | The band of the file manager: the question, the typed name, the buttons. |
| `install-job` | own name | `root`, `phases`, `phase`, `track` | `busy`, `failed`, `done`, `current` | none | The running install or update: its phases and the one that is now. |
| `page-block` | own name | `root`, `list`, `grip`, `band` | `editing`, `hover` | none | One editable block of a wiki page, with its grip. |
| `page-tree-row` | own name | `root`, `grip`, `icon`, `label`, `dot`, `ticks`, `tick`, `folder`, `state` | `current`, `quiet`, `selected`, `hover`, `expanded`, `collapsed` | `dot-ok`, `dot-old`, `dot-other` | One page in the wiki tree: grip, icon, title, state dot. |
| `place` | own name | `root`, `icon`, `label` | `current`, `disabled`, `hover`, `selected` | `top`, `trash`, `settings`, `network`, `pinned` | One place in the sidebar of the file manager. |
| `program-card` | own name | `root`, `icon`, `title`, `summary`, `badge`, `action`, `head`, `version` | `installed`, `update`, `offered`, `busy`, `selected`, `hover` | none | The card of one program in the Store: icon, title, summary, badge, action. |
| `program-detail` | own name | `root`, `head`, `heading`, `sub`, `track`, `preview`, `facts` | `busy`, `invalid` | none | The page of one program: head, description, install progress, what to know. |
| `program-row` | own name | `root`, `list`, `icon`, `name`, `summary`, `version`, `badge`, `action`, `cell`, `count`, `search`, `filters` | `installed`, `update`, `busy`, `selected`, `hover` | none | One program as a row: name, version, badge, action. |
| `quick-add` | own name | `root`, `plus`, `field` | `default`, `focus`, `busy` | none | The always-present row for adding the next entry. |
| `revision-row` | own name | `root` | `current`, `hover` | none | One row of the page history. |
| `task-row` | own name | `root`, `head`, `main`, `title`, `sub`, `reason`, `more`, `facts`, `code` | `hover`, `selected`, `checked`, `muted` | none | One task as a line: done mark, title, a sub line, facts and labels. |
