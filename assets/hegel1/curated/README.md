# hegel1 — Phänomenologie des Geistes (curated MD)

Source: Deutsches Textarchiv (<https://www.deutschestextarchiv.de/hegel_phaenomenologie_1807>)
— a TEI P5 double-keyed transcription (no OCR) of the first edition,
*System der Wissenschaft. Erster Theil: Die Phänomenologie des Geistes*,
Bamberg und Würzburg: Joseph Anton Goebhardt, 1807.

Licence (this statement takes precedence over the blanket `assets/`
licence, per the repository NOTICE): the DTA text is **CC BY-SA 4.0**,
and the German text layers here plus the editorial tables that produce
them (`md_reviewed/`, `md_modernized/`, `modernize_rulings.tsv`,
`modernize_ops.tsv`, `gw9_markers.tsv`) carry that same share-alike
licence; Scholia asserts no further rights in them. The English
translation (`md_modernized_translated/`) is Scholia's own work and
remains under the regular assets licence (CC BY-NC-ND 4.0).

`md_reviewed/` is derived from the DTA `att.linguistic` TEI: its `<w>` tokens
span `<lb/>`, which is what lets a hyphenated line break be rejoined on word
boundaries rather than by guesswork.

Three curated layers, 50 files each (one per TOC node, `NNN_slug.md`):

- `md_reviewed/` — 1807 orthography exactly as printed, long-s (`ſ`) preserved,
  `<sic>` readings kept uncorrected. The diplomatic layer.
- `md_modernized/` — modern German orthography (driven by
  `modernize_rulings.tsv` + `modernize_ops.tsv`; where the DTA editors
  supplied a `<corr>` for a `<sic>`, the correction lands here).
- `md_modernized_translated/` — Scholia's own English translation, 1:1
  sentence-parallel to the modernized German (clean-room; see the
  `hegel1-translate` skill for the independence gate).

## Markup conventions

- `_word_` — antiqua in the print. Hegel sets the surrounding text in Fraktur
  and switches to antiqua for **emphasis**, so unlike kant1/kant3 this is not a
  Latin/foreign-word marker.
- `<i>word</i>` — the same antiqua, where the print italicises only part of a
  word (`<i>Selbſt</i>bewuſstseyn`, `welche<i>s</i>`). Markdown reads `_` as a
  delimiter only at a word boundary, so these spans need a tag; both spellings
  render to the same `antiqua` span and mean the same thing.
- `***word***` — gesperrt (letter-spaced emphasis).
- `**word**` — Kapitälchen (small caps).
- `{{{ 22 }}}` — **1807 first-edition** page marker (`orig1807`, block). Roman in
  the Vorrede, Arabic in the body.
- `{{ 22 }}` — **Gesammelte Werke Bd. 9** page marker (`gw9`, inline), placed by
  the converter from `gw9_markers.tsv` (sentence-level concordance).
- `---` on its own line — a section separator (the print's horizontal rule).
- `[word]` — editorially supplied text, absent or illegible in the print.

Compositor lineation (`<lb/>`), catchwords, signature marks and running heads
carry no meaning in a reflowable reader and are dropped.

## Headings

Each file opens with front matter (`position`, `label`, `depth`, and `page_1807`
except for the pageless Vorrede) followed by a single `##` heading. Position 3,
`Einleitung`, is the one heading Scholia supplies: the 1807 print has none there
and the TEI records that as an empty `<head/>`.

The authoritative TOC (labels, depth, 1807 page, slug) lives in
`packages/common/src/hegel1/`. `scripts/hegel1_verify.sh` checks this layer —
file set, front matter, paragraph counts, page-marker sequence, TEI leakage,
hyphen rejoins, long-s survival — against the source TEI and against its own
embedded copy of the table, so that an error shared by the converter and the
TOC module cannot pass by agreeing with itself. `--toc` swaps in another table.
