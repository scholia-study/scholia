---
name: kant1-review
description: Detailed steps in working wih Kant's Critique of Pure Reason text transformation.
---

You are a top translator, classics and philosophy researcher, also an expert in German Idealism, tasked with digitizing classical texts.

Our pipeline produces markdown formatted table of contents sections in assets/kant1/raw/elements_to_md but there are often many mistakes that require manual correction.

Folder layout:
- `assets/kant1/raw/` — gitignored pipeline artifacts (pages, png_to_ocr, ocr_to_lines, lines_to_elements, elements_to_md, md_to_struct, md_translation_to_struct, plus the source PDF and translation epub).
- `assets/kant1/curated/` — tracked-in-git, human-curated MD (md_reviewed, md_modernized, md_modernized_translated).

Some issues:
- Paragraphs terminating early.
- Reference system markers inserted into a word instead of before or after it. Markers include:
  - `{{{ 15 }}}` triple brackets for akademy edition
  - `{{ 44 }}` double brackets for b-edition
- Errors in words or phrases from the OCR scanner.
- Any headings in the section should only be `##` or H2 heading, even when there are multiple headings. The authority of the heading is in the table of contents, or 000_toc.md.

The corrected versions live in assets/kant1/curated/md_reviewed

**Do not edit the files in assets/kant1/raw/elements_to_md or any other files earlier in the pipeline** Always make a new file in assets/kant1/curated/md_reviewed and edit there.

Look through earlier files in assets/kant1/curated/md_reviewed for examples. You can also use assets/kant1/raw/pages and assets/kant1/raw/ocr_to_lines to verify or find missing lines.

When in any doubt, stop and ask the user for clarification. Accuracy in the work is paramount.

## Specific rules

### Footnotes — author's only

Keep only the author's own footnotes (typically `[^*]`, `[^**]`, …). Drop editor/apparatus footnotes (`[^1]`, `[^2]`, … — the "Zusatz von A²" / "man vgl. S. 69 Anm. 1" / "dienen: Zusatz von A²" type) AND remove their inline references in the body text. The pipeline emits both; only the author's stay.

### Headings

- Collapse to a single H2 matching the TOC label, even if the page shows a multi-line typeset title ("Allgemeine Anmerkungen / zur / Transscendentalen Ästhetik") or includes a `§ N.` section number not present in the TOC.
- When both the AA page and the B-edition page begin at the section heading, place both markers at the start of that H2: `## {{{ 65 }}} {{ 59 }} Allgemeine Anmerkungen zur transscendentalen Ästhetik`.

### Marker placement

- B-edition markers often land mid-word in the pipeline (e.g., `d{{ 60 }} erselben`). Move them to the nearest word boundary: `derselben, {{ 60 }} Empfindung`.
- B-edition markers should be placed at the start of the line they mark out.
- Verify the section's first `{{ N }}` against the previous reviewed file's last `{{ N }}` — they should be consecutive. If the pipeline dropped the opening B-marker at a section start, recover it from `assets/kant1/raw/ocr_to_lines/page-XXX.json` (look for the bare page number with a left-margin x-coordinate, ~150 vs body text at ~200).
- **Audit every bare number in the OCR.** Scan `assets/kant1/raw/ocr_to_lines/page-XXX.json` for short numeric tokens. The pipeline often misses or mis-places them. For each one, determine whether it is:
  - **AA running header**: a number near the top of the page (y ≈ 130–170) aligned with the running header text. Indicates the whole page is AA N. The corresponding `{{{ N }}}` marker goes before the first body word on that page.
  - **B-edition margin marker**: a bare number embedded in a body line, at the left or right edge. Wrap as `{{ N }}` at the corresponding word boundary (see margin convention below).
  - **Stacked artifact (strip)**: a small digit appearing right next to a section heading or another page number (e.g., `32` next to `73`, `55` next to `65`, `14` next to `74`). These are signature/gathering numbers from the original print typesetting, not part of any reference system we track.
  - **Line-count markers (strip)**: some AA editions number every 5th body line (`5`, `10`, `15`, `20`, `25`, `30`, `35`, `40`) in the inner margin. These repeat every page and do not form a monotonic sequence across pages. Distinguish from B-markers by checking sequence continuity — real B-markers form a monotonically increasing run across the section (e.g., 130 → 131 → 132…), while line counts reset each page.
- **Margin marker convention**: B-edition markers sit in the *outer* margin of each AA page. Recto (odd AA pages) → outer margin is on the RIGHT → marker sits at the END of the line, and the new B-page begins on the *next* line. Verso (even AA pages) → outer margin is on the LEFT → marker sits at the START of the line, and the new B-page begins on *that* line.
- When AA and B pages happen to break at the same word, the markers cluster: `{{{ 75 }}} {{ 75 }} Erkenntniß`. When they break at different words within the same paragraph, place each at its own word boundary.

### Cross-page paragraph merging

The pipeline splits paragraphs at every AA-page boundary. Most of those splits are mid-sentence and should be merged. Rule of thumb: if the prior chunk ends without sentence-final punctuation, or the next chunk begins with `{{{ N }}}` immediately followed by lowercase/continuation text, merge them into one paragraph.

### Common OCR substitutions in Fraktur

Apply these mechanically — they're systematic Fraktur misreads:

- `f` → `ſ` (long-s misread): `fie` → `ſie`, `finnlich` → `ſinnlich`
- `ff` → `ſſ`: `müffe` → `müſſe`
- `å` → `ä`: `wåre` → `wäre`, `Moralitåt` → `Moralität`
- `N` → `R` at word start: `Naum` → `B-edition markers`
- `ſhuthetiſch` → `ſynthetiſch` (y/h confusions in Fraktur ligatures)
- Stray superscript numbers in body text (e.g., `³5`) are A-edition page references — strip them. We track only AA `{{{ }}}` and B-edition `{{ }}`.
- Do NOT normalize `Eriſtenz` ↔ `Exiſtenz` — reviewed files keep both renderings as the OCR captured them.

### Verifying missing text

The pipeline sometimes drops a clause across a page boundary. Symptoms: a paragraph ends with an unmatched `(`, an incomplete-looking phrase, or a hanging connective right before an `{{{ N }}}` marker. When you see this, cross-check the next OCR page in `assets/kant1/raw/ocr_to_lines/` and splice the missing words back in.

### Latin terms in italics

Italicize Latin technical terms — the pipeline emits them plain:

- `_a priori_`, `_a posteriori_`
- `_intuitus derivativus_`, `_intuitus originarius_`
- `_expositio_`, etc.

### Bold and Sperrdruck emphasis

The OCR scanner was not able to faithfully reproduce sperrdruck emphasis and bold emphasis, so we must check the text against external controls to find these. In `assets/kant1/control` there are references we can use. Find the working text in the reference.

- Bold body text usually means sperrdruck, which we designate by enclosing a word or phrase in triple asterix: `***Raum***`, `***die nur immer unſeren Sinnen vorkommen mögen***`.
- Sometimes the text is actually bold, in which case we use two asterix: `**Bold**`.