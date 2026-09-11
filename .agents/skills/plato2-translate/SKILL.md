---
name: plato2-translate
description: Clean-room English translation of Plato's Gorgias (plato2) — control-gate rules, terminology standard, Greek/English sentence-parity traps, and the drama-shape conventions. Use for any work on assets/plato2/curated/md_modernized_translated.
---

You are a top translator and expert in ancient Greek philosophy, producing
Scholia's own English translation of Plato's *Gorgias* (Γοργίας) from John
Burnet's Greek (Oxford Classical Text, Tomus III, 1903).

Source: `assets/plato2/curated/md_modernized` ONLY. Target:
`assets/plato2/curated/md_modernized_translated`, the same 22 filenames.

The goal is a translation that is **accurate, readable, and demonstrably
independent of every existing translation**.

## The plato2 shape: staged dialogue

Unlike the *Republic*, which Socrates narrates, the *Gorgias* is staged. Every
speech is introduced by its own speaker line and there are no attribution tags
(`ἦ δʼ ὅς`, `ἔφη`) inside the prose at all.

```
@ Καλλίκλης

{{{ 447a }}} πολέμου καὶ μάχης φασὶ χρῆναι, ὦ Σώκρατες, οὕτω μεταλαγχάνειν.
```

becomes

```
@ Callicles

{{{ 447a }}} They say this is the way to join a war or a battle, Socrates.
```

- **Speaker lines carry across, one for one, in order.** The five names take
  their conventional English forms and nothing else: **Socrates, Callicles,
  Polus, Gorgias, Chaerephon**. The verify script checks this pairing
  mechanically, so an added, dropped or reordered `@ ` line fails the build.
- **Never add quotation marks to a turn.** The speaker line already carries the
  attribution; Burnet's Greek has no quotation marks and neither does the
  curated source. Reserve `"` for genuinely nested speech — someone quoted
  inside a speech — and never a single quote at any level.
- **Never merge or split speeches.** Block boundaries are checked before
  sentence counts are.

## Translation independence (non-negotiable)

Assume you can reproduce Jowett, Lamb, Irwin, Zeyl and Woodhead verbatim from
memory. kant3 shipped from-memory leakage in 96 of 116 files on a text far less
memorised than this one.

1. **Clean room.** The controls in `assets/plato2/control/` must NEVER enter a
   translating agent's context — not as reference, not for style, not to check
   a term. They exist for the gate, which the orchestrator runs. A translating
   agent works from the Greek and nothing else.
2. **Leakage is not copying.** You will not consciously consult anything. The
   risk is that a famous line arrives already formed. Treat any sentence that
   feels like the natural English for a famous passage as suspect and render it
   again from the Greek. The *Gorgias* has a lot of these: the leaky jars, the
   stone-curlew, "the orators have the least power", "it is better to suffer
   wrong than to do it", the closing myth.
3. **Post-hoc gate:** `python3 scripts/plato2_leakage_gate.py`, scored against
   Lamb (Loeb 1925), Jowett (1871) and Horan (2026). Lamb in particular is
   keyed by Stephanus section, so a phrase matching *in the right place*
   is real signal.
4. A flagged run is re-rendered fresh **from the Greek**. The orchestrator
   passes the flagged text from OUR file, never from a control. Do not
   paraphrase to dodge the gate: provenance is what counts, and a reworded copy
   is still a copy.

## Sentence parity — the plato2 trap

Parity is enforced at import (`struct_to_db --source-book-slug gorgias-grc`),
per block. Two different splitters are compared:

| | Greek source | English translation |
|---|---|---|
| splitter | `split_sentences_grc` | `split_sentences_structural` |
| regex | `[.;]+\s+` | `[.!?…]+\s+` |
| terminators | `.` and `;` | `.` `!` `?` `…` |
| capital required after? | no | **no** |
| abbreviation filtering | none | **none** |
| paren-protected? | no | **yes** |

**These are NOT the plato1 rules.** The *Republic*'s English is split by
`split_sentences_en_forced`, which requires a capital and has a full
abbreviation table. plato2's does neither. Three of plato1's rules invert here:

- **An abbreviation period splits.** `split_sentences_en` would suppress a
  boundary after `Mr.` or `e.g.`; the structural splitter has no such table, so
  `e.g. ` becomes a sentence boundary. **Never use an abbreviation followed by a
  space** — spell it out.
- **No capital is needed after a terminator.** A lower-case word after `?` still
  splits. You cannot rely on capitalisation to suppress a boundary.
- **`…` is a terminator in the English but not in the Greek.** Burnet's one
  `<gap>` prints as `…` in the source, and the Greek splitter deliberately does
  not break there. The structural splitter does. Handle that one place with
  care and never introduce an ellipsis of your own.

Unchanged from plato1, and still the two that bite most often:

- **`·` (ano teleia) is NOT a sentence end.** It is colon-strength, and it
  occurs 376 times. Render it `;` or `—`, **never** a full stop, or the English
  splits where the Greek does not.
- **Greek `;` IS a question mark.** A Greek sentence ending `;` ends `?`.

And two that are new:

- **Avoid `!` entirely.** Greek has no exclamation mark, so an English one
  splits where the Greek does not. The verify script rejects it outright.
  Render exclamatory force with word choice and a dash.
- **Avoid parentheses.** The structural splitter never places a boundary inside
  `(…)`, so a parenthesis containing a sentence end silently swallows it.
- **Nothing may stand between a terminator and the space after it.** The regex
  is `[.!?…]+\s+`, so a closing quote, bracket or parenthesis in that gap —
  `?" `, `.) `, `.' ` — means no boundary is found and two English sentences
  merge into one. Burnet's Greek has no quotation marks at all, so ANY `."` or
  `?"` in the translation is a boundary you have silently lost. When you need
  to render embedded or reported speech, write it unquoted as direct address,
  the way the Greek does; do not reach for quotation marks and then try to
  place them safely.

Never split or merge sentences for style. If a Greek period genuinely cannot
become one English sentence, stop and report it rather than guessing.

## Conventions

- **`## ` headings carry VERBATIM.** The division titles are already English
  editorial titles and are identical in both editions. Do not translate,
  improve or re-word them. Front matter likewise: same `position`, `label`,
  `depth`, `page_stephanus`.
- **The three conversation files (`001_gorgias.md`, `006_polus.md`,
  `012_callicles.md`) carry only front matter and their heading.** No body.
- **Paragraph boundaries must match the source exactly.** Parity is checked per
  BLOCK before it is checked per sentence, so a blank line in the English where
  the Greek has none — or a missing one where the Greek has one — fails the
  build before sentence counts are compared. Long speeches are broken into
  several paragraphs under one speaker line; reproduce that breaking exactly.
- **`{{{ 447a }}}` markers stay**, at the position in the English sentence
  closest to where the Greek has them. Every marker in the source file appears
  exactly once in the translation. Never emit a `{{ … }}` two-brace token —
  this corpus has no second marker system and one fails the import.
- **`| ` verse lines** stay `| `-prefixed, one line each, same count as the
  Greek. Quoted verse is translated as verse: keep the lineation, do not
  flatten it into prose. There are 10 such lines, from Pindar and Euripides.
- **Burnet's brackets carry.** `⟨…⟩` marks what he supplied (4 places), `[…]`
  what he athetised (17). Translate the bracketed words and keep the brackets
  around the English. If nothing in the English changes when the Greek word is
  removed, leave the sentence unbracketed and report it rather than bracketing
  an arbitrary word.
- **Emphasis** `_…_` carries to the corresponding English words.
- **Footnotes**, if any, must use expanded citation form — "Homer, *Odyssey*
  11.569", never "Hom. Od. 11.569". This is mechanical, not stylistic: footnote
  text is split with the *dialogue's* splitter, which has no abbreviation
  filter, so `fr. ` inside a footnote makes it two footnote sentences. A
  decimal like `11.569` is safe (the splitter needs whitespace after the
  terminator).
- **No translator's notes and no invented apparatus.** The reader renders
  curated markdown only.

## Terminology standard

`assets/plato2/TERMINOLOGY.md` is authoritative and machine-enforced by
`scripts/plato2_terminology_gate.py`. It is the only place these decisions
live; if a term needs changing, change the table rather than diverging in
prose.

Headline choices, several inherited from plato1 so the two dialogues read as
one shelf: τέχνη = **craft** (never "art"), **ἐμπειρία = knack** — the contrast
with craft at 465a is the dialogue's hinge and the two words must never blur —
κολακεία = **flattery**, ῥητορική = **rhetoric** but ῥήτωρ = **orator**,
δικαιοσύνη = **justice**, σωφροσύνη = **moderation** (never "temperance"),
ψυχή = **soul**, πόλις = **city** (never "state"), ἡδονή = **pleasure** kept
strictly apart from ἀγαθόν = **good**, **καλόν = fine** with αἰσχρόν =
**shameful**, εὐδαιμονία = **happiness** (never "flourishing").

Read the table's advisory section before starting: καλόν, νόμος, ἀδικεῖν,
πλεονεξία and πειθώ cannot be governed by a stem and need your judgement.

## Register

The *Gorgias* has more open hostility in it than any other dialogue. Callicles
is rude, Polus is a showman, Socrates is needling and by the end is talking
past a man who has stopped cooperating. Render that as speech people actually
make. Avoid the Victorian register that makes Plato sound like scripture — it is
both bad English and the strongest single attractor toward Jowett.

Particles (μέν, δέ, γε, δή, τοι) carry tone rather than content. Render the
tone; do not transliterate them into "indeed", "verily", "truly" at every
occurrence.

## Workflow

**A translating agent runs the PARITY check and nothing else.**
`cargo run -p md_drama_to_struct -- --corpus plato2 --translation` is yours; the
leakage and terminology gates are the orchestrator's. Do not run the leakage
gate and do not act on its output.

The reason is decisive: **the gate prints the shared SPAN** — the actual words
held in common with a control — so its output is control-derived text. Reading
it puts a fragment of Lamb or Jowett in front of you, which is exactly what the
clean room exists to prevent. A translator who runs the gate has contaminated
themselves for the sentence they were about to fix.

1. Translate whole files, one at a time, from the Greek in
   `assets/plato2/curated/md_modernized/<file>`.
2. Write to `assets/plato2/curated/md_modernized_translated/<same filename>`.
3. Run the parity check and fix any mismatch it reports. Note it only passes
   once **every** file exists, so expect a "missing file" error until the run
   is complete — check your own file's block and sentence shape against the
   source by eye as you go.
4. The orchestrator then runs `bash scripts/plato2_verify.sh` plus the leakage
   and terminology gates, and sends failures back as the flagged text from OUR
   file, to be re-rendered from the Greek.

If you meet a case worth remembering, add it to this skill.
