# hegel2 — Wissenschaft der Logik (curated MD)

The complete *Wissenschaft der Logik* as modern scholarship reads it: the
1832 second edition of the Doctrine of Being together with the 1813
Doctrine of Essence and the 1816 Doctrine of the Concept, following the
text of the *Gesammelten Werke* (GW 21, GW 11, GW 12).

Sources — both credited equally; Scholia's version is constructed from the
two together and asserts no rights in the German text:

- **Digitale Hegel-Edition** by Giuliano Infantino
  (<https://www.hegeledition.com>, database CC BY 4.0) — transcriptions of
  the public-domain primary text of the Gesammelte Werke volumes, with GW
  pagination and line breaks.
- **Deutsches Textarchiv** (<https://www.deutschestextarchiv.de>, CC BY-SA
  4.0) — TEI P5 double-keyed transcriptions (no OCR) of the original
  prints: Bd. 1,2 (Nürnberg 1813, `hegel_logik0102_1813`) and Bd. 2
  (Nürnberg 1816, `hegel_logik02_1816`), used as independent witnesses to
  verify and correct the text where the volumes overlap.

Licence (this statement takes precedence over the blanket `assets/`
licence, per the repository NOTICE): the German text layers here and the
editorial tables that produce them carry **CC BY-SA 4.0** (the stricter of
the two source licences), attribution: Giuliano Infantino, Digitale
Hegel-Edition, hegeledition.com; Deutsches Textarchiv. Scholia asserts no
further rights in them. The English translation
(`md_modernized_translated/`, when it exists) is Scholia's own work and
remains under the regular assets licence (CC BY-NC-ND 4.0).

## What is included

- GW 21 (1832 *Die Lehre vom Seyn*, incl. both Vorreden and the
  Einleitung), GW 11 pp. 233–409 (1813 *Die Lehre vom Wesen*), GW 12
  pp. 5–253 (1816 *Die Lehre vom Begriff*).
- **Excluded**: the printed Inhaltsanzeigen (tables of contents), and the
  Nachlass appendices GW prints after the texts — GW 21's "Beilage"
  (pp. 385 ff.) and GW 12's "Beilagen" (pp. 255 ff., "Zum Erkennen",
  "Notiz zu Leibniz", "Notiz zu Fries", …). These are Hegel's manuscript
  notes, not part of the published work.

## Curated tables

- `gw_dta_rulings.tsv` — adjudicated GW-vs-DTA word differences (below).
- `page_joins.tsv` — whether a paragraph continues across each GW page
  break, witness-derived (DTA for 1813/1816, the Werke text for 1832);
  `uncertain` rows were decided by prose sense.
- `modernize_rulings.tsv` — word surface → modern spelling, composed from
  hegel1's human-decided layers, the DTA CAB readings, and hand rulings
  for the 1832-only vocabulary; drives `md_modernized`.

## Reconciling the two sources

`gw_dta_rulings.tsv` records every word-level difference between the
hegeledition transcription and the DTA witness (1813 and 1816 parts;
~1% of tokens, most of it the DTA faithfully carrying print errors that
GW emends). Each site carries a ruling:

- `keep_gw` — GW's reading stands (typically a deliberate GW emendation
  of a misprint in the original edition).
- `use_dta` — the DTA reading is applied (a transcription slip in the
  digitization).

and a `certainty` column: `uncertain` marks substitutions where the
printed GW volume would be needed to adjudicate (we lack it); per the
standing default these take the DTA reading and stay logged for a later
pass. The converter refuses to emit if the built text still disagrees
with a witness at an unruled site. The 1832 Being has no DTA witness;
the Zeno.org Werke text (`assets/hegel2/control/wdl.json`) serves as its
collation control in the verify gate only.

## Layers

Three curated layers, one file per TOC node (`NNN_slug.md`):

- `md_reviewed/` — the GW text as transcribed: Hegel's orthography as GW
  prints it (`Seyn`, `zweyten`, `Verhältniß`; no long-s — GW sets antiqua).
- `md_modernized/` — modern German orthography (rulings-table driven,
  hegel1-style).
- `md_modernized_translated/` — Scholia's own English translation (later;
  clean-room, gated).

## Markup conventions

- `_word_` — emphasis (GW's italics, which render the print's Sperrdruck).
  Intraword spans use `<i>…</i>`.
- `{{{ 21.15 }}}` — **Gesammelte Werke** page marker (`gw`, block): GW
  volume and page joined by a dot, one system across the whole book
  (`21.x` Seyn, `11.x` Wesen, `12.x` Begriff). This is the only reference
  system and the citation default ("GW 21.15"), matching hegel1's GW
  margin convention (`9.53`).
- `[^1]` / `[^1]: …` — authorial footnotes, numbered per file in document
  order (the print's `*)`-style glyphs are not preserved).
- `---` on its own line — a section separator.
- Mathematical expressions (Quantity chapters) are kept as inline text
  exactly as the source transcribes them.

## Headings and TOC

The book's TOC mirrors the printed GW tables of contents (every entry,
all levels — chapters, A/B/C, a/b/c, Anmerkungen), arranged under the
work's own two parts:

1. *Erster Theil. Die objective Logik* (GW 21 title page), containing the
   Vorreden, Einleitung, *Erstes Buch. Die Lehre vom Seyn* (GW 21) and
   *Zweytes Buch. Die Lehre vom Wesen* (GW 11 half-title, p. 233).
2. *Zweyter Theil. Die subjective Logik oder die Lehre vom Begriff*
   (GW 12 title page; Hegel's 1832 re-framing calls the 1816 "Zweyter
   Band" the second Theil).

Each file opens with front matter (`position`, `label`, `depth`, and
`page_gw` except for pageless title nodes) followed by a `##` heading.
Node slugs derive from labels; repeated labels (the many "Anmerkung"
nodes) are disambiguated with a counter in the slug.

The authoritative TOC (original labels, modernized labels, depth, GW
page, slug) lives in `packages/common/src/hegel2/`.
