# Kant1 Curated Text Audit Report

**Date:** 2026-06-11
**Scope:** all 74 curated files (prefixes 000–073) in `assets/kant1/curated/` — `md_reviewed` (REV), `md_modernized` (MOD), `md_modernized_translated` (TRA); ~110k words per layer.
**Method:** (1) mechanical consistency pass (marker sequences, paragraph/heading/footnote/emphasis parity, 1:1 sentence ratio, Fraktur-misread patterns); (2) 14 parallel content-review agents, each auditing a file batch in all three layers against the rules in the `kant1-review` and `kant1-modernize-translate` skills; suspect readings verified against the raw OCR (`raw/ocr_to_lines/page-{AA+9}.json`), the 1911 reprint page scans (`raw/pages/`), and the Cambridge/Guyer-Wood control (`assets/kant1/control/text/`, covers B169+ only). Critical findings independently re-verified.

**Status legend:** ✅ fixed in the curated files · ⬜ open · ❓ policy decision needed before fixing · ⏭️ skipped — the 000 TOC files are human-reviewer convenience, not part of the import pipeline (it never reads them; TOC structure is hardcoded in `common/src/kant1/toc*.rs`); revisit only if needed.

**Remediation log:**
- 2026-06-11: all §1 criticals fixed and imported. The 008 splice (#3) required two import passes — an in-place edit, then a sentence insert — because the reconcile aligner (`packages/reconcile/src/align.rs`, `SPLIT_MERGE_MIN_SIM = 0.90`) rejects splits whose halves are simultaneously reworded. The same two-pass pattern applies to any future fix that adds/removes a sentence boundary while changing its text.
- 2026-06-11: all §2.1 marker issues fixed (awaiting next import run; marker/spacing-only changes, single normal pass).
- 2026-06-11: §2.3 fixed (#91–173) except #90 (deferred by request) and #100/#139, which turned out to be intentional `\|\|\|` forced-sentence-split markers, not junk (see those rows). No fix in this batch changes a block's sentence count — single normal import pass.

---

## 1. Critical (meaning-changing)

| # | Status | File | Layer | Current | Correct | Note |
|---|---|---|---|---|---|---|
| 1 | ✅ | 003 | tra | "we must at least be able to ***cognize*** these same objects as things in themselves, even if we cannot ***think*** them" | "even if we cannot ***cognize*** …, we must at least be able to ***think*** them" | B XXVI: erkennen/denken inverted — stated the opposite of Kant's central doctrine; MOD was correct |
| 2 | ✅ | 007 | rev+mod+tra | "Dieſer aber kann {{{ 32 }}} ohne daß sie…" | insert "vermieden werden, wenn man ſeine Erdichtungen nur behutſam macht," | clause dropped at AA 32 seam (raw page-041) |
| 3 | ✅ | 008 | rev+mod+tra | "durch keine Zergliedealle {{{ 34 }}} Körper ſind ausgedehnt" | "durch keine Zergliederung {{{ 34 }}} deſſelben hätte können herausgezogen werden. Z. B. wenn ich ſage: alle Körper…" | text dropped at AA 34 seam; restores a sentence boundary — landed as two import passes (see log) |
| 4 | ✅ | 046 | rev+mod+tra | "Folge der Wahrnehmung {{{ 171 }}} nicht objectiv beſtimmt" | insert "nur lediglich in der Apprehension, d. i. bloß ſubjectiv, aber dadurch gar" | clause dropped at AA 171 seam (raw page-180) |
| 5 | ✅ | 052 | rev+mod+tra | REV "(not aus dieſem Begriffe hinauszugehen)" → MOD "(Not …)" → TRA "(need to go beyond this concept)" | "(nicht aus dieſem Begriffe hinauszugehen)" / "(not to go beyond this concept)" | negation inverted down the chain; raw page-235 has "nicht" |
| 6 | ✅ | 059 | rev+mod+tra | "ſeine objective Nichtigkeit habe oder nicht" / "objective nullity" | "Richtigkeit" / "objective correctness" | R→N misread inverted the meaning (page-252 scan; Cambridge "correctness") |
| 7 | ✅ | 034 | tra | "the ***notnecessary unity*** of space" | "the ***necessary unity*** of space" | §26 house example; garbled half-translation of "notwendige" |
| 8 | ✅ | 049 | tra | "false and ***unimpossible***" | "false and ***impossible***" | non-word reads as negation; MOD "unmöglich" |
| 9 | ✅ | 036 | tra | "a lack of the latter (the _secunda Petri_)" | "a lack of the power of judgment (the _secunda Petri_)" | footnote: German "an jener" = Urteilskraft; "the latter" pointed at understanding — referent inverted |
| 10 | ✅ | 003 | rev | "meinem Begriffe feine Anschauung unterlegen" | "keine Anschauung" | f↔k misread, meaning-flip; MOD/TRA correct |
| 11 | ✅ | 051 | rev | "den legteren feinen Gegenstand bestimmen" | "den leßteren keinen Gegenstand" | f↔k flip ("fine" vs "no object"); MOD correct |
| 12 | ✅ | 068 | rev | "aber ***feine Reihe*** ausmacht" | "***keine Reihe***" | f↔k flip ("a fine series" vs "no series"); control "but not a series" |
| 13 | ✅ | 068 | rev | "das Substantiale fein Glied in derselben" | "kein Glied" | f↔k flip; MOD/TRA correct |
| 14 | ✅ | 003 | rev | "Vertrauen in unſere Vernunft zu ſehen" | "zu ſeßen" | tz-ligature→h: place trust, not see; MOD/TRA correct |
| 15 | ✅ | 010 | rev | "beſtimmte und fichere Schranken zu ſehen" | "ſichere Schranken zu ſeßen" | f↔ſ + tz→h: set limits, not see them; MOD/TRA correct |
| 16 | ✅ | 043 | rev+mod | "Stellen aber ſehen/sehen jederzeit jene … voraus" | "ſeßen/setzen … voraus" | presuppose, not foresee (p.160 scan); TRA "presuppose" correct |
| 17 | ✅ | 043 | rev+mod | "Einheit des Syſtems zu verlegen" | "verleßen/verletzen" | violate, not misplace (p.164 scan); TRA "violating" correct |
| 18 | ✅ | 049 | rev+mod+tra | "ſehe/sehe ich freilich mehr…" / "I indeed see more than possibility" | "ſeße/setze ich" / "I indeed posit more" | footnote [^**]; page-207 scan reads "ſetze"; Cambridge "posit" |
| 19 | ✅ | 068 | rev | "ſondern vielmehr vorausſehen" (B437) | "vorausſeßen" | tz→h; control "but rather presuppose them"; MOD/TRA correct |
| 20 | ✅ | 068 | rev | "weiter keine andere vorausſehen" (B444) | "vorausſeßen" | tz→h; control "presuppose no farther premise"; MOD/TRA correct |
| 21 | ✅ | 065 | rev | two `[^****]:` definitions (lines 147+149) | second def → `[^*****]:` | ref `[^*****]` had no def — footnote resolution broke; MOD/TRA correct |
| 22 | ✅ | 071 | mod | "ohne Substanz aufeinander sein" (¶28) | "außer einander sein" | "outside one another" became "upon one another"; TRA correct |
| 23 | ✅ | 071 | mod | "ein Mannigfaltiges aufeinander, mithin reale" (¶46) | "außerhalb einander" | same corruption; TRA correct |
| 24 | ✅ | 036 | rev+mod+tra | "im Gebrauch theils der reinen Verſtandesbegriffe" / "in the use, partly, of the pure concepts" | "der wenigen reinen Verſtandesbegriffe" / "the few pure concepts" | print "der wenigen" (page-142; Cambridge "the few"); "theils" corrupt |

Verified genuine, do **not** "fix": "die fein geſponnenen Argumente" (003), "Mark fein Silber" (043), "Sinnen feiner wären" (049), "Monadiſten fein genug" (071) — real "fein".

---

## 2. Major

### 2.1 Missing or misplaced reference markers

| # | Status | File | Layer | Current | Correct | Note |
|---|---|---|---|---|---|---|
| 25 | ✅ | 046 | rev+mod+tra | `{{ 238 }} … {{ 240 }}` — `{{ 239 }}` missing | "vor einer {{ 239 }} Begebenheit vorhergeht" / "precedes {{ 239 }} an event" | margin verified raw page-179 |
| 26 | ✅ | 052 | rev+mod+tra | `{{ 317 }} … {{ 319 }}` — `{{ 318 }}` missing | "zu Erzeugung {{ 318 }} ***beſonderer***" / "generation of {{ 318 }} ***particular***" | margin page-224 |
| 27 | ✅ | 052 | rev+mod+tra | `{{ 330 }} … {{ 332 }}` — `{{ 331 }}` missing | "***Gemeinſchaft*** {{ 331 }} ***der Subſtanzen***" / "***community*** {{ 331 }} ***of substances***" | margin page-232; emphasis span split around marker |
| 28 | ✅ | 070 | tra | `{{{ 296 }}}` missing | "as a whole, {{{ 296 }}} the successive synthesis" | REV/MOD have it |
| 29 | ✅ | 072 | tra | `{{{ 312 }}}` missing | "a series in time {{{ 312 }}} entirely ***of itself***" | REV/MOD have it |
| 30 | ✅ | 073 | tra | `{{{ 318 }}}` missing | "For {{{ 318 }}} something must be regarded as a condition" | REV/MOD have it |
| 31 | ✅ | 017 | rev+mod+tra | "vorſtellen würde, {{{ 71 }}} ſondern" | "{{{ 71 }}} vorſtellen würde, ſondern" / "would {{{ 71 }}} represent itself" | AA 71 begins two words earlier (page-079/080 seam) |
| 32 | ✅ | 049 | mod+tra | "der also {{ 283 }} mit keinen" / "which therefore {{ 283 }} cannot" | "{{ 283 }} der also" / "{{ 283 }} which therefore" | REV placement was right (p-204 margin) |
| 33 | ✅ | 003 | rev | "Saz dar{{ XXVII }} aus folgen" | "daraus {{ XXVII }} folgen" | marker inside word; matches MOD/TRA placement |
| 34 | ✅ | 003 | rev | "dogmatiſch {{{ 22 }}}und" | "{{{ 22 }}} und" | jam |
| 35 | ✅ | 003 | rev | "Hauptſtücks {{ XXXIX }}der" | "{{ XXXIX }} der" | jam |
| 36 | ✅ | 003 | tra | "to the public{{ XXXV }}." | "to the public {{ XXXV }}." | jam |
| 37 | ✅ | 009 | rev | "und {{ 17 }}nicht als Principien" | "{{ 17 }} nicht" | jam |
| 38 | ✅ | 014 | rev+mod | "{{34}}" | "{{ 34 }}" | missing inner spaces; TRA already correct |
| 39 | ✅ | 016 | rev | "Idealität{{ 55 }} des" | "Idealität {{ 55 }} des" | jam |
| 40 | ✅ | 016 | rev | "{{ 57 }}obzwar" | "{{ 57 }} obzwar" | jam |
| 41 | ✅ | 017 | rev | "{{ 65 }}erlangt" | "{{ 65 }} erlangt" | jam |
| 42 | ✅ | 045 | rev | "{{ 231 }}(die Subſtanz)" | "{{ 231 }} (die" | jam |
| 43 | ✅ | 051 | rev+mod | "Verſtandeswesen,{{ 309 }} auch" | ", {{ 309 }} auch" | jam; TRA had the space |
| 44 | ✅ | 062 | rev | "geleite {{ 386 }} t wird" | "geleitet {{ 386 }} wird" | marker inside word |
| 45 | ✅ | 063 | rev+mod | "transſcendental {{ 392 }} en" / "transzenden {{ 392 }} talen" | "transſcendentalen {{ 392 }}" / "transzendentalen {{ 392 }}" | marker inside word; TRA correct |
| 46 | ✅ | 068 | rev | "unter den Er441 scheinungen" | "Erscheinungen" | margin number jammed into word; the `{{ 441 }}` marker itself is correctly placed |

### 2.2 Wrong, missing, or spurious emphasis (verified against page scans / control)

| # | Status | File | Layer | Current | Correct | Note |
|---|---|---|---|---|---|---|
| 47 | ⬜ | 003 | rev+mod+tra | "ſondern … Verengung" unemphasized | "***Verengung***" / "***narrowing***" | page scan AA 16 (B XXIV–XXV) Sperrdruck; Cambridge ⟪narrowing⟫ |
| 48 | ⬜ | 011 | rev+mod+tra | "nenne alle Erkenntniß transscendental, die ſich…" | "***transscendental***" / "***transzendental***" / "***transcendental***" | definition term letter-spaced in print (page-052) |
| 49 | ⬜ | 014 | rev+mod+tra | §1 ¶1: "Anſchauung", "gegeben", "Anſchauungen", "gedacht", "Begriffe" unemphasized | wrap each in `***…***` (EN: intuition, given, intuitions, thought, concepts) | page-058 letter-spacing; only bold **Sinnlichkeit** is marked |
| 50 | ⬜ | 014 | rev+mod+tra | "alle Vorſtellungen rein (im transſcendentalen…" | "***rein***" / "***pure***" | page-059 |
| 51 | ⬜ | 014 | rev+mod+tra | "ſelber reine Anschauung heißen" | "***reine Anschauung***" / "***pure intuition***" | page-059 |
| 52 | ⬜ | 014 | rev+mod+tra | "trausscendentale Logik genannt wird" | "***transscendentale Logik***" / "***transcendental logic***" | page-060 |
| 53 | ⬜ | 014 | rev+mod+tra | "die Sinnlichkeit iſoliren" | "***isoliren***" / "***isolate***" | page-060 |
| 54 | ⬜ | 014 | rev+mod+tra | "nämlich Raum und Zeit" | "***Raum*** und ***Zeit***" / "***space*** and ***time***" | page-060 |
| 55 | ⬜ | 015 | rev+mod+tra | "als außer uns und dieſe" | "***außer uns***" / "***outside us***" | §2 opening, page-060 |
| 56 | ⬜ | 015 | rev+mod+tra | "die ***Möglichkeit*** der Geometrie" | "***Möglichkeit der Geometrie***" / "***possibility of geometry***" | span too short (page-063) |
| 57 | ⬜ | 052 | rev+mod+tra | "daraus verneinende Urtheile werden können" | "***verneinende***" / "***negative***" | page-225; parallel to ***bejahende*** |
| 58 | ⬜ | 052 | rev+mod+tra | "in ihre Leibniz-Wolffianische Lehrgebäude" | "***Leibniz-Wolffianiſche***" | page-231 Sperrdruck |
| 59 | ⬜ | 068 | rev+mod+tra | "nur den Verstandesbegriff von den unvermeidlichen … frei mache" | "***Verſtandesbegriff*** … ***frei mache***" / "***frees*** the ***concept of the understanding***" | page-292 letter-spacing; control bolds |
| 60 | ⬜ | 070 | rev+mod+tra | "nicht vorgeſtellt, wie groß es ſei" (¶38) | "***wie groß***" / "***how large***" | page scan; Cambridge **how great** |
| 61 | ⬜ | 070 | rev+mod+tra | "der Begriff eines Maximum" (¶38) | "***Maximum***" / "***Maximums***" / "***maximum***" | letterspaced Fraktur (Sperrdruck), not Antiqua |
| 62 | ⬜ | 071 | rev+mod+tra | "die transſcendentale Atomiſtik nennen" (¶40) | "***Atomiſtik***" / "***atomistics***" | page scan: only "Atomiſtik" spaced |
| 63 | ⬜ | 003 | rev | "**Thales**" (bold) | "***Thales***" | print letterspaces; MOD/TRA already *** |
| 64 | ⬜ | 003 | rev | "**Reduction**" (bold) | "***Reduction***" | AA 14 footnote; MOD/TRA correct |
| 65 | ⬜ | 003 | rev | "**Seele**" (bold) | "***Seele***" | AA 18, like Gott/einfache Natur; MOD/TRA correct |
| 66 | ⬜ | 003 | mod | "zum äußeren **Sinne** gehören" | "***Sinne***" | REV ***Sinne***, TRA ***sense*** — degraded to bold |
| 67 | ⬜ | 003 | mod | "das ***synthetische Verfahren nennen***" | "das ***synthetische Verfahren*** nennen" | span wrongly extended over the verb; Cambridge excludes "call" |
| 68 | ⬜ | 016 | rev+mod | "nämlich nach ***einander***" | "***nach einander***" (rev) / "***nacheinander***" (mod) | print letterspaces both words (page-068); TRA "***successively***" fine |
| 69 | ⬜ | 030 | tra | "***community*** is the ***causality***" | de-emphasize "community" | REV/MOD + control emphasize only "causality" |
| 70 | ⬜ | 033 | tra | "to cognize something ***as an object***" | "to ***cognize*** something ***as an object***" | print Sperrdruck includes the verb |
| 71 | ⬜ | 034 | tra | "***must*** be able to accompany" (§16) | "***must*** ***be able*** to accompany" | third span lost; control bolds "be able" |
| 72 | ⬜ | 034 | tra | "does ***not*** find some such combination" (§24) | "does not ***find***" | emphasis on wrong word; control "not **find** … **produces**" |
| 73 | ⬜ | 042 | rev+mod+tra | "***Das Principium derſelben iſt: Alle…" | span starts at "Alle Anſchauungen…" / "All intuitions…"; lead-in roman | print letterspaces only the principle (p.157) |
| 74 | ⬜ | 042 | rev+mod | "Das Principium derſelben iſt:" | "Das Princip derſelben iſt:" / "Das Prinzip…" | print reads "Princip" (p.157) |
| 75 | ⬜ | 043 | rev+mod+tra | "**Ihr Principium derſelben iſt:**" and "**d. i. einen Grad.**" bold | remove both bold spans | print roman; only "In allen Erſcheinungen … intenſive Größe" gesperrt |
| 76 | ⬜ | 043 | rev+mod | "Ihr Principium derſelben iſt:" | "Das Princip derſelben iſt:" / "Das Prinzip…" | doubled possessive not in print (p.160); TRA "Its principle is" fine |
| 77 | ⬜ | 043 | rev+mod+tra | "in welcher Epikur ſeinen Ausdruck" | "***Epikur***" / "***Epicurus***" | print letterspaces the name — apply proper-name policy (§4) |
| 78 | ⬜ | 044 | rev+mod+tra | "**Das Princip derſelben iſt:**" bold | remove bold (wording itself correct here) | print roman (p.167) |
| 79 | ⬜ | 049 | tra | "our ***inner*** experience, which is indubitable" | "our ***inner*** ***experience***" | second span dropped; REV/MOD + Cambridge bold both |
| 80 | ⬜ | 052 | rev+mod+tra | "die ***Einerleiheit*** (vieler Vorstellungen" | "**Einerleiheit**" / "**identity**" | print BOLD here, like **Verſchiedenheit**/**Einſtimmung** (p.224) |
| 81 | ⬜ | 052 | rev+mod+tra | "den ***Widerſtreit***, daraus verneinende" | "**Widerſtreit**" / "**opposition**" | print bold in this sentence (p.225) |
| 82 | ⬜ | 052 | rev+mod+tra | "Mit einem Worte: **Leibniz** ***intellectuirte***" | plain "Leibniz" | print plain here (p.230); gesperrt ***Leibniz*** opening the next paragraph is correct |
| 83 | ⬜ | 057 | tra | "it is no _principium_" ×2 | drop italics | print sets it in Fraktur, not Antiqua (p.247); REV/MOD plain are right |
| 84 | ⬜ | 065 | tra | "In his _Phaedo_" | drop italics (or sanction book-title italics globally) | REV/MOD "Phädon" unemphasized |
| 85 | ⬜ | 070 | rev+mod+tra | "nicht als ***zugleich gegeben*** angeſehen" (¶20) | "***zugleich*** gegeben" / "***simultaneously*** given" | print letterspaces only "zugleich" |
| 86 | ⬜ | 070 | rev+mod+tra | 2nd + 3rd "***Sinnenwelt***" in ¶48 | de-emphasize (keep only 1st: "ſtatt einer ***Sinnenwelt***") | print emphasizes only the first instance |
| 87 | ⬜ | 071 | tra | "which, ***given separated*** (at least in thought)" | "***separated*** (at least in thought) ***given***" | two separate spans merged, paren displaced (¶38) |

### 2.3 Wrong words, dropped words, corruptions

| # | Status | File | Layer | Current | Correct | Note |
|---|---|---|---|---|---|---|
| 88 | ⏭️ | 000 | mod | 28 TOC link targets use modernized slugs (…urteile, …transzendentale…) while MOD files on disk keep REV-style names | repoint the link hrefs to the on-disk filenames; do **not** rename the files (md_to_struct matches both German dirs against REV-style names from `filenames.rs`) | repo-navigation only — the importer never reads 000_toc.md (excluded in `kant1_md_to_struct/src/main.rs:42`; TOC structure is hardcoded in `common/src/kant1/toc*.rs`), so all files **are** imported. Affected: 008, 009, 011–013, 017–020, 022–025, 029, 032–034, 036, 039, 040, 053, 055–059, 062, 063 (057–059 overlap #89) |
| 89 | ⏭️ | 000 | rev+mod+tra | links to 057/058/059 omit the `a_`/`b_`/`c_` filename segment | add the segment to the link hrefs (3 per dir) | confirmed in all three dirs; on-disk names carry `a_`/`b_`/`c_` because the hardcoded TOC labels read "A./B./C. …" and filenames are slugified from them — same impact class as #88: repo navigation only, importer unaffected |
| 90 | ⬜ | 002 | rev+mod | "Excellenz, dem Königl." | "Sr. Excellenz, …" | raw page-012 reads "Sr. Excellenz"; TRA "His Excellency" correct |
| 91 | ✅ | 003 | rev | "fünftige Zeit", "fünftigen System" | "künftige(n)" | f↔k (raw OCR confirms); MOD/TRA correct |
| 92 | ✅ | 003 | rev | "Philodorie" | "Philodoxie" | x→r in Kant's coinage; Cambridge "philodoxy"; MOD/TRA correct |
| 93 | ✅ | 003 | mod | "Denfart" | "Denkart" | OCR non-word (k→f) left standing in MOD |
| 94 | ✅ | 003 | mod | "um eines willen dasein" | "da sind" | REV "dasind" = finite verb, turned into infinitive |
| 95 | ✅ | 003 | mod | "diesen Periodo" | "Period" (or "diese Periode") | invented form |
| 96 | ✅ | 003 | mod | "wenn man annahm" ×2 | "annimmt" | tense change; REV/original present; TRA "assumes" |
| 97 | ✅ | 003 | mod | Latin italics dropped ×2 ("a priori") | "_a priori_" | blocks 12 & 16; REV/TRA have them |
| 98 | ✅ | 003 | tra | "that royal road to {{ XI }}; rather I believe" | "…to hit upon, {{ XI }} or rather to pave for itself, that royal road; rather…" | clause truncated at marker |
| 99 | ✅ | 003 | tra | footnote **: "no experiment can be made with the ***objects*** of the propositions" | "for testing the propositions of pure reason, no experiment can be made with their ***objects***" | "zur Prüfung der Sätze" dropped |
| 100 | ⏭️ | 003 | rev+mod+tra | "umzuändern:\|\|\| ***„Dieses Beharrliche" | leave as is | reclassified: \|\|\| is the pipeline's forced sentence-split marker (`sentences.rs strip_forced_splits`) — the splitter never splits on ":", so the marker deliberately makes the quoted passage its own sentence |
| 101 | ✅ | 006 | rev | "iſt er überdiger {{{ 29 }}} dem auch" | "über {{{ 29 }}}dem" (= überdem) | hyphen-join grabbed "diger" from "nothwen=diger" two lines below (raw p-037/038) |
| 102 | ✅ | 006 | mod | "ist er überdies {{{ 29 }}} dem auch" | "überdem auch" (or "überdies", dropping stray "dem") | ungrammatical as is |
| 103 | ✅ | 006 | tra | "the unconditioned universality" | "the unrestricted universality" | unbeschränkt ≠ unbedingt (distinct Kantian term) |
| 104 | ✅ | 006 | tra | "everything that is empirical in it, the color" | "…in it one by one: the color" | "nach und nach" omitted |
| 105 | ✅ | 009 | rev | "Wiſſenschaft anſteht, sollen" | "anſieht" | raw page-048; "ansehen für" required; MOD corrects |
| 106 | ✅ | 014 | rev | "alodŋta xai vonta" | "αἰσθητὰ καὶ νοητά" | Greek garbled by OCR |
| 107 | ✅ | 015 | rev | "ausgedehnten Wesen zu reden" / "Geſchmack zu mit Recht" | "Weſen ꝛc. reden" / "Geſchmack ꝛc." | ꝛc. (etc.) garbled to "zu" (raw p-064/065) |
| 108 | ✅ | 015 | mod+tra | "von ausgedehnten Wesen reden" / equivalent | "… Wesen etc. reden" / "of extended beings, etc." | the dropped ꝛc. propagated |
| 109 | ✅ | 015 | rev | "dergleichen Säge aber" | "Säße" | ß→g produced real word "saws"; MOD/TRA correct |
| 110 | ✅ | 016 | rev+mod+tra | "Die Zeit iſt  kein empiriſcher Begriff" | restore list numeral "1)" | arguments 2)–5) are numbered; REV double space marks the gap |
| 111 | ✅ | 016 | rev+mod+tra | heading "4. Metaphysische Erörterung des Begriffs" | "§ 4. …" | print has "§ 4."; §5–§7 in same file carry § |
| 112 | ✅ | 017 | rev+mod+tra | heading "Allgemeine Anmerkungen zur transscendentalen Ästhetik" | prepend "§ 8." | print page-074; breaks the §1–§8 sequence |
| 113 | ✅ | 017 | mod | "(not die Regentropfen" | "nicht" | English word in German text |
| 114 | ✅ | 017 | tra | "(_intuitus originarius_), mithin not intellectual" | "consequently not intellectual intuition" | untranslated "mithin" |
| 115 | ✅ | 023 | tra | "embellishment of every empty assertion" | "pretension" (if REV emended per §4) else "procedure" | matches neither MOD "Vorgehens" nor canonical "Vorgebens"; see §5 print-faithful list |
| 116 | ✅ | 030 | mod | "dass man auf die letztere Art" | "dass man noch auf die letztere Art" | "noch" dropped (raw p-102); TRA inherits ("that even in the latter way") |
| 117 | ✅ | 030 | tra | "lay at their foundation the categories" | "lay at its foundation" | referent = der Erkenntnis (sg.); control "ground it in the categories" |
| 118 | ✅ | 032 | tra | "We were able above, with little trouble" | "…above, in the case of the concepts of space and time, …" | omission leaves "these" without antecedent; Cambridge keeps it |
| 119 | ✅ | 033 | rev | "Wir ſind ſeßt im Begriffe" | "jeßt" | misread; "ſeßt" renders as wrong word (setzt) |
| 120 | ✅ | 036 | rev | "am natürlichen Talent deſſelben mangelt" | "derſelben" | = der Urtheilskraft; print/OCR; MOD silently corrected |
| 121 | ✅ | 037 | rev+mod | "erfordert wird, aufhalten, wollen wir" | "aufzuhalten" | print (p-146); missing "zu" is ungrammatical; TRA unaffected |
| 122 | ✅ | 037 | rev | "mit dem letzteren ***gleichartig*** ſein" | "mit der letztern" (restore) or document emendation | print reads "der letztern"; MOD follows REV |
| 123 | ✅ | 041 | rev | doubled "des" in Tafel figure | delete duplicate `<span>des</span>` | print p-156 single "des"; MOD/TRA figures correct |
| 124 | ✅ | 042 | rev | "eine Anſchauung im Raume und der Zeit" | "im Raum und Zeit" | print p-157; REV added "-e" and "der" |
| 125 | ✅ | 042 | rev | "beſtimmnt werden" | "beſtimmt" | typo introduced in curation (not in OCR) |
| 126 | ✅ | 043 | rev+mod+tra | "Wenn ich Thaler ein Geldquantum" | "Wenn ich 13 Thaler…" / "If I call 13 talers" | "13" dropped in all three (OCR p-163; Cambridge "thirteen talers") |
| 127 | ✅ | 044 | tra | "set the category alongside it as its restricting condition" | "set it alongside the category as its restricting condition, under the name of a formula of the former" | schema/category role reversal |
| 128 | ✅ | 046 | rev+mod+tra | "Gegenſtand zwar unbekannt iſt: was verſtehe ich denn" | "Gegenſtand unbekannt iſt; was verſtehe ich alſo" / "is unknown; what, therefore" | "zwar"/"denn" interpolated, colon for semicolon (raw p-178) |
| 129 | ✅ | 046 | rev+mod | "vorigen {{{ 170 }}} Beiſpiele eines Hauſes" | "von einem Hauſe" | print p-179; TRA "example of a house" unaffected |
| 130 | ✅ | 046 | rev+mod+tra | "eigenthümliches {{{ 177 }}} Kriterium der Subſtanz" | "Kennzeichen" / "characteristic" | print p-185/186; avoids collision with the real "Kriterium" two sentences later |
| 131 | ✅ | 046 | rev+mod+tra | "Und doch hat die Auflöſung der Frage doch" | "Allein nach unſerm Vorigen hat …" / "But according to what we said above, …" | back-reference dropped; causes doubled "doch" |
| 132 | ✅ | 046 | rev+mod+tra | "Zuſtänden und {{{ 179 }}} gehören alſo mit" | "als solche" / "as such" | print p-187: "gehören als solche mit zu der ganzen Veränderung" |
| 133 | ✅ | 046 | rev | "nicht {{ 254 }} plöglich ich (auf einmal" | delete stray "ich"; "plötzlich" | not in print (p-188); tz misread; MOD/TRA correct |
| 134 | ✅ | 046 | rev | "zu erkennen.  We anticipiren nur" | "Wir anticipiren" | English "We" + double space (p-189); MOD/TRA correct |
| 135 | ✅ | 046 | tra | "precedes the other before {{ 234 }} the other" | "the one state precedes {{ 234 }} the other" | garbled duplication of MOD wording |
| 136 | ✅ | 046 | mod | "in diesem oder jemem Zeitverhältnisse" | "jenem" | typo introduced in MOD |
| 137 | ✅ | 046 | mod | "der Ursache and deren unmittelbaren Wirkung" | "und" | English "and" introduced in MOD |
| 138 | ✅ | 047 | mod | "ist jeder Wahrnehmung … abgebrochen" | "jede Wahrnehmung" | case corruption; REV/TRA correct |
| 139 | ⏭️ | 050 | mod+tra | "sein könne usw.\|\|\| So" / "a magnitude, etc.\|\|\| As" | leave as is | reclassified: forced-split marker — the splitter treats "usw." as an abbreviation and would not split, so the marker forces the boundary (REV "u. f. w. So" splits naturally, hence no marker there); deleting it would merge sentences |
| 140 | ✅ | 049 | rev | "Zuſammenhaltung with den Kriterien" + "Zusammenhange with dem" | "mit" ×2 | English "with" jammed into German (raw has "mit") |
| 141 | ✅ | 049 | rev | "nach empirischen Gesehen hinreicht" | "Geſeßen" | tz→h (p-199); "Gesehen" = seen |
| 142 | ✅ | 049 | rev | "einer bloß fritischen Anmerkung" | "kritiſchen" | f↔k (p-206) |
| 143 | ✅ | 049 | rev | "das Anschauungsver. mögen" | "Anſchauungsvermögen" | line-break hyphen rendered ". " — splits word and fakes a sentence break |
| 144 | ✅ | 051 | rev | "Noumena neunt. Aber" | "nennt" | u↔n ("ninth"); MOD correct |
| 145 | ✅ | 051 | mod | "die Möglichkeit des Dinge" | "der Dinge" | MOD-introduced corruption |
| 146 | ✅ | 051 | tra | "***object in itself*** {{ 307 }} makes and therefore represents" | delete second "makes" | duplicated verb |
| 147 | ✅ | 051 | tra | "brought _a priori_ zustande / into being" | "(although brought about _a priori_)" | untranslated German + draft-alternative slash left in text |
| 148 | ✅ | 052 | rev+mod+tra | heading "Reflexionsbegriffe Von der Amphibolie der…" | single "Von der Amphibolie der Reflexionsbegriffe" | heading phrase duplicated (print p-223 has it once) |
| 149 | ✅ | 052 | rev | "denn die Sinnlichkeit was ihm nur" | "war ihm" | raw p-229 reads "war"; MOD/TRA correct |
| 150 | ✅ | 052 | rev | "aufheben, und 3-30=0 sei" | "3-3=0" | spurious 0 kept when "=0" was restored; MOD/TRA correct |
| 151 | ✅ | 052 | mod | "ohne auf die Anschauung, welche" | "ohne auf die Anschauung zu sehen, welche" | verb dropped; REV has "zu ſehen"; TRA conveys it |
| 152 | ✅ | 055 | rev+mod+tra | "diese von Beſtimmung abweichend machen" | "von ihrer Beſtimmung" / "from their determination" | "ihrer" missing (raw p-244) |
| 153 | ✅ | 055 | rev+mod | "deren application/Applikation ſich ganz" | "Anwendung" | print reads "Anwendung" (raw p-244); TRA "application" is correct English |
| 154 | ✅ | 055 | rev | "daß so uns das Meer in der Mitte" | delete "so" | AA margin line-number 30 jammed into text (page-245); MOD/TRA omit it |
| 155 | ✅ | 058 | tra | "the consequence (consequence)" | "the inference (consequence)" | "Schlußfolge (Conſequenz)" — gloss voided; Cambridge "the inference (consequence)" |
| 156 | ✅ | 060 | rev+mod | "Da dieſe aber allererſt" | "dieſes" | print (raw p-254); "dieſe…kann" ungrammatical, shifts referent; TRA correct |
| 157 | ✅ | 064 | mod+tra | "keine Erkenntnis, obzwar einen problematischen Begriff" / "no cognition" | "keine Kenntnis" / "no acquaintance" | print "Kenntniß" (raw p-270); Kenntnis ≠ Erkenntnis in Kant's vocabulary |
| 158 | ✅ | 065 | rev | "betrachtet werden kann, gelten müſſen" | "gelten müſſe" | OCR page-276 reads "müſſe" (wrong number/mood in REV) |
| 159 | ✅ | 065 | rev | "Im Unterſage aber iſt nur" | "Im Unterſaße" | tz misread, non-word; MOD/TRA correct |
| 160 | ✅ | 065 | mod | "in Nichts verwandelte werden könne" | "verwandelt werden könne" | corrupted participle |
| 161 | ✅ | 065 | mod | "weil sie gehört zum Denken überhaupt" | "weil sie zum Denken überhaupt gehört" | broken subordinate word order |
| 162 | ✅ | 065 | mod | "anhängend betrachtet werden kann, gelten muss" | "gelten müsse" | indicative loses Kant's subjunctive |
| 163 | ✅ | 065 | tra | "and mithin do not give myself" (×11) | "consequently"/"hence" | untranslated German "mithin", lines 64–147 |
| 164 | ✅ | 065 | tra | "which can be called rational psychology" | "which can be called the rational doctrine of the soul" | Psychologie/Seelenlehre distinction erased (circular); Cambridge keeps both |
| 165 | ✅ | 065 | tra | "contrary to the style of good style" | "contrary to the taste of good writing" | "Geschmack der guten Schreibart"; "style" duplicated |
| 166 | ✅ | 065 | tra | "possibility of thought after its cessation" | "…thought even after the cessation of life, of which they have an example only in…" | "nach dessen Aufhörung" = of LIFE; "its" inverts the example |
| 167 | ✅ | 068 | rev | "trifft auch den Kaum" | "Raum" | K↔R Fraktur misread; MOD/TRA correct |
| 168 | ✅ | 068 | tra | "condition under which it is necessary, and which" | "a condition under which it is necessary to refer this to a higher condition" | impersonal es-ist-notwendig-zu construction misparsed; control agrees |
| 169 | ✅ | 071 | tra | "also no simple part would remain, and consequently" (¶18) | "…no simple part, and thus nothing at all would remain; consequently no substance…" | middle step "mithin gar nichts übrig bleiben" omitted; Cambridge has it |
| 170 | ✅ | 072 | rev | "Sehet: es gebe eine Freiheit" | "Seßet:" | page PNG shows "Setzet" (Posit, not Behold); MOD/TRA correct |
| 171 | ✅ | 072 | rev | "entgegen und ſolche Verbindung" | "und eine ſolche Verbindung" | "eine" dropped (raw OCR has it); MOD/TRA have it |
| 172 | ✅ | 072 | mod | "ist eigentlich nur ***transzental***" | "***transzendental***" | garbled key term inside Sperrdruck (REV "transſcendental") |
| 173 | ✅ | 072 | tra | "allow different series of causality to begin" | "allow different series to begin of themselves, as regards causality, in the midst of the course of the world" | "der Cauſalität nach" is adverbial, not genitive of "Reihen"; Cambridge agrees |

---

## 3. Minor (orthographic OCR residue, modernization leftovers, italics, nuances)

| # | Status | File | Layer | Current | Correct | Note |
|---|---|---|---|---|---|---|
| 174 | ⬜ | 001 | mod | "silemus; de re autem" | "silemus: De re autem" | Latin quotation altered; raw OCR matches REV — don't modernize quotations |
| 175 | ⬜ | 002 | rev+mod | "unterthänig gehorsamster Diener" | "unterthänig-gehorsamſter" / "untertänig-gehorsamster" | raw OCR hyphenates |
| 176 | ⬜ | 003 | rev | f/ſ misreads: finnreichen; fie ×4; alfo; finnlichen; Überfinnlichen; Grundfäße; widerfinniſche; find; Metaphyfik ×3 | ſinnreichen; ſie; alſo; ſinnlichen; Überſinnlichen; Grundſäße; widerſinniſche; ſind; Metaphyſik | raw OCR confirms all |
| 177 | ⬜ | 003 | rev | n/u + letter misreads: Gaug; au die Bahn; denu; vou; Auschauungsvermögens; unvenommen; Mernunft; Vernunstwissenschaft; derselbent; Idealisut; ius Publicum; fenes; Sak; Möglichfeit; dieſelbe (sg. for pl.) | Gang; an; denn; von; Anschauungsvermögens; unbenommen; Vernunft; Vernunftwiſſenſchaft; derselben; Idealism; ins; jenes; Saß; Möglichkeit; dieſelben | raw OCR confirms; MOD "ebendieselbe" needs same plural fix |
| 178 | ⬜ | 003 | rev | å/ā junk ×9: Umånderung; ſpåterer; gångeln; geråth; Gegenſtånde; Moralitåt; gemåßer; Vollſtåndigkeit; Popularitāt | ä in all | systematic accent junk |
| 179 | ⬜ | 003 | rev | ſs-mix ×9: Wiſsenschaft ×5; beſsere; Wiſsen; Wiſsenſchaft; Wasſersäule | ſſ forms | house style |
| 180 | ⬜ | 003 | rev | line-join scars: derVerluſttrifft; jed es; Gesichts. punkte; Experimental methode; der. ſelben; be wußt; u. f. w.; jest; erfeßt; legteren; letteren; Erperiment; Vorausſetung | der Verluſt trifft; jedes; Gesichtspunkte; Experimentalmethode; derſelben; bewußt; u. ſ. w.; jeßt; erſeßt; letzteren; letzteren; Experiment; Vorausſetzung | raw OCR confirms |
| 181 | ⬜ | 003 | mod | irgende ein; Neueung; die inner Anschauung; Beweisgründen; Diogenes Laertius; übergehe; Missverständnisse; hiebei/hievon | irgendein; Neuerung; innere; Beweisen; Diogenes der Laertier; vorbeigehe; Missverstande; hierbei/hiervon | typos + lexical drift + internal inconsistency |
| 182 | ⬜ | 005 | rev | Zuſammengefeßtes; Sazz; Saţ | Zuſammengeſeßtes; Saß; Saß | OCR junk |
| 183 | ⬜ | 006 | rev | Sak; ab geleitet; manniginal; felbſt; körperlichenObjects/könntihr/odereiner/aufdringt,geſtehen (lost spaces); Siz | Saß; abgeleitet; mannigmal; ſelbſt; insert spaces; Sitz/Siß | raw OCR confirms |
| 184 | ⬜ | 006 | mod | darthun; weglasset | dartun; weglasst | th leftover; archaizing |
| 185 | ⬜ | 006 | tra | Merkmal/Kennzeichen/Kriterien all rendered "criterion/criteria" | Merkmal → "mark"; Kennzeichen → "marks/sure signs"; reserve "criteria" for Kriterien | three terms collapsed in one passage |
| 186 | ⬜ | 007 | rev | Metaphyfik; find; fommen; stray "s" token; laſsen; besſer; thr | Metaphyſik; ſind; kommen; delete; laſſen; beſſer; ihr | raw p-040/041 |
| 187 | ⬜ | 007 | tra | "judgments beyond all its boundaries" | "beyond all boundaries of experience" | "derselben" = der Erfahrung |
| 188 | ⬜ | 008 | rev | Prádicats/Prādicat ×2; lettere; Algemeinheit; find | Prädicat(s); leßtere; Allgemeinheit; ſind | raw confirms |
| 189 | ⬜ | 009 | rev | Schlüffe; Säze; Saz; Sah; fich; Såßen | Schlüſſe; Säße; Saß; Saß; ſich; Säßen | raw p-045–048 |
| 190 | ⬜ | 009 | rev+mod+tra | "(Physica)" / "(Physik)" / "(physics)" | "(_Physica_)" / keep Latin italic in all three | Cambridge keeps "physica" |
| 191 | ⬜ | 009 | tra | "***actually think***" merged span; "go so far beyond it"; "added entirely from outside" | "***actually*** … ***think***" (two spans per print); "indeed even go so far"; "is entirely an addition" | print has two Sperrdruck spans; "wohl gar" dropped; spatial metaphor added |
| 192 | ⬜ | 010 | rev | genan; wissſen; Sat; Schlüffen; find; Metaphyfik ×2; Nicht-Wiſsen; Zuverläſsigkeit; fann; Wisſenſchaft; hervorgeschosſenen; footnote: be. zweifeln/lettere/eigent- lichen/Phyfit | genau; wiſſen; Saß; Schlüſſen; ſind; Metaphyſik; -Wiſſen; Zuverläſſigkeit; kann; Wiſſenſchaft; -geſchoſſenen; bezweifeln/leßtere/eigentlichen/Phyſik | raw p-048–051 |
| 193 | ⬜ | 010 | mod | darnach | danach | 008 MOD modernizes the same word |
| 194 | ⬜ | 011 | rev | Propādeutik; transfcendentale; derfelben; Syftem; ſsie; find; daſind; Erkenntnisſe/Wisſenschaft ×3/müsſen | Propädeutik; transſcendentale; derſelben; Syſtem; ſie; ſind; da ſind; ſſ forms | raw p-052–054; MOD "dasind" → "da sind" |
| 195 | ⬜ | 011 | tra | "hesitation"; "purposeful"; "motives"/"determining ground" | "difficulty/precariousness" (Bedenklichkeit); "expedient" (zweckmäßig); Triebfedern → "incentives", Bewegungsgrunde → "motive" | terminology |
| 196 | ⬜ | 012 | rev+mod | "Der Transscendentale Elementarlehre" | "Der Transſcendentalen / Transzendentalen Elementarlehre" | genitive -n lost (leads into 013's "Erster Theil") |
| 197 | ⬜ | 014 | rev | transfcendentalen ×2; trausscendentale; nåämlich; gewisſen Verhältniſsen; Vernunſtprincipien; Baum- garten; Geſezen; lektere | transſcendental-; nämlich; gewiſſen Verhältniſſen; Vernunftprincipien; Baumgarten; Geſeßen; leßtere | raw confirms |
| 198 | ⬜ | 014 | rev+mod | "a posteriori" plain; mod: "aisthēta kai noēta" plain; mod "wornach" | "_a posteriori_"; "_aisthēta kai noēta_"; "wonach" | TRA already italicizes |
| 199 | ⬜ | 014 | tra | "a disappointed hope" | "a failed hope" | verfehlte Hoffnung = inherently misguided |
| 200 | ⬜ | 015 | rev | find; Erflärung; fich; Sig; Neceptivität; be kommen; Saz; Gefichts; heading "transcendentalen" | ſind; Erklärung; ſich; Sitz/Siß; Receptivität; bekommen; Saß; Geſichts; transſcendentalen | raw confirms |
| 201 | ⬜ | 016 | rev | missing ")" after "Theilvorſtellungen"; Ariomen; metaphyfiſchen; Verånderung; wåre; entgegengefetter; lettere/lettern; "sc."; dieſser; Anschauun; stray "s"; finnlichen ×2; unfere; Da gegen; vorausſekt; Nealität; abfolute; de das; Erkenntnisſe/Verhältniſsen; fie; dafind; fagen; threr; Zeitfølge; Din- gen | ")"; Axiomen; metaphyſiſchen; Veränderung; wäre; entgegengeſetzter; leßtere(n); ꝛc.; dieſer; Anſchauung; delete; ſinnlichen; unſere; Dagegen; vorausſeßt; Realität; abſolute; da das; ſſ forms; ſie; daſind; ſagen; ihrer; Zeitfolge; Dingen | raw + print confirm; "ſebſt" is the reprint's own slip (→ ſelbſt or annotate) |
| 202 | ⬜ | 016 | rev | "a posteriori" plain (block 18) | "_a posteriori_" | MOD/TRA have it |
| 203 | ⬜ | 016 | mod | darthun; "widersprechend-entgegengesetzter" ×2 | dartun; "kontradiktorisch entgegengesetzter" | technical term replaced beyond orthography; TRA "contradictorily opposed" right |
| 204 | ⬜ | 016 | tra | "even"; "only add"; "without their reality … being disputed"; "constitution of which" | "themselves" (selbst); "further add" (noch hinzu); "without it being permissible to dispute their actuality" (darf + Wirklichkeit); "of which object" | nuance drift |
| 205 | ⬜ | 017 | rev | Jch; lettern/lettere; Saz; Geseze | Ich; leßtern/leßtere; Saß; Geſeße | Fraktur J; tz strays |
| 206 | ⬜ | 017 | mod | frontmatter label "transscendentalen Ästhetik" | "transzendentalen" | label unmodernized though heading is |
| 207 | ⬜ | 017 | tra | "does the representation of ***outer senses***"; "***actively (by self-activity)***"; "precede any action of thinking something as representation"; "We cognize nothing but"; "therefore" | plural "representations"; "***self-actively***" (no gloss in span); "that which, as representation, can precede any act of thinking"; "We know nothing but" (kennen); "also" (auch) | nuance/terminology |
| 208 | ⬜ | 018 | mod | frontmatter label "Beschluß der transscendentalen" | "Beschluss der transzendentalen Ästhetik" | label unmodernized |
| 209 | ⬜ | 021 | rev | lettere; round-s cluster ¶1–2 (Gegenstand, Anschauung ×6, Empirisch, Vorstellung, beigemischt, unterscheiden, Eigenschaften, "so daß", "eben so") | leßtere; long-ſ forms | see §4 round-s policy |
| 210 | ⬜ | 021 | rev+mod+tra | "in concreto" plain | "_in concreto_" | Latin italics rule |
| 211 | ⬜ | 021 | mod | "keine empirische Prinzipien" | "empirischen" | archaic plural inflection |
| 212 | ⬜ | 022 | rev | "ausschließen"; H2 "transſcendentalen" (long-s) | "ausſchließen"; round-s in heading | only heading in batch using ſ |
| 213 | ⬜ | 022 | mod | darthut | dartut | th leftover |
| 214 | ⬜ | 023 | tra | "or rather, solely to test them"; "the demand to use it" | "or, better, solely to test" (noch besser); "the presumption of using it" (Zumutung) | control: "or, better" / "effrontery" |
| 215 | ⬜ | 025 | rev | Zufäße; zuſammengeſetten | Zuſäße; zuſammengeſeßten | f↔ſ; tz |
| 216 | ⬜ | 026 | mod | "das eigentümliche Geschäfte" | "Geschäft" | archaic neuter; modernized elsewhere (023, 030) |
| 217 | ⬜ | 026 | tra | heading "First Book." vs label "Book I." | align | other files' labels match headings |
| 218 | ⬜ | 029 | mod | "d i."; "Diese unendliche Urteile" | "d. i."; "unendlichen" | typo; inflection |
| 219 | ⬜ | 029 | tra | "For for this very reason"; "non-mortal" for nichtsterbend | "For this very reason"; "non-dying" | doubled word; lost distinction (control "undying") |
| 220 | ⬜ | 030 | rev+mod+tra | "Erzeugung des Quantum" plain | "_Quantum_" / "the _quantum_" | Latin italics |
| 221 | ⬜ | 030 | mod | "ihre ebenso reine abgeleiteten Begriffe" | "reinen" | half-modernized agreement |
| 222 | ⬜ | 030 | tra | "§10. 3. Section."; "***definite principles***" partial span; "under them"; Kennzeichen → "criteria" | "Section 3."; also emphasize "dividing"; "under it"; "marks/indications" | heading pattern; control comparison |
| 223 | ⬜ | 032 | mod | "müsste … müsste" | "müsse … müsse" | Konjunktiv I (indirect discourse) in print |
| 224 | ⬜ | 033 | tra | "if and only if"; "synthetic representations and their objects"; "appropriate activity" | plain conditional; singular "representation and its objects"; "purposive activity" | alsdann…wenn; REV singular; zweckmäßig |
| 225 | ⬜ | 034 | rev | §18/§22/§26 headings use long-s; "in dem es"; "correspondirte"; footnote "des Mannigfaltigen" | round-s headings per convention; "indem es"; "correſpondirte"; "des mannigfaltigen" (adj., raw lowercase) | heading convention + parse |
| 226 | ⬜ | 034 | mod | §16–§26 headings unmodernized (Apperception, Princip, Urtheile, objectiv, Bewußtsein, ſ, "Erkenntniſße" corrupt, "Transscendentale Deduction"); "gegebenen Erkenntnisse"; "erreicht werde"; "könnte"; "Ebendasselbe synthetische Einheit"; "Zusammenreihung"; "irgendwo"; dropped "dadurch"; "dieser letzteren" | modernize headings ("zur Erkenntnis der Dinge" etc.); "gegebener"; "werden"; "könne"; "Ebendieselbe"; "Zusammentreffung"; "irgendworan"; insert "dadurch"; "dieses letzteren" | raw OCR verified for all (p-117–133) |
| 227 | ⬜ | 034 | tra | "In the what follows"; "that intuit itself"; "For for example"; "Erfolg"→"event"; "added"→(gezählt); "an intuition" (Einer); "beyond more laws than"; "recognized" ×3 | "In what follows"; "intuits"; "For example"; "consequent"; "ascribed"; "one intuition"; "does not extend to more laws than…"; "cognized" | control agrees on all |
| 228 | ⬜ | 034 | tra | footnote *******: "corresponding to the inner intuition that corresponds" | "to the inner intuition" (directional "zur") | doubled, loses direction |
| 229 | ⬜ | 035 | mod | "das Erkenntnis {{{ 131 }}} {{ 171 }} über" | "die Erkenntnis" | gender; cf. §4 Erkenntnis policy |
| 230 | ⬜ | 036 | rev | Mutterwiſſes; erſezen; comma after "analytiſch" (also mod); Verſtandeſeinſicht | Mutterwißes; erſeßen; drop comma; Verſtandeseinſicht | print/OCR p-140–142 |
| 231 | ⬜ | 036 | mod | label unmodernized; Verstandsgesetzen/-gebrauchs/-begriffe(n) ×4 | modernized label; Verstandes- | dropped linking-e |
| 232 | ⬜ | 037 | rev | "unter die erſtere" ×2; "= o ="; vermittelst | "unter die erſte" (print); "= 0 ="; vermittelſt | p-143–146 |
| 233 | ⬜ | 037 | mod | Verstands; wornach; Apperception | Verstandes; wonach; Apperzeption | inconsistent within file |
| 234 | ⬜ | 038 | rev | Verſtandeſerkenntniß; apodictiſche | Verſtandeserkenntniß; apodiktiſche | morpheme boundary; corpus norm |
| 235 | ⬜ | 039 | rev | diefes; fann; überflüſsiger; spurious ¶ break before "Denn wenn das Urtheil analytiſch iſt" (all three); stray hard line-breaks (039 ×2, 041 ×1) | dieſes; kann; überflüſſiger; merge ¶; join lines | p-151; control one paragraph |
| 236 | ⬜ | 039 | rev+mod+tra | "conditio sine qua non" plain | "_conditio sine qua non_" | control `<em>` |
| 237 | ⬜ | 039 | mod | zuvorderst | zuvörderst | different word (= at the very front) |
| 238 | ⬜ | 040 | mod | hiezu | hierzu | MOD elsewhere uses hierzu |
| 239 | ⬜ | 040 | tra | "It is only a sum total" | "There is only one sum total in which…" | existential (G-W "There is only one totality") |
| 240 | ⬜ | 041 | rev+mod | footnote Latin plain: conjunctio, compositio, nexus ×2 | italicize | print Antiqua; TRA already italic |
| 241 | ⬜ | 041 | mod | Akkidens | Akzidens | nonstandard |
| 242 | ⬜ | 041 | tra | "carry with them an expression" | "at the same time carry with them" | "zugleich" dropped |
| 243 | ⬜ | 042 | rev | heading "Axiome der Anſchauung" | "Axiomen" | print heading (p.157); body keeps "Axiomen" |
| 244 | ⬜ | 042 | rev+mod+tra | "(quanti)", "(quanta)", "(quantitas)", "(indemonſtrabilia)" plain | italicize all four | print Antiqua; control `<em>` |
| 245 | ⬜ | 043 | rev | missing colon "Das iſt das Reale"; gewiſße ×2; nenuen; Continuitât; Bewußtſeine | "Das iſt: das Reale"; gewiſſe; nennen; Continuität; Bewußtſein | print p-162; junk |
| 246 | ⬜ | 043 | mod | "Mangel sofern zu ergänzen"; "benenne ich es sofern richtig, als" | "so weit … dass"; "insofern … als" | wrong sense of sofern |
| 247 | ⬜ | 044 | rev | "da dieſe ſich" (also mod "diese"); "Eben daſelbe"; Verhältnisſe/Verhältnisſes/Erkenntniſse | "dieſes"/"dieses"; "daſſelbe"; ſſ forms | print: das Dasein referent |
| 248 | ⬜ | 045 | rev | "Beweiſ"; Verhältnisſe/Zeitverhältnisſe | "Beweis"; ſſ forms | terminal long-ſ introduced in curation |
| 249 | ⬜ | 045 | mod | "z. B." (REV "z. E."); "Akzidenz" ×2 | restore "z. E."; "Akzidens" | silent change; nonstandard form |
| 250 | ⬜ | 045 | tra | "coexistence" (Begleitung); "apprehension" (Besorgnis); abiding/persistent/permanent mix | "accompaniment"; "worry/concern"; unify "permanence" family | collides with technical Apprehension |
| 251 | ⬜ | 046 | rev | Entſehen; "etwas andere" (also mod); "***ſubjectives***" case (also mod); empirifche; Caufalität ×2; diefes; find; fich; fo; ſs-mix ×7 (ſucceſsiv, gewiſse, Succeſsion ×2, ſucceſsiven, Küsſen, Succesſion); tz strays (leztere, Geſeze, sezen, feſtsezen, legten, letteren); modern "dass" ×2; letter O for 0 | Entſtehen; "anderes"; "***Subjectives***/***Subjektives***" (substantivized); empiriſche; Cauſalität; dieſes; ſind; ſich; ſo; ſſ forms; print forms; "daß"; digit 0 | scan/raw verified |
| 252 | ⬜ | 046 | mod | giebt; Induction; So bald | gibt; Induktion; Sobald | unmodernized/inconsistent |
| 253 | ⬜ | 046 | tra | "where I had to begin"; "receptivity" (Aufnahme); "specific" added; "as its determining cause" (Folge!); "the parts of time" | "when"; "their being taken up"; drop; "determining it as its consequence"; "its parts" (of the progress) | control comparisons |
| 254 | ⬜ | 047 | rev | gefeßt→(f); Einflüſße; Ge. meinschaft; Zuſammengeſettes; Grundsazes | geſeßt; Einflüſſe; Gemeinſchaft; Zuſammengeſeßtes; Grundſaßes | f/ſ on tz; junk period |
| 255 | ⬜ | 047 | rev+mod | communio, commercium, communio spatii, Commercium, compositum reale plain | italicize | control `<em>`; TRA already italic (resolves mech emphasis flags blocks 7–8) |
| 256 | ⬜ | 047 | tra | "to A itself"; "permanent" | drop "itself"; "persistent" | spurious; terminology |
| 257 | ⬜ | 049 | rev | å-junk ×7 (Prådicate, Gegenſtånde, Realitåt, erwåge, gånzlich, vollſtåndig, nāmlich); Ansſchauung; find; Nugung; "Teil"; transfcendentalen; "Dingen" (footnote); deſsen | ä; Anſchauung; ſind; Nuzung; Theil; transſcendentalen; Dinge; deſſen | raw confirms |
| 258 | ⬜ | 049 | rev+mod | Latin law-phrases plain (in mundo non datur casus/fatum/saltus/hiatus; assertio also in tra) | italicize (decide "vacuum" too) | Cambridge italicizes |
| 259 | ⬜ | 049 | mod | "nur eine empirische Behauptung"; "natürlicher Weise" | "nur Eine" (print capital = only ONE); "natürlicherweise" | p-199 |
| 260 | ⬜ | 049 | tra | "the transcendental {{ 267 }} [use]"; "we could underlie"; "idealizing reason" | "use" (no brackets); "on which we could base"; "ideal reason" | only editorial bracket in corpus |
| 261 | ⬜ | 051 | rev | wißen; lettere(n) ×4; weglaffe; Grundſake; transfcendentale ×2 (+missing space "(außerordentliche)"); müſse/Lasſe/zulasſen/Succesſion; "ſi nnliches"; "die Sinnen ſtellen"; Sphaere | wiſſen; leßtere(n); weglaſſe; Grundſaße; transſcendentale + space; ſſ forms; ſinnliches; die Sinne; Sphäre | raw p-212–222 |
| 262 | ⬜ | 051 | mod | Verstandsgesetzen/Verstandsgebrauch; "intelligible oder sensible heißen" | Verstandes-; "intelligibel oder sensibel" | linking-e; anglicized inflection |
| 263 | ⬜ | 051 | tra | "undisputably"; "he/his" for the understanding; Latin italics inconsistent (mundi sensibilis/intelligibilis, Noumeni, commercium); "permanence" | "indisputably"; "it/its"; italicize consistently; "persistence" | terminology |
| 264 | ⬜ | 052 | rev | find ꝛc.; "ſind z."; letteren; Sag; Grundsag; Sak; Sat; Auschauung; phyfischen; transfcendentalen ×2; transßcendentale ×2; eigenthůmlichen; Realitât; Grundſāße; Philofophen; gewiſze/gewiſße/gewiſse; laſsen; Verhältniſsen/Verhältniſße; Caufalität; u. f. w. | ſind ꝛc. (×2); leßteren; Saß; Grundſaß; Saß; Saß; Anſchauung; phyſiſchen; transſcendental- ×4; eigenthümlichen; Realität; Grundſäße; Philoſophen; gewiſſe; laſſen; ſſ forms; Cauſalität; u. ſ. w. | raw confirms all |
| 265 | ⬜ | 052 | mod | "u. f. w." carried; darthut; "sie Objekte" (REV "Gegenſtände"); "sucht" (REV "suchte"); "ihr Leibniz-Wolffianisches Lehrgebäude" (print plural) | u. s. w.; dartut; Gegenstände; suchte; "ihre Leibniz-Wolffianischen Lehrgebäude" | TRA mostly unaffected |
| 266 | ⬜ | 052 | rev+mod | figcaption "Tafel der Nichts" | "Tafel des Nichts" | curator-added, ungrammatical |
| 267 | ⬜ | 052 | tra | "epitome" (Inbegriff); "do not have to be counted" ×2; "similar and equal to itself"; "necessary in themselves"; "faculty of cognition" (Erkenntnißkraft) | "sum total"; "must not be counted" (modal scope); reciprocal "they"; "of itself … not only possible but also necessary"; "power of cognition" | control comparisons |
| 268 | ⬜ | 053 | mod | heading "Zweite Abtheilung. Die transscendentale Dialektik" | "Zweite Abteilung. Die transzendentale Dialektik" | 013/019/025 MOD headings are modernized |
| 269 | ⬜ | 055 | rev | Marimen; gânzlich; legteren; Grundsäge; fubjectiven Grundfäßen; Grundfäße; gefünstelten; angemesſenen | Maximen; gänzlich; letzteren; Grundſäße; ſubjectiven Grundſäßen; Grundſäße; gekünſtelten; angemeſſenen | raw confirms; x→r and f/k |
| 270 | ⬜ | 055 | mod | heading "Vom transscendentalen Schein"; Verstandsgesetzen | "transzendentalen"; Verstandesgesetzen | also 056 heading |
| 271 | ⬜ | 055 | tra | "does not likewise cease" | "nevertheless does not cease" | gleichwohl |
| 272 | ⬜ | 057 | mod | Verstandserkenntnis | Verstandeserkenntnis | linking-e |
| 273 | ⬜ | 058 | rev + toc + filenames | "logichen" (REV heading + label, 000 TOC line 63, German filenames) | "logischen" | raw OCR itself reads "logischen"; MOD/TRA headings already corrected; renaming files touches the pipeline |
| 274 | ⬜ | 059 | mod | "wirklich Statt hat" | "statthat" | leftover |
| 275 | ⬜ | 060 | mod | wornach | wonach | leftover |
| 276 | ⬜ | 060+061 | tra | "## First Book." / "## 1. Section." | "Book I." / "Section 1." | match labels and 062/063/064 style |
| 277 | ⬜ | 061 | rev | Anſehrung ×3; Erkenntniſße | Anſehung; Erkenntniſſe | curation-stage corruption (raw reads Anſehung) |
| 278 | ⬜ | 061 | mod | "ins Licht"; "möglichst größten" | "in Licht" (print); "möglich größten" | silent rewording |
| 279 | ⬜ | 061 | tra | "nor even to itself {{ 369 }}." | move marker before "properly intelligible" | four words follow it in REV/MOD |
| 280 | ⬜ | 062 | rev | ſpeculatiwen ×3; gewiſße; Erkenntnißſe; "_per epiſyllogiſmos_" | ſpeculativen; gewiſſe; Erkenntniſſe; "_per episyllogismos_" | v→w; round s in Latin |
| 281 | ⬜ | 062 | mod | Erkenntnis gender flips ×3; "erwogenen reinen" | keep neuter as REV; "erwogene reine" (print) | see §4 policy |
| 282 | ⬜ | 063 | mod | "müsste" ×2; "Sie bedarf ihrer nicht zum Behufe" | "müsse"; "Sie bedarf sie nicht zum Behuf" | mood; print wording (raw p-269) |
| 283 | ⬜ | 065 | rev | "transscendentalen" (1 of 13); _remisſio_; Interesſe; Verhältnisſes; Oberſaze | transſcendentalen; _remissio_; Intereſſe; Verhältniſſes; Oberſaße | consistency |
| 284 | ⬜ | 065 | mod | heading "Beweiſes"; "schließt … in sich" (REV "hält"); "als bloßen Gegenstandes" | "Beweises"; "hält … in sich"; "als bloß Gegenstandes" | ſ leftover; lexical swap; adverbial bloß |
| 285 | ⬜ | 065 | tra | "converted in {{ 414 }} into"; "(sofar as"; "***I***, as thinking, is an object"; "unground"; "destination" | drop first "in"; "insofar as"; "…am an object of the inner sense, and am called the soul"; "groundlessness"; "vocation" | person/grammar/lexicon |
| 286 | ⬜ | 066 | rev | ſeze; Geſezt; trailing double blank lines | ſeße; Geſeßt; strip | tz strays; EOF hygiene |
| 287 | ⬜ | 066 | mod | "Missverständnisses" | "Missverstandes" | REV "Mißverſtandes"; align with 065 |
| 288 | ⬜ | 067 | rev | fſich; gewiſse; lasſen/laſsen; Geſeze | ſich; gewiſſe; laſſen; Geſetze | raw confirms |
| 289 | ⬜ | 067 | rev+mod | "Principium" plain | "_Principium_" / "_Prinzipium_" | Antiqua in print; TRA "principle" fine |
| 290 | ⬜ | 068 | rev | transfcendentale; abfolute(n) ×2; Jdee; im gleichen; lettere; progreffive; wcil; Meſsen; dom; Caufalität; Totalitāt; wiſsen; fucceſſive; Regreffus; Veruunft | transſcendentale; abſolute(n); Idee; imgleichen; letztere; progreſſive; weil; Meſſen; vom; Cauſalität; Totalität; wiſſen; ſucceſſive; Regreſſus; Vernunft | raw confirms; MOD "von dem" → "vom" for 1:1 |
| 291 | ⬜ | 068 | mod | "schlechthin unbedingte wäre" | "unbedingt" | stray -e vs REV/raw |
| 292 | ⬜ | 068 | rev+mod+tra | "(Noumenis)" plain | "(_Noumenis_)" / "_noumena_" | Antiqua in print; control italicizes |
| 293 | ⬜ | 068 | tra | "of subordinated (not coordinated) conditions" | "of conditions subordinated (not coordinated) to one another" | "einander" dropped |
| 294 | ⬜ | 069 | rev | vor züglichen; letteren; gewisſer; Gegensage; vernünfteľnde; rüftige; leßzten; lasſen; Gefeße; Grundfäße; untergråbt; transfcendentalen | vorzüglichen; letzteren; gewiſſer; Gegenſatze; vernünftelnde; rüſtige; letzten; laſſen; Geſeße; Grundſäße; untergräbt; transſcendentalen | raw confirms |
| 295 | ⬜ | 069 | mod | "könnte gegeben werden" | drop "werden" or record as accepted completion | print has archaic ellipsis "könnte gegeben, noch" |
| 296 | ⬜ | 069 | tra | sceptical/scepticism ×4 | skeptical/skepticism | corpus + control standard |
| 297 | ⬜ | 070 | rev | missing colon "Gegentheil an ſo wird"; gefeßt; "(transſcendental)"; gewiſser; Verhältniſse; round-s slips (Erster, ersten, solchen, Ursache, unterscheidende, diese, absolute, Gegenstände, statt, ist, derselben) | "an: ſo wird"; geſeßt; "(transſcendentale)"; gewiſſer; Verhältniſſe; long-ſ | print confirms |
| 298 | ⬜ | 070 | rev+mod | footnote "Quantum", "Correlatum" plain | "_Quantum_", "_Correlatum_" | Antiqua in print; TRA already italic |
| 299 | ⬜ | 070 | tra | "Space can therefore"; footnote "but they are rather bound together"; "consists in this" | "A space can therefore" (ein Raum); "but are only bound together" (nur); "consists precisely in this" (eben) | dropped words |
| 300 | ⬜ | 071 | rev | Simplicitāt; "(molecularum)" plain; round-s "schlechthin" ×3 | Simplicität; "(_molecularum_)"; ſchlechthin | print confirms |
| 301 | ⬜ | 071 | mod | "das Ganzes"; bemerkke; "außer einander" (¶30) | "das Ganze"; bemerke; "außerhalb einander" | REV wording |
| 302 | ⬜ | 072 | rev | fahen; Einflüſße; round-s slips (Zustand, verschaffen, Wechsel, unumschränkten, verspricht, sondern, Zusammenhang) | ſahen; Einflüſſe; long-ſ | raw confirms |
| 303 | ⬜ | 072 | mod | beiweitem; "den Zusammenhang und die Ordnung" | "bei weitem"; "den Zusammenhang und Ordnung" | added article vs REV |
| 304 | ⬜ | 072+073 | tra | "Of the Antinomy…" / "Observation on" | "The Antinomy…" / "Remark on" | normalize with 070/071 |
| 305 | ⬜ | 072 | tra | "beginning of the world" (Urſprung) | "origin of the world" | Anfang occurs separately in same sentence |
| 306 | ⬜ | 073 | rev | Schlüſse; round-s (verschiedenen, sondern) | Schlüſſe; long-ſ | |
| 307 | ⬜ | 073 | rev+mod+tra | "(infit)", "(fit)" plain (footnote) | "(_infit_)", "(_fit_)" | Antiqua in print |
| 308 | ⬜ | 073 | mod | "als formal-Bedingung" | "als formale Bedingung" | REV/TRA correct |
| 309 | ⬜ | 073 | tra | "unconditioned-necessary"; "quite in accordance with common human reason"; "after considering" | "unconditionally necessary"; "even with common human reason" (ſelbſt); "when it considers" (nachdem non-temporal) | nuances |
| 310 | ⏭️ | 000 | mod | "Idealismusus"; TOC labels out of sync with MOD file H2s (Function/Funktion, Deduction/Deduktion ×4, Anticipationen/Antizipationen, Beschluß/Beschluss; unmodernized Sceptische/Disciplin for future files) | "Idealismus"; modernize labels to match headings | line 82 + passim |
| 311 | ⏭️ | 000 | rev (+mod) | "mathematischtranszendentalen"; TOC entries 074–086, 095 in modern orthography vs REV TOC's own 18th-c. style | "mathematisch-transzendentalen"; normalize per layer | hyphen jam; style consistency |

---

## 4. Systematic patterns & policy questions

| # | Status | File | Layer | Current | Correct | Note |
|---|---|---|---|---|---|---|
| 312 | ❓ | corpus | rev | tz-ligature rendered six ways: sanctioned `ß` (ſeße, Saß — 231 tokens), `ſetz` (113), plus strays as `z` (Saz, Geſeze, Siz), `k` (Sak, lektere), `t/tt` (Sat, lettere), `g` (Sag, Säge), `h` (ſehen — meaning-changing, fixed in §1) | pick one canonical form; mechanically normalize the z/k/t/g/h strays (unambiguous non-/wrong words) | §1/§3 list the meaning-relevant cases individually |
| 313 | ❓ | corpus | rev | round s freely mixed where print has `ſ` (whole blocks in 021, 065, 070–073; scattered everywhere) | bulk-normalize to long-ſ, or accept as-captured and document | skill prescribes faithful long-s |
| 314 | ❓ | corpus | rev | x→r misreads beyond the sanctioned Existenz family: Marimen (055), Contert ×2 + Ariomen (049), Ariomen (016), erponirt (068), Erperiment (003), Philodorie (003) | decide whether the Eriſtenz sanction generalizes; if not, fix all (some already rows above) | Eriſtenz 11 / Exiſtenz 41 tokens coexist by design |
| 315 | ❓ | corpus | rev | `ꝛc.` (etc.) rendered as "2c.", "zc.", "sc.", "z.", or dropped/garbled to "zu" | pick canonical form (`ꝛc.`) and sweep | the "zu" cases are meaning-affecting (015) |
| 316 | ⬜ | several | mod | frontmatter labels inconsistently modernized (017, 018, 036, 053, 055, 056 old; 058, 062, 063 modernized) | unify | |
| 317 | ❓ | corpus | mod | lexical policy inconsistent: hiebei/hievon/hiemit/hiezu/darnach/wornach sometimes modernized; Mißverstand→Missverständnis (066, 069) vs kept (003, 065); "in Stecken"→"ins Stocken" (003); rhapſodiſtiſch→rhapsodisch (030); Erkenntnis gender flips (021/023 vs 025/028/029/030/034/035/062); Verstands- vs Verstandes- (036, 037, 051, 055, 057) | write a MOD style policy, then sweep (~30 flags collapse) | |
| 318 | ❓ | corpus | tra | terminology drift: Erkenntnis="knowledge" throughout 065; erkennen→"recognized" (034); Beharrlichkeit permanence/persistence (045, 047, 051, 065); Vernunftschluss "syllogism"/"inference of reason" (058–064); Größe→"quantity" (029); Kennzeichen→"criteria" (006, 030, 046); sceptical (069); "Of the…"/"The…" + "Remark/Observation" heading styles (060/061, 070–073) | fix the glossary, then sweep | glossary: cognition, persistence, magnitude, mark |
| 319 | ❓ | corpus | all | Latin/Greek typography: Antiqua terms → italics (quanti/quanta/quantitas, conjunctio/compositio/nexus, communio/commercium, casus/fatum/saltus/hiatus, assertio, Noumenis, molecularum, infit/fit, in concreto, conditio sine qua non, Physica) vs Fraktur-set terms → plain (Correlatum 046, principium 057); Greek garbled (014) or transliterated (043 προληψις) | decide policy, then sweep (individual rows above) | REV/MOD lag TRA on most |
| 320 | ❓ | several | all | §-heading conventions differ: "Einleitung §1"/"§ 2." (014), missing "§ 4." (016), missing "§ 8." (017), "§9" trailing (029) vs "§10" leading (030), unmodernized § headings (034) | pick one shape | individual rows above |
| 321 | ❓ | corpus | rev | proper names in Sperrdruck: print letterspaces names (Baumgarten, Berkeley, Epikur, Thales…); REV marks some (***Copernicus***) not others | confirm convention | affects #77-area rows |
| 322 | ❓ | 007, 011 | rev | files start mid-AA-page without repeating the running `{{{ N }}}` marker (005/006/008/009/010 repeat it) | confirm ingest relies on frontmatter `aa_page`, else add markers | |
| 323 | ⏭️ | corpus | all | `\|\|\|` markers (003 all layers, 050 mod+tra) | none — do not sweep | reclassified: `\|\|\|` is the pipeline's forced sentence-split marker (`sentences.rs strip_forced_splits`), placed where the auto-splitter would not split (after a colon in 003; after the abbreviation "usw." in 050); see #100/#139 |

---

## 5. Verified non-issues (do not "fix"; future audits skip these)

- AA/B marker repeats at file boundaries (sections starting mid-page) and the antinomy facing-page interleave (070–073) are by design.
- AA page-number gaps at title pages are blank pages: AA 4, AA 6 (002/003), AA 48 + B 31/32 (012 half-title).
- "Unbalanced parens" from `1)`, `2)`, `a)` list markers are list-item punctuation.
- All mechanical sentence-count mismatches were read and resolved as abbreviation noise (z. B./z. E./d. i./u. ſ. w./etc.) except those that became findings; the 1:1 sentence rule holds corpus-wide (the three §1 truncations are now fixed).
- Königsberg umlaut in TRA: proper noun.
- Genuine "fein" (not f/k): 003 "fein geſponnenen", 043 "Mark fein Silber", 049 "Sinnen feiner", 071 "Monadiſten fein genug".
- TRA marker word-position drift caused by English word order (047 {{ 257 }}, 052 {{ 349 }}, 065 {{ 414 }}): same page boundary, acceptable.
- B-markers {{ 240 }}, {{ 250 }}, {{ 441 }} spot-checked against margins: correctly placed.
- Print-faithful oddities (REV matches the 1911 reprint): "erfolgt werden" (003; MOD/TRA emend to "verfolgt" — vetted), "Vorgehens" for received "Vorgebens" (023 — reprint typo; decide follow-print vs emend), "Teile" amid "Theil" (035), "ein Größe" (046), "ſeiner Urſprung" (052), "Eiſenfeiligs" (049), "Diallele" (Akademie emendation, 023), archaic "das Scandal" (003), "Categorie"/"Kategorie" mixing (030), "ſebſt" (016 — reprint slip, flagged in its row).
- Prefix 048 is unused — TOC and all three dirs jump 047→049 with no content gap (047 ends {{ 265 }}/AA 185, 049 opens {{ 266 }}/AA 185).

---

## 6. Suggested fix order

1. ~~§1 criticals~~ — ✅ done 2026-06-11 and imported (008 via two-pass import).
2. ~~§2.1 markers~~ — ✅ done 2026-06-11; awaiting next import run (single pass, no sentence-count changes).
3. §2.2 emphasis — verify each against the cited page scan while applying.
4. ~~§2.3 wrong words~~ — ✅ done 2026-06-11 (all except #90, deferred by request; #100/#139 reclassified as intentional forced-split markers); awaiting next import run.
5. Decide §4 policies (tz-rendering, long-s, x→r scope, ꝛc., MOD style, TRA glossary), then do the §3 sweeps file-by-file — many minors collapse into a handful of regex passes once policies are fixed.

Reminder: any fix that changes a block's sentence count must land as two import passes (see Remediation log).
