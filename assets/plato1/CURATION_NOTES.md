# plato1 curation notes

## Provenance

Greek text: John Burnet, *Platonis Opera*, Tomus IV (Oxford: Clarendon Press,
1905), as transcribed in the Perseus Digital Library's `canonical-greekLit`.

- File: `data/tlg0059/tlg030/tlg0059.tlg030.perseus-grc2.xml`
- Repository: https://github.com/PerseusDL/canonical-greekLit
- Commit: `91595f89e15b4d3000cd93efcf8990720c8be2b9` (2026-07-20)
- Local copy: `assets/plato1/raw/` (gitignored)

Burnet's words and his Stephanus milestones are public domain and were taken.
Perseus's own contributions were discarded and the structure re-derived: their
`resp="perseus"` section divisions, `<said who>` speaker attribution, `<bibl>`
identifications and URNs. Their `ed="P"` paragraph milestones were used as a
drafting seed for paragraphing only, which the curated markdown then supersedes.

## The converter is a one-time bootstrap

`packages/plato1_tei_to_md` generated `curated/md_modernized/` once. Per DEC-16
it is not run again: the markdown is the editorial surface from that point, and
re-running would discard hand corrections. Fixes go into the markdown, never
back through the converter.

## Corrections to the transmitted text

Two places where the Perseus TEI we ingested is corrupt. Both errors are
**upstream, in the Perseus digitisation** — verified against the raw XML in
`raw/` — and neither was introduced by our converter. Both are recorded here
because they are the only points where the curated Greek departs from the file
it was built from.

### 521e — a misplaced full stop

Perseus prints a full stop between `φθίσεως` and `ἐπιστατεῖ`, leaving
`ἐπιστατεῖ` as a one-word sentence cut off from the genitives it governs.
`ἐπιστατεῖ` takes a genitive object, and its genitives sit on the other side of
the stop, so the reading cannot stand. The curated text reads
`σώματος γὰρ αὔξης καὶ φθίσεως ἐπιστατεῖ.` as one sentence.

This changed the sentence counts, which GOLDEN.md records.

### 585b — γῆς for τῆς

Perseus prints `κενότης ἐστὶ γῆς περὶ ψυχὴν αὖ ἕξεως` — "an emptiness *of
earth*". `γῆς` is the noun γῆ, where the sense requires the article `τῆς`
reaching forward to `ἕξεως`. Without it, `ἕξεως` has nothing to attach to and the
clause does not construe.

The warrant is the parallel two turns earlier, which builds the identical frame
for the body: `κενώσεις τινές εἰσιν τῆς περὶ τὸ σῶμα ἕξεως`. The passage argues
body-filling against soul-filling, so the symmetry is the argument. Γ and Τ are
also an easy one-letter confusion in a manuscript hand or in OCR.

Treated as a transcription slip rather than an editorial emendation: the question
is not what Plato wrote but whether this is Burnet's reading or a mistyping of
it, and a non-construing reading printed without an obelus or apparatus note is
not plausibly Burnet's. **Residual uncertainty:** this was inferred, not
confirmed against a printed OCT — the local `canonical-greekLit` checkout holds
the byte-identical Perseus file, so it is not an independent witness. A scan of
the OCT page or a transcription from a different family would settle it.
Reverting is one character.

## Known items for the editorial pass

- **Three literal `>` characters** appear in Burnet's transcribed text, a
  Perseus artefact, and were passed through verbatim rather than silently
  stripped. A `>` opening a line is markdown blockquote syntax, so each needs a
  decision.
- **The 61 quotation citations** are not yet written. `derived/quotations_report.tsv`
  lists every quotation site with Perseus's own identification as a starting
  hint; each must be verified against the actual Homer, Hesiod, Pindar or
  Aeschylus text before being written as a footnote. Perseus's strings are not
  shipped (they are inconsistent — `Hes. WD 232` where the standard abbreviation
  is `Hes. Op.`, and one cites a Loeb volume Burnet never used).

## Editorial conventions

- Burnet's brackets are kept: `⟨ ⟩` for what he supplied, `[ ]` for what he
  athetized; `<sic>` readings stand as printed.
- A truncated quotation opens with `…` (U+2026). Never three ASCII dots — the
  Greek splitter would read those as a sentence terminator.
- Quoted verse is one `+ `-prefixed run per line.
- Division headings are Scholia's English titles, in both editions.
