# hegel1 translation campaign followups (reviewer pass)

## Consistency-pass fixups (after all segments land)
- 001.seg1: idiomatic Schein rendered "shine" at ¶1, ¶15, ¶20 → semblance per locked policy.
- 039.seg1: Selbstständigkeit rendered "self-standingness" → self-subsistence.
- Sweep all: self-sufficiency/independence/self-standingness → self-subsistence; "Notion" must not occur; Horos capitalization decision.

## German-layer defects spotted by translators (md_modernized QA sweep later)
- OCR damage read through: Mornente, Tronnung, anerkanrt, ebeuso, Erfällung, alo, Realitat, Ehensowenig, Magnetismns, Bilgung, unmittelbarbar, coneretes, seheinend, Nahme, Ungeschicklichkkeit, dzs (006/039/001 segs).
- 001 ¶66: missing period mid-sentence in German (joined with comma in EN).
- 001 ¶68: suspended-hyphen artifact "form — wie geschmacklose".
- 049.seg1: Selbstständigkeit as independence → self-subsistence; Rechtszustand 'condition of legal right' vs toc_en 'The Condition of Right' — align
- 012 ¶8: German splitter artifact 'u. s. w. Körper' pseudo-split; EN mirrors as 2 real sentences. If German re-splits, re-merge EN.
- 001.seg2 ¶28/¶45: 'the thing itself' for ordinary Sache selbst → 'the matter itself' (Ding owns 'thing'); also add Bekannt=familiar to addendum
- 048.seg1: verify Begeistung→Begeisterung call (Begeistung is a real Hegel term in 012/030); check übertägigen rendering
- 048.seg1: 'Begeisterung' → render Begeistung as 'inspiriting' (real term, see 030 lock)
- 006.seg2: Entzweiung 'sundering' → 'splitting-in-two' per lock (agent launched pre-lock)
- 035 ¶2: German stray period "in der reinen. Form" forces 7-sentence count; EN split at grammatical seam. If German repaired → re-merge.
- 035 ¶11: German corruption "_ist. Ansicht_" = "_ist_. _An sich_"; EN renders intended text (16 vs 15 spans, deliberate).
- 046 ¶7: German stray period 'um dieser. Allgemeinheit' — EN mirrors split; re-merge if German repaired
- 018 ¶9: 'b[ut]' → 'but' (policy: letter-level restoration brackets do not carry into English)
- Anschein→'show of' lock: sweep 001.seg1 ¶3 'appearance of seriousness' → 'show of seriousness' in fixups
- 017 ¶11: '[im]mediate' → 'immediate' (letter-restoration bracket policy)
- 031 ¶8: 'congregation' → 'community' (Gemeine unification)
- 033.seg2: 'fetch up' for herbeibringen reads colloquial — reviewer eyeball
- 028: Persönlichkeit 'personhood' → 'personality' (unify with 035)
- 001.seg3: some paragraphs carry surplus EN parenthetical em-dashes vs German (draft-density kept) — reviewer eyeball
- 019: 'comportment' → 'relating' (Verhalten lock; comportment is Pinkard's signature)
- 003 ¶12: 'for an other' vs 'for another' — unify at review (check gate impact)
- 037 adopted "harmony between morality and X" (gate-forced; "harmony of" is the controls' shared cadence). Check 036/038/039 use the same connective for Harmonie der Moralität.
- Nichtigkeit not in ruled table: 033 uses "nothingness" (¶18, gate-forced) and "nullity" (¶13); 009.seg2 patched to avoid "nullity" run. Decide a default (nullity) and note gate-forced exceptions.
- German OCR sweep candidate: 033 ¶2 "Lüfte" likely OCR for "Lüste" (translated as lusts); verify in md_modernized + md_reviewed.
- German OCR sweep candidate: 035 ¶11 "was sie _ist. Ansicht_ ist sie" should be "was sie _ist_. _An sich_ ist sie" (emphasis mis-spanned in md_modernized; English renders intended sense, validator warns 16 vs 15 spans).
- 046/047 gate-forced deviations to eyeball at review: Aufgang = "the rising" (not sunrise); Gemeinschaft der Seligkeit = "fellowship of blessedness" (046 ¶11); Besonnenheit = "level-headedness" (046 ¶8).
- German OCR sweep candidates from 039.seg2: ¶31 "Allgemnine"→"Allgemeine", ¶40 "hst"→"hat" (translated by intended sense).
- 001.seg3: "the relation of subject and predicate" renders Verhältnis (lock says relationship) in the same sentence as a Verhalten fix; full Verhältnis-vs-Beziehung audit was never run campaign-wide — reviewer sweep item.

## GW 9 concordance (sentence-level by design)
- GW 50–52 have no markers: blank/divider pages between Vorrede (ends GW 49) and Einleitung (GW 53); the 1807 Zwischentitel (file 002) sits there. Could be pinned from a physical GW 9 if ever wanted.
- GW 78: margin number missing from the Cambridge print (77 jumps to 79); placed by interpolation at the start of Pinkard ¶123 — flagged `interpolated` in gw9_markers.tsv.
- All placements snap to the nearest German sentence start; a library copy of GW 9 could refine any of them by editing gw9_markers.tsv anchors and re-running the converter + translated-layer insertion.
