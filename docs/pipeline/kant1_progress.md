# Kant1 Review Progress

Progress log for manual review of `kant1_elements_to_md/*.md` → `kant1_md_reviewed/*.md`. One H2 per file with the changes applied and reasoning. Conventions and shared rules live in `.claude/skills/kant1-review/SKILL.md`.

## File 017 — Allgemeine Anmerkungen zur transscendentalen Ästhetik (AA 65)

- **Heading collapsed** from a 3-line typeset title (`## Allgemeine Anmerkungen` / `zur` / `## Transscendentalen Ästhetik.`) to a single H2 matching the TOC label. Pipeline emitted one H2 per visual line.
- **Missing B-marker `{{ 59 }}`** at section start recovered from `kant1_ocr_to_lines/page-074.json` — pipeline dropped the opening B-marker.
- **Mid-word B-markers moved to word boundaries** for `{{ 60 }}` through `{{ 72 }}` (e.g., `d{{ 60 }} erselben` → `derselben, {{ 60 }} Empfindung`). Pipeline consistently inserts B-markers between adjacent OCR tokens, which lands them inside words.
- **OCR errors fixed**: `fie`→`ſie`, `ſhuthetiſch`→`ſynthetiſch`, `Naum`→`Raum`, `wåre`→`wäre`, `finnlich`→`ſinnlich`, `müffe`→`müſſe`, `Saturu`→`Saturn`. All are systematic Fraktur misreads (long-s as `f`, `R` as `N`, `ä` as `å`, `n` as `u`).
- **Stray "³5" stripped** from body text — A-edition page reference artifact that doesn't belong to either reference system we track.
- **Missing text spliced**: "Daſein in Beziehung auf gegebene Objecte bestimmt) abhängigen Weſen" between "(die ſein" and "zuzukommen ſcheint", recovered from page-082 OCR. Pipeline drops clauses across some page boundaries.
- **Editor footnotes `[^1]`, `[^2]`, `[^3]` removed** (and their body references) on user instruction — only the author's `[^*]` footnotes are kept. Editor footnotes typically begin "Zusatz von A²" or are cross-references like "vgl. S. 69 Anm. 1".
- **Cross-page paragraph splits merged** where the pipeline split mid-sentence at AA page boundaries.

## File 018 — Beschluß der transscendentalen Ästhetik (AA 73)

- Heading collapsed to single H2 with both `{{{ 73 }}}` (AA) and `{{ 73 }}` (B) markers since both editions begin a new page at this section.
- **Stripped stacked `{{ 32 }}`** artifact — small digit appearing right next to the "73" marker near the heading. Doesn't fit AA or B sequence; matches the same pattern as `55` near `65` in file 017 and `14` near `74` in file 019. Signature/gathering numbers from the original print typesetting.
- Editor footnote `[^1]` (`Man vgl. S. 69 Anm. 1.`) and its body reference removed.
- Trailing `{{{ 74 }}} 14` stripped — that's the start of file 019's page plus a signature mark, doesn't belong here.
- `Säge` → `Säße` (OCR misread of `ß` as `g`); `_a priori_` italicized; `ſynthetiſche` normalized to match file 017's long-s convention.

## File 019 — Zweiter Theil. Die transscendentale Logik (AA 74)

- Title page only. Collapsed multi-line typeset ("Der / Transscendentalen Elementarlehre / Zweiter Theil. / Die transſcendentale Logik.") into single H2 matching TOC label. The "Der Transscendentalen Elementarlehre" is the parent section's title (already covered by file 012) so it's redundant here.
- Both `{{{ 74 }}}` (AA) and `{{ 74 }}` (B) markers added at heading since this is a new section starting both AA and B pages.
- Stripped stacked `14` artifact next to the AA page number (signature/gathering mark).
- Editor footnote `[^1]: A1: fann.` dropped.

## File 020 — Einleitung. Idee einer transscendentalen Logik (AA 74)

- Title-only section. Single H2 matching TOC; no markers (continuation from file 019 on the same AA/B page).

## File 021 — I. Von der Logik überhaupt (AA 74)

- Heading collapsed from 5 typeset lines to single H2 matching TOC.
- **Missing text spliced** between "Materie der sinnlichen" and "{{{ 75 }}} unter welcher etwas angeſchaut wird" — pipeline dropped the line "Erkenntniß nennen. Daher enthält reine Anſchauung lediglich die Form,". Restored from `page-084.json`.
- `{{{ 75 }}}` repositioned to before "Erkenntniß nennen" (true start of AA 75 = first body line of page 084).
- `{{ 75 }}` added before "unter welcher" — pipeline had failed to wrap the B 75 marker even though OCR clearly shows "75" at the right margin of body line 1.
- Five mid-word B-markers repositioned: `{{ 76 }}` (Deswegen darf → man aber), `{{ 77 }}` (kennen, wenn → man die Regeln), `{{ 78 }}` (Kanon → des Verſtandes), `{{ 79 }}` (Gebrauch hindern → oder befördern). All used the even/odd page margin convention to derive correct target word.
- Three cross-page paragraph splits merged (pipeline splits at AA page breaks even when sentences continue).
- OCR errors: `fann`→`kann`, `find`→`ſind`, `darans`→`daraus`, `Spontaneitat`→`Spontaneität`, `2.`→`2c.` (et cetera ligature), `Em pfindung`→`Empfindung`, `Pſy= chologie`→`Pſychologie`, `u. f. m.`→`u. ſ. w.` (long-s misread as `f`, `w` misread as `m`), `Säge`→`Säße`. Editor footnote `[^1]` removed.

## File 022 — II. Von der transscendentalen Logik (AA 77)

- Heading collapsed; no AA marker (continues on AA 77 from file 021).
- Three mid-word B-markers repositioned (`{{ 80 }}`, `{{ 81 }}`, `{{ 82 }}`). Cross-page paragraph at AA 77→78 split merged.
- `Ge= seßen` → `Geſeßen`, `finnliche` → `ſinnliche`, `Wiſsenschaft` → `Wiſſenſchaft`, `Gegenſtånde` → `Gegenſtände` (Fraktur umlaut OCR error).

## File 023 — III. Von der Eintheilung der allgemeinen Logik in Analytik und Dialektik (AA 79)

- Heading collapsed (III. / Von der Eintheilung der allgemeinen Logik / in / Analytik und Dialektik.) → single H2.
- **`{{ 33 }}` artifact stripped** — small digit stacked with the real `{{ 83 }}` margin marker (same pattern as file 018's `{{ 32 }}` near `{{ 73 }}`).
- `{{ 83 }}` repositioned: pipeline placed it before paragraph 2 ("Es iſt ſchon ein großer..."), but OCR shows the marker is in the right margin between paragraphs 2 and 3. Correct position is before paragraph 3 ("Wenn Wahrheit in der Übereinſtimmung...").
- Three more mid-word B-markers fixed (`{{ 84 }}`, `{{ 85 }}`, `{{ 86 }}`).
- `mau`→`man` (Fraktur u/n confusion), `Unwiſsenheit`→`Unwiſſenheit`, italicized `_conditio sine qua non_`.

## File 024 — IV. Von der Eintheilung der transscendentalen Logik in die transscendentale Analytik und Dialektik (AA 81)

- **`{{{ 81 }}}` stripped** from heading — already placed in file 023's body (AA 81 begins mid-paragraph in 023, not at file 024's heading).
- **`{{ 1 }}` artifact stripped** — stacked with `{{ 87 }}` (real B-marker at this heading).
- **Colophon artifacts stripped**: `## Kant's Schriften. Werte.` and `## III.` — page-090 bottom footer that pipeline mistook for headings. Also `{{ 6 }}` (signature mark at bottom of page).
- Cross-page sentence split at "daß uns Gegenſtände {{{ 82 }}} in der Anschauung" merged.
- Many Fraktur OCR errors: `trausscendentalen`→`transſcendentalen` (×2), `Deukens`→`Denkens`, `feinen`→`ſeinen`, `Erfenntniß`→`Erkenntniß`, `Bedingnug`→`Bedingung`, `transfcendentalen`→`transſcendentalen`. `{{ 88 }}` moved to before `(Objecte)` per left-margin (even page) convention.

## File 025 — Erste Abtheilung. Die transscendentale Analytik (AA 83)

- Heading collapsed from 4-line typeset (Der / Transfcendentalen Logik / Erste Abtheilung. / Die Transscendentale Analytik;) to single H2 matching TOC.
- **Trailing `## Der` / `## Transſcendentalen Analytik` stripped** — those belong to next section (file 026), pipeline included them by mistake at the end of file 025.
- **`60` stacked artifact stripped** — next to `{{ 89 }}` (real B-marker at this heading).
- `{{{ 83 }}}` + `{{ 89 }}` clustered at heading.
- `{{ 90 }}` moved from "ſich {{ 90 }} ſelbſt" to before "keine äußerlich" (right-margin convention: marker at end of line → next line begins B-page).
- OCR errors: `Wiſsenschaft`→`Wiſſenſchaft`, `feien`→`ſeien`, `deſsen`→`deſſen`, `Transfcendentalen`→`transſcendentalen`.

## File 026 — Erstes Buch. Die Analytik der Begriffe (AA 83)

- Heading collapsed to single H2 (no AA marker — AA 83 continues from file 025).
- Cross-page paragraph merged at AA 83→84 boundary (page 092 ended at "Unterſuchungen,", page 093 begins at "Begriffe, die ſich darbieten").
- `{{ 91 }}` moved from mid-word "dieſe{{ 91 }} s" to before "denn dieſes iſt" — page 093 line 6 has "91" at left margin (even page), so B 91 starts at that line.

## File 027 — 1. Hauptstück. Von dem Leitfaden der Entdeckung aller reinen Verstandesbegriffe (AA 84)

- Heading collapsed from 5-line typeset to single H2 (no AA marker — continues on AA 84).
- `{{ 92 }}` moved from "werden{{ 92 }}  zulezt" to before "ſyſtematiſchen Einheit" — page 093 line 26 shows "92" at left margin where the hyphenated "ſyſte-matischen" split lands. Reassembled the word and placed the marker at the natural boundary.
- Cross-page sentence "zuſammenhängen" → "müſſen" merged; `{{{ 85 }}}` placed at "zuſammenhängen {{{ 85 }}} müſſen".
- `feßt`→`ſeßt` (long-s OCR error), `lâßt`→`läßt` (Fraktur umlaut), `Staude`→`Stande` (u/n).

## File 028 — 1. Abschnitt. Von dem logischen Verstandesgebrauche überhaupt (AA 85)

- Heading collapsed; no AA marker (AA 85 already in file 027's body).
- Single long paragraph merged across AA 85→86 page break (pipeline split it).
- `{{ 93 }}` (right margin, odd page 094): moved to before "andere Art zu erkennen" — pipeline had "Vermögen {{ 93 }} der Anschauung" but OCR shows B 93 begins at the *next* line after "...Anschauung keine 93".
- `{{ 94 }}` (left margin, even page 095): moved to before "vorkommende Erſcheinungen" (joined word from "vor-"/"kommende" split).

## File 029 — 2. Abschnitt. Von der logischen Function des Verstandes in Urtheilen. §9 (AA 86)

- Heading: collapsed Zweiter Abschnitt. / § 9. / Von der logischen Function… → single H2 matching TOC.
- `{{ 95 }}` added at heading — OCR shows "95" at the left margin near the start of the section title on page 095.
- **Categories of judgments table** rendered as a markdown table. Pipeline had emitted each cell as a separate H2 heading; reconstructed the 4-column structure (Quantität / Qualität / Relation / Modalität, three values each). `Affertorische` → `Aſſertoriſche` (OCR misread `ſſ` as `ff`).
- Eight mid-word B-markers repositioned: `{{ 96 }}` through `{{ 101 }}` plus `{{{ 87 }}}`, `{{{ 88 }}}`, `{{{ 89 }}}`, `{{{ 90 }}}`.
- **`{{ 100 }}` recovered**: pipeline emitted bare `100.` in body text ("Inhalte 100. des Urtheils") without wrapping it as a marker. Converted to `{{ 100 }}` and placed before "des Urtheils beiträgt".
- Author footnote `[^*]` about Function des Verstandes / Urtheilskraft / Vernunft preserved.
- `affertoriſch` → `aſſertoriſch` throughout (multiple occurrences), `Confequenz`→`Conſequenz`, `ur.zertrennlich`→`unzertrennlich`, `Sah`→`Saß`, `Naum`→`Raum`, `Geſseße`→`Geſeße`, `vou`→`von`, `Prådicats`→`Prädicats`. Italicized Latin `_judicium singulare_`, `_judicia communia_`, `_antec._`, `_consequ._`.

## File 030 — 3. Abschnitt. Von den reinen Verstandesbegriffen oder Kategorien. §10–12 (AA 90)

- **Second categories table** (Quantität / Qualität / Relation / Modalität with 12 cells) rendered as markdown table. Original layout was a 2×2 compass (top/bottom centered, middle row two columns); flattened to a 4-column markdown table for readability.
- Latin terms italicized in the table cells: `_substantia et accidens_`, `_quando_, _ubi_, _situs_, _prius_, _simul_, _motus_, _actio, passio_`.
- Editor footnotes `[^1]` (`Der § 11 ist ein Zusatz von A³`) and `[^2]` (`Der § 12 ist ein Zusatz von A²`) dropped along with their body references. Author footnote `[^*]` (`Metaphyſ. Anfangsgr. der Naturwiſſenſch.`) preserved.
- 8 mid-word B-markers (`{{ 107 }}` to `{{ 116 }}`) and 9 AA markers (`{{{ 91 }}}` to `{{{ 99 }}}`) repositioned to verified OCR positions.
- **Missing text spliced** between AA 94→95 break: "suchung aus den Augen bringen, indem ſie Zweifel und Angriffe erregten," (page-104 line 1) was dropped by pipeline.
- **Missing text spliced** between AA 96→97 break: "Glied der Eintheilung geſeßt wird, alle übrige ausgeſchloſſen werden und" (page-106 line 1).
- **Missing text spliced** between AA 98→99 break: "wieder liefern und dazu zuſammenſtimmen. Also wird durch die" (page-108 line 1).
- Cross-page paragraph merges throughout.
- `Anschaunng`→`Anschauung`, `Eiutheilung`→`Eintheilung`, `denft`→`denkt`, `Eirkel`→`Cirkel`, `Vollzähligkeit` joined across page break, etc.

## File 031 — 2. Hauptstück. Von der Deduction der reinen Verstandesbegriffe (AA 99)

- Title-only section. Single H2 matching TOC; no markers needed (heading is mid-AA-99, not at start of any B-page transition).
- Stripped trailing pipeline noise including the start of file 032's heading.

## File 032 — 1. Abschnitt. Von den Principien einer transscendentalen Deduction überhaupt. §13 (AA 99)

- Heading collapsed; no AA marker.
- `{{ 117 }}` moved from "Hand haben{{ 117 }} ," to "indeſſen {{ 117 }} auch uſurpirte Begriffe" — page-108 OCR shows "117" at end of line "...indeſſen 117" (right margin, odd page), so B 117 starts at next line.
- `{{ 118 }}` moved from mid-sentence "Art, 118 die doch darin" to start of new paragraph "{{ 118 }} Wir haben jetzt ſchon zweierlei Begriffe" (page-109 has "118" at left margin of even page aligned with that paragraph's first line).
- `{{{ 100 }}}` repositioned with the joined word "beſtimmt ſind" — pipeline had it before the wrong paragraph and dropped the missing text "stimmt sind, und dieser ihre Befugniß…" between pages 108 and 109.
- `{{{ 101 }}}`, `{{ 119 }}`, `{{ 120 }}`, `{{ 121 }}`, `{{{ 102 }}}` repositioned (verified against pages 109–110 OCR for B-markers; AA 102+ approximated since pages 111+ not read in this round).
- Italicized `_quid iuris_`, `_quid facti_`, `_quaestionem facti_`, `_generatio aequivoca_`. Editor footnote `[^1]: A1: dieses.` dropped.

## File 033 — Übergang zur transscendentalen Deduction der Kategorien. §14 (AA 104)

- Heading collapsed; no AA marker (`{{{ 104 }}}` is at end of file 032's body, just before the §14 title).
- Body and markers from pipeline output with mid-word B-markers moved to nearby word boundaries.
- `inconsequent`→`inconſequent`, `fö allgemeine`→`ſo allgemeine`, `menschlische`→`menschliche`, `fategorischen`→`kategoriſchen`, `F{{ 129 }} unction` → `{{ 129 }} Function`. Editor footnote `[^1]: In A1 steht statt der nachstehenden drei Absätze...` dropped.
- **Verification pass against OCR pages 113–115**: corrected 4 marker positions:
  - `{{ 125 }}` moved one word earlier ("wenn der Gegenſtand {{ 125 }} die Vorſtellung" → "wenn {{ 125 }} der Gegenſtand die Vorſtellung") — OCR page 113 line at y=745 has "125" at left margin of line beginning with "der Gegenstand".
  - `{{ 126 }}` moved earlier in paragraph ("Begriffen nothwendiger Weiſe {{ 126 }} gemäß" → "denn alsdann iſt alle {{ 126 }} empiriſche Erkenntniß") — OCR page 113 line at y=1822 has "126" at left margin of line beginning with "empirische Erkenntniß der Gegenſtände".
  - `{{ 127 }}` moved later ("Ohne {{ 127 }} dieſe urſprüngliche" → "auf mögliche Erfahrung, {{ 127 }} in welcher alle") — OCR page 114 line at y=956 has "127" at end of line "...auf mögliche Erfahrung, 127", so B 127 starts on next line beginning with "in welcher".
  - `{{ 129 }}` moved one word earlier ("Begriffen die {{ 129 }} Function" → "Begriffen {{ 129 }} die Function") — OCR page 115 line at y=1354 has "129" at left margin of line beginning with the second half of joined "Begriffen"; marker placed after the joined word.
  - `{{{ 105 }}}` and `{{{ 106 }}} {{ 128 }}` cluster confirmed correct.

## File 034 — 2. Abschnitt. Transscendentale Deduction der reinen Verstandesbegriffe. §15–27 (AA 107)

The B-deduction, including §§15–27 and 11 author footnotes (`[^*]` through `[^**********]`, plus a `[^*-26]` for a second `*` footnote within §26).

- Heading collapsed; AA 107 marker at start of section.
- All 11 author footnotes preserved as `[^*]` through `[^**********]`, plus `[^*-26]` (a distinct `*` footnote that appears on page 134, separately from `[^*]` of §15).
- Editor footnote `[^1]` about A1 origin (`Der obige zweite Abschnitt, die §§ 15-27, bis zum Anfang des zweiten Buchs (S. 130) umfassend, ist eine Neubearbeitung...`) dropped.
- Section sub-headings (`Von der Möglichkeit einer Verbindung überhaupt.`, etc.) preserved as H3 since they're sub-divisions of the §§ structure.
- Categories table-like layouts (none in this file).
- **Initial pass** placed markers using pipeline positions with mid-word corrections moved to nearest word boundaries, but without OCR verification.
- **Verification pass against `page-116.json` through `page-135.json`** corrected ~17 marker positions:
  - `{{ 130 }}` moved from "kann alſo auch {{ 130 }} nicht" → "Anſchauung zugleich mit {{ 130 }} enthalten ſein" (OCR shows "130" at end of line "...zugleich mit 130", so B 130 starts at next line).
  - `{{ 131 }}` moved from third occurrence of "Mannigfaltigen" → second occurrence (with `[^*]` footnote) where page 117 line 1 begins with "131 Mannigfaltigen.*)".
  - `{{ 132 }}` moved one word earlier ("können; {{ 132 }} denn" vs "können; denn {{ 132 }} ſonſt").
  - `{{ 133 }}` moved from mid-clause to paragraph break ("angehören würden. {{ 133 }} Aus dieſer").
  - `{{ 134 }}` and `{{{ 110 }}}` clustered at "Anſchauung {{{ 110 }}} {{ 134 }} gegebene Vorſtellungen" — both AA and B page breaks land at the same joined word from page 118→119.
  - `{{ 135 }}` moved to "allererſt {{ 135 }} aufgenommen werden".
  - `{{ 136 }}` moved to "ſynthetiſche Einheit {{ 136 }} der Apperception heißt".
  - `{{ 137 }}` moved to "denn ohne das {{ 137 }} kann nichts".
  - `{{ 138 }}` moved to "im Raume zu {{ 138 }} erkennen" (joined word from "er-"/"kennen" split).
  - `{{ 139 }}` moved to "zugleich das {{ 139 }} Mannigfaltige der Anſchauung" (joined word "Mannigfaltige").
  - `{{ 140 }}` moved to "durch Aſſociation {{ 140 }} der Vorſtellungen" (joined word "Aſſociation" from "Aſſo-"/"ciation").
  - `{{ 141 }}` moved to "allenfalls nur auf {{ 141 }} kategoriſche" (joined word "kategoriſche" from "kate-"/"goriſche").
  - `{{ 142 }}` + `{{{ 114 }}}` clustered at "Verhältnißwörtchen {{{ 114 }}} {{ 142 }} iſt" (joined word from p.122→123 split).
  - **`{{ 162 }}` was at wrong position** ("Aber in der Zeit, {{ 162 }}…") — that's actually B 163's location per OCR page 135. Renamed existing to `{{ 163 }}` and moved one word earlier ("ſtehen. Aber {{ 163 }} in der Zeit").
  - **`{{ 162 }}` newly added** at the section-separator paragraph break before "Wenn ich alſo z. B. die empiriſche Anſchauung eines Hauſes" — page 134 OCR shows "162" at right margin of a position between paragraphs 3 and 4 of §26.
  - **`{{ 164 }}` was misplaced mid-sentence**; moved to start of paragraph "{{ 164 }} Es iſt um nichts befremdlicher" per page 135 OCR (left margin, even page → marker aligns with that line's start).
- **Verification pass against OCR pages 136–139** for `{{ 165 }}`–`{{ 169 }}`: corrected 4 of 5 positions:
  - `{{ 165 }}` moved one word later ("alle Erſcheinungen {{ 165 }} der Natur" → "alle Erſcheinungen der {{ 165 }} Natur") — OCR page 136 y=1054 has "165" at end of line "...alle Erscheinungen der 165"; right margin on odd page → next line ("Natur, ihrer Verbindung…") begins B 165.
  - `{{ 166 }}` already correctly placed after joined "derſelben" (page 137 y=275 has "166" at left margin of line "selben gegeben iſt…", joined from "der-"/"selben" split).
  - `{{ 167 }}` moved from second "Begriffe" to after first "Begriffe" ("oder dieſe {{ 167 }} Begriffe machen" → "macht dieſe Begriffe, {{ 167 }} oder dieſe Begriffe machen") — OCR page 137 y=783 has "167" at left margin of line beginning with second half of joined first "Begriffe" ("griffe, oder diese Begriffe machen").
  - `{{ 168 }}` moved much later ("Mittelweg {{ 168 }} entſcheidend" → "den Kategorien {{ 168 }} die Nothwendigkeit") — OCR page 138 y=528 has "168" at end of line "...den Kategorien 168"; right margin → next line ("die Nothwendigkeit mangeln…") begins B 168.
  - `{{ 169 }}` moved much later ("in Raum und {{ 169 }} Zeit überhaupt" → "der urſprünglichen {{ 169 }} ſynthetiſchen Einheit") — OCR page 138 y=1682 has "169" at end of line "...der urſprüng- 169"; right margin → next line ("lichen ſynthetiſchen Einheit…") begins B 169, marker placed after joined word "urſprünglichen".
- File 034 is now fully OCR-verified across all B-markers (B 130–B 169) and AA markers (AA 108–AA 130).

## Convention notes (used throughout)

- **Page-marker placement rule** (from `SKILL.md`): for recto/odd AA pages, the B-marker sits at the right margin at the end of a line and B-page begins on the *next* line. For verso/even AA pages, the B-marker sits at the left margin at the start of a line and B-page begins on *that* line.
- **Joined words across page breaks**: when a word is hyphenated across pages (e.g., "Mannig-" / "faltigen"), the marker is placed at the natural boundary AROUND the joined word — usually right before the joined word for clarity, since the second half is where the new B-page begins.
- **Footnote categories**: author footnotes (`[^*]`, `[^**]`, etc., keyed to `*)`, `**)` in the original print) are preserved; editor footnotes (`[^1]`, `[^2]`, etc., keyed to apparatus-style `¹)`, `²)`) and their inline references are stripped. Editor notes typically begin "Zusatz von A²", "A¹: …", or are scholarly cross-references like "vgl. S. 69 Anm. 1".
- **Stacked-number artifacts**: small digits stacked next to real page numbers (e.g., `32` near `73`, `55` near `65`, `14` near `74`, `12` near `79`, `33` near `83`, `1` near `87`, `60` near `89`) are signature/gathering numbers from the original print typesetting. Stripped throughout. They form no monotonic sequence and don't fit any reference system.
- **Colophon footer stripping**: `Kant's Schriften. Werke. III.` appearing as `## ` heading in pipeline output is the running footer of the Akademie-Ausgabe Volume III, not content — stripped along with associated signature digits (`6`, `7`, `8`, `9`, etc.) at page bottoms.
- **Long-s normalization**: normalized to Fraktur convention (`ſ` at syllable-initial positions before vowels, short `s` at syllable-end) where the OCR was inconsistent, matching the style established in files 014, 017, etc.
