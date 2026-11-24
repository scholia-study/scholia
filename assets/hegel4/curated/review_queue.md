# hegel4 review queue

Working checklist; delete items as they are settled.

## TOC / headings

- The stray `II.` / `NATURPHILOSOPHIE.` and `III.` / `PHILOSOPHIE DES GEISTES.`
  half-titles are excluded.

## Sentence splitting

Citation apparatus (day ordinals before a month, number/Roman ordinal before
a Roman numeral or reference abbreviation) is protected by the corpus knob
`reference_protected_splits`. Still splitting, to be ruled on:

- [ ] Abbreviation runs: "Hrn. | Th.", "Vergl. | Anm. z. vorh. §.", "Ep. | I. 118.",
  "Princ. phil. | I. 15.", "Ch. | I. den einfachen …", "Phys. | III. ch. 4.",
  "S. 5 vergl. | S. 56 ff.", "S. 119. ff." and similar "S. <n>" fragments
  (about 28 mid-block starts).
- [ ] Enumerators standing alone as sentences: "a." | "Sein" in about 65 node
  headings, and "a." | "Die Bedingung …" in 16 list items (shared-splitter
  defect, also in hegel1/hegel2/kant1).
- [ ] `u.s.f.` / `u.s.w.`: 39 splits, at least 8 of them mid-sentence ("dem
  Rechte u.s.f. Befriedigung", "spartanischer u.s.f. Bürger").

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

## Source gaps

- [ ] § 96 breaks off after "α) Das Fürsichsein als Beziehung auf sich
  _selbst_ ist _Unmittelbarkeit_, und" (GW 20.133, line 25). The XML and the
  JSON agree, and both English controls carry the rest of the §. The missing
  German has to come from a print (zeno.org refuses automated access).

## Source notes

- A `)` after a note mark `∗` was a stray in the source and is dropped (3 note
  heads).
- Footnotes: 20 (12 single, 8 chained, 31 `<note>` parts in the XML).
- 671 `#vr` milestones and 5 vertical spaces dropped.
- "Kentnissen" occurs once in the source; left as printed.
