# kant3 — textual compromises & provenance notes

Deliberate deviations from a perfectly diplomatic edition, recorded so they
aren't mistaken for errors.

## Source orthography (md_reviewed)

- **No long-s (ſ).** PG #55925 normalizes ſ→s, so kant3's "original orthography"
  is period *spelling* (Urtheilskraft, nothwendig, Uebersinnliche) **without**
  Fraktur typography. Unlike kant1 (reconstructed from Fraktur scans), kant3
  cannot and does not restore ſ.
- **Sperrdruck IS preserved** — the source tags it (`<em class="gesperrt">`), so
  `***…***` is faithful (not a guess).
- **AA editorial apparatus dropped.** Only Kant's own footnotes are kept (41).
  The Academy's "Anmerkungen"/Lesarten (AA p. 510+) and their inline refs are
  excluded; they are editorial, not authorial.

## Reference systems

- `aa_v` (Akademie-Ausgabe Band V, block) and `e1790` (1790 first edition,
  inline) are both extracted automatically from the source's `pb`/`opn` spans.
- AA *line numbers* (`<span class="ln">`) are dropped (apparatus, not content).

## Translation (md_modernized_translated)

- A **Scholia Community Edition** English translation produced with LLM
  assistance, held to **1:1 sentence parity** with the modernized German
  (enforced at build time by `md_prose_to_struct --corpus kant3 --translation`).

## Sentence-splitter tuning (common::sentences)

Verification surfaced gaps in the shared German splitter, fixed without
regressing kant1 (kant1 struct output is byte-identical before/after):

- Added abbreviations: `Hofr.`, standalone `" v."` (von in names; NOT word-final
  `-v.` like *objektiv.*), `gl.`, and the multi-token `r. V.` (= reinen Vernunft).
- Added the German closing guillemet `«` as a sentence boundary and an opening
  quote (`"`/`"`) as a valid sentence start (symmetric with how German already
  breaks before `»`).
