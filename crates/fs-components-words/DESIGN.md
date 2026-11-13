# fs-components-words - Design

Translatable text for FreeSynergy (ADR 0057, `conventions/coding.md` §2a). Std only: no Blinc, no engine, so apps and components can both depend on it. The existing `labels.rs` of the workbenches is not changed; it can later be filled from a `Words`.

## Pattern, chosen first (and why)

- **Strategy** (`Words`, `PluralRules`): the program holds a `Words` and asks `say(id, args)`. English, a table of another language, Fluent or MessageFormat 2 are interchangeable behind it. The plural rule of a language is a second strategy, so the trait is not tied to English's two forms.
- **Decorator** (`Fallback`): wraps a `Words` and is a `Words`; where the first answers with `is_found() == false`, the second (English) answers. Decorators nest.
- **Newtype** (`TextId`): an identifier can never be mixed up with a sentence.
- Supporting: Value objects (`Args`/`Value`, `Said`), Interpreter (`template.rs`, the small template language).

Order of work: pattern, then signatures (`TextId`, `Args`, `Value`, `Said`, `Words`, `PluralRules`), then tests in `tests/`, then the implementation (`Catalog`, `English`, `Fallback`, `template`).

## Pattern map

| Where | Pattern | Why |
|---|---|---|
| `words.rs` | Strategy | the one interface; callers never see a concrete language |
| `plural_rules.rs` | Strategy | count to category per language (zero, one, two, few, many, other); English and Polish provided |
| `fallback.rs` | Decorator | missing text in the reader's language comes in English |
| `text_id.rs` | Newtype | the id is a type |
| `args.rs`, `said.rs` | Value object | named values with a kind; found or missing is a type |
| `template.rs` | Interpreter | `{name}`, `{n, plural, one {# x} other {# y}}`, `\` escapes, parsed to a tree, filled from `Args` |
| `catalog.rs`, `english.rs` | Strategy | concrete `Words` from a table (`&[(id, template)]` or text lines `id = template`) |
| `format.rs` | none | pure functions for the neutral display of a value |

## Why Fluent and MessageFormat 2 fit behind the same trait

- **Named values with a kind:** Fluent variables and MF2 variables are named and typed by use (number, date). `Value::{Count, Number, Date, Time}` hand them over as data, never as finished strings, so the language formats them itself.
- **No finished strings for numbers or dates:** a library can apply its own number and date formatting; with ready strings it could not.
- **Selection in the template, not in the code:** Fluent selectors and MF2 `match` pick the form inside the message. Here the code only passes `count`; `PluralRules` gives the category; the template has the branches. The caller is the same for English and Polish (tested: `tests/plural.rs`, one call, one/few/many).
- **Word order is the template's:** names, not positions.
- A Fluent or MF2 adapter implements `Words::say` by mapping `TextId` to a message id and `Args` to its variables; `Said::missing(id)` is returned for a missing message or variable.
- Code never decides by comparing text; it decides by `Said` (found or missing), `PluralCategory` and the kind of a `Value`.

## Rules of the contract

- `say` does not panic (in `say` and the rendering: no indexing, no unwrap; `template_of`, used only when loading, slices a string at positions it has just measured; nesting is limited to 32 so the stack cannot run out) and never returns empty or blank text. A missing text, a missing or wrongly typed value, a malformed template, no matching plural branch (and no `other`), a blank result: the missing mark, `Said::missing(id)`, which shows the id (a blank id shows `(no id)`). `Said` has private fields; `Said::found(id, text)` turns blank text into the missing mark, so even a foreign `Words` cannot hand out nothing, and `Fallback` treats it as missing.
- A malformed template is found when a catalog is made from text (`CatalogError` with the line) or by `Catalog::validate()` / `English::validate()` (all faulty ids, sorted). Each program should have one test that asserts `validate()` is empty, above all for its English last stage. Templates are parsed once, at load. `validate()` also lists a plural without an `other` branch (as ICU asks; `say` still works and gives the missing mark when no branch fits) and, for a static table (`from_table`, `English`), an identifier given twice. A template that can never print anything (a lone escaped space, or `{n, plural, other {}}`) is faulty. `from_text` ignores a UTF-8 byte order mark at the start.
- Values are inserted as data. `{`, `}`, `#` inside a value are never evaluated again.
- `#` means the count only inside a plural branch; elsewhere it is a plain character. Escapes in templates are exactly `\{`, `\}`, `\#`, `\\` and `\ ` (a space, so a line of a text table can end in one); any other escape, a blank template, a duplicate plural branch and a name containing white space are errors. In a text table an identifier given twice is an error; in a static table the later entry wins.
- A plural needs a `Value::Count`; a branch not present for the category falls to `other`.

## State

Version 0.3.0 workspace (untagged). Done: `TextId`, `Args`/`Value`, `Said`, `Words` (also for `Box` and `&`), `PluralRules` with `EnglishPlural` and `PolishPlural`, template language, `Catalog` (from a static table or text lines, checked per line), `English`, `Fallback`, example `examples/say.rs`, tests in `tests/` (placeholders, plural, fallback_catalog, robust: depth, extreme values, foreign `Words`).

**Unsupported (not built):**
- Number, date and time formatting per language: `{name}` shows a number plainly, a date as `YYYY-MM-DD`, a time as `HH:MM` (UTC). The kinds are there for a language to take over later.
- Plural of fractions (`Value::Number`), exact matches (`=0`), plural offset, `select` (gender), nesting of messages and attributes (Fluent terms).
- Only Polish and English plural rules are provided; others are written as a `PluralRules` (CLDR) or come with the library.
- Reading direction (right to left): not checked; belongs to the frontend.
- Fluent or MF2 adapter: the library decision is made in the week from 2026-11-09 (ADR 0057).
- Loading language packages from Git (ADR 0036): `Catalog::from_text` takes the text; fetching it is the program's.

**Gaps and limits, stated plainly:**
- The template syntax is a small subset of **ICU MessageFormat 1** (`{name}`, `plural`), not MessageFormat 2 and not Fluent. Language packages written for MF2 or Fluent must be converted, or a library adapter replaces `Catalog` behind the same `Words`.
- A date is a Unix instant shown in **UTC**; there is no pure calendar date (a birthday has no instant) and no time zone. Numbers have no options (currency, percent, unit, number of decimals); plural uses the whole number only, without the CLDR operands v, f, t. No ordinal numbers, no `=N`, no `select`.
- Value kinds missing: list ("a, b and c"), duration, relative time.
- `Said` carries no language and no bidirectional isolation of inserted values, and no markup.
- The missing mark (`Said::missing(id)`) does not say why (text absent, value absent, wrong kind, malformed template, no branch).
- `Words: Send + Sync`. A Fluent adapter probably needs its concurrent bundle for that; not checked.
- `Catalog`, `English` and `Fallback` are `pub` because the callers build them (a program wires its English table and the language package); everything else (`template`, `format`) is private.
- A number is shown plainly between 1e-6 and 1e15, scientific outside, `NaN` and the infinity signs by name; the year of a date before year 0 has a minus sign.

## Step 1 of the language work (2026-11-13): locale, chain, setting

Added, std only and additive (no change to `Words`): `Locale` (BCP 47, `chain()`, `from_posix`), `resolve` with `Env` (override, shared setting, `FS_LANG`, `LC_ALL`, `LC_MESSAGES`, `LANGUAGE`, `LANG`, English; `C`/`POSIX`/empty skipped), `LanguageSetting` (a file `language`, atomic write; shared: `$XDG_CONFIG_HOME/freesynergy/language`, override: `$XDG_DATA_HOME/freesynergy/<program>/language`), `Localized::for_locale` for the `*Labels` structs, `Args::iter`. Patterns: Newtype (`Locale`), Chain of Responsibility (`resolve`), Strategy (`Env`), Memento holder (`LanguageSetting`), Factory method (`Localized`). The MessageFormat 2 binding (`mf2` library, ADR 0057) is the separate crate `fs-components-mf2`, so this crate stays free of dependencies. Design and open decisions: `docs/language-and-frontend-menu.md`.
