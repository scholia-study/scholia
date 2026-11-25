---
name: hegel4-translate
description: Clean-room English translation of Hegel's Enzyklopädie der philosophischen Wissenschaften (1830, hegel4) — control-gate rules, terminology standard, emphasis, marker and sentence-parity conventions. Use for any work on assets/hegel4/curated/md_modernized_translated.
---

You are a top translator and expert in German Idealism, producing Scholia's
own English translation of Hegel's *Enzyklopädie der philosophischen
Wissenschaften im Grundrisse*, third edition (1830, GW 20): the Logic, the
Philosophy of Nature and the Philosophy of Spirit, **without the Zusätze**
(the editors' lecture-note Additions are not Hegel's published text and are
not in our source).

Source: `assets/hegel4/curated/md_modernized` ONLY (the modernized German
reading layer). Target: `assets/hegel4/curated/md_modernized_translated`,
same 178 files, same German filenames (the translated file's front-matter
label is the English authority; there is no toc_en table).

The goal is a translation that is **accurate, modern, and demonstrably
independent of every existing translation**.

## Translation independence (non-negotiable)

Inherited from hegel1/hegel2 (read `.claude/skills/hegel1-translate/SKILL.md`
for the history; the kant3 incident showed clean-room retranslation still
fails from training memory). The short form:

1. **Clean room.** The control translations in `assets/hegel4/control/`
   must NEVER be in a translating agent's context — not as reference, not
   as style guide, not to check a term. Controls are for QC/orchestrator
   sessions only, which write no translation prose:
   - Logic (§§ 1–244, prefaces): `bd` (Brinkmann/Dahlstrom, Cambridge
     2010), `gsh` (Geraets/Suchting/Harris, Hackett 1991);
   - Nature (§§ 245–376): `petry` (Petry, 1970; prints no § 245);
   - Spirit (§§ 377–577): `inwood` (Wallace/Miller rev. Inwood, Oxford
     2010), `wallace` (Wallace 1894).
   All except Wallace 1894 interleave the Zusätze; that is harmless for the
   gate. `controls_report.md` documents the extraction and its residuals.
2. **From-memory leakage is the primary risk, not copying.** The
   Encyclopaedia Logic and the Philosophy of Spirit are heavily quoted in
   the literature; assume the model can reproduce gsh, bd and Wallace
   verbatim and gate accordingly.
3. **Post-hoc gate, per control:**
   `python3 .claude/skills/hegel4-translate/overlap_gate.py <file>`
   measures against each control **separately** (`--spans K` prints the
   matched runs for remediation). PASS = <3% word-8-gram overlap AND no
   common run ≥ 15 words, against every control. Text carried verbatim
   from the German file (Latin, French, Greek and English quotations,
   formulas, citation numbers) is masked first and reported as
   "discounted". Calibration: gsh against bd on §§ 3–4 measures 2.4% with a
   14-word longest run, so the bar sits right at published practice.
4. A flagged run is re-rendered fresh from the German (the orchestrator
   passes the flagged text from OUR file, never from the control), then the
   file is re-gated; replacement sentences can newly converge — iterate to
   PASS.
5. **Do not paraphrase away overlap.** Provenance is what counts; fresh
   generation from the German is the only clean path.
6. **First drafts of the Logic converge heavily.** In the pilot, Sonnet's
   first pass over §§ 19–131 measured 4–18% against gsh/bd, with runs up
   to 37 words; the most natural "textbook" rendering of a sentence is the
   recalled one. The Philosophy of Nature converges far less (13 of 54
   measurable first drafts failed against Petry, mostly on a single 15–21
   word run; one remediation round sufficed). The Philosophy of Spirit
   converges like the Logic (54 of 59 measurable files failed against
   Inwood, runs to 42 words, three rounds), above all on definitional
   theses: a rewrite that keeps the "X is Y" frame and changes only the
   edges does not move the run; recast the frame itself. Build each sentence's English architecture from the German
   yourself (clause order, verbal vs. nominal, voice) rather than reaching
   for the first fluent version. Locked terms stay; the syntax around them
   is yours. The opposite failure is as bad: never contort a short thetic
   sentence ("This pure being is now pure abstraction") or drop into a
   casual register to dodge overlap. The gate is per file, so short plain
   sentences can converge; spend the variation on long sentences.
   Restructuring also flips relations: after a re-render, re-read the
   German for every rewritten sentence and check that datives, relative
   pronouns and "dem … dem" chains still point where Hegel's do (§ 236's
   "dem die Idee als solche der Gegenstand, dem das Objekt sie ist" came
   back reversed).

## Terminology standard

`assets/hegel4/curated/translate_terms.tsv` is authoritative. Policy
(user-confirmed 2026-11-25): **hegel2's terms — Di Giovanni's glossary —
are the starting point**, so Scholia's Hegel corpora read consistently;
hegel1's rulings carry for the Phenomenology vocabulary that recurs in the
Philosophy of Spirit; rows marked `user-approved` (Nature and Spirit
vocabulary neither earlier corpus ruled on, plus the pilot's additions) were
ruled on 2026-11-25. New terms a translator coins go into the final report;
the orchestrator logs them in `assets/hegel4/curated/translation_followups.md`.
Headline choices: Begriff = **concept**, aufheben = sublate, Dasein =
**existence**, Existenz = **concrete existence**, Schein = shine, Geist =
**spirit** (never "mind", also in the part title), Schranke = restriction,
Urteil = judgment, Schluss = syllogism, Zweck = purpose, Einzelheit =
singularity, Wirklichkeit = actuality, Herr/Knecht = mastery/servitude,
Sittlichkeit = ethical life, Materie = matter. Sache stays hegel2's "the
matter" ("die Sache selbst" = the matter itself) in the Logic, the prefaces
and wherever no confusion arises; it becomes "the fact" only where it would
collide with Materie, mainly in the Philosophy of Nature (user-ruled
2026-11-25).
English node labels come from `assets/hegel4/curated/translate_labels_en.tsv`
(one row per position).

Terminology-locked renderings (titles, fixed formulae like "in and for
itself", "being-for-itself") legitimately converge with the controls; runs
< 15 words from locked terms are expected and pass.

## Markup

- Front matter: same `position`/`depth`/`page_gw`; `label:` from the
  English label table.
- **`{{{ 20.39 }}}` GW page markers and `{{ 84 }}` / `{{ 19 A }}` § markers
  stay**, at the position closest to where the German has them. § headings
  `## {{ 84 }} §. 84.` render `## {{ 84 }} § 84.`; the Remark's marker
  `{{ 19 A }}` opens the first Remark paragraph exactly as in the German.
- **`::: remark` / `:::` fences carry exactly**: same paragraphs inside,
  same paragraphs outside.
- **Emphasis is per word**, as in hegel2: `_in_ _Form_ _des_ _Gedankens_`
  → `_in_ _the_ _form_ _of_ _thought_` on the corresponding English words.
  Intraword `An<i>sichsein</i>` maps to the corresponding English element
  (`being-<i>in-itself</i>`); where the English word does not decompose,
  emphasize the whole word.
- **`+ ` display lines** (list items, verse) stay `+ `-prefixed, one line
  each, same count.
- **Footnotes carry:** `[^1]` refs at the corresponding position, `[^1]:`
  definitions after the same paragraph.
- Em-dashes (–/—) as in the German. `[supplied]` brackets carry.
- No translator's notes or synthesized apparatus: the reader renders
  curated markdown only.

## Sentence parity

**1:1 sentence ratio with the German, per block and per footnote.** Never
split or merge sentences; if impossible, stop and note the case. The
German side splits with `reference_protected_splits` (citation apparatus
protected) and `\.` escapes; the English side splits with the English
paren-protected splitter, which knows NEITHER the German citation rules
NOR the German abbreviations. Check every file with
`md_prose_to_struct --corpus hegel4 --translation --output-file <scratch>`
(missing files are skipped, so a partial corpus builds); footnote parity is
checked only at import (`struct_to_db --dry-run` against a scratch DB), and
translation import refuses a partial corpus ("present in source but missing
from translation"), so until all 178 files exist the orchestrator compares
footnote sentence counts between the German and English struct JSONs
(matching `toc_nodes` by `sort_order`).

- **`\.` works in English too**: a period that ends no sentence. Where the
  German holds an abbreviation run or a list together (`u.s.f\. Stoff`,
  `+ II\. Die Naturphilosophie`, `Hrn\. Th\. S. 119`), the English must
  hold it together as well — rephrase so no period sits before a capital,
  or escape the period (`II\. The Philosophy of Nature`). An escape that
  suppresses no split stops the build; remove it.
- **Spell out German abbreviations** (`z. B.` → "for instance", `d. h.` →
  "that is", `u.s.f.` → "and so forth", `vergl.` → "compare" or "see")
  rather than `e.g.`/`i.e.`/"etc.".
- **Citations go in parentheses where the German has them**; inside a
  parenthesis nothing splits. Outside parentheses, a citation chain
  (`Cart. Princ. phil. I. 15.`, `Ep. I. 118.`) splits in English at every
  "abbreviation. Capital": carry it with `\.` at the sites the German
  holds together.
- **Dash junctions split.** A sentence terminator followed by a spaced
  dash and a capital (`. – Nichts…`) is a boundary on both sides; mirror
  the German dash.
- **Splitter glue traps:** never end an English sentence on bare "I.", on a
  word ending "…lie."/"…eg.", or on "…sect." — restructure or append a
  faithful trailing word. INVERSE CASE: when the German under-splits (a
  single capital + period reads as an initial), reproduce the same
  under-split in English rather than restructuring away from it.
- **Honorifics:** German `Hrn.` before a name is held with `\.` in our
  German (`Hrn\. Tholuk`); English "Mr." is protected and merges by itself,
  so no escape is needed there.
- **Spell out month abbreviations** ("2 February", never "2 Feb.") —
  inside a footnote, "(2 Feb.) Brougham" splits after the parenthesis and
  breaks footnote parity, which the build does not report.
- **Datelines are one sentence.** German "den 25. May 1827." is protected;
  English drops the day-period ("Berlin, 25 May 1827.").
- Mathematical expressions (`s = ct³`, `V/t`) carry verbatim.

## Quotations

Hegel quotes Latin (Cicero, Descartes, Anselm, Spinoza), French (Biot,
Lagrange, Brougham's speech is in English) and Greek. **Foreign-language
quotations stay in the original language**, verbatim from our German file
— the gate discounts them. German verse (Rückert's ghazals in the § 573
footnote, Goethe) is translated, line for line, keeping the `+ ` lines.

## Workflow

Translation runs as lanes: Sonnet translators, each with a batch of whole
files, the German source, this skill, the terms table and the label table
— never the controls. Opus verifies each finished file against the German
(accuracy, terminology, emphasis, markers), then the orchestrator runs the
parity build and the gate, and sends failures back for fresh re-rendering.

If you encounter odd cases worth remembering, add them to this skill.
