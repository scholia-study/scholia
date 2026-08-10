---
name: hegel1-translate
description: Clean-room English translation of Hegel's Phänomenologie des Geistes (hegel1) — control-gate rules, terminology standard, emphasis and sentence-parity conventions. Use for any work on assets/hegel1/curated/md_modernized_translated.
---

You are a top translator and expert in German Idealism, producing Scholia's
own English translation of Hegel's *Phänomenologie des Geistes* (1807).

Source: `assets/hegel1/curated/md_modernized` ONLY (the modernized German
reading layer — post-1996 orthography, errata applied).
Target: `assets/hegel1/curated/md_modernized_translated`, same 50 files, 1:1.

The goal is a translation that is **accurate, modern, and demonstrably
independent of every existing translation**.

## Translation independence (non-negotiable)

Inherited from the kant1/kant3 incidents — read the "Translation
independence" section of `.claude/skills/kant1-modernize-translate/SKILL.md`
for the full history. The short form:

1. **Clean room.** The three control translations in `assets/hegel1/control/`
   (Miller 1977, Baillie 1910, Pinkard 2018) must NEVER be in a translating
   agent's context — not as reference, not as style guide, not to check a
   term. Controls are for QC/orchestrator sessions only, which write no
   translation prose.
2. **From-memory leakage is the primary risk, not copying.** kant3's
   clean-room retranslation still failed 96/116 files against the real
   control from training memory. Miller's PhG is among the most-quoted
   philosophy translations in existence; assume the model can reproduce it
   verbatim and gate accordingly.
3. **Post-hoc gate, per control:**
   `python3 .claude/skills/hegel1-translate/overlap_gate.py <file>` measures
   against each control **separately** (`--spans K` prints the matched runs
   for remediation). PASS = <3% word-8-gram overlap AND no common run ≥ 15
   words, against every control. This gate is hegel1's own; the kant skill's
   gate is untouched and kant-only.
4. **Calibration (2026-08-09):** the published translations against each
   other: Baillie↔Miller 7.1% / run 35, Baillie↔Pinkard 2.5% / run 34,
   Miller↔Pinkard 3.3% / run 32 — ordinary prose, not quotations. Our bar is
   therefore *stricter than published practice*; expect remediation rounds.
   A flagged run is re-rendered fresh from the German (the orchestrator
   passes the flagged text from OUR file, never from the control), then the
   file is re-gated; replacement sentences can newly converge — iterate to
   PASS.
5. Baillie is public domain (J. B. Baillie d. 1940) — similarity to it is a
   quality problem, not a legal one, but the user wants dissimilarity from
   all three; the gate treats them equally.
6. **Do not paraphrase away overlap.** Provenance is what counts; fresh
   generation from the German is the only clean path.

## Terminology standard

`assets/hegel1/curated/translate_terms.tsv` is authoritative — key-term
policy: **defer to Di Giovanni** (Cambridge *Science of Logic* glossary)
wherever he rules; PhG-specific terms follow the table. Headline choices:
Begriff = **concept** (never "Notion"), aufheben = sublate, Einzelnheit =
singularity, Wesen = essence, Dasein = existence, Wirklichkeit = actuality,
Erkennen = cognition / Wissen = knowledge, Geist = spirit, Gestalt = shape,
Kraft = force, Sittlichkeit = ethical life. Rows marked DECIDE in the note
column are unresolved — do not translate affected passages before the user
has ruled; the resolved table is part of this skill's contract.

Terminology-locked renderings (chapter titles, fixed formulae like
"in and for itself") legitimately converge with the controls; runs < 15
words from locked terms are expected and pass. A title alone carrying a run
≥ 15 gets reworded label + heading together, kant3-style.

## Conventions

- **1:1 sentence ratio with the German.** Never split or merge sentences; if
  impossible, stop and ask, offering options. (`just struct hegel1` parity
  will enforce this once the corpus is wired into `md_prose_to_struct` —
  until then the ratio is on the translator's honor and checked at wiring.)
- **Emphasis carries exactly.** In hegel1, `_…_` is antiqua = emphasis (NOT
  Latin, unlike kant): render as `_…_` on the corresponding English words.
  Intraword antiqua `<i>Ansich</i>seins` maps to the corresponding English
  element: `being-_in-itself_`; where the English word does not decompose,
  emphasize the whole word and note the case in the file's QC notes.
  `***gesperrt***` and any `**…**` carry as-is.
- **Page markers `{{{ N }}}` stay**, at the position in the sentence closest
  to where the German has them. Em-dashes (—) as in the German. `[supplied]`
  brackets carry.
- Front matter: same `position`/`depth`/`page_1807`; `label:` will use the
  English label table (`common::hegel1::toc_en`, to be created with the
  resolved terminology) and filenames the English slugs, kant1-style.
- The German layer's editorial decisions (errata applied, print errors
  fixed, `Itzt` kept) are already in `md_modernized` — translate what it
  says; the This/Now/Here vocabulary per the terms table.
- No translator's notes or synthesized apparatus: the reader renders curated
  markdown only.

## Campaign learnings (2026-08-10, first full pass)

- **Splitter glue traps (English side).** `common::sentences::split_sentences_en`
  refuses to split after a bare capital ("… the pure I. Since …" — INITIAL_RE
  reads "I." as an initial) AND after any word ending in "ie."/"eg." (the
  `i\.?\s*e\.` / `e\.?\s*g\.` abbreviation regexes match "lie.", "die.").
  Hegel ends many German sentences on *Ich*; the first full build glued 15
  sentence pairs across 13 blocks this way. Never end an English sentence on
  bare "I." or on "…lie." — restructure ("the I in its purity", "the I that
  is its own", "reside" for *liegen*) or append a faithful trailing word
  ("the I itself", "as well", kant1-style). Fix the English, not the splitter.
- **Mechanical sweeps can create overlap.** Unifying "an other" → "another"
  produced a brand-new 22-word Miller run in a file that had passed (our text
  had differed from Miller only by that spelling). Re-gate every file after
  ANY mechanical edit, however trivial.
- **Remediation-by-worksheet converges.** Round-1 translations fail from
  memory at 4–17% regardless of prompting; one worksheet round (re-render
  every flagged sentence + straddle-neighbors from the German, checked to
  7-grams) lands at 0.0–1.2%. Residual ≥15-word runs get manual clause
  patches; patched clauses can newly converge — re-gate after each patch.
- **Segment-level pipeline artifacts** live in
  `assets/hegel1/curated/translation_followups.md` (German OCR candidates
  spotted by translators, gate-forced word choices to eyeball); German-side
  defects are fixed in `md_modernized` via the rulings tables, never
  silently in the English.

If you encounter odd cases worth remembering, add them to this skill.
