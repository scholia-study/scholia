# hegel3 — Wissenschaft der Logik, Das Seyn (1812) (curated MD)

The first-edition Doctrine of Being: *Wissenschaft der Logik. Erster Band.
Die objektive Logik* — its first book, *Das Seyn* — Nürnberg: Johann
Leonhard Schrag, 1812. Hegel rewrote this book completely for the 1832
second edition (which the *Wissenschaft der Logik* corpus, hegel2,
follows), making the 1812 text a work of its own: the original opening of
the Logic as the system first had it.

Source: Deutsches Textarchiv
(<https://www.deutschestextarchiv.de/hegel_logik0101_1812>) — a TEI P5
double-keyed transcription (no OCR) of the first edition.

Licence (this statement takes precedence over the blanket `assets/`
licence, per the repository NOTICE): the DTA text is **CC BY-SA 4.0**, and
the German text layers here plus the editorial table that produces them
(`md_reviewed/`, `md_modernized/`, `modernize_rulings.tsv`) carry that
same share-alike licence; Scholia asserts no further rights in them.

The converter is `hegel1_tei_to_md` — the same DTA `att.linguistic` TEI
shape serves both Hegel corpora (the `<w>` tokens span `<lb/>`, which is
what lets a hyphenated line break be rejoined on word boundaries rather
than by guesswork). The layers are generated once and thereafter treated
as curated; should a table-driven regeneration ever be needed, the exact
invocations (verified byte-stable against the checked-in layers) are:

    hegel1_tei_to_md assets/hegel3/raw/hegel_logik0101_1812.xml \
        assets/hegel3/curated/md_reviewed --layer reviewed \
        --reports assets/hegel3/derived --page-key page_1812 \
        --promote-head "Erstes Buch. Das Seyn" --gw-markers /dev/null
    hegel1_tei_to_md assets/hegel3/raw/hegel_logik0101_1812.xml \
        assets/hegel3/curated/md_modernized --layer modernized \
        --rulings assets/hegel3/curated/modernize_rulings.tsv \
        --ops assets/hegel3/curated/modernize_ops.tsv \
        --page-key page_1812 --promote-head "Erstes Buch. Das Seyn" \
        --gw-markers /dev/null

(`--gw-markers /dev/null` because the 1812 volume has no `{{ }}`
concordance system.) The one structural intervention: the DTA nests *Erstes Buch*
inside the *Logik* division, while the print's own contents list it at
top level — `--promote-head` lifts the Buch and its subtree one level to
match the print.

One label emendation lives only in the curated files, not the converter:
node 010's body head prints bare "A.", but the print's own Inhaltsanzeige
reads "A. Seyn" — the label and heading follow the contents page
("A. Seyn" / "A. Sein" / "A. Being"; the slug and filename keep the body
head's bare `a`). A regeneration would emit "A" again and must re-apply
this in all three layers plus the `common::hegel3` TOC tables.

Three curated layers, 120 files each (one per TOC node, `NNN_slug.md`):

- `md_reviewed/` — 1812 orthography exactly as printed, long-s (`ſ`)
  preserved, `<sic>` readings kept uncorrected. The diplomatic layer.
- `md_modernized/` — modern German orthography, driven by
  `modernize_rulings.tsv` (seeded from hegel1's human-decided table and
  layer pairs, hegel2's table, and the DTA CAB readings; where the DTA
  editors supplied a `<corr>` for a `<sic>`, the correction lands here)
  plus `modernize_ops.tsv` (grammatically compelled print-error repairs;
  the diplomatic layer keeps every error as printed, and uncertain
  readings stay unrepaired — see `translation_followups.md`).
- `md_modernized_translated/` — Scholia's own English translation
  (clean-room, gated against the 1832 translations as proxy controls —
  no published English translation of the 1812 first edition existed
  before this one). Kant3-style: shared German filenames, front-matter
  labels are the English authority
  (`translate_labels_en.tsv` is the label table). Unlike the German
  layers (CC BY-SA 4.0 per the DTA terms), the translation is Scholia's
  own work under the regular assets licence (CC BY-NC-ND 4.0).

## Markup conventions

Same as hegel1 (see `assets/hegel1/curated/README.md`), plus what the
1812 volume adds:

- `[^1]` / `[^1]: …` — the four authorial footnotes, numbered per file.
- `2/7` — the stacked fractions of the ratio discussions, as plain text.
- `[…]` — a char-level lacuna in the damaged copy (six sites); the DTA's
  supplied readings around them carry the usual `[word]` brackets.
- `{{{ 22 }}}` — **1812 first-edition** page marker (`orig1812`, block,
  the citation default). Roman in the front matter (the DTA's supplied
  brackets around unprinted numbers are stripped: `[III]` → `III`),
  Arabic in the body; the misprinted page 93 (printed "95") carries its
  corrected number.
- Repeated heading labels (the many *Anmerkungen*) get a counter suffix
  in the slug (`anmerkung_2`, …), never in the label.

The authoritative TOC lives in `packages/common/src/hegel3/`; the
modernized labels there are the converter's own renderings, byte-for-byte.
The GW 11 half of the Digitale Hegel-Edition (see hegel2) is this text in
its GW-edited form; a word-level comparison lives at
`assets/_unassigned/hegel/reports/gw11_1812_diff_sites.tsv` for a later
review pass, and could seed a GW 11 concordance if hegel3 ever grows a
`{{ }}` system.
