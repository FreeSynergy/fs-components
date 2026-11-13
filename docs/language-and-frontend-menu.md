# One language per program, and the Frontends menu in the shell (design, not built)

Status: draft for the strategy. Covers finding B-L (mixed German/English in Store, Wiki, menus) and B-M (a "Frontends" menu in every window). Bases: ADR 0065 (standard window shell), ADR 0074 (layer "Sprache und Formate", BCP 47), ADR 0057 (words), `fs-components-words/DESIGN.md`. Everything marked **[open]** needs a decision; **[not verified]** was not checked.

## 1. What is there today (evidence)

**Words.** `fs-components-words` exists (`Words`, `Catalog`, `Fallback`, template language, `Cargo.toml` std only) but **no crate uses it**: `grep fs_components_words` finds only its own tests and example. Every widget and workbench has a `*Labels` struct with an English `Default` (20 structs, 19 `impl Default`; about 694 `pub ...: String` fields in 91 files of the component crates, an upper bound because it also counts non-label fields). Programs fill these in. `fs-components-app` says of itself "It brings no words" (`crates/fs-components-app/src/lib.rs:5-6`).

**Language is decided three different ways, and the shell does not know it:**

| Program | How it reads the language | Where |
|---|---|---|
| Store | `FS_LANG`, then `LANG` (no `LC_*`); only ~23 functions `fn x(lang: Lang)` have German, the rest of the window is English | `fs-store-core/src/language.rs:13`, `fs-store-app/src/words_lang.rs:3-22` (its own doc says "the rest of the window is English") |
| Wiki | `--lang`, else `LC_ALL`, `LC_MESSAGES`, `LANG` starting with `de` (`is_german`); a second copy of the same rule for the page language | `fs-wiki-app/src/frontend.rs:134-142`, `mode_read.rs:175-185`; German only for the reader labels (15 fields, `frontend.rs` `german_words`) and the menu words (`frontend_window.rs:34-38`); program name and "Search pages" stay English (`mode_reader.rs:73,89`, `mode_read.rs:125`) |
| Tasks | none; all English (`words.rs` header: "translation will replace this file's content") | `fs-tasks-app/src/words.rs` (33 constants) |
| Files, People, Groups, Calendar, Maps, Whiteboard | no environment read found [not verified further] | grep for `"LANG"`/`"LC_ALL"`/`"FS_LANG"` over their `crates/` |

**The mixture the person saw is built in:** the Store menu bar is `Menu::new(words::MENU_STORE ...)` (English constant "Store"), `words_lang::menu_view(lang)` ("View"/"Ansicht" by language) and `words::MENU_HELP` (English constant "Help") side by side (`fs-store-app/src/view/menu.rs:69-82`). With `LANG=de` the bar reads "Store | Ansicht | Help".

**Language is not remembered anywhere**: no `language` setting exists in any program (the three settings stores are: Store `settings.toml` with key `face`; Tasks a JSON file with key `frontend`, written atomically, `fs-tasks-app/src/frontend.rs:70-125`; Wiki a plain text file `frontend`, written aside and renamed, `fs-wiki-app/src/frontend.rs:63-110`).

**Menus today (B-M):** `Menu`/`MenuEntry` are plain values; `Shell::set_menus(menus, handler)` is the only door (`fs-components-app/src/shell.rs:98-113`, `menu.rs`). A `MenuEntry` has no check mark and a menu has no sections or submenus; `menu_view.rs` draws one flat panel per menu. The shell builds **no standard menu**: ADR 0065's Program/View/Help does not exist in code; each program builds its own:

- Wiki: menus Program (Quit) and View with one line per frontend, label `"✓ Frontend: Notion"` (mark typed into the text) (`fs-wiki-app/src/frontend_window.rs:33-57`).
- Store: Store, View, Help; View lists the frontends as `"● Frontend: Storefront (tiles or a table, Craft look)"` (`view/menu.rs:57-82`, `words_lang.rs:77-93`). The code still says "Face" (`enum Face`, key `face`, `face_line`), against the word rule.
- Tasks: a **top-level menu "Frontend"** next to Task and View; the shown frontend is `disabled()` instead of marked (`fs-tasks-app/src/frontends.rs:201-225`). The View menu there is the workbench views (List, Status), a different thing.

Three programs, three shapes. The choice is remembered in three formats.

## 2. Language (B-L)

### 2.1 Where it lives: ONE place
Extend `fs-components-words` (std only, already the ADR 0057 home; Blinc-free so programs and component crates can both use it). New modules, additive, no change to `Words`:

```rust
/// BCP 47 language tag, normalised (`de`, `de-DE`, `zh-Hans-CN`). Newtype: never a bare string.
pub struct Locale(String);
impl Locale { pub fn parse(tag: &str) -> Option<Locale>; pub fn primary(&self) -> &str; pub fn as_str(&self) -> &str; pub fn from_posix(value: &str) -> Option<Locale>; }
// from_posix: "de_DE.UTF-8@euro" -> de-DE; "C", "POSIX", "" -> None.

pub enum LanguageSource { Setting, Environment(&'static str), Portal, Fallback }
pub struct Chosen { pub locale: Locale, pub source: LanguageSource }

/// Strategy for the environment so tests never read the real one.
pub trait Env { fn var(&self, name: &str) -> Option<String>; }
pub fn resolve(setting: Option<&str>, env: &dyn Env) -> Chosen;
```

Order of `resolve` (what the task asked, with the existing `FS_LANG` kept so the Store does not regress): **setting `language`** > `FS_LANG` (compat, to be dropped later **[open]**) > `LC_ALL` > `LC_MESSAGES` > `LANG` > portal > `en`. POSIX rule: the first of `LC_ALL`, `LC_MESSAGES`, `LANG` that is set and not empty wins; a value that is `C` or `POSIX` does not count as a language and the search goes on **[open]: or stops with `en`; the POSIX meaning is "no translation"**. `LANGUAGE` (GNU priority list) is not read **[open]**.

**Portal:** ADR 0074 names only the `org.freedesktop.appearance` keys (`color-scheme`, `accent-color`, `contrast`, `reduced-motion`). A language key in `org.freedesktop.portal.Settings` is **not known to me and not verified** (no network was used). Until it is shown, the step is a no-op behind the trait `Portal { fn language(&self) -> Option<Locale> }`, default `None`, so adding it later changes no caller. A desktop session normally exports `LANG` to its programs anyway.

### 2.2 The setting
- Key `language`, value a BCP 47 tag, or absent (= follow the system). Written atomically (temporary file in the same folder, `sync`, `rename`), the pattern both Tasks and Wiki already use.
- Home: the program's data folder `freesynergy/<program>/` (`data_dir_in` of Tasks: `$XDG_DATA_HOME` if absolute, else `$HOME/.local/share`). Words provides `LanguageSetting { file }` with `load() -> Option<Locale>` and `save(Option<&Locale>) -> Result<(), String>`: a broken or unreadable file means "follow the system" and is named on stderr once (as `Preference::load` does).
- Format: one plain file `language` (as the Wiki's `frontend`) in step 1, so no program needs a TOML or JSON parser; Store's `settings.toml` may adopt the key later **[open: one settings file per program for all keys, with a migration like Tasks' `face` to `frontend`]**. One setting per program here; "for all programs" (ADR 0074 section 8, order: person for this program > person for all > frontend default > system) needs the shared settings place of the admin frontend and is **[open]**, not built in this step.

### 2.3 The text-table interface (both languages are a must)
Goal: a text without German or without English **cannot be written**. Two layers:

1. **Compile time.** A macro `texts!` (in `fs-components-words`, `macro_rules!`, no proc-macro dependency) is the only way to declare UI text; its grammar requires both fields:

```rust
texts! { pub RELOAD_TIP { en: "Reload catalog", de: "Katalog neu laden" } }
```
A missing `de:` or `en:` is a compile error (the pattern does not match). It expands to a `const Text { id, en, de }` and a static table that feeds `Catalog::from_table` for each language, so the existing `Words`, `Fallback` (English as the last stage), plural and template language are used unchanged. A third language is a language package (ADR 0036), not a field: `Fallback` handles a missing text there.
2. **Test and script.** `Table::validate()` (exists on `Catalog`) plus a test per crate that: every `Text` has a non-blank `en` and `de`, the sets of `{placeholders}` are equal, `de != en` unless listed under `[same]` (one word, a name, a unit: "Frontends", "OK"). The same rules run as the gate script (section 3) so a crate that does not use the macro is still caught.

Components keep their `*Labels` structs (public API, 20 structs). Each gets `fn for_locale(&Locale) -> Self` built from a `texts!` table next to it (`labels_text.rs`, files of at most 300 lines); `Default` becomes `for_locale(en)`. The workbenches still know no language: the **program** calls `TaskLabels::for_locale(&locale)`.

### 2.4 Changing the language while the program runs
The menu entry sets the setting and calls `on_language(locale)`. Menus are rebuilt at once (the shell owns them, 2.5). Widgets hold their labels from construction (`TasksWidget::new(adapter, labels())` in `fs-tasks-app/src/frontends.rs`), so a live switch needs either `set_labels` on every widget or building the frontend again. **[open]: live (rebuild the frontend, state kept) or "applies at the next start" with a note in the status line.** Recommendation: live for the shell menus in step 2, "next start" with an honest status message for the widgets until every widget has `set_labels`.

### 2.5 The Language entry in the shell
Menu **Program > Language** (de: "Sprache"): "System default (Deutsch)", "Deutsch", "English", the chosen one marked. Language names are always in their own language (endonyms), never translated, so a person who cannot read the current language can still find theirs. Entries come from the installed language packages (ADR 0074: layer `language`); in step 2 the list is `de`, `en` plus what the program passes. Needs a check mark on `MenuEntry` (section 4.2). The shell calls the program with `on_language`.

## 3. The gate (B-L)

`contract/texts-check` (Python, standard library only, pattern of `contract/check`; `--selftest` runs a mutation check). Three rules:

- **R1** hard-coded UI text: a string literal given to `text(`, `label(`, `title(`, `tooltip(`, `placeholder(`, `Menu::new(`, `MenuEntry::new(`, `Status::Info/Problem(` or assigned `= "..".into()` outside `texts!` and outside a `#[cfg(test)]` module.
- **R2** single-language table: in files named `words*`, `labels*`, `lang*`, a UI literal outside a `texts!` block.
- **R3** every `texts!` entry has `en` and `de`, equal placeholders, not blank, `de != en` unless listed.

"UI text" is a heuristic: has a letter, is no path, option, identifier or `fs-` name, and has a space or a capital first letter. An accepted case goes into `contract/texts-exceptions.toml` (`[literal]` key `file:text`, `[same]` key the text) **with a reason**, as `contract/exceptions.toml` does. It finds candidates; a person judges.

**Run on the four trees today** (`contract/texts-check <tree>`): components 458 (R1 1, R2 457), Store 295 (1, 294), Wiki 34 (4, 30), Tasks 49 (4, 45); R3 0 everywhere because no `texts!` exists yet. Part of R2 is noise (date patterns like `'%I %p'` in `fs-components-charts/src/labels.rs:36-38`, which belong under `[literal]`, or better in the locale formats, ADR 0074 "Formate"). Mutation self-test: 9 of 9 fixtures (good file, each rule once, test module ignored, placeholder mismatch) give the meant result. **Not covered, named:** texts built from pieces (`format!("{} files", n)` is neither R1 nor R2: the plural rule of the words crate must replace it, found by review), text in `.css`/`.toml`/`.xml` catalogs, text drawn by Blinc widgets of a dependency, a UI call with another name than the list. The list grows with each miss.

The gate runs with `--stats` (counts, never fails the build) until the migration of a crate is finished, then **hard (exit 1)** for that crate, tree by tree, so it is never red for work not yet due **[open: who runs it: CI of each repo, or `fs-release/scripts`]**.

### Picture test (both languages)
Skeleton in this branch: `crates/fs-components-app/tests/language_picture.rs`. It draws the same window with the bar in German and in English (offscreen, `draw_to_png`, skipped without a graphics adapter), keeps `shell-de.png` and `shell-en.png` in `FS_COMP_SHOTS`, and asserts that the bar differs and the work area does not. Run: 1 passed, pictures looked at (bar "Programm / Ansicht / Hilfe" against "Program / View / Help"). **Skeleton, named:** the language is a local table in the test, not the future `Locale`; it sets nothing through `LANG`. Later: the program under test starts with `LANG=de` and `LANG=en` (real process, temp `HOME`/`XDG_*`) and the stored reference pictures of `fs-components-pictures` (`Group::check`, `Tolerance`, branch `frontend-umbau-3`, read only) replace the pixel asserts; Store and Wiki each get one such pair. A scan that no visible word is English under `LANG=de` is the picture's job only by eye, so the picture pair is the "images of both" for the strategy, not a proof.

## 4. The Frontends menu (B-M)

### 4.1 Meaning
**Frontends** (de: "Frontends", the same word; listed under `[same]`) are the layouts of the program (Store: Storefront, Compact; Wiki: Classic, Confluence, Notion; Tasks: Classic, Board, Day list). **Views** (de: "Ansichten": tiles, list, table, status) stay what the program offers inside one frontend. They are two sections of the menu **View** (de: "Ansicht"), never mixed: first the program's own views, a separator, then the heading "Frontends" with one line per frontend of the program, the active one marked. The marked one is a choice, not a disabled line (Tasks disables it today, which reads as "not available").

### 4.2 API of the shell (`fs-components-app`, additive: `Menu`, `MenuEntry` are `#[non_exhaustive]`)
```rust
pub struct FrontendItem { pub id: String, pub label: String, pub reason: Option<String> /* locked, with the why (ADR 0066) */ }
impl AppWidget {
    /// The frontends this program has (loaded or installed), the active one, and what a choice does.
    pub fn set_frontends(&self, items: Vec<FrontendItem>, active: &str, on_choose: impl Fn(&str) + Send + Sync + 'static) -> Result<(), AppError>;
    pub fn set_active_frontend(&self, id: &str) -> bool;   // marks; menus rebuilt
}
impl MenuEntry { pub fn checked(self, on: bool) -> Self; }   // the mark is drawn by the shell, not typed into the label
impl Menu { pub fn with_id(self, id: &'static str) -> Self; } // `view`, `program`, `help`: so the shell can merge its section
```
The shell owns the menu **View** and merges its "Frontends" section into it; ids of the shell's entries carry the prefix `fs.frontend.`/`fs.language.` and are answered by the shell before the program's handler (today one handler gets every id, `shell.rs:98-113`, so a program cannot mix up the two). Persistence of the choice stays with the program (three formats today, section 1) in step 2; one shared setting is **[open]**. A program with one frontend shows the section with a single marked line (never hidden, so a person sees the program is one of a family). Keyboard and mouse behaviour is ADR 0065 5a and comes from the existing menu code, no second implementation.

**Submenu vs section [open]:** the task asked for a submenu. `menu_view.rs` draws one flat panel per menu and `menu_overlay.rs` one overlay (`shell.rs` holds one `overlay`), so a real submenu (second overlay, Right/Left keys, hover timing) is a larger change in the code that frontend-umbau group 10 converts. Recommended: section with heading inside **View** now (entries and marks as above); a real submenu later as one more `MenuEntry` kind without changing the API above.

### 4.3 Standard menus
ADR 0065's Program (Settings, Quit), View, Help (F1, About) become one builder in the shell, so no program types `"Quit"` again: `AppWidget::standard(StandardParts { program_entries, view_entries, help_page })`. The shell therefore **must get words of its own** (Program, View, Help, Frontends, Language, Settings, Quit, About, Help; in de and en via `texts!`), which changes the sentence "It brings no words" in `lib.rs`/DESIGN to "it brings only the words of the standard shell". **[open: confirm]** (alternative: the program hands in all nine words, which brings back the Store's mixture).

### 4.4 How each program feeds it (merge of the three shapes)
- Wiki: `Frontend::ALL`, label = name (no "Frontend: " prefix, no typed mark) -> `set_frontends`; replaces `menus()` in `frontend_window.rs:33-57`; `german: bool` goes away.
- Store: `Face::ALL` -> `set_frontends`; `view/menu.rs:57-82` shrinks to the program's own views (Programs, Sources, Settings, Messages); rename `Face` to `Frontend` with settings key migration like Tasks' `face` -> `frontend`.
- Tasks: the top-level "Frontend" menu (`frontends.rs:209`) goes; its entries go to View > Frontends; its classic View (List, Status) stays as the program's views.
- Programs with one frontend today (Files, People, Groups, Calendar, Maps, Whiteboard, Messenger): one line, `set_frontends` with a single item [not verified: whether their code can list a second].
"Loaded" (hot plugins, ADR 0074 section 8): the list comes from the program as data; when the store installs or removes a frontend package the program calls `set_frontends` again, and the menu is rebuilt (set_menus already rebuilds and closes an open menu).

## 5. Plan

| # | Step | Where | Depends on | Files (est.) | Risk |
|---|---|---|---|---|---|
| 1 | `Locale`, `Env`, `resolve`, `LanguageSetting`, `texts!`, tests (resolution table incl. `de_DE.UTF-8@euro`, `C`, empty, setting over env, broken file) | `fs-components-words`, new files, no change to `Words` | nothing; no overlap with frontend-umbau | ~6 new, 2 edited | low |
| 2 | Shell: `MenuEntry::checked`, `Menu::with_id`, `set_frontends`, language section, standard menus, own words (de/en), draw the mark; tests of `shell.rs`, `menu_*` | `fs-components-app` (`menu.rs`, `shell.rs`, `widget_menu.rs`, `menu_view.rs`, new `standard.rs`, `texts.rs`) | step 1; **frontend-umbau group 10 touches `menu_view.rs`, `menu_overlay.rs`, `styles.rs`** and the three picture tests `menu_picture.rs`, `picture.rs`, `own_bar.rs`: do step 2 first and let group 10 rebase, or after it; never both at once | ~8 | medium to high (every program draws the shell) |
| 3 | Gate script in CI per repo, `--stats` | `contract/texts-check`, exceptions file | step 1 (macro) | 2 | low |
| 4 | Component labels get `for_locale` from `texts!`, by crate, easiest first: widgets small ones (`agent_tree`, `message_composer`, `message_list`, `transfer_list`, `tool_call`), `llm_chat`, `people`, `tasks`, `calendar`, `charts`, `file_manager` (4 label files, 170+101+290 lines), `page_info` (WikiLabels) | components crates | step 1; **groups 2, 11, 12, 14, 15, 16 of frontend-umbau** convert styles in the same crates: separate files (`labels*.rs` against `view/`, CSS), so overlaps are few, but `workbenches/file_manager` and `people`/`tasks` dirs are shared: one run per group, never in parallel with its umbau group | ~20 label files + ~20 new | medium (about 694 fields; a long German word breaks a layout: the band-height estimate by characters in `file_manager/band_size.rs` is already known to be too low for longer words, `DESIGN.md` "Another language with longer words") |
| 5 | Components release (one tag), `fs-release/scripts/bump-components.sh`, programs lift | release | 2, 4 | release repos | low (known procedure) |
| 6 | Programs, one run each, picture pair de/en + gate hard for that crate: Store (about 216 constants plus 23 `Lang` functions; drop `Lang`, `FS_LANG` read; rename Face), Wiki (15 + the menu words + 3 environment reads; `--lang` stays as override), Tasks (33 + plan view texts), then Files, People, Groups, Calendar, Maps, Whiteboard, Messenger | program repos (10 `-app` crates) | 5 | per program 3 to 10 | medium; Store the largest |
| 7 | Gate hard everywhere; delete the old per-program reads | all | 6 | few | low |

Estimated size in numbers: component crates 457 literals in R2 plus 694 fields (overlap), Store 294, Wiki 30, Tasks 45; the seven smaller programs are not counted yet (run `contract/texts-check` on them in step 3).

## 6. Open decisions (for strategy and the person)
1. **Submenu or section** for Frontends (4.2). Recommended: section in View now.
2. **The shell gets words of its own** (4.3), instead of programs handing them in.
3. **Live switch of language** or "next start" for widgets (2.4).
4. Order **setting > `FS_LANG` > `LC_ALL` > `LC_MESSAGES` > `LANG` > portal > en**: keep `FS_LANG`? treat `C`/`POSIX` as "no language"? read `LANGUAGE`? (2.1)
5. **Portal language key**: none known; check before promising it (2.1).
6. **Where the language setting lives**: one file `language` per program now; one settings file per program and a shared "for all programs" setting later (2.2).
7. Names: "Frontends" in both languages; "Sprache"/"Language"; the endonym list.
8. Who runs the gate and from when it is hard (3).
9. Whether the Store's `Face` rename (setting key `face` -> `frontend`) is part of this work or of the Store's own.
10. Order against frontend-umbau group 10 (step 2) and groups 11 to 16 (step 4): decided by the strategy, since they write to the same crates.

## 7. Not verified
No program was started with `LANG=de`/`LANG=en`; the evidence is the source (file and line above) and the picture skeleton. The programs Files, People, Groups, Calendar, Maps, Whiteboard, Messenger were only searched for the environment variables. The portal language key was not checked. The gate has not been run in CI. No production code was changed.
