# plato2 — curation notes

Plato's *Gorgias*, Greek reading edition plus a clean-room English translation
locked 1:1 to it.

## Provenance

| | |
|---|---|
| Copy-text | John Burnet, *Platonis Opera*, Tomus III (Oxford: Clarendon Press, 1903) |
| Transcription | Perseus Digital Library, `canonical-greekLit` |
| File | `data/tlg0059/tlg023/tlg0059.tlg023.perseus-grc2.xml` |
| Perseus commit | `91595f89e15b4d3000cd93efcf8990720c8be2b9` |
| Taken | 2026-09-10 |

The raw TEI is kept at `assets/plato2/raw/` and is gitignored, as in every other
corpus.

## What the source contains

Verified by the converter's `--survey` mode, which refuses any element outside
the set it was written for:

| | |
|---:|---|
| 1,107 | `<said>` elements, each with one `<label>` and one `<p>` |
| 1,081 | genuine speech turns — 26 `<said>` carry `rend="merge"` and continue the speech before them |
| 527 / 236 / 207 / 97 / 14 | genuine turns for Socrates / Callicles / Polus / Gorgias / Chaerephon |
| 284 | of the 404 sections are in force at some turn start, so any of those may anchor a division |
| 404 | Stephanus section milestones, 447a–527e |
| 81 | bare page milestones (dropped as redundant — the section value carries the page) |
| 8 | Perseus paragraph milestones (`ed="P"`) — a drafting hint only |
| 17 / 4 / 1 | `<del>` athetized / `<add>` supplied / `<gap>` lost |
| 15 / 10 / 6 | `<quote>` / `<l>` verse lines / `<bibl>` citations |
| 27,341 | Greek words |

### There is no 447e

Burnet's page 447 ends at `d`. The dialogue therefore has 404 sections, not the
405 that 81 pages × 5 letters would suggest. Any gate that generates the
expected key set arithmetically has to special-case it.

### `rend="merge"` is a continuation, not a turn

26 `<said>` elements carry `rend="merge"`. They continue the speech before them
across one of Perseus's section-div boundaries, and none of them changes speaker
(verified: 0 exceptions of 26). They must not emit a second speaker label. Joined
up, the two long speeches are Socrates' closing myth (1,639 words from 522e, five
fragments) and Callicles' speech on nature and convention (1,350 words from 482c,
five fragments).

Those fragment boundaries are **not** paragraph breaks. They fall where Perseus's
markup divides sections, which has nothing to do with where the speech breathes.

### The speech turn is the block structure

The *Republic* is narrated, so its paragraphing had to be reconstructed from
4,234 Perseus `ed="P"` hints. The *Gorgias* is staged: it has 8 such hints,
because the turn boundaries do that work instead. Paragraph breaks *within* a
long speech (Polus's, Callicles' great speech, the closing myth) are Scholia's
and are reviewed.

### Speaker names

The curated markdown carries full names (`@ Σωκράτης`), not Burnet's printed
abbreviations (`ΣΩ.`). Both are in the source — the abbreviation in `<label>`,
the expansion in `<said who>` — against the five-person `<particDesc>` roster.
The reader renders speaker labels as a flush-left column, which a column of
two-letter abbreviations would not serve. Lamb's Loeb TEI independently confirms
the conventional English forms: Socrates, Callicles, Polus, Gorgias, Chaerephon.

## The English edition

A clean-room translation from Burnet's Greek, sentence-parity locked to it and
imported with `struct_to_db --source-book-slug gorgias-grc`. Both editions build
to the same shape — 22 nodes, 2,202 blocks, 2,828 sentences, 404 markers — which
is what the lock means in practice.

Scored per sentence against three controls in three eras (Lamb, Loeb 1925;
Jowett, 1871; Horan, rev. 2026), none of which any translating agent ever saw.
One passage failed at 24 shared words — the chiasmus at 506c, whose vocabulary
this corpus's own terminology table mandates — and was re-rendered from the Greek
by an agent that had not seen the flagged span. Six passages sit in the review
band (13–16 words) and were left alone: they are forced convergence, phrases like
"the doctor as your slave and the trainer as your slave" where the Greek's shape
plus a governed term leaves English almost no choice.

### Burnet's brackets in the English

`⟨ ⟩` marks what Burnet supplied and `[ ]` what he athetized. All four supplied
readings carry into the English, bracketing the English element whose presence
depends on the Greek one — `this very thing ⟨which⟩ you're so pleased with` for
`τοῦτο ⟨ὃ⟩ δὴ ἀγαπᾷς`.

Five of the seventeen athetized readings do not appear in the English, which is
deliberate. Three are dittographies — `[οἱ ἀγαθοὶ καὶ οἱ κακοί]` at 498d,
`[βλέποντες]` and `[πρὸς τὸ ἔργον τὸ αὑτῶν,]` at 503e — where the same words
already stand unbracketed in the same sentence, so the English says them once.
The other two, `[ἐπὶ]` and `[αὑτούς]`, are a preposition and a reflexive that
English carries inside a verb rather than as a separate word; bracketing an
arbitrary English word to stand in for them would assert a dependency that is
not there. The rule is in the `plato2-translate` skill: bracket the English
element whose presence depends on the Greek one, and where none does, leave the
sentence unbracketed and say so.
