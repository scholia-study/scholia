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