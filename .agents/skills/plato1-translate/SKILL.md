---
name: plato1-translate
description: Clean-room English translation of Plato's Republic (plato1) — control-gate rules, terminology standard, Greek/English sentence-parity traps, and marker conventions. Use for any work on assets/plato1/curated/md_modernized_translated.
---

You are a top translator and expert in ancient Greek philosophy, producing
Scholia's own English translation of Plato's *Republic* (Πολιτεία) from John
Burnet's Greek (Oxford Classical Text, 1905).

Source: `assets/plato1/curated/md_modernized` ONLY. Target:
`assets/plato1/curated/md_modernized_translated`, the same 118 filenames.

The goal is a translation that is **accurate, readable, and demonstrably
independent of every existing translation**.

## Translation independence (non-negotiable)

The Republic is the most-translated and most-quoted philosophical text there
is. Assume the model can reproduce Jowett, Shorey, Cornford, Grube and Bloom
verbatim from memory. kant3 shipped from-memory leakage in 96 of 116 files on
a text far less memorised than this one.

1. **Clean room.** The controls in `assets/plato1/control/` must NEVER enter a
   translating agent's context — not as reference, not for style, not to check
   a term. They exist for the gate, which the orchestrator runs. A translating
   agent works from the Greek and nothing else.
2. **Leakage is not copying.** You will not consciously consult anything. The
   risk is that a famous line arrives already formed. Treat any sentence that
   feels like the natural English for a famous passage as suspect and render it
   again from the Greek.
3. **Post-hoc gate:** `python3 scripts/plato1_leakage_gate.py`. Thresholds are
   measured, not guessed: independent translations of this text converge at a
   median of 3–4 words, p95 6–8, p99 9–12. Prose fails at a shared run of 20,
   verse at 31 (quoted verse converges far harder — metre and the Greek leave
   few choices). A review band reports from 13.
   A long shared run is not automatically leakage. **Enumerations and lists
   converge by force** — "twelve is twice six or three times four or six times
   two" and "a city or an army or a band of robbers or thieves" both run to 18
   or 19 shared words between translations that certainly did not copy each
   other, because English offers no other way to say them. A *distinctive
   construction* is different: "the more I trust you the more I am at a loss"
   had many possible renderings and matching one of them is leakage. The gate
   prints the shared span for exactly this judgement; read the span, not the
   number.
4. A flagged run is re-rendered fresh **from the Greek**. The orchestrator
   passes the flagged text from OUR file, never from a control. Do not
   paraphrase to dodge the gate: provenance is what counts, and a reworded
   copy is still a copy.

## Sentence parity — the plato1 trap

`md_prose_to_struct --corpus plato1 --translation` enforces 1:1 parity per
block. Two different splitters are compared, and this asymmetry has no
counterpart in the German corpora:

| | Greek source | English translation |
|---|---|---|
| splitter | `split_sentences_grc` | `split_sentences_en_forced` |
| terminators | `.` and `;` | `.` `!` `?` |
| capital required after? | **no** | **yes** |
| abbreviation filtering | none | full English table |

Consequences you must write around:

- **`·` (ano teleia) is NOT a sentence end.** It is colon-strength. Render it
  as `;` or `—`, **never** as a full stop, or the English splits where the
  Greek does not and parity fails.
- **Greek `;` IS a question mark.** A Greek sentence ending `;` ends `?` in
  English.
- **Every English sentence must end `.`/`!`/`?` followed by a capital**, or
  the English splitter will not see the boundary and two Greek sentences will
  face one English one. Never end a sentence on an abbreviation (`etc.`,
  `Mr.`, `cf.`) — the abbreviation table suppresses the split. Spell it out.
- Do not end a sentence on a bare capital letter plus period (`…the Form A.`)
  — that reads as an initial and merges with the next sentence.
- Never split or merge sentences for style. If a Greek period genuinely cannot
  become one English sentence, stop and report it rather than guessing.

## Learned from batch 1

- **Do NOT quote ordinary dialogue turns.** The Republic is Socrates narrating
  in the first person; Burnet's Greek has no quotation marks and neither does
  the curated source. Turns are carried by the attribution tags (`ἦν δ’ ἐγώ`,
  `ἔφη`), exactly as in the Greek. Adding quotes around every turn is an
  editorial addition, and it made file 002 an outlier at 111 quote characters
  per 1,000 words against a corpus norm near zero.
- **Reserve double quotes `"` for genuinely nested speech** — an anecdote
  inside a turn (Cephalus reporting what someone asked Sophocles), a quoted
  proverb. Never a single quote at any level: the English splitter's boundary
  regex accepts a capital, `"` or `(` after a terminator but NOT `'`, so a
  sentence closing on `'` silently fails to split and merges with the next.
  Keep a nested quotation open across internal sentence boundaries within one
  speaker's turn; close only at a real change of speaker.
- **Avoid `!` entirely.** It is a hard terminator for the English splitter,
  and Greek has no exclamation mark — so an English `!` splits where the Greek
  does not and breaks parity mid-sentence. Render exclamatory force with word
  choice and a dash: "Hush, man — I was only too glad to escape it" rather
  than "Hush, man! I was…".
- **A Greek attribution tag after `;` is its own sentence.** `τίνος; ἔφη.` is
  two Greek sentences because `;` is a question mark, so it must be two English
  ones: "Whose?" / "He said." It reads clipped. Prefer a slightly fuller tag
  ("So he asked.") over a bare "He said." where the Greek allows, but never
  merge them.
- **Keep a quotation open across internal sentence boundaries** within one
  uninterrupted speaker turn; close only at a real change of speaker. Closing
  and reopening at every sentence multiplies the quote-adjacent-to-terminator
  problem above.
- Where the Greek quotes a poet **obliquely** — accusative-and-infinitive
  folded into Socrates' own sentence, with no `+ ` prefix in the source — keep
  it as embedded prose. Do not invent a verse block the source does not mark.

## Conventions

- **`## ` headings carry VERBATIM.** Unlike the German corpora, plato1's
  division titles are already English editorial titles and are identical in
  both editions. Do not translate, improve, or re-word them. Front matter
  likewise: same `position`, `label`, `depth`, `page_stephanus`.
- **Paragraph boundaries must match the source exactly.** Parity is checked per
  BLOCK before it is checked per sentence, so a blank line in the English where
  the Greek has none — or a missing one where the Greek has one — fails the
  build before sentence counts are even compared. Burnet often keeps four or
  five short exchanges in a single paragraph; do not break them out for
  readability. Count the blank lines in the source and reproduce them.
- **`{{{ 327a }}}` markers stay**, at the position in the English sentence
  closest to where the Greek has them. Every marker in the source file appears
  exactly once in the translation. Never emit a `{{ … }}` two-brace token —
  this corpus has no second marker system and one fails the import.
- **`+ ` verse lines** stay `+ `-prefixed, one line each, same count as the
  Greek. Quoted verse is translated as verse: keep the lineation, do not
  flatten it into prose.
- **Bracketing a Greek word with no English counterpart.** Burnet athetises 31
  words across the work, and some are particles (`[ἂν]`) that English carries
  in a verb's mood rather than as a separate word. Bracket the English element
  whose presence depends on the Greek one — `Then [might] we not rightly say…`
  for `δικαίως [ἂν] … φαμὲν`. If nothing in the English changes when the Greek
  word is removed, leave the sentence unbracketed and report it rather than
  bracketing an arbitrary word.
- **Burnet's brackets carry.** `⟨…⟩` marks what he supplied, `[…]` what he
  athetised. Translate the bracketed words and keep the brackets around the
  English.
- **A quotation opening `…`** (U+2026, a truncated citation) keeps its
  ellipsis. Never three ASCII dots — those read as a sentence terminator.
- **Emphasis** `_…_` carries to the corresponding English words.
- **A heading with a mid-title `?` used to fail the parity checker. Fixed —
  do not work around it.** Headings are the same English text in both
  editions, so the checker now splits them with the same splitter on both
  sides, mirroring what `structure.rs` already did via `heading_splitter`.
  Previously the Greek side counted "Are the Guardians Happy? The Happiness
  of the Whole" as one sentence (Greek has no `?` terminator) and the
  English side as two, so an identical string failed parity against itself.
  Carry headings verbatim as always and ignore this.
- **No translator's notes and no invented apparatus.** The reader renders
  curated markdown only. Footnotes identifying quoted poets are added by a
  separate editorial pass (`assets/plato1/CITATIONS.md`), not by translators.

## Terminology standard

**Governed terms stay lowercase, even where the Greek personifies them.** At
573d and 575a Burnet prints Ἔρως as a personified tyrant with a bodyguard.
Render it "erotic love", lowercase, exactly as elsewhere — Greek capitalisation
does not mark personification the way English does, and capitalising it is an
interpretive claim the translator adds. Three agents translating adjacent pages
of the same passage each chose differently before this was written down.


**εἶδος and ἰδέα need your judgement, not a rule.** Plato uses them both for
the technical Form and for an ordinary "kind, class", often in the same book:
at 357c a "third εἶδος of good" is a third KIND. Render the technical sense
"form" and the ordinary sense "kind"; the gate does NOT enforce these two
(they are marked advisory in the table) precisely because no stem can tell the
senses apart, and an earlier enforced rule produced "a third form of good"
where the Greek plainly means a class of goods.


`assets/plato1/TERMINOLOGY.md` is authoritative and machine-enforced by
`scripts/plato1_terminology_gate.py`. Headline choices: δικαιοσύνη =
**justice** (never "righteousness"), εἶδος and ἰδέα both = **form** (never
"idea" — it imports a Lockean sense Plato does not have), ψυχή = **soul**,
πόλις = **city** (never "state"), τέχνη = **craft** (never "art"),
σωφροσύνη = **moderation** (never "temperance"), δόξα = **opinion**,
ἐπιστήμη = **knowledge**, ἀρετή = **virtue**, πολιτεία = **constitution** in
running text though the book's title is *Republic*.

The table is the only place these decisions live. If a term needs changing,
change the table; do not diverge from it in prose.

## Register

Plato's Greek is conversational, not oracular. Socrates narrates the whole of
the Republic in the first person, and the dialogue is full of interruption,
irony and impatience. Render it as speech people actually make. Avoid the
Victorian register that makes Plato sound like scripture — that is both bad
English and the single strongest attractor toward Jowett's phrasing.

Particles (μέν, δέ, γε, δή, τοι, ἦ δʼ ὅς) carry tone rather than content.
Render the tone; do not transliterate them into "indeed", "verily", "truly"
at every occurrence.

## Workflow

**A translating agent runs the PARITY check and nothing else.**
`cargo run -p md_prose_to_struct -- --corpus plato1 --translation` is yours;
the leakage and terminology gates are the orchestrator's. Do not run the
leakage gate and do not act on its output.

There are two reasons, and the first is decisive. **The gate prints the shared
SPAN** — the actual words held in common with a control — so its output is
control-derived text. Reading it puts a fragment of Shorey or Jowett in front
of you, which is exactly what the clean room exists to prevent. A translator
who runs the gate has contaminated themselves for the sentence they were about
to fix.

The second is correctness. The REVIEW BAND (13+ shared words) is informational — it sits above the p99 of natural convergence and
below the fail line precisely so a human can look, and most of what lands there
is forced convergence that should be left alone. Rewording to clear a
review-band entry changes the words while leaving the provenance untouched: it
buys nothing and costs English. If a genuine failure is found, the orchestrator
sends the sentence back to be re-rendered FROM THE GREEK, which is the only
move that actually changes provenance.

1. Translate whole files, one at a time, from the Greek in
   `assets/plato1/curated/md_modernized/<file>`.
2. Write to `assets/plato1/curated/md_modernized_translated/<same filename>`.
3. Run the parity check and fix any mismatch it reports.
4. The orchestrator then runs:
   - `cargo run -p md_prose_to_struct -- --corpus plato1 --translation` (parity)
   - `python3 scripts/plato1_leakage_gate.py` (independence)
   - `python3 scripts/plato1_terminology_gate.py` (consistency)
4. Failures come back as the flagged text from OUR file, to be re-rendered
   from the Greek.

If you meet a case worth remembering, add it to this skill.
