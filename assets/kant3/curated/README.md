# kant3 — Kritik der Urteilskraft (curated MD)

Source: Project Gutenberg #55925 (<https://www.gutenberg.org/ebooks/55925>) —
Kant's gesammelte Schriften, **Akademie-Ausgabe Band V**, a born-digital
transcription (no OCR). `md_reviewed/` was produced once from the HTML.

Three curated layers, 122 files each (one per TOC section, `NNN_slug.md`):

- `md_reviewed/` — original period orthography (Urtheilskraft, nothwendig). The
  diplomatic layer.
- `md_modernized/` — modern German orthography (Urteilskraft, notwendig). The
  reader's primary `text`/`html`.
- `md_modernized_translated/` — English translation (Scholia Community Edition),
  1:1 sentence-parallel to the modernized German.

## Markup conventions

- `***word***` — Sperrdruck (spaced emphasis in the original print).
- `**word**` — regular bold (rare).
- `_word_` — Latin / antiqua (e.g. `_a priori_`, `_sensus communis_`).
- `{{{ 203 }}}` — **Akademie-Ausgabe Band V** page marker (`aa_v`, block).
- `{{ 17 }}` / `{{ III }}` — **1790 first-edition** page marker (`e1790`, inline;
  Roman in the Vorrede, Arabic in the body).
- `[^*]`, `[^**]` — Kant's own footnote refs; defined `[^*]: …` at section end.
- `|||` — manual sentence split (forced boundary), rare.

The authoritative TOC (labels, depth, AA page, slug) lives in
`packages/common/src/kant3/{toc,toc_mod}.rs`; front matter is validated against it
by `md_prose_to_struct --corpus kant3`. See `KANT3_TEXTUAL_COMPROMISE.md` for fidelity notes.
