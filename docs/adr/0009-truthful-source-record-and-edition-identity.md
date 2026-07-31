# 0009. One truthful source record + edition identity as display fields

**Status**: Accepted
**Date**: 2026-07-30

## Context

`sources` carried no place of publication. Every bibliographic standard
has one (CSL `publisher-place`, BibTeX `address`, MARC 264$a, Chicago),
and it matters more here than in a typical application: Scholia's corpus
is almost entirely pre-1900 imprints, where place is how editions are
told apart. ESTC/USTC key on imprint place; MLA keeps place specifically
for pre-1900 works.

Worse, the Kant rows described two different books at once.
`title`/`publication_year` asserted the 1787 Riga printing of the
*Kritik der reinen Vernunft*, while `publisher` held
"Akademie-Ausgabe Band III" — a reference to a 1911 Berlin volume, and
not a publisher at all. The volume Scholia actually transcribed is on
disk (`assets/kant1/raw/pages/page-001.png`): *Kant's gesammelte
Schriften*, hrsg. von der Königlich Preußischen Akademie der
Wissenschaften, Band III, Berlin: Druck und Verlag von Georg Reimer,
1911. That fact existed only as hand-written prose in `meta::ABOUT`.

This is the shape of the whole corpus — a modern edition of an old work —
and the two facts diverge in two ways. A critical reprint presents an
edition printed decades earlier: the Akademie-Ausgabe volumes of 1911 and
1913 carry Kant's 1787 and 1790 texts. And where Scholia's contributors
do the editorial work — the translations, and the modernizations that
rest on sustained judgment — Scholia itself is the publisher of a 2026
edition presenting a much older work. They coincide only when the
transcription is faithful and nobody here added editorial judgment:
Shakespeare's Sonnets are the 1609 Quarto via EEBO-TCP, with a modern
layer taken from PoetryDB rather than prepared here.

Two questions were being answered by one set of columns:

- **Provenance** — "where did this text come from?" The concrete edition
  Scholia publishes or transcribed it from.
- **Identity** — "which text is this?" What a reader choosing from the
  library cares about: is this A or B, is this the 1609 Quarto.

Separately, passage locators ("B 132", "AA V 212") never involved
`sources` at all: they come from `reference_systems` + embedded page
markers, selected by `cite_priority`. Both Kant systems were
`cite_priority: None`, so Kant citations fell back to `Node · s. N`
while the `AA III {ref}` / `B {ref}` templates sat unused.

## Decision

1. **The `sources` row records the edition Scholia actually publishes or
   transcribes.** Where the text is transcribed, that is the volume in
   hand: kant1 is the AA Band III printing — year 1911, publisher "Georg
   Reimer", place "Berlin", volume "III", not a fictional record of a
   1787 Riga printing nobody here has touched. Where Scholia's
   contributors prepare the text, Scholia is the publisher: the three
   translations and the two judgment-heavy modernizations (*Paradise
   Lost*, *Keiser og Galileer*) are 2026 editions published by **Scholia
   Sodalitas**, with the historical imprint they displace moving into
   `about_text`. The dividing line is degree of editorial judgment: the
   Kant reading texts are also modernized here, but German orthographic
   normalization does not make a new edition — those rows keep the AA
   record and the fellowship is credited in `about_text` instead. The
   imprint itself is publicly declared (about page, `llms.txt`) so a
   citation-follower can find its publisher. Sodalitas rows carry no
   `publication_place` — a
   distributed fellowship publishing on the web has no imprint city, and
   inventing one is the failure this ADR exists to stop. Chicago and MLA
   both drop place for online and self-published works, and
   `format_bibliography_entry` already renders publisher-without-place
   correctly.

2. **Identity is not a bibliography record.** It is carried by the work
   title, a new `original_year`, the existing `edition` field, and the
   reference systems. Two new columns:

   - `publication_place TEXT` — imprint place of the record itself.
     Useful for every source type.
   - `original_year SMALLINT` — the year of the edition this text
     *presents*, when it differs from the edition recorded (CSL's
     `original-date`). NULL means "same as `publication_year`" — true
     for the Bibles and the Sonnets, while every Kant, Milton, and Ibsen
     book carries one.

   `SMALLINT`, not a label string, because the library **sorts** works
   by year; display strings are composed per surface.

3. **Display rule: reading surfaces show identity; provenance surfaces
   show the record.** Library card and sort, book detail, and SEO
   `datePublished` use `COALESCE(original_year, publication_year)`, so
   the Critique still reads "1787" everywhere a reader chooses a text
   (`reading/library/db.rs:577-580`, `reading/books/db.rs:17,45`). The
   About-this-text panel shows the record as it is: 1911, Georg Reimer,
   Berlin, volume III, plus the `about_text` prose. Kept in code per
   surface, not as a per-corpus flag — the answer is the same for every
   corpus, so a knob would only be a way to get it wrong inconsistently.

4. **Article citations implement Chicago's original-then-edition year
   form**, which covers reprints and translations of older works alike.
   It applies whenever `original_year` and `publication_year` are both
   present and differ (`articles/db.rs:442-453`). Bibliography entries
   use the parenthesised form, text citations the bracketed one — "Kant,
   Immanuel. (1787) 1911." for the German record, "(1787) 2026" for the
   Sodalitas translation, "(Kant [1787] 1911, 132)" in text. The
   facts-of-publication segment degrades with the data: both →
   "Place: Publisher.", publisher alone → "Publisher.", place alone →
   "Place.", neither → omitted. An `edition` statement, when the record
   carries one, follows the title per Chicago — "…*Kritik der reinen
   Vernunft*. 2. Auflage (B). Berlin: Georg Reimer."

5. **Kant cites by page, not by sentence.** `cite_priority` is set in
   the corpus config — kant1 `b_edition` → 0 (B pagination is *the* KrV
   convention, and the AA III margins reproduce it), kant3 `aa_v` → 0
   (non-KrV Kant is cited by AA volume and page). The secondary system
   in each stays `None`: shown in margins, absent from the default
   citation. Sentence numbers step back to fallback-only — they are
   Scholia-internal and unverifiable against print, while page
   citations are portable in both directions. Sentences remain the
   anchor mechanism for deep links and projection regardless.
   Translations inherit the source book's systems, so the English
   editions cite identically.

6. **Bibliographic corrections live in corpus `meta.rs`; existing
   databases get one-off SQL.** `struct_to_db`'s reconcile path never
   updates the `sources` row — bibliographic fields are written only on
   fresh insert (`import.rs:330`) — and that stays as-is. Importers and
   `meta.rs` carry the corrected values so fresh ingests are right from
   the start; already-populated databases receive the same values via
   idempotent `UPDATE` statements keyed on stable identifiers, run after
   the migration. `--replace` is never a substitute: it is destructive
   and cascades quotations.

## Reasoning

**Why one row rather than two.** The first design kept the 1787 record
as the primary row and demoted the volume actually transcribed to a
satellite, linked by `copy_text_source_id`. Rejected as a category
error: it preserved the invented record and made the real one an
appendix. It also cost more than it returned — the importer would own a
second row's lifecycle (creation, upsert identity, `--replace`
orphaning) for every corpus; multi-volume sets collide on
`UNIQUE (title, source_type, publication_year)` because `volume` is not
part of that key; and its main justification, somewhere to put licence
facts, was aspirational since `sources` has no licence column either
way. If licence metadata lands later it belongs on the one truthful
record, which this model keeps as the primary row.

**Why `original_year` and not a second identity label.** `edition`
already carries "2. Auflage (B)", and the About panel renders it. A
composed string like "1787 · B-Edition" is `original_year` + `edition`,
built where a surface wants it. Even hosting both the A and B editions
needs no new column: `original_year` 1781 vs 1787 separates the cards.

**Naming.** "canonical_year" was considered and dropped — `canonical_*`
already means cross-translation identity in this schema
(`canonical_passages`, ADR 0008).

**SEO uses the identity year.** Search intent is the work, not the
printing.

**`publisher` is load-bearing for Bibles and was left alone.**
`bible_to_db` uses it as the same-language version-pill label *and*
queries on it — `s.publisher = 'KJV'` gates canonical verse-count
seeding (`packages/bible_to_db/src/main.rs:592,1239`). Bible rows take
no `original_year` and their `publisher` values are untouched.

**Projection is unaffected.** Cross-translation identity remains
`COALESCE(translation_of_id, id)` (`quotations/db.rs`,
`article_passage_references/db.rs`, `canonical_passages.work_root`).
Nothing here touches those columns.

**Extract, don't infer.** Every corrected value was read off the
transcribed volume's own title page or TEI header. An earlier draft
credited AA Band III to Benno Erdmann; no volume editor is named on
those title pages, so no editor link was made. Likewise EEBO A50924
turned out to be the 1674 second edition rather than 1667, which
`milton1.rs` already had right.

**Accepted limitation.** Later bibliographic edits in `meta.rs` still
will not propagate to populated databases, because reconcile does not
upsert the source row. If that becomes a real workflow, a reconcile
source-upsert is the fix; it is not needed for a one-time correction.

**Where the containing volume's title lives.** *Kant's gesammelte
Schriften* has no structured field and stays in `about_text` prose. If
citation export ever needs it structurally, `parent_source_id` already
models "work contained in volume" — no schema was reserved for it.

## Schema deltas

One append-only migration, `0019_source_place_and_original_year.sql`:

```sql
ALTER TABLE sources ADD COLUMN publication_place TEXT;
ALTER TABLE sources ADD COLUMN original_year SMALLINT;
```

`UNIQUE (title, source_type, publication_year)` is unchanged — place is
deliberately not part of source identity, and the Kant year moving
1787 → 1911 collides with nothing.

`text_struct::model::BookData` grew `publication_place`, `original_year`,
`edition`, `volume`, and `url` so the ingest waist can carry what the
column set now holds; every genre parser wires them, and
`struct_to_db`'s source insert writes them.
