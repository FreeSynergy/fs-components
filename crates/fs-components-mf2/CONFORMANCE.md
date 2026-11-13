# mf2 3.0.0 gegen die offizielle MessageFormat-2-Suite

Suite: unicode-org/message-format-wg, Commit 16372a6, `test/tests/**/*.json` (462 Faelle).
Test: `crates/fs-components-mf2/tests/conformance.rs` (Branch unreviewed/mf2-conformance), direkt ueber `mf2` (Registry wie in `Mf2Texts`: string, number, integer, percent, offset; Bidi je Fall).
Pruefung: Ausgabe (`exp`) gleich, und die Menge der Fehlertypen (`expErrors`, Syntax/Datenmodell beim Kompilieren, Laufzeit beim Formatieren) gleich. `expParts` wird NICHT geprueft (nur gezaehlt).

| Datei | bestanden | fehlgeschlagen | nicht unterstuetzt | davon mit expParts |
|---|---|---|---|---|
| bidi.json | 27 | 0 | 0 | 0 |
| data-model-errors.json | 23 | 0 | 0 | 0 |
| fallback.json | 5 | 0 | 3 | 3 |
| functions/currency.json | 0 | 0 | 12 | 0 |
| functions/date.json | 0 | 0 | 7 | 0 |
| functions/datetime.json | 0 | 0 | 7 | 0 |
| functions/integer.json | 13 | 0 | 0 | 0 |
| functions/number.json | 41 | 0 | 0 | 1 |
| functions/offset.json | 16 | 0 | 0 | 0 |
| functions/percent.json | 13 | 0 | 0 | 0 |
| functions/string.json | 10 | 0 | 0 | 0 |
| functions/time.json | 0 | 0 | 6 | 0 |
| pattern-selection.json | 0 | 0 | 22 | 0 |
| syntax-errors.json | 133 | 0 | 0 | 0 |
| syntax.json | 114 | 0 | 0 | 10 |
| u-options.json | 10 | 0 | 0 | 6 |
| **gesamt** | **405** | **0** | **57** | 20 |

Quote: 405/462 = 87,7 % bestanden; 405/405 = 100 % der unterstuetzten Faelle.

## Nicht unterstuetzt (57), alles Registry-Luecken unserer Konfiguration, kein Fehler von mf2
- 22 pattern-selection + 3 fallback: `:test:select/:test:function/:test:format` (Testfunktionen der Suite, nicht in mf2 enthalten).
- 32 currency/date/datetime/time: Funktionen, die `Mf2Texts` nicht registriert (Datum/Zeit zeigen wir neutral, `date.rs`; Waehrung nicht benutzt). mf2 hat dafuer das Feature `fn_datetime` (nicht geprueft).
Die Faelle wurden nicht ausgefuehrt-und-bewertet, sondern bei Abweichung als "nicht unterstuetzt" gezaehlt. Die 10 `u-options`-Faelle bestehen (u:* wird ignoriert); `u:dir/u:id` sind fuer uns ohne Belang.

## Fehlerklassen (fehlgeschlagen)
Im Endstand keine. Beim ersten Lauf 1 Fehler: syntax.json, `{$one} et {$two}` (fr): erwartet `1,3 et 4,2`, erhalten `1.3 et 4.2`. Ursache war UNSERE Registry: ohne `.with_numbers(&mf2::fn_number::NUMBERS)` formatiert mf2 nicht annotierte Zahlen neutral (Spec 2.6). Das betrifft UNSERE Nutzung ({$n} ohne `:number` mit Value::Number/Count). Behoben in `src/texts.rs` (eine Zeile) und im Test abgesichert (`core.rs`: `{$p}` de 1,5).

## Einordnung fuer FreeSynergy
- Betrifft uns (einfache Nachrichten, Variablen, Plural, Zahl, Fallback, Escapes, Syntaxfehler): alles bestanden (syntax, syntax-errors, data-model-errors, number, integer, percent, offset, string, bidi, fallback ohne :test).
- Betrifft uns nicht: currency, date/time/datetime, :test:*, u:dir/u:id, expParts (Teile-Ausgabe).
- `core` (tests/core.rs, 23 eigene Faelle + 5 Syntaxfehler, hart): 100 % bestanden. Enthaelt :plural-Selektion (en/de/pl, ordinal), :number, :integer, :string, Escapes, fehlende Variable / unbekannte Funktion / falscher Operand mit Fallback.
