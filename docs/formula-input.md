# Formula language

View → Interface language is the existing, persisted language preference. Choose **Español**
for Spanish, **Deutsch** for German or **Français** for French formula entry and display. On first run the preference
follows the system language. English and the other interface languages currently use the English
formula table and separators.

| Spanish | Canonical English |
| --- | --- |
| `SUMA` | `SUM` |
| `PROMEDIO` | `AVERAGE` |
| `SI` | `IF` |
| `CONTAR` | `COUNT` |
| `SUMAR.SI` | `SUMIF` |
| `CONTAR.SI` | `COUNTIF` |
| `BUSCARV` | `VLOOKUP` |
| `HOY` | `TODAY` |
| `VERDADERO`, `FALSO` | `TRUE`, `FALSE` (literals or function calls) |

Names are case-insensitive. Spanish formulas use decimal commas (`1,5`), semicolon arguments
and reference unions, and backslashes between array columns (`{1\2;3\4}`). For example,
`=SI(VERDADERO;SUMA(1,5;2,25);0)` evaluates to 3.75. English mode retains comma arguments,
decimal points and comma array columns; it does not silently translate Spanish aliases.

German uses the function names German Excel 365 shows, all 420 that differ from English
(`SUMME`, `WENN`, `SVERWEIS`, `MITTELWERTWENNS`, `BETA.VERT` …), as observed for PR #1201
(`crates/formula/src/input_de.rs`); the 99 others, such as `MAX`, `INDEX` and `FILTER`, keep their
English name. `WAHR` and `FALSCH` are the booleans. German formulas use decimal commas, semicolon
arguments and points between array columns (`{1.2;3.4}`): `=WENN(WAHR;SUMME(1,5;2,25);0)` is 3.75.
Error values in formulas are German too: `#WERT!`, `#BEZUG!`, `#ZAHL!`, `#NV`, `#DATEN_ABRUFEN` and
`#ÜBERLAUF!`; `#NULL!`, `#DIV/0!`, `#NAME?` and `#CALC!` are the same in both. The English spelling
is still understood when typed. Error values shown as cell results, and table specifiers (`#Alle`,
`#Daten`), are not translated yet. Typed cell values follow the number locale (File › Options ›
Number and date format, German by default with Deutsch): `1.234,56`, `31.12.2026`, `12,5 %` and
`1.234,56 €` are numbers. Spanish takes the decimal comma only (`1.234,5` stays text).

French uses French Excel's names, the 402 that differ from English (`SOMME`, `SI`, `RECHERCHEV`,
`NB.SI`, `MOYENNE.SI.ENS`, `AUJOURDHUI` …), from the tables of upstream #2310 and #1639
(`crates/formula/src/input_fr.rs`); `MAX`, `INDEX`, `LET` and the others keep their English
name. `VRAI` and `FAUX` are the booleans. Separators are as in German: decimal commas, semicolon
arguments and points between array columns (`{1.2;3.4}`, from Microsoft's French array-constant
page; not checked in French Excel): `=SI(VRAI;SOMME(1,5;2,25);0)` is 3.75. Error values are
`#VALEUR!`, `#NOM?`, `#NOMBRE!`, `#NUL!`, `#PROPAGATION!` and `#CHARGEMENT_DONNEES`; `#DIV/0!`,
`#REF!`, `#N/A` and `#CALC!` are the same in both. An English name still works unless French
gives it to another function (`TRIM` is French for MIRR, `VAR.P` for VARP). Typed cell values
follow the French number locale (`1 234,5`, `31/12/2026`).

The cell editor, formula bar, Show Formulas, autocomplete, the Formula ribbon, Insert Function, and argument hints
use the selected formula language. Common Spanish function help is translated; functions
without a Spanish entry retain their English names/help. General interface labels and errors
fall back to English where Spanish translations are not provided. An edit captures its language
when it starts so switching the preference cannot reinterpret unfinished input.

Workbook/sheet names, table names, LET bindings and LAMBDA parameters take precedence over
aliases. Text, sheet names and table column names are unchanged. When a localized spelling
would collide with a name, displayed formulas retain the canonical spelling to preserve meaning.

Stored formulas, XLSX import/export, engine commands, control/API calls, CLI evaluation and
clipboard exchange retain strict English syntax. This also applies to advanced dialogs such as
Name Manager and conditional-format rules. Localization happens at the cell/formula-bar editor
boundary, including filling the selected range with Ctrl+Enter; it does not change the workbook
format or teach the calculation engine ambiguous aliases.
