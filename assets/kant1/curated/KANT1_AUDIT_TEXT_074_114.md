# Kant1 Curated Text Audit Report — prefixes 074–114

**Date:** 2026-06-13
**Scope:** the 41 newly added files (prefixes 074–114) in `assets/kant1/curated/` — `md_reviewed` (REV), `md_modernized` (MOD), `md_modernized_translated` (TRA); ~84k words per layer (Antinomy sections 3–9 + solutions, Ideal of pure reason, Appendix to the Dialectic, complete Doctrine of Method).
**Method:** (1) mechanical consistency pass (marker sequences/continuity/parity, block parity, footnote ref/def parity, emphasis-span parity, suspicious-pattern scan); (2) 14 parallel content-review agents, each owning a disjoint file batch in all three layers, auditing against the rules in the `kant1-review` and `kant1-modernize-translate` skills; every suspect reading verified against the raw OCR (`raw/ocr_to_lines/page-{AA+9}.json`), the 1911 reprint page scans (`raw/pages/*.png` — authority for Sperrdruck/bold/Antiqua), and the Cambridge/Guyer-Wood control (`assets/kant1/control/text/part0051–0059.html`). Fixes were applied directly; documented-only items follow the open policy rows of `AUDIT_TEXT.md` §4.
**Verification:** `kant1_md_to_struct` + `kant1_md_translation_to_struct` pass after all edits — blocks 1267/1267, sentences 4652/4652, footnotes 75/75, page markers 1509/1509 (German vs English). **No DB import was run.**

**Status legend:** ✅ fixed in the curated files · ⬜ documented only (open corpus policy / next pass) · ❓ needs a human/policy decision.

**Headline numbers:** ~45 critical (meaning-changing) fixes; ~340 Sperrdruck spans + ~70 Latin italic spans restored to the German layers (the Sperrdruck-restoration step had been skipped on REV/MOD for 083, 086–088, 090–095, 097, 100, 105, 108–114); 17 missing page markers inserted (5×B in 086, B 611 in 091/092, AA 418 in 096, B 680/691/693 brace repairs in 099, AA 455–460 + B 705/707/732 in 100, B 789/796 in 106, B 777/857 + 29 TRA-only markers in 105/111/112/113); ~130 markers repositioned to OCR-verified print line breaks; 1 broken footnote pair repaired (113); 1 dropped footnote sentence restored (091 — **sentence-count change, see §7**).

---

## 1. REV (md_reviewed)

### 1.1 Critical (meaning-changing or non-word corruptions)

| # | St | File | Before | After | Evidence / note |
|---|---|---|---|---|---|
| R1 | ✅ | 074 | "zeigt die Philoſophie eine Würöe" | Würde | page-331 OCR |
| R2 | ✅ | 076 | "das Nichtmaß, wornach" | Richtmaß | page-346.png, R↔N misread |
| R3 | ✅ | 077 | "Object können, wir allen Umfang" (spurious comma; `{{ 523 }}` misplaced) | "können wir … Wahrnehmungen {{ 523 }} zuſchreiben" | page-350 OCR margin |
| R4 | ✅ | 078 | logiſehe; ***unbefehen***; "als dort vorausſehen"; "Schein, der daher entſpricht" | logiſche; ***unbeſehen***; vorausſeßen (presuppose); entſpringt | pages 353/356; control "sight unseen"/"presuppose" |
| R5 | ✅ | 083 | "keinen undren Gegenſtand" | andern | page-370 |
| R6 | ✅ | 084 | "umgeben haben. Freiheit im praktiſchen…" (dropped word) | "haben. Die ***Freiheit im praktiſchen Verſtande***" | page-372.png |
| R7 | ✅ | 086 | "anders beestimmtt?" | beſtimmt | page-385 |
| R8 | ✅ | 086 | "ſo, daß nicht die Bedingungen…" | "nicht" removed (1911 print lacks it) | page-382.png zoom; MOD/TRA keep the received-text emendation — see ❓ P6 |
| R9 | ✅ | 088 | "transſcedent(e)" ×2 | transſcendent(e) | pages 390/391 |
| R10 | ✅ | 090 | "der aber mit die Idee der Weisheit" | mit der | page-393 |
| R11 | ✅ | 091 | "durchgängig bemitteilt vorgeſtellt" | beſtimmt | page-397 |
| R12 | ✅ | 091 | footnote `[^*]`: second print sentence missing | restored "Die ***Beſtimmbarkeit*** eines jeden ***Begriffs*** … (_Universalitas_) … untergeordnet." (all three layers) | page-395.png; **+1 sentence — two-pass import, §7** |
| R13 | ✅ | 091 | fn `[^***]` "ſondern ſo auf der Verknüpfung" | spurious "ſo" removed | page-401.png |
| R14 | ✅ | 092 | `dadu{{ 612 }} rch` (marker mid-word) | "viel zu leicht, {{ 612 }} als daß ſie dadurch" | page-401 margin |
| R15 | ✅ | 093 | "ſo entſpricht ein/kein Widerſpruch" ×2 | entſpringt ×2 | page-407 (arises, not corresponds) |
| R16 | ✅ | 093 | "my Begriff nicht den ganzen" | mein Begriff | English leakage in the 100-Thaler sentence |
| R17 | ✅ | 097 | "nothwendig vorausſehen" ×2 + footnote | vorausſetzen | pages 430/431 tz-ligature; TRA "presuppose" |
| R18 | ✅ | 099 | `{{{ 680 }}}` / `{{{ 691 }}}` (+dup `{{ 691 }}`) / `{{{ 693 }}}` — B-pages typed as AA markers | `{{ 680 }}` before "Art…"; single `{{ 691 }}` before "allen unſeren Beobachtungen"; `{{ 693 }}` at "Bedin{{ 693 }}gungen" | pages 441/447/448 margins |
| R19 | ✅ | 099 | "nothwendig vorausſehen müſſen" | vorausſeßen | page-441.png |
| R20 | ✅ | 100 | "bei Seite ſehen wollen"; "folgend vorauszuſehen" | ſetzen; vorauszuſetzen | pages 455/465 tz-ligature |
| R21 | ✅ | 100 | "entſproſſen wären" ×2 | entſprungen wären | pages 453/461 |
| R22 | ✅ | 100 | "eines Jrrthumes überzeugt werden" | überführt (convicted) | page-462; TRA "convicted" |
| R23 | ✅ | 100 | "nur eine Jnheit mehr" | Einheit | page-462 |
| R24 | ✅ | 104 | heading duplicate `{{ 741 }}` | `{{ 740 }}` (B 740 in progress at section start; 103 ends `{{ 740 }}`) | page-477 margins; 105/107 heading-repeat precedent |
| R25 | ✅ | 104 | "laſſen ſich transſcendentaler Sätze" | transſcendentale | page-482 |
| R26 | ✅ | 105 | "eine andere … entgegen zu ſehen" | entgegen zu ſetzen | page-503.png tz-ligature; control "set against" |
| R27 | ✅ | 106 | "Mißtrauen auf ſeine … Principien zu ſehen" | zu ſetzen | page-507.png |
| R28 | ✅ | 110 | "gleichgültig bei Seite ſehen können" | ſeßen | page-531.png 4× zoom |
| R29 | ✅ | 111 | "zur unmittelbaren Erkenntniß neuer Gegenſtände" | Kenntniß | page-539; Kenntnis≠Erkenntnis; control "acquaintance" |
| R30 | ✅ | 113 | f↔ſ / misread cluster (13): fehr; fubjectiv; wachfen; Vernunſtwiſſenſchaften; fie; philofophiren; dcs; anfeßt; Metaphyfif/Metaphyfik ×3; inithin; Bunkte; find; Marimen | ſehr; ſubjectiv; wachſen; Vernunft-; ſie; philoſophiren; des; anſeßt; Metaphyſik; mithin; Punkte; ſind; Maximen | pages 547–558 |
| R31 | ✅ | 114 | Kuinen; fie; folle; laffen | Ruinen; ſie; ſolle; laſſen | pages 559–561 |
| R32 | ✅ | 114 | "dreifache Abſicht, in so welcher" | in welcher | page-559: margin line-count "30" read as "ſo" |

### 1.2 Major (wrong/dropped words, grammar breaks, marker placement)

| # | St | File | Before → After | Evidence |
|---|---|---|---|---|
| R33 | ✅ | 074 | ſchlechteſtens → ſchlechthin; idealiſerenden → idealiſchen | page-332/337.png; control "simply"/"ideal" |
| R34 | ✅ | 075 | dropped clause at B507/AA332 seam: "haben allein, {{ 507 }}…" → "haben allein das Eigenthümliche an ſich, {{ 507 }}…"; fn word order restored ("außer der Sphäre … geſetzt wird, die…"); spurious comma before {{ 510 }} | page-340/341/342 |
| R35 | ✅ | 078 | vermuthlich (was vermutlich); Univerſum → Univers (print); "einer und" → "einer- und" (suspended hyphen) | pages 354/357 |
| R36 | ✅ | 086 | "Charakter ſeinem Willkür" → ſeiner; gewiſße ×2 → gewiſſe | pages 379/381 |
| R37 | ✅ | 087 | "ſuchen müſße" → müſſe; `_substantia phaenomenon_`, `_ens extramundanum_` italicized | page-388; Antiqua |
| R38 | ✅ | 088 | "irgend einen Begriff" → einigen | page-391.png zoom |
| R39 | ✅ | 091 | "aller Verneinungen" → alle; "ableiten" → abzuleiten; "vorausſetzen?" → vorauszuſetzen; "ableitet" → ableite; Verhältniſſe ×2 → Verhältniß; Antheile → Antheil; Correlat → Correlatum (fn); Beſtimmung → Beſtimmungen (fn, print); prototypon → _Prototypon_; Copien → Copeien | pages 394–400 |
| R40 | ✅ | 092 | Selbſtbeſitze → Selbſtbeſitz; ſchicke → ſchicken; müſße/Vernunftſchluſße → müſſe/Vernunftſchluſſe; Überfürenderes → Überführenderes | pages 402–404 |
| R41 | ✅ | 093 | "dass die Illuſion in Verwechslung" → "das … Verwechſelung" (print); Exiſtentialſatz → Exiſtenzialſatz; "der Sinnen" → der Sinne | pages 409/411 |
| R42 | ✅ | 094 | allerrealſten → allerrealſte; transscendentalen → transscendentale (fem.); Keihe → Reihe; find → ſind; Anfehen/Abficht → Anſehen/Abſicht; Unterſag/Grundſay/sett → Unterſaß/Grundſaß/ſeßt; "ab soluten"/"dasei"/vernünfteľnden → absoluten/"da ſei"/vernünftelnden | pages 413–419 |
| R43 | ✅ | 095 | "zweite Regel gebietet" → "Regel euch gebietet" (dropped word); hypostafirt/fich/folchen/Grundfäße → ſ-forms; Vernunſtprincipien → Vernunftprincipien | pages 420–422 |
| R44 | ✅ | 096 | Caſualität ×3 → Cauſalität; missing `{{{ 418 }}}` inserted | pages 424/427 |
| R45 | ✅ | 097 | Gegenſände → Gegenſtände; Beweien → Beweiſen | pages 431/432 |
| R46 | ✅ | 099 | "könneu" → können; stray " g" at `{{ 689 }}` removed | pages 442/446 |
| R47 | ✅ | 100 | ſodern ×2 → ſondern; lerteren → letzteren; Anthroporphismen → Anthropomorphiſmen; transſcendenter → -ten; unbeſtimmmt → unbeſtimmt; "kein poſitives Hinderniß" → keine poſitive (archaic fem.); Jch → ich; Subſtanz → Subſtanzen (Analogie list) | pages 454–467 |
| R48 | ✅ | 103 | "das eigentliche Geſchäfte" → eigenthümliche | page-475; control "special job" |
| R49 | ✅ | 104 | Syntheſe ×13 → Syntheſis; "deſſen" ×2 → desjenigen; Gegenteile → Gegentheile; Ausdruc ×2 → Ausdruck; Axiome ×4 → Axiomen; Algebra → Algeber (print); Dogmata → Dogmate (print); dropped-clause check at all AA seams clean | pages 478–492 |
| R50 | ✅ | 105 | forſehende → forſchende; "wohldenkenden Menſchen" → Manne; billigerweiſe → billigermaßen; + 11 minor print-inflection restorations (Beſiße→Beſitz, zurechte, andres, giebts ×2, wahre, gewiſſer, ächten ×2, minderem, anvertrauet, wohlgemeinte, dieſelbe, eigenthümlichen) | pages 493–503 |
| R51 | ✅ | 106 | transſcendenden → transſcendenten; "aus derſelben" → "als ſelbiger" (print; see ❓ P7); ***Principien*** de-emphasized; kein → keinen; ſeines → ſeiner; Beſitze → Beſitz; Eigentume → Eigenthume; `_Facta_` ×3 italicized; 2 stray backticks removed | pages 504–511 |
| R52 | ✅ | 107 | zuletßt → zuleßt; deſſenjenigen → desjenigen; Vernichtung → Vernichtigung (print); "der derſelben" → derſelben; problematiſch → problematiſche | pages 513–518 |
| R53 | ✅ | 108 | Eigentheit → Eigenthümlichkeit; rauſen → raufen; "von einem" → "von Einem" (Kant's capital-E emphasis); Beweiſe → Beweiſen; transſcendentalen → transſcendentale | pages 519–525 |
| R54 | ✅ | 110 | Grundmarime → Grundmaxime; "alle unſere Erkenntniß überſteigt" → Kenntniß; Zuruſtung → Zurüſtung; afficiert → afficirt; vorausgeſett → vorausgeſeßt; "nämlich iſt" → "nämlich: iſt"; heading "Erster Abschnitt." → "1. Abschnitt." (label authority) | pages 528–531 |
| R55 | ✅ | 111 | "keine Naturgeſeße hervorbringen" → nicht; ſchlechtherdings → ***ſchlechterdings***; Geſehen/Naturgeſehen → Geſeßen/Naturgeſeßen (tz→h); beſtimmmen; morališch; Geſetze-junk; "Gebrauche" → Gebrauch; anderes → andres | pages 532–539 |
| R56 | ✅ | 112 | fich/folche/fittliche → ſ-forms; Ahsichten → Abſichten; entriffen → entriſſen; Überredung capitalization ×2 per print; Fürwahr= halten/ge= ſezt joined; `_consentientia…_`, `_a priori_` italicized | pages 540–547 |
| R57 | ✅ | 113 | "³0 Philoſophen"/"Moraliſten ³)" margin/apparatus digits removed; "Wissen ſchaft" joined; 5 hyphenation scars joined; "2c" → print has "2c." but period reverted — see ❓ P8; "nennt" → nenut (print-faithful reprint slip, kept per §5 precedent); `da Philo- {{{ 546 }}} sophen` → "da Philoſophen {{{ 546 }}} selbst" | pages 547–558 |

REV documented-only residue (⬜, open policies `AUDIT_TEXT.md` §4 #312–#315, #321): ſſ/ſß/ſs-mixing (Intereſße, gewiſße families, passim); long-ſ vs round-s mixing in whole blocks; readable tz-strays (Saz, lettere, Säge, jezt, zulezt, Sittengeſez families); å/ā/ť/ľ accent junk; J/I mixing (Jdee/Idee); ꝛc. rendered "2c."/"zc."/"u. ſ. w."; x→r Eriſtenz family (eriſtirt, erponirt — sanctioned both ways); proper names letterspaced in print but unemphasized in every layer: **Lambert** (075), **Leibniz** (093, 111), **Plato** (090), **Wolffiſche/Wolffiſchen, Epikur, Ariſtoteles, Wolff, David Hume** (113/114) — apply once #321 is decided. Print-faithful oddities kept: 083 "Dagegen das durchgängig Bedingte" anacoluthon (no "kann" in print); 086 "geräth etwa in einen neuen Zuſtand" (negation distributes); 088 "unſere neue Kenntniſſe"; 090 "obgleich es niemals erreichen können" / "Hirngeſpinnſte"; 100 centered "\*" divider before final paragraph not represented; 113 "diejenige thun", "im ſyſtematischem Zusammenhange", body "Philoſoph nenut"; 114 "ok es gleich" (defective glyph, ❓), "aus einen bloß", "vor dem rohen Zuſtande", "keine gründliche und zuverläſſigere", "Grundsägen".

---

## 2. MOD (md_modernized)

### 2.1 Critical

| # | St | File | Before → After | Note |
|---|---|---|---|---|
| M1 | ✅ | 077 | "die reality einer empirischen Vorstellung" → Wirklichkeit | English leakage |
| M2 | ✅ | 078 | "nicht als such ein solches" → "als ein solches"; voraussehen → voraussetzen | English leakage; tz fix |
| M3 | ✅ | 086 | "die/der lettere" ×2 → letztere; beestimmtt → bestimmt | non-words in modern layer |
| M4 | ✅ | 087 | leaked LLM editorial bracket "[Wait, original was 'bloß intelligibelen Bedingung'…]" + broken sentence → "einer bloß intelligiblen Bedingung … zu gründen; sondern" | drafting artifact removed |
| M5 | ✅ | 091 | fn `[^***]` "ihres manifolden" → Mannigfaltigen; fn `[^**]` "Abgrund der Ignoranz"/"so gross"/"eine grosse" → Unwissenheit/groß/große; fn `[^*]` missing sentence restored; bemitteilt → bestimmt | mirrors REV |
| M6 | ✅ | 097 | voraussehen ×3 → voraussetzen | mirrors REV |
| M7 | ✅ | 099 | voraussehen → voraussetzen; markers 680/691(dup)/693 repaired | mirrors REV |
| M8 | ✅ | 100 | 6 criticals mirrored (setzen, voraussetzen, entsprungen ×2, überführt, Einheit) + "zuzuschreiben gedechte" → gedächte | |
| M9 | ✅ | 103 | "sehr beträglich" → betrüglich | non-word |
| M10 | ✅ | 104 | dropped clause restored: "dadurch zu unterscheiden vermeinten, dass sie habe" → "dass sie von jener sagten, sie {{{ 470 }}} habe"; "durch diesen arbitrarily Begriff" → willkürlichen | page-478; English leakage |
| M11 | ✅ | 105/106 | entgegenzusehen → entgegenzusetzen; "zu sehen" → "zu setzen" | mirrors REV |
| M12 | ✅ | 110 | "bei Seite sehen" → setzen | mirrors REV |
| M13 | ✅ | 111 | corpus-outlier ß/OCR residue swept (~50 tokens: daß ×19, muß ×8, Geseße/Gesehen family ~15, Interesße ×4, Kenntniß ×3, jeßt, lezten…) → valid modern forms; Erkenntnis → Kenntnis (B 833) | layer must be valid modern German |
| M14 | ✅ | 113/114 | criticals mirrored from REV + "Matematik/matematisch" ×13 → Mathematik; 114 broken German (ok→ob, einen→einem, Ruinen, sie, solle, lassen, Grundsätzen, Lehrsatz) | |

### 2.2 Major

| # | St | File | Before → After | Note |
|---|---|---|---|---|
| M15 | ✅ | 074 | schlechtestens → schlechthin; idealisierenden → idealischen (B504 only) | mirrors REV |
| M16 | ✅ | 076 | "anträffe" → antreffe (mood) | Konjunktiv I as REV |
| M17 | ✅ | 077 | "sensible Anschauung" → sinnliche | lexical drift |
| M18 | ✅ | 079 | "der Sinnen" → Sinne; hinzugefügt → hinzugesetzt | drift vs REV |
| M19 | ✅ | 086 | "seinem Willkür" → seiner | mirrors REV |
| M20 | ✅ | 091 | Verhältnis ×2, Anteil, ableite, abzuleiten, _Prototypon_, "keine andere Gegenstände" → anderen | mirrors REV + MOD inflection |
| M21 | ✅ | 092 | müsste → müsse (Konj. I); "verständlicher gewordener" → verständlich gewordener; Selbstbesitz; `d i.` ×2 → `d. i.` | print mood/wording |
| M22 | ✅ | 093 | menge → Menge; Existenzialsatz; "der Sinnen" → der Sinne; "ohne … Anschauung zu sehen, welche" verb restored (052-class; pre-existing) | mirrors REV |
| M23 | ✅ | 094 | einzigiger → einziger; "dasei" → da sei; "euch" restored (095) | typos/mirror |
| M24 | ✅ | 096 | "Nun thut man" → tut; `{{{ 418 }}}` inserted | modernization consistency |
| M25 | ✅ | 097 | "irgende ein(e)" ×3 → irgend; "spekulativ Beweise" → spekulative | non-words |
| M26 | ✅ | 100 | mass Fraktur-residue sweep (seßt, Geseße-family ×17, Jdentität, Jnhalt, Jnteresße, Kenntniß ×5, gewisße, Jrrthum, Jst, "kann Ich"…) → modern forms | layer validity |
| M27 | ✅ | 103 | heading "Disziplin" → "Disciplin" (label authority); "das eigentliche Geschäft" → eigentümliche | |
| M28 | ✅ | 104 | müsste → müsse; "dessen" ×2 → desjenigen; "möglicher empirischen" → empirischer; "ebenso sowohl" → ebensowohl; spurious comma removed | print |
| M29 | ✅ | 105 | "ein Frommer" → frommer (adj.); müsste → müsse; Prinzipien → Grundsätze (B767); billigermaßen; Besitz | print/REV |
| M30 | ✅ | 106 | vollständiger → völliger; darthun → dartun; Besitz | REV/modernization |
| M31 | ✅ | 107 | "ohne Sinnen" → Sinne; dessenjenigen → desjenigen; behauptung → Behauptung | |
| M32 | ✅ | 108 | "Sätze" ×2 → Beweise; Bewerber → Bewunderer (meaning flip); billigerweise → billigermaßen; "transzendentaler Satz" → transzendentale; Einem; Reziprozität → Reziprokabilität; argument → Argument; Gegenstände → Gegenstande | REV/print |
| M33 | ✅ | 110 | heading → "1. Abschnitt."; Erkenntnis → Kenntnis; thun/Thun ×5 → tun/Tun; "nämlich: ist" | mirrors REV |
| M34 | ✅ | 112 | Matematik → Mathematik; an sich/sittliche/solche/Absichten/entrissen/Fürwahrhalten/gesetzt/Überredung ×2; ~16 invalid modern forms (lettere, Säße, Troße, stußig, besize, paßt, müßte ×2, Vorausseßung…) | layer validity |
| M35 | ✅ | 113 | tz-ligature ß-artifacts ~30 tokens (unterstüßen, Grundsäße, Gesezgeber, zulezt, jeziger, Pläßchen, Schußwehr…) → modern tz forms; Kenntniß ×2 → Kenntnis | layer validity |

MOD documented-only (⬜, policies #316/#317): frontmatter-label modernization depth; hiebei/hievon/hiezu/darnach/wornach retained; th-forms (Theil, Urtheil…), -iren verbs, c-spellings (Principien, objectiv, Construction), "giebt", "ahnden", "gesammten" — await the MOD style policy; 086 keeps the received-text "nicht" (❓ P6); 093 "diese gedachten hundert Taler" grammatical modernization; 095 "konstitutives" k-form; 075 fn "außerhalb"+clause-order modernization; 045-class "Akzidens" not present here.

---

## 3. TRA (md_modernized_translated)

### 3.1 Critical

| # | St | File | Before → After | Evidence / note |
|---|---|---|---|---|
| T1 | ✅ | 078 | "The series of appearances is to be encountered" → conditions | German "Reihe der Bedingungen"; control epub shares the error |
| T2 | ✅ | 082 | "regress in the composition of what" → decomposition | page-369 "Decompoſition"; Guyer-Wood epub error, meaning-flipping |
| T3 | ✅ | 086 | footnote dropped clause restored: "But how much of it is pure effect of freedom, how much…" | control epub also lacks it; REV/MOD + page-382.png |
| T4 | ✅ | 091 | fn `[^*]` run-on split into the restored second sentence; over-emphasis trimmed; "subordinated" restored | part0052 + page-395.png |
| T5 | ✅ | 104 | three untranslated rubrics "1./2./3. Von den ***Definitionen/Axiomen/Demonstrationen***." → "1. On **definitions**." etc. (bold per print) | part0056 |
| T6 | ✅ | 108 | "The third eigentümliche rule" → "***third*** peculiar"; "The eigentümliche cause" → "real" | untranslated German; MOD "eigentliche" |
| T7 | ✅ | 112 | "surrendering the former … the latter … torn away" → latter/former swapped back | German B 856; control shares the inversion |
| T8 | ✅ | 113 | `[^*]`/`[^**]` body refs restored (both were missing; raw `*` asterisks normalized); Guyer page-ref "***A834/B862***" leaked as emphasized text removed; "***architectonic I***" span trimmed | page-552/556 anchors |
| T9 | ✅ | 113 | 20 missing markers inserted (AA 539–546, B 861–878 odd set) + 5 repositioned; stray `{{{ 549 }}}` relocated | REV/MOD correspondence |
| T10 | ✅ | 114 | Persius motto repaired: unclosed italics, "non ego euro" → "non ego curo", "(Pers.)" restored | page-561 Antiqua |
| T11 | ✅ | 111 | 9 missing markers inserted (AA 526/527/528/531, B 837/838/840/842/844); `{{ 843 }}` moved a sentence | REV/MOD anchors |
| T12 | ✅ | 092/093 | `{{ 611 }}` repeat added to 092 heading; "unconditioned down through" junk word removed; 16 markers realigned | margins |

### 3.2 Major

| # | St | File | Before → After | Note |
|---|---|---|---|---|
| T13 | ✅ | 074 | `_a priori_` restored ("pure rational unity _a priori_)") | Latin parity |
| T14 | ✅ | 075 | — (no TRA-specific fixes beyond markers) | |
| T15 | ✅ | 077 | "they would not be objects" → "+ except insofar as they are contained…" (dropped "als ſofern"; control also drops it) | MOD |
| T16 | ✅ | 078 | "put in place certain of the concepts" → "put ourselves in a position…"; + restored "to us", "perhaps", "in itself", "unconditionally necessary"; "had of course to say"; "either…or"; "actual" (wirklich) | control shares several of these errors |
| T17 | ✅ | 079/080/081 | emphasis converted to print: **can/could/a regress to infinity** bold; +bounded absolutely, in itself, to infinity, anticipate, affirmative, the world; italic clause de-italicized ×2 | PNGs; some spans control missed |
| T18 | ✅ | 085 | "not an object of intuition through which" → "not an object of sensible intuition, but through which it can nevertheless be" | dropped words |
| T19 | ✅ | 087/088 | "***regulative principle*** ***of reason***" → emphasis only on "regulative principle"; marker realignments | page-388.png |
| T20 | ✅ | 091 | "through the ***idea***" → "***ideal***" (control inverted); +***given***, ***being of all beings***, ***distributive***/***collective***; fn "cognition" → "information" (Kenntnisse) | PNG; control corrections |
| T21 | ✅ | 092 | "gradually more intelligible" — "more" dropped per print ("verſtändlich"); G-W divergence noted | print is project authority |
| T22 | ✅ | 094–096 | `{{{ 418 }}}` inserted (096) | |
| T23 | ✅ | 097 | "_summa intelligence_" → "_summam intelligentiam_" | half-English garble |
| T24 | ✅ | 099 | "go, the entfernteren parts" → "unite in their course the more distant parts" (untranslated German + missing verb); 2 spans de-emphasized (print roman); +***maxims***, ***law of specification*** (❓ PNG ambiguous) | MOD; page-447/449 |
| T25 | ✅ | 100/101/103 | ~30 fidelity fixes: "ideas of pure reason"; "the series" (not nature); "highest being" (not reason); transcendent (not transcendental); presuppose; cognize/acquaintance glossary; original being; "confirms"; 101 "desist"/"estimate"/"each to build separately"; 103 tense repair (present), "utility", "embellishment" | German + control |
| T26 | ✅ | 104 | "with with" → "with which"; unclosed `_a priori_`; merged span split into four (***exposition, explication, declaration*** + declaration); +***expositions***/***constructions***; `_quanta_`/`_principium_`; "in pure intuition" → "in intuition"; `{{{ 478 }}}` moved | part0056/PNGs |
| T27 | ✅ | 105/106 | `{{ 777 }}`, `{{ 789 }}`, `{{ 796 }}` inserted; ***principles*** de-emphasized; `_custom_` de-italicized; "critique of reason" (not "pure"); marker realignments | print |
| T28 | ✅ | 107 | "attractive force" → "force of extension" (print "Ausdehnungskraft"; control wrong); "for which they are nevertheless supposed to speak"; "stirring scruples … cannot well be"; +"would really", "should", "him who", "often ×2", "pure private" | print/MOD |
| T29 | ✅ | 108 | "of and acquaintance with" (Kenntnis); "no suspicion at all"; "mark" (Kennzeichen); "reciprocability"; "that parliamentary advocate"; italic split `_non entis nulla {{ 821 }} sunt praedicata_` | glossary/MOD |
| T30 | ✅ | 109/110 | "footing somewhere"; "now set aside … of pure reason"; "the investigation of nature"; "exceeds all our acquaintance"; "reason, therefore"; "causes that are represented" | MOD |
| T31 | ✅ | 111 | "transcendental use" → transcendent (control error); "speculative principles of reason" de-emphasis ("der Vernunft" not letterspaced); "realm of grace" span merged | Kant's distinction; PNG |
| T32 | ✅ | 112 | "moral questions" → "moral laws"; "everything should be cognized _a priori_, where everything is necessary"; "knowing***. ***Having an opinion" span split; fn "!" → "."; `[^*]:` spacing | German; control deviations |
| T33 | ✅ | 113 | "very seldom"; "share … in the rambling use" (Antheil mistranslation); restored "indeed even through long periods of time"; doubled "One would ask," removed; "***learned***" de-emphasized (print roman); ontology list span split ×4; "_teleologia_"/"_per intussusceptionem_" repaired | MOD/print |
| T34 | ✅ | 114 | "in this department of inquiry" (invented "natural" dropped); "through common reason" (Vernunft); "***sensible objects***" full span; "from before the rude state"; markers 881/883/884 moved | MOD/print |

TRA documented-only (⬜): Guyer-Wood renderings kept where meaning-preserving and control-verbatim (074 "is demanded" voice, 080 da-clause subordination, 082 decomposition-attachment ambiguity, 092 "absolute necessity", 093 GW interpretive renderings, 105/106 weigern/Maxime/Interesse renderings, 108 "antiquity", 110 "rule of conduct", 111 "that is, in the moral use", 113 span extents wider than print); punctuation-inside-emphasis corpus convention; English word-order span splits (see §5).

---

## 4. Markers — final state

All three layers now share **byte-identical marker sequences** for every file 074–114 (verified mechanically): AA 322–552, B 490–884, no gaps, no duplicates except the by-design file-boundary repeats (092 heading `{{ 611 }}` after 091's in-paragraph `{{ 611 }}`; 104 heading `{{ 740 }}` after 103's `{{ 740 }}`; 107 heading `{{ 797 }}`; AA repeats at section starts). Title-page gaps B 733/734, AA 462–464 (100→101) are by design.

| File | Inserted | Repositioned (to OCR-margin-verified print breaks) |
|---|---|---|
| 075 | — | — (B 507 seam got its dropped clause back) |
| 076–078 | — | 18 (incl. 4 pipeline-inserted spurious commas removed) |
| 086 | B 573, 575, 580, 583, 584 (all layers) | B 576 (was mid-word) |
| 087–091 | B 611 (091, all layers — true break "in ſich enthält, {{ 611 }} welches denn vermittelſt…") | B 588–610: 21 moved (3 were mid-word; `{{ 608 }}` a paragraph off) |
| 092–093 | B 611 heading repeat (092) | 16 |
| 096 | AA 418 (all layers; anchor "zufälligen Einrichtung, ▸auf das Daſein") | — |
| 099 | — | B 680/691/693 brace repairs + duplicate collapse (REV+MOD) |
| 100 | AA 455–460, B 705, 707, 732 (all layers; no text lost at any seam) | ~28 |
| 104 | — | heading `{{ 740 }}` + 21 B markers (one a full page off) + `{{{ 478 }}}` (TRA) |
| 105–106 | B 777 (TRA), B 789, 796 (all layers) | 25 |
| 111–113 | TRA: AA 526–528, 531, 539–546; B 837/838/840/842/844 (111), 857 (112), 861–878 odd set (113) | 113 REV/MOD: 17; 114: 4 |

## 5. Emphasis — final state

The Sperrdruck/Latin restoration step had been skipped on REV/MOD for 083, 086, 087, 088, 090–095, 097, 100, 105, 108–114 (and partially missing everywhere else). Every restored span was letterspace-verified on the 1911 page scan (control `<strong>`/`<em>` as corroboration only — the control epub itself **misses** spans the print has (e.g. 081 "to infinity"/"anticipate", 100 "für uns" B723, 103 "positiven", 108 "erſte/zweite/dritte", 110 "Geſetze der Freiheit") and **adds** spans the print lacks (099 ×2, 094 ×5 — see ❓ P5).

Final `***`-span counts per file (REV/MOD/TRA): 074 35/35/35 · 075 7/7/7 · 076 34/34/35 · 077 9/9/9 · 078 33/33/36 · 079 24/24/25 (+3 `**` each) · 080 6/6/6 · 081 17/17/17 · 082 11/11/11 · 083 23/23/23 · 084 24/24/23 · 085 20/20/20 · 086 37/37/38 · 087 8/8/9 · 088 2/2/2 · 090 11/11/11 · 091 ~38 aligned · 092 11/11/11 · 093 18/18/18 · 094 9/9/14 (❓ P5) · 095 11/11/11 · 096 13/13/14 · 097 38/38/37 · 099 59/59/59 · 100 51/51/52 · 101 6/6/6 · 103 14/14/14 · 104 54/54/54 (+3 `**`) · 105 30/30/30 · 106 23/23/23 · 107 16/16/16 · 108 17/17/17 · 109 5/5/5 · 110 15/15/15 · 111 45/45/45 · 112 43/43/42 · 113 70/70/70 · 114 24/24/24. All numeric deltas are sanctioned English word-order splits/merges, each documented in the per-batch reports (e.g. 078 +3 splits; 084 German pathologiſch/afficirt split; 086 vorhergehen → preceded+precede; 096 span split by `{{{ 414 }}}`; 097 atheiſtiſch/deiſtiſch/anthropomorphiſtiſch 3↔1; 112 ſowohl/als 2↔1) — except 094 (❓ P5). Latin italics aligned per file (e.g. 094 0→18 in REV/MOD; 113 0→46) after PNG Antiqua checks; Fraktur-set terms left plain (085 Noumenon de-italicized, 106 _custom_ removed).

## 6. Footnotes

- 113 (all layers): two defs both `[^*]:` with missing body refs → now `[^*]` (Weltbegriff, anchor B 866–868 "Weltbegriffe (conceptus cosmicus)") and `[^**]` (_physica generalis_ note, anchor "physica rationalis" B 874), refs + defs 1:1 in every layer. German/English footnote totals now 75/75.
- 091 `[^*]`: dropped second print sentence restored in all three layers (R12/M5/T4).
- No editor/apparatus notes leaked anywhere (the OCR `¹)`/`³)` A-variant markers were verified absent; two stray margin digits removed from 113 REV/MOD).

## 7. Sentence-count changes (import planning)

**Exactly one block changed sentence count:** 091, footnote `[^*]` — 1 → 2 sentences in all three layers (restored dropped print sentence). Everything else is intra-sentence. Per the reconcile-aligner rule (`AUDIT_TEXT.md` remediation log), this fix must land as **two import passes** (in-place edit pass + sentence-insert pass) when the next DB import runs. No import was performed in this audit.

## 8. New policy flags (❓ — decide before/with the next pass; P7 + P8 resolved 2026-06-13)

| # | Item | Detail |
|---|---|---|
| P1 | 074 body-text-as-H2 | `## Dies iſt der Gegenſatz des ***Epikureisms*** … ***Platonism***` is ordinary body text in print (paragraph indent, body font, page-336), rendered as a heading in all three layers. Structural change (block type) — needs owner decision + two-layer-consistent edit. |
| P2 | 075/090/093/111/113/114 proper names | print letterspaces names (Lambert, Leibniz, Plato, Wolff, Epikur, Ariſteles, Hume); no layer emphasizes → blocked on #321. |
| P3 | 106 "Richtigkeit" vs Guyer "nullity" | print clearly reads "Richtigkeit"; REV/MOD follow print, TRA follows Guyer ("nullity" = Nichtigkeit emendation). Same class as the vetted 023 "Vorgehens" row — decide follow-print vs emend. |
| P4 | 099 "law of specification" | TRA span kept/added on control evidence; PNG ambiguous. Verify on a better scan if it matters. |
| P5 | 094 "abyss" passage | TRA/Guyer emphasize five words (satisfy, content, measures, sustain, but whence) that the 1911 reprint sets in roman (letter-pitch verified). REV/MOD intentionally NOT given these spans; TRA left as control. Decide: trim TRA to print, or accept asymmetry. |
| P6 | 086 "ſo, daß [nicht] die Bedingungen" | 1911 print lacks "nicht"; received text has it. REV now print-faithful (no "nicht"); MOD/TRA keep the emendation. Confirm or unify. |
| P7 | 106 "als ſelbiger" | ✅ **Resolved 2026-06-13.** Raw OCR (page-504) confirms the 1911 reprint itself reads "als ſelbiger", so REV stays print-faithful while MOD ("aus derselben") and TRA ("from it") keep the emendation — exactly the vetted 003 "erfolgt/verfolgt" precedent (`AUDIT_TEXT.md` §5). No file change: `md_to_struct` aligns REV↔MOD per block, so the divergence is harmless. |
| P8 | "2c." was an OCR error for ꝛc. | ✅ **Resolved 2026-06-13.** "2c." is the OCR misread of the Fraktur "ꝛc." (= etc.; the ꝛ rotunda scans as a "2"). Corrected the three sites to the corpus-standard tokens — REV "ꝛc." (matching 16 existing sites), MOD "etc." (matching ~18) — at 078, 112, 113. **No code change** (an earlier `SINGLE_ABBREVS` edit was reverted): ꝛc./etc. legitimately *ends* sentences in most places (051, 065, 078, 104 all split correctly), so it must not be globally non-splitting. The only residual is 113, where ꝛc. sits mid-sentence before the capitalized noun "Jahrhundert"; the toolchain has no "no-split" sentinel and the abbreviation-list route would require editing out-of-scope files (051/065), so the trailing abbreviation period is dropped at this one site ("zehnte ꝛc Jahrhundert" / "etc Jahrhundert") to avoid a spurious split. Full period-faithfulness there (via the project's usw.-style abbreviation + `\|\|\|` design, applied corpus-wide) folds into the #315 ꝛc. canonicalization sweep, which also covers the other renderings ("sc." in 112, "zc.", "z.", "u. ſ. w."). Both struct gates + `cargo test -p common sentences` (48) pass. The 113 period-omission is logged as compromise **C1** in `assets/kant1/curated/KANT1_TEXTUAL_COMPROMISE.md`. |
| P9 | 114 "ok es gleich" | reprint glyph defective (reads ot/ok for "ob"); REV kept as captured (uncertain), MOD emends to "ob". |

## 9. Verification gates (final run, 2026-06-13)

- Mechanical pass: 110 findings → 0 actionable (remainder = `[^***]`-label false positives, by-design title-page gaps, and the sanctioned emphasis word-order deltas of §5).
- `cargo run -p kant1_md_to_struct`: ✅ — 1267 blocks (1060 ¶ / 189 headings), 4652 sentences, 75 footnotes (198 fn sentences), 1509 page markers (583 AA / 926 B), AA 2–552, B II–884.
- `cargo run -p kant1_md_translation_to_struct`: ✅ — identical counts (1267 / 4652 / 75 / 1509); per-block MOD↔TRA sentence parity holds.
- `cargo test -p common sentences`: ✅ — 48 pass (`sentences.rs` reverted to baseline; this audit ships **no** code change).
- **No code change:** the P8 fix is data-only — "2c." (OCR error) → faithful "ꝛc." (REV) / "etc." (MOD) in three files; the abbreviation period is dropped only at the single mid-sentence site 113 (see P8). `packages/common/src/sentences.rs` is unchanged from baseline. (The pre-existing `M packages/kant1_md_to_struct/src/main.rs` debug-`eprintln` in the working tree is not from this audit.)
- **Not done (next steps):** DB import (two passes required, §7); the remaining §8 policy decisions (P1–P6, P9 — P7 and P8 resolved 2026-06-13); the §4-policy orthography sweeps (round-s, tz-strays, the ꝛc. renderings incl. restoring the 113 period, proper-name Sperrdruck) once `AUDIT_TEXT.md` §4 / #315 is decided.
