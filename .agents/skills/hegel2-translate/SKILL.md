---
name: hegel2-translate
description: Clean-room English translation of Hegel's Wissenschaft der Logik (hegel2) — control-gate rules, terminology standard, emphasis and sentence-parity conventions. Use for any work on assets/hegel2/curated/md_modernized_translated.
---

You are a top translator and expert in German Idealism, producing Scholia's
own English translation of Hegel's *Wissenschaft der Logik* (GW text: 1832
Seyn + 1813 Wesen + 1816 Begriff).

Source: `assets/hegel2/curated/md_modernized` ONLY (the modernized German
reading layer). Target: `assets/hegel2/curated/md_modernized_translated`,
same 274 files, same German filenames (kant3-style: the translated file's
front-matter label is the English authority; there is no toc_en table).

The goal is a translation that is **accurate, modern, and demonstrably
independent of every existing translation**.

## Translation independence (non-negotiable)

Inherited from hegel1 (read `.claude/skills/hegel1-translate/SKILL.md` for
the full history; the kant3 incident showed clean-room retranslation still
fails from training memory). The short form:

1. **Clean room.** The control translations in `assets/hegel2/control/`
   (Miller 1969, Di Giovanni 2010) must NEVER be in a translating agent's
   context — not as reference, not as style guide, not to check a term.
   Controls are for QC/orchestrator sessions only, which write no
   translation prose.
2. **From-memory leakage is the primary risk, not copying.** The Science
   of Logic is Di Giovanni's home turf and Miller's rendering is among the
   most-quoted in the literature; assume the model can reproduce both
   verbatim and gate accordingly.
3. **Post-hoc gate, per control:**
   `python3 .claude/skills/hegel2-translate/overlap_gate.py <file>`
   measures against each control **separately** (`--spans K` prints the
   matched runs for remediation). PASS = <3% word-8-gram overlap AND no
   common run ≥ 15 words, against every control. Caveat: the Miller
   control on hand is an abridged text (~109k words); Di Giovanni is
   complete.
4. A flagged run is re-rendered fresh from the German (the orchestrator
   passes the flagged text from OUR file, never from the control), then
   the file is re-gated; replacement sentences can newly converge —
   iterate to PASS.
5. **Do not paraphrase away overlap.** Provenance is what counts; fresh
   generation from the German is the only clean path.

## Terminology standard

`assets/hegel2/curated/translate_terms.tsv` is authoritative — key-term
policy: **defer to Di Giovanni's glossary** wherever he rules (his term
choices, never his prose). Headline choices: Begriff = **concept** (never
"Notion"), aufheben = sublate, Sein = being, Dasein = **existence**,
Existenz = **concrete existence**, Schein = shine, Schranke = restriction,
Grenze = limit, quantitative Verhältnis = ratio, Anzahl = amount,
Urteil = judgment, Schluss = syllogism, Zweck = purpose, Einzelheit =
singularity, Wirklichkeit = actuality, Wechselwirkung = reciprocity.
English node labels come from `assets/hegel2/curated/translate_labels_en.tsv`
(one row per position) — the label table is part of this skill's contract.

Terminology-locked renderings (chapter titles, fixed formulae like "in and
for itself", "being-for-itself") legitimately converge with the controls;
runs < 15 words from locked terms are expected and pass.

## Conventions

- **1:1 sentence ratio with the German.** Never split or merge sentences;
  if impossible, stop and note the case. `md_prose_to_struct --corpus
  hegel2 --translation` enforces parity per block.
- **Emphasis carries exactly.** `_…_` renders as `_…_` on the
  corresponding English words; intraword `<i>Ansich</i>seins` maps to the
  corresponding English element (`being-<i>in-itself</i>`); where the
  English word does not decompose, emphasize the whole word.
- **Page markers `{{{ 21.68 }}}` stay**, at the position in the sentence
  closest to where the German has them. Em-dashes (–/—) as in the German.
  `[supplied]` brackets carry.
- **Footnotes carry:** `[^1]` refs at the corresponding position, `[^1]:`
  definition blocks after the same paragraph.
- **`+ ` display lines** (set-off propositions, quote couplets) stay
  `+ `-prefixed, one line each. In-file `## ` headings are translated.
- Front matter: same `position`/`depth`/`page_gw`; `label:` from the
  English label table.
- **Splitter glue traps (English side):** never end an English sentence on
  bare "I.", on a word ending "…lie."/"…eg.", or on "…sect." (intersect,
  dissect, bisect) — restructure or append a faithful trailing word. Fix
  the English, not the splitter. INVERSE CASE: when the German splitter
  itself under-splits (a capital-letter formula like "A ist A." merges
  with the next sentence, since a single capital + period reads as an
  initial), reproduce the same under-split in English (end on bare "A.")
  rather than restructuring away from it — parity mirrors the German's
  actual splits, artifact or not.
- No translator's notes or synthesized apparatus: the reader renders
  curated markdown only.
- **Spell out German abbreviations in English** (`z. B.` → "for instance",
  `d. h.` → "that is", `u.s.f.` → "and so forth", `z. E.` → "say") rather
  than `e.g.`/`i.e.`/"etc." — no abbreviation period can then create a
  false sentence boundary for the parity-enforcing splitter.
- **Citations live inside parentheses, and parentheses never split.**
  hegel2 sets `paren_protected_splits` (md_prose_to_struct corpus knob,
  both editions): no sentence boundary inside a parenthesis or directly
  before one. Citation apparatus is therefore parenthesized — the German
  side via the converter's `CITATION_PARENS` table (one flat paren per
  run), the English mirroring the same parens — and its abbreviation
  periods stop mattering: inside the parens both languages are free-form,
  so no verbatim-carry gymnastics are needed. Only citations OUTSIDE
  parens still interact with the abbreviation tables (the tables carry
  `resp.`, `lect.`, `philos.`, `folg. Anm(erkung)` for the prose cases
  that cannot be parenthesized). German `W. z. E.` (QED) renders `Q.E.D.`
  (272 precedent).
- **Footnote definitions have their own parity check** at import time
  (struct_to_db compares per-footnote sentence counts, which the
  block-level parser check does not cover). German honorifics `Hrn.`,
  `resp.`, `Aufl.` are UNprotected in the German table and artifact-split
  mid-phrase, while natural English renderings (`Mr.`, `Prof.`) ARE
  protected and merge — recast the English into a genuine sentence
  boundary nearby rather than importing the honorific. Verify with the
  importer's `--dry-run` against a scratch DB when in doubt.
- Mathematical expressions (`2/7`, `xᵐ−1=0`, series) carry verbatim.

If you encounter odd cases worth remembering, add them to this skill.
