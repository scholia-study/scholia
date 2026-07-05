---
name: kant1-modernize-translate
description: Detailed steps when working with Kant's Critique of Pure Reason text modernization and translation.
---

You are a top translator, classics and philosophy researcher, also an expert in German Idealism, tasked with digitizing classical texts.

You are to take `assets/kant1/curated/md_reviewed` and produce a up-to-date German modernization `assets/kant1/curated/md_modernized` and _from that base_ produce accurate English translation in `assets/kant1/curated/md_modernized_translated`.

Folder layout:
- `assets/kant1/raw/` — gitignored pipeline artifacts (pages, png_to_ocr, ocr_to_lines, lines_to_elements, elements_to_md, md_to_struct, md_translation_to_struct, plus the source PDF and translation epub).
- `assets/kant1/curated/` — tracked-in-git, human-curated MD (md_reviewed, md_modernized, md_modernized_translated).

**Do not edit the files in assets/kant1/raw/elements_to_md or any other files earlier in the pipeline** Always make a new file the target folders and edit there.

Look through earlier files in assets/kant1/curated/md_modernized and md_modernized_translated for examples.

When in any doubt, stop and ask the user for clarification. Accuracy in the work is paramount.

If you encounter odd cases and notable issues that would be good to remember for next run, add it to this skill. 

## Translation independence (licensing — read before translating a single sentence)

**Incident (2026-07-05):** an overlap audit found the entire `md_modernized_translated` layer was derived from the copyrighted Cambridge/Guyer–Wood translation (median 29.5% verbatim word-8-gram overlap; file 093 had an unbroken 908-word verbatim run; files ~076–114 were 90–95% identical). Calibrated baseline: two genuinely independent translations of the same German (Meiklejohn vs Guyer/Wood) share <1% 8-grams, longest run 12 words. Scholia is commercial; that layer can never ship. See auto-memory `kant1-translation-licensing`.

Rules, non-negotiable:

1. **Clean room.** Translate from `md_modernized` ONLY. The Cambridge text (`assets/kant1/control/**`, the epub in `assets/kant1/raw/`) must NEVER be in the translating agent's context — not as reference, not as "style guide", not "just to check a term". The same applies to the old tainted English files: do not read them while translating.
2. **Permitted reference:** Meiklejohn 1855 (public domain) only, and only for terminology sanity checks — never copy its sentences either; its Victorian register is wrong for this edition.
3. **Post-hoc gate:** every translated file must pass `python3 .claude/skills/kant1-modernize-translate/overlap_gate.py <file>` — verbatim word-8-gram overlap vs the Cambridge control **< 3%** and **no common run ≥ 15 words**. Honest convergence on technical formulae passes this comfortably (independent baseline is <1% / 12 words). A failing file is retranslated, not patched around.
4. The control text is for **QC agents only** (emphasis arbitration, fidelity spot-checks), in sessions that do not write translation prose.
5. **Do not "paraphrase away" overlap.** Rewording a Guyer/Wood-derived text still yields a derivative work no matter what the gate measures; provenance is what counts, and git history documents provenance. Fresh generation from the German is the only clean path.

### Terminology standard

Erkenntnis = cognition (NOT knowledge; knowledge renders Wissen/Kenntnis). Anschauung = intuition, Verstand = understanding, Vernunft = reason, Vorstellung = representation, Grundsatz = principle, Vorschrift = precept, Willkür = free choice. Selbsterkenntnis = self-knowledge (corpus convention, exception to the Erkenntnis rule).

### Heading conventions (hard-won, do not "fix")

- English headings begin **"On …"**, not "Of …" (user decision 2026-07-05; matches modern scholarly usage).
- English ordinal style follows the file's **label** style ("Section N.", "Chapter N.") — EXCEPT where the sentence-splitter forces mirroring the German: "Erstes Buch." must become "First Book." (not "Book I." — a lone "I." is not a sentence boundary and breaks the 1:1 parity check in `just struct kant1`), and "§10. 3. Abschnitt." must stay "§10. 3. Section." (3 sentences). When in doubt, run `just struct kant1` — it enforces parity.
- Frontmatter labels must match `packages/common/src/kant1/toc_mod.rs` / `toc_en.rs` exactly, and file names derive from those labels via `slugify` (position number `NNN` is the stable identity, so renames are safe for the DB but the Rust tables and files must move together).
- German modernized layer keeps "u. s. w." (not "usw.") and Germanized "Prinzipium" (not italic Latin).
- An English sentence must never END on a bare capital letter ("…from E to A. For…") — the splitter reads "A." as an initial and glues the sentences (same reason "Book I." doesn't split). Append a faithful trailing word ("…from E to A as well.") or restructure.
- Abbreviated cross-references must mirror the German's sentence-split behavior under the pipeline splitter (`packages/common/src/kant1`… list in `common/src/sentences.rs`): German "(Einleit. II)." pseudo-splits, so the English must use "(Intro. II)." (unprotected abbrev → splits) NOT "(Introd. II)." (protected → doesn't split). When `just struct kant1` reports an off-by-one at such a block, this is why.
- Untranslated public-domain quotation nodes (e.g. 001 Motto = Bacon's Latin epigraph) legitimately FAIL the overlap gate — every edition carries the identical Latin. Verify the quoted text is PD and byte-identical to the German curated file, then accept despite the gate.

## Specific rules

### Keep an exact 1:1 sentence ratio

Do not break up or combine German sentences. Find ways to make the syntax and meaning work in the English translation. If it is completely impossible to find a 1:1 translation, stop and ask the user for clarification, offering a selection of possibilities.

### Keep emphasis

It is very important that to carry over emphasis EXACTLY
- ***words and phrases in sperrdruck***
- **normal bold**
-  _latin words_

### Use English file names in `md_modernized_translated`

Refer to `000_toc.md` for the English file names in `kant1/curated/md_modernized_translated`

### Port figures faithfully but translate content

Figures should already come with correct HTML structure and styling, but the actual text contents within divs, spans and figcaption will need modernization/translation.