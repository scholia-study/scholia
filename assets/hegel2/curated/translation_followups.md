# hegel2 translation follow-ups

## RESOLVED 2026-08-24: citation-fragment sentences

The splitter used to shed citation apparatus as fragment "sentences"
("Vern. 2te Aufl.", "Tom.", "Band I. Abth."). Fixed structurally: the
`paren_protected_splits` corpus knob (never split inside or directly
before a parenthesis, hegel2 both editions) plus the converter's
`CITATION_PARENS` table (eight non-parenthesized citation runs re-set
inside one flat paren each, both German layers) plus four abbreviation
entries for unparenthesizable prose (`resp.`, `lect.`, `philos.`,
`folg. Anm(erkung)`). Sentence count 8,174 → 8,059; zero citation
fragments remain; scratch-DB import re-verified 1:1. The English
citation apparatus was then anglicized across 18 files (S. → p., Band →
Vol., Th./Abth. → Part/Division, Aufl./Ausg. → ed., Anm. → Remark,
ebendas. → ibid., a. a. O. → loc. cit., übers. von → trans.; canonical
English titles for Kant/Hegel classics; non-canonical German titles and
Latin/French stay verbatim) — safe because parens are split-free on
both sides. The parity checker in md_prose_to_struct now honours the
corpus splitter knob (it used to hardcode the base splitters).

Items noted during the 2026-08-23 translation campaign that were deliberately
deferred rather than fixed by sweeping edits. None blocks publication; each
would need DE↔EN sentence alignment to fix reliably, and every file below
already passes the overlap gate and the importer's 1:1 sentence parity.

## Beziehung: "reference" vs "relation"

The brief locked Beziehung = reference (Di Giovanni's choice) partway
through the campaign, at the Essence opening. Files translated before the
lock (roughly the Doctrine of Being, 001–118) and two later batches
(140–142, and parts of 125–131) render Beziehung as "relation" where it
reads naturally. "Relation" is also the locked rendering of Verhältnis, so
occurrences cannot be told apart by grep — a cleanup needs per-sentence
alignment against the German. Both renderings are faithful; the collision
is stylistic, not semantic.

## Grundlage: "groundwork" / "foundation" / "base"

Never locked in the brief. Batches variously chose groundwork (144–151),
base (153–157, deliberately, to keep "foundation" free for Unterlage in
the house example), and foundation (163+, the majority). All three stay
clear of Grund = ground. Harmonizing to one word would misfire in 154,
where two distinct German words need two English ones.

## Minor register variants

- Reich: normalized to "realm" corpus-wide 2026-08-23 (three "kingdom of"
  sites rewritten) — EXCEPT 273's two-worlds passage, whose three
  "kingdom" renderings are gate-driven (the "realm" phrasing converged on
  a 26-word control run) and stay deliberately.
- color/colour: normalized to British "colour" corpus-wide 2026-08-23; no
  action left.
- Art und Weise: "way and manner" (187–189) vs "manner and mode" (221–225).
- Gegenstoß: "counter-thrust" (125, 204-, 163–167) vs "recoil" (190–191).
- monstrieren (Hegel's deliberate non-"demonstrieren"): "indicate" (209),
  "monstration" (210), "outward showing" (215–220), "point out" (274) —
  four renderings; a harmonization would pick one and touch all four files.

## Documented gate exemptions (tracker carries the details)

- 066: Spinoza Latin scholium (73-word run) — verbatim rule-10 carry.
- 087: the equation (x+dx/2)(y+dy/2)… (18-token run) — verbatim carry.
- 088: Descartes French quotation (22-word run) — verbatim carry.
- 233: Ploucquet Latin sentence (30-word run) — verbatim carry.
- 200: title-only file after the Wechselwirkung re-homing; overlap
  percentage is a label-only artifact.
