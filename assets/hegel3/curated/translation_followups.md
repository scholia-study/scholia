# hegel3 follow-ups

## Lost formulas (facsimile recovery pending)

The DTA transcription carries 19 empty `<formula/>` elements — formulas
of the 1812 print it did not transcribe (its `[Formel]` apparatus tokens
are dropped by a `drop` ruling; the eleven TeX-transcribed fractions
render as plain `2/7`). The lost formulas sit on 1812 pages 214, 215,
218, 221 (×2), 222 (×5), 223 (×2), 224, 225, 236, 237 (×3), 238 — all in
the quantitative-infinity/ratio stretch (nodes 087–092 territory). Both
German layers and the translation read past them silently, as the
diplomatic layer always has. Recovery means reading the DTA facsimile
pages and inserting the printed expressions at the marked positions in
both German layers (and mirroring them in the English); until then the
affected sentences carry the prose without the displayed expressions.

## Print errors left standing

- 014 (1812 p. 27): "leichter, als ihn angeben zu geben" — evidently a
  printer's slip (dittography?); the intended reading is uncertain
  without the facsimile, so both German layers keep it and the English
  renders the evident sense ("than to specify it"). Clear single-word
  print errors ARE repaired in the reading layer via
  `modernize_rulings.tsv` (Richts → Nichts, Richtſeyns → Nichtseins,
  zn → zu, weißt → weist) and `modernize_ops.tsv` (the copula the
  page-23 break dropped); the diplomatic layer keeps them all as
  printed.

- 037 (1812 p. 66): "verschiedener, und ihn Beziehung in einer und derselben
  Rücksicht" — "ihn" is evidently corrupt (ihre? in?); reading uncertain
  without the facsimile, so both German layers keep it as printed and the
  English renders the evident sense ("their reference").
- 031: the print's stray sentence break ("…oder das Ansichsein. Das
  herausgetreten ist, …") and verbless fragment ("sie am Etwas selbst.")
  are authentic 1812 pointing, mirrored in the English.

- 088 (1812 p. 182): "abgewonnen; ſondere wieſes iſt vor wie nach…" —
  garbled in the print (sondern dieses? sondern es?); reading uncertain
  without the facsimile, so both German layers keep it as printed and the
  English renders the evident sense.

- 091 (1812 p. 204): "es entzieht nur wieder ein gleichgültiges Quantum"
  — "entzieht" is likely a compositor's error (entsteht?); reading
  uncertain without the facsimile, so both German layers keep it as
  printed and the English renders it literally ("withdraws").

- 104 (1812 p. ~277): "der ganze Unterschied des Quantitäten" —
  ungrammatical in the print (der? den?); kept as printed, English renders
  the evident sense ("of the quantities").

- 120 (1812 p. 331): "durch seine, ihr gleiche, Negativität, sich mit zu
  vermitteln" — the print drops the object of "mit" (sich? ihr?); reading
  uncertain, so both German layers keep it as printed and the English
  renders the evident sense ("to mediate itself with itself").
- 119 (closing sentence): the print's anacoluthon (the "wodurch dann der
  Begriff des Wesens…" clause never receives a finite verb) is authentic
  and mirrored in the English.

- 092 (1812 p. ~219): "Die Eltern unter den Neueren" — evidently "die
  Älteren" (period spelling or slip); kept as printed, English renders
  "The elder among the moderns".

## Deliberate translation liberties (gate-driven, documented)

- 004: the German's triple "die Logik des Seins … die Logik des Wesens …
  die Logik des Begriffs" list is rendered with English list-ellipsis
  ("the logic of being; that of essence and that of the concept") — the
  literal repetition is an unavoidable ≥15-word lock against both proxy
  controls.


## Minor cross-corpus register variants (vs hegel2)

- Ein Eins: hegel3 renders "one One" (label "1. One One"); hegel2's
  parallel passages have "One one" / label "The One One of Attraction".
- Vielheit: hegel3 = "plurality" (per the locked table); hegel2's
  early files 065/066 render the same word "multiplicity".
