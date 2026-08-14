# hobbes1 md_modernized — provenance and review notes

The modernized layer was produced 2026-08-11 from `md_reviewed` (EEBO-TCP
A43998, 1651 first edition) by five parallel modernization passes plus a
validation/harmonization pass. ~11,700 word-level edits across 58 files.
Machine-enforced invariants held throughout: front matter, page markers,
margin-note token count/order, block structure, sentence-boundary
punctuation, emphasis placement, capitalization.

## Corpus-wide rulings applied

- **Spelling only, never the word**: u/v–i/j normalization, `vv→w`,
  doubled/Latinate endings (naturall→natural, publique→public,
  soveraign→sovereign), `-nesse→-ness`, `shew→show`, `-t`/syncopated
  preterites (exprest→expressed, entred→entered). Archaic morphology and
  diction kept: hath/doth/-eth, thou/thee, spake, fromward, middest,
  bewray, apparence, unsufficient, `-our` forms as printed.
- **No harmonization of variants Hobbes himself mixes**: both `injust` and
  `unjust`, both `-our`/`-or` (honour/honor), both cognisance/cognizance
  stay as printed per occurrence.
- **humane → human** corpus-wide (29×): 1651 spelling of *human*; keeping
  it invites the modern "compassionate" misreading.
- **Daemon family → Demon** (Demonology, Demoniacs, Demons);
  `Lacedaemonians` (the Spartans) untouched.
- **phantastical → fantastical**.
- **Latin/Greek verbatim**, scripture citation formats as printed
  (`1 Sam. 8. 3.`, `ver.`), Counsell/Councell distinction preserved
  (counsel = advice, council = assembly).
- **OCR/print corruptions** (not 1651 spellings) fixed silently in this
  layer: ~90 items of the `aud→and`, `nothiug→nothing`, `Ptoiemy→Ptolemy`
  class, macron contractions expanded (Testamēt→Testament), run-together
  words split. The diplomatic layer keeps them as transcribed.
- **Gap resolution**: the 383 TCP illegibility markers were resolved from
  context or verified against 1651-derived control transcriptions
  (Wikisource page scans, Gutenberg #3207); Greek restored in polytonic
  script only where a control shows it in script, never reconstructed
  from transliterations. Full audit trail in
  `assets/hobbes1/derived/modernize_log_agent{1..5}*.tsv` (gitignored;
  regenerate-and-review before discarding).
- **TOC labels** are editorial metadata: the three artifact-bearing labels
  (`Mis•…ry`, `s•…cond`, `RÉDEMPTION`) are resolved by ruling in the
  converter (`LABEL_RULINGS`); the diplomatic heading lines keep the gap
  displays.
- **Per-layer labels**: each layer's front matter carries its own label —
  diplomatic in `md_reviewed` (validated against `common::hobbes1::toc`),
  modernized in `md_modernized` (validated against `toc_mod`, derived from
  the modernized heading lines). Reader TOC labels and node slugs use the
  modernized forms, like every other corpus.

## QA sweep (2026-08-13)

Full-corpus dictionary scan (en_GB+en_US wordlists), stitch detector
(out-of-dictionary tokens that split into two dictionary words), mixed-case
scan, and a dictionary-invisible-archaism scan. Fixed: 33 drop-cap
transcription artifacts normalized (`THe→The`, `BEsides→Besides`,
`NAture→Nature`, …— typographic, not authorial capitals), `inConscience →
in Conscience` (TCP word-join, margin note in XV), `imagin → imagine`,
`commest → comest`, `now adays → nowadays`, `aswell → as well`,
`dye → die` (6×, dictionary-valid so invisible to spell sweeps).
Deliberately NOT changed after inspection: `_Here-alt_` (Hobbes's Dutch
etymology gloss for *Herald*), `White _ness,_ Round _ness_` (the abstract-
suffix passage), `limetwigs` (1651 prints it joined), `Thensa/Thensam`
(Roman ceremonial chariot, Latin), errata-quoted forms (`writt`,
`signied`, …), accented Latin (`vivâ voce`, `definitivè`), `mens`/
`anothers` (possessives without apostrophe — apostrophes are frozen
punctuation; see open items).

## Later rulings applied (2026-08-13)

- **`then` → `than`** where comparative (63×, every occurrence hand-
  classified in context; 277 temporal `then` remain untouched — consequence
  clauses like "if more, then it is the Assembly" and the attributive
  "the then Preachers" were the trap cases).
- **Possessives without apostrophe** → apostrophized (179×): `mans→man's`,
  `mens→men's`, `anothers→another's`. This and the comparative pass are the
  two sanctioned exceptions to the frozen-punctuation rule; neither touches
  sentence-boundary punctuation, so layer parity is unaffected.

- **`▪` damaged-punctuation glyphs** (88×, TCP `char:punc`): resolved in the
  modernized layer against the Wikisource page-scan transcription at
  letter-exact positions — 32 commas, 16 semicolons, 8 periods, 3 colons,
  29 spurious (broken type/ink beside surviving punctuation; deleted, one
  as a space where deletion would weld words). Zero unresolved. The two
  main-flow period resolutions carry `|||` forced-splits in md_reviewed.
  Audit trail: `derived/modernize_log_agent5_punc.tsv`. The diplomatic
  layer keeps all 88 `▪` verbatim.

- **Garbled 1651 Greek corrected** (10×): the compositor's errors flagged in
  the restoration logs print corrected in the modernized layer (λόλος→λόγος,
  μακαεισμός→μακαρισμός, πιστεύω εἰς/αὐτῷ, ἀποσυνάγωγον, προεστῶτες,
  παραγγελίας ἐδώκαμεν, ὑπακούει, Δουλεία); the diplomatic layer keeps them
  as printed.
- **Printed Contents and Errata excluded** (with the bookplate, in the
  converter's `front_div_excluded` ruling): print apparatus with no reading
  value — Scholia generates its own TOC, and the errata's corrections live
  on in the modernized layer. 56 nodes; positions 004+ renumbered. This
  also retired the one unresolvable gap (an errata line-reference).
- **1651 ordinal style modernized** (17×): "in the 35. Chapter" → "in the
  35th Chapter" (likewise Psalm/Epistle/Cha.; "the 70. Elders" → cardinal
  "70 Elders"). The sentence splitter gained an ordinal-reference
  suppression (`ORDINAL_REF_NOUN_RE`, English nouns only) so the diplomatic
  layer's "35. Chapter" no longer false-splits either; scripture verse
  citations and Hobbes's numbered enumerations keep their splits.

- **Strong-colon sentence splitting** (2026-08-14): Hobbes points with
  period-strength colons, so hobbes1 uses `split_sentences_en_strong_colon_forced`
  (corpus flag `strong_colon_splits`): a capitalized word after ": " begins
  a new sentence (661 sites); lowercase continuations (579) stay joined.
  Other corpora are untouched. Sentence p95 dropped from 146 to 82 words,
  max from 777 to 211. The six diplomatic-layer `|||` forced splits now
  live in the converter's `FORCED_SPLIT_RULINGS` (regeneration-stable):
  two gap-hidden boundaries, two `▪`→period, two `▪`→colon-before-capital.

## Open items for reviewer decision

1. **`Circaean`** (XLV): kept — the 1651 form for the Circensian games;
   `ae→e` would wrongly yield "Circean" (of Circe).
