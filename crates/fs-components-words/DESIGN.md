# fs-components-words - Design

Translatable text for FreeSynergy (ADR 0057, `conventions/coding.md` §2a). Std only: no Blinc, no engine, so apps and components can both depend on it. The existing `labels.rs` of the workbenches is not changed; it can later be filled from a `Words`.

## Pattern, chosen first (and why)

- **Strategy** (`Words`, `PluralRules`): the program holds a `Words` and asks `say(id, args)`. English, a table of another language, Fluent or MessageFormat 2 are interchangeable behind it. The plural rule of a language is a second strategy, so the trait is not tied to English's two forms.
- **Decorator** (`Fallback`): wraps a `Words` and is a `Words`; where the first says `Missing`, the second (English) answers. Decorators nest.
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
- A Fluent or MF2 adapter implements `Words::say` by mapping `TextId` to a message id and `Args` to its variables; `Said::Missing` is returned for a missing message or variable.
- Code never decides by comparing text; it decides by `Said` (found or missing), `PluralCategory` and the kind of a `Value`.

## Rules of the contract

- `say` never panics and never returns empty text. A missing text, a missing or wrongly typed value, a malformed template, no matching plural branch (and no `other`), an empty result: `Said::Missing(id)`, the id is what is shown (an empty id shows `(no id)`).
- Values are inserted as data. `{`, `}`, `#` inside a value are never evaluated again.
- `#` means the count only inside a plural branch; elsewhere it is a plain character. Escape `\{`, `\}`, `\#`, `\\` in templates.
- A plural needs a `Value::Count`; a branch not present for the category falls to `other`.

## State

Version 0.2.0 workspace. Done: `TextId`, `Args`/`Value`, `Said`, `Words` (also for `Box` and `&`), `PluralRules` with `EnglishPlural` and `PolishPlural`, template language, `Catalog` (from a static table or text lines, checked per line), `English`, `Fallback`, example `examples/say.rs`, tests in `tests/` (placeholders, plural, fallback_catalog).

**Unsupported (not built):**
- Number, date and time formatting per language: `{name}` shows a number plainly, a date as `YYYY-MM-DD`, a time as `HH:MM` (UTC). The kinds are there for a language to take over later.
- Plural of fractions (`Value::Number`), exact matches (`=0`), plural offset, `select` (gender), nesting of messages and attributes (Fluent terms).
- Only Polish and English plural rules are provided; others are written as a `PluralRules` (CLDR) or come with the library.
- Reading direction (right to left): not checked; belongs to the frontend.
- Fluent or MF2 adapter: the library decision is made in the week from 2026-11-09 (ADR 0057).
- Loading language packages from Git (ADR 0036): `Catalog::from_text` takes the text; fetching it is the program's.
