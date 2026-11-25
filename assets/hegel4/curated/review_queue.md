# hegel4 review queue

Working checklist; delete items as they are settled.

## TOC / headings

- The stray `II.` / `NATURPHILOSOPHIE.` and `III.` / `PHILOSOPHIE DES GEISTES.`
  half-titles are excluded.

## Sentence splitting

Citation apparatus (day ordinals before a month, number/Roman ordinal before
a Roman numeral or reference abbreviation) is protected by the corpus knob
`reference_protected_splits`. Abbreviation runs and enumerated list items in
the body are held together with `\.` in both layers. Still splitting:

- [ ] Enumerators standing alone as sentences in about 65 node headings
  ("a." | "Sein").

## Hyphen rulings

`hyphen_rulings.tsv` holds the 27 line-end hyphens the JSON could not settle
(25 join, 1 `ck` Entwik-kelung GW 20.145, plus keep for Reflexion-in-sich
GW 20.162 and Magneto-Electro-Chemismus GW 20.309). Check them against the
print.

## Modernization

- 14 rulings flagged `uncertain` in `modernize_rulings.tsv`: Göthe/Göthes
  (→ Goethe), Kalkspath, Repercussion, Thone, Staab/Staabes, Cirkels,
  Oscilliren, Sclave/Sclaven, existire, definire, qualirt (a source typo?).
- 708 pattern-derived rulings (`hegel4-rule`, certainty `rule`) added beyond
  the 702 seeded from hegel2/hegel3: `ey→ei`, `Ae/Ue/Oe→Ä/Ü/Ö`, old `th→t`
  in German roots, `-irt/-iren→-iert/-ieren`, `c→k/z` in German-assimilated
  loans, pre-reform `ß→ss`, `aa→a`. Latin, French and English quotations
  and personal names were excluded from the c→k/z rule by an explicit list;
  skim the tsv for any that slipped through.
- `Bestimmtheit` and similar `-heit` words keep their `th`.
- "presentire", "produire", "détruire" and other French/Latin forms are
  deliberately untouched.

## Source notes

- A `)` after a note mark `∗` was a stray in the source and is dropped (3 note
  heads).
- Footnotes: 20 (12 single, 8 chained, 31 `<note>` parts in the XML).
- 671 `#vr` milestones and 5 vertical spaces dropped.
- "Kentnissen" occurs once in the source; left as printed.
