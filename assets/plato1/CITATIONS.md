# Citations for quoted verse in Plato's *Republic* (plato1)

This is a **draft for review**, not yet applied to the curated markdown. It proposes a footnote for every quotation site Perseus's TEI apparatus flagged in `assets/plato1/derived/quotations_report.tsv` (130 rows), verified directly against the primary Greek texts on disk wherever a text was available (`canonical-greekLit`: Homer, Hesiod, Aeschylus, Pindar's four books of extant odes, Herodotus, Euripides' *Orestes*). Where no on-disk text exists (fragments of lost plays and lyric poets, known only through ancient quotation), standard modern fragment numbers were established from secondary scholarship instead, and are flagged as such.

- **71 rows — LITERARY** (quotation of another author; footnote proposed)
- **49 rows — DIALOGUE** (Socrates/an interlocutor, or a self-repeat within the *Republic*; no citation needed)
- **10 rows — UNIDENTIFIED** (literary in character; no source could be located — reported honestly rather than invented)

Of the 71 LITERARY rows, **61 were verified against an on-disk primary text** (exact line(s) given, which sometimes differ from Perseus's own line number — see the notes). The remaining LITERARY rows are fragments of lost works (Pindar, Simonides, Aeschylus's lost plays) for which no primary text is available in this repository; those are cited by standard modern fragment number where scholarship allowed one to be established, and marked unverified otherwise.

**4 rows where Perseus's citation is flatly wrong** (rows for stephanus 364e, 383b, 386c, 545e — see notes for each; two of these mislabel a verbatim Homer quotation as an Aeschylus fragment, one cites the wrong Iliad line entirely, one cites a non-existent Iliad match for Socrates' own mock-Homeric invocation). One further row (stephanus 469a) is off by one line, which may be an edition-numbering difference rather than an error.

**15 new identifications** — LITERARY quotations Perseus gave no `perseus_bibl` for at all, now identified and verified on disk (notably a full Homeric-formula tag at 328e, an entire half-line of Odyssey 9 at 390a, and several partial lines that are the un-tagged other half of a citation Perseus did give on a neighboring row).

**Never invented:** every UNIDENTIFIED row is reported as such rather than guessed. This includes the six anonymous anti-philosophy tags at 607b–c, an unattested Aeschylus couplet (380a) whose authenticity is independently unconfirmable, and a couple of proverbial lines that ancient sources themselves never assign to an author.

| stephanus | class | proposed footnote | source verified | perseus said | note |
|---|---|---|---|---|---|
| 328e | LITERARY | Homer, *Iliad* 22.60 | yes — Iliad 22.60 (on-disk) |  | Perseus gave no identification. Plato attributes the phrase generically to "the poets" (οἱ ποιηταί), not a single line — it is a recurring Homeric formula: also *Iliad* 24.487 and *Odyssey* 15.348 (all verified on disk). |
| 329b | DIALOGUE |  |  |  |  |
| 329c | DIALOGUE |  |  |  |  |
| 329c | DIALOGUE |  |  |  |  |
| 329c | DIALOGUE |  |  |  |  |
| 331a | LITERARY | Pindar, fr. 214 Snell–Maehler | no — fragment, not in on-disk corpus |  | Cephalus's own word, immediately preceding his verbatim quotation of the same Pindar fragment at 331a (next row); the word γηροτρόφος recurs verbatim there. Same reference as the following row. |
| 331a | LITERARY | Pindar, fr. 214 Snell–Maehler (Loeb ed. Race) | no — fragment; not in on-disk corpus (only Pindar's 4 books of extant epinicians are on disk, not the fragments) | Pindar Frag. 214, Loeb | Perseus's "Frag. 214, Loeb" confirmed correct against modern scholarship. Survives only through ancient quotation (chiefly this Plato passage). |
| 332c | DIALOGUE |  |  |  |  |
| 334b | LITERARY | Homer, *Odyssey* 19.395–396 | yes — Odyssey 19.395-396 (on-disk, exact) | Hom. Od. 19.395 |  |
| 337b | DIALOGUE |  |  |  |  |
| 337c | DIALOGUE |  |  |  |  |
| 339a | DIALOGUE |  |  |  |  |
| 341e | DIALOGUE |  |  |  |  |
| 350e | DIALOGUE |  |  |  |  |
| 362a | LITERARY | Aeschylus, *Seven Against Thebes* 593 | yes — Seven Against Thebes 593 (on-disk; Plato's ‑ον for the source's ‑ος, adapting the case to his sentence) |  | Same citation as the next row (Aesch. *Seven* 592–594); together rows for 362a–b quote lines 593–594. |
| 362b | LITERARY | Aeschylus, *Seven Against Thebes* 594 | yes — Seven Against Thebes 594 (on-disk, exact) | Aesch. Seven 592-594 |  |
| 363b | LITERARY | Hesiod, *Works and Days* 232–234 | yes — WD 232-234 (on-disk; Plato paraphrases in indirect infinitive construction rather than quoting the verse form directly) | Hes. WD 232 | Perseus cites only the opening line (232); the paraphrase draws on 232–234 together. |
| 363b | LITERARY | Homer, *Odyssey* 19.109 | yes — Odyssey 19.109 (on-disk, opening words) |  | Opening words of the same continuous Homer quotation as the next two rows (Od. 19.109–113); Perseus tagged the citation only on the row for 363c. |
| 363b | LITERARY | Homer, *Odyssey* 19.109–111 | yes — Odyssey 19.109-111 (on-disk, exact) |  | Continues the quotation begun in the previous row. |
| 363c | LITERARY | Homer, *Odyssey* 19.112–113 | yes — Odyssey 19.112-113 (on-disk, exact) | Hom. Od. 19.109 | Perseus's citation ("19.109") marks the start of the whole quoted passage (rows for 363b–c together = Od. 19.109–113), not specifically this row's text. |
| 364c | LITERARY | Hesiod, *Works and Days* 287 | yes — WD 287 (on-disk; Plato drops the particle τοι) |  | Same citation as the next row (Hes. WD 287–289); Perseus tagged only that row. |
| 364d | LITERARY | Hesiod, *Works and Days* 288–289 | yes — WD 288-289 (on-disk, exact) | Hes. WD 287-289 |  |
| 364d | LITERARY | Homer, *Iliad* 9.497, 499 | yes — matches Iliad 9.497 (variant λιστοί for στρεπτοί) + 9.499 (θυσίαισι for θυέεσσι), with 498 silently dropped |  | Part of the same Phoenix "Prayers" speech as the next row; see that row's note for Perseus's line-number error on the companion citation. |
| 364e | LITERARY | Homer, *Iliad* 9.500–501 | yes — Iliad 9.500-501 (on-disk, exact, cov=1.00) | Hom. Il. 9.497 | Perseus's citation ("Hom. Il. 9.497") is WRONG: line 497 reads νηλεὲς ἦτορ ἔχειν· στρεπτοὶ δέ τε καὶ θεοὶ αὐτοί, which is not this text at all. The quoted words are verbatim 9.500–501, three lines further on in the same Phoenix speech quoted (in adapted form) at the previous row. |
| 365b | LITERARY | Pindar, fr. 213 Snell–Maehler (preserved by Maximus of Tyre) | no — fragment; not in on-disk corpus | Pindar, Fr. | Perseus gave only "Pindar, Fr." with no number. Modern scholarship identifies this as fr. 213 S–M, immediately adjacent to fr. 214 quoted at 331a (row 6). |
| 365c | DIALOGUE |  |  |  | Plato's own philosophical keyword ("seeming"), introducing the Simonides quotation at the next row. |
| 365c | LITERARY | Simonides, fr. 598 PMG (Page) [= fr. 76 Bergk]; cf. Euripides, *Orestes* 236 | Euripides part: yes — Orestes 236 (on-disk, exact: κρεῖσσον δὲ τὸ δοκεῖν, κἂν ἀληθείας ἀπῇ). Simonides part: no — fragment not in on-disk corpus | Simonides, Fr. 76 Bergk, and Eur. Orest. 236. | Perseus's dual citation confirmed correct. Bergk's 19th-century numbering (fr. 76) corresponds to Page's PMG 598 in the modern standard edition; the Simonides fragment is itself preserved via a scholion to this line of Euripides' Orestes, which is presumably why Perseus paired the two. |
| 365c | DIALOGUE |  |  |  |  |
| 365d | DIALOGUE |  |  |  |  |
| 365e | DIALOGUE |  |  |  | Plato's own prose ("θυσίαις τε καὶ εὐχωλαῖς ἀγανῇσιν καὶ ἀναθήμασιν"), echoing vocabulary from the Iliad 9 quotation used two Stephanus pages earlier (rows for 364d, ≈ Il. 9.499 εὐχωλῇς ἀγανῇσι) but not itself a fresh, separately citable quotation. |
| 366a | DIALOGUE |  |  |  |  |
| 366e | DIALOGUE |  |  |  |  |
| 367a | DIALOGUE |  |  |  |  |
| 368a | LITERARY | an unnamed elegist, "Glaucon's lover" (Γλαύκωνος ἐραστής) — traditionally conjectured to be Critias (Schleiermacher, 1828) | no — text not extant outside this citation |  | Valuable: Perseus gave no identification at all. Plato names the author only by relationship to Glaucon, not by name; the standard scholarly conjecture (via Debra Nails and others) identifies him as Critias, later of the Thirty Tyrants, who is independently attested as a poet in several genres. The elegy itself does not survive elsewhere. |
| 379d | LITERARY | Homer, *Iliad* 24.527–528 | yes — Iliad 24.527-528 (on-disk, exact) | Hom. Il. 24.527-8 |  |
| 379d | LITERARY | Homer, *Iliad* 24.530 | yes — Iliad 24.530 (on-disk, exact) | Hom. Il. 24.530 |  |
| 379d | LITERARY | Homer, *Iliad* 24.532 | yes — Iliad 24.532 (on-disk; Plato's τὸν δὲ for the source's καί ἑ) | Hom. Il. 24.532 |  |
| 379e | UNIDENTIFIED |  |  | unknown | Perseus itself flags this "unknown." Loosely echoes *Odyssey* 4.392 (κακόν τʼ ἀγαθόν τε τέτυκται) but the word order and case forms differ too much to call it the same line; more likely a continuation of the "two jars of Zeus" theme just quoted from *Iliad* 24.527ff than an independently citable verse. No verbatim match found on disk. |
| 380a | LITERARY | "as Aeschylus says" (Plato's own words) — fragment of disputed authenticity, unattested outside this citation | no — fragment not in on-disk corpus; not confirmed by any independent ancient source | Aesch. Fr. | This couplet is the source of the later tag "whom the gods would destroy, they first make mad." Plato explicitly names Aeschylus (ὡς Αἰσχύλος λέγει), but because no other ancient author quotes or alludes to it, its authenticity as genuine Aeschylus cannot be independently verified — flagged honestly rather than asserting a fragment number I cannot confirm. |
| 381d | LITERARY | Homer, *Odyssey* 17.485–486 | yes — Odyssey 17.485-486 (on-disk, exact) | Hom. Od. 17.485-486 |  |
| 381d | LITERARY | Aeschylus, fr. 168 Radt (Hera, disguised as a wandering priestess, hymning the nymphs of Argos; play uncertain — perhaps *Semele* or *Xantriai*) | no — fragment not in on-disk corpus | Aesch. | Perseus gave only the bare author name "Aesch." with no fragment or play. Modern scholarship (Radt's TrGF) identifies fr. 168–168b and situates the speaker as Hera in priestess disguise, consistent with Plato's context ("μηδ' ἐν τραγῳδίαις... εἰσαγέτω Ἥραν ἠλλοιωμένην ὡς ἱέρειαν ἀγείρουσαν"). |
| 383b | DIALOGUE |  |  | Hom. Il. 2.1 | IMPORTANT — Perseus's citation ("Hom. Il. 2.1") is WRONG. *Iliad* 2.1 reads ἄλλοι μέν ῥα θεοί τε καὶ ἀνέρες ἱπποκορυσταὶ, entirely unrelated. This phrase is not a quotation at all: it is Plato's own indirect-discourse paraphrase introducing the Aeschylus fragment quoted verbatim in the next row ("οὐδὲ Αἰσχύλου, ὅταν φῇ ἡ Θέτις τὸν Ἀπόλλω ἐν τοῖς αὑτῆς γάμοις ᾄδοντα ἐνδατεῖσθαι τὰς ἑὰς εὐπαιδίας —"). No citation applies to this row; the real citation belongs to the next row (Aesch. fr. 350 Radt). |
| 383b | LITERARY | Aeschylus, fr. 350 Radt (Thetis reproaching Apollo for his false prophecy about Achilles) | no — fragment not in on-disk corpus | Aesch. Frag. 350 | Explicitly introduced by name in Plato's own prose (previous row). Well-known fragment, quoted elsewhere in antiquity too. |
| 386c | LITERARY | Homer, *Odyssey* 11.489–491 | yes — Odyssey 11.489-491 (on-disk, verbatim, cov=1.00) | Aesch. Frag. 350 | IMPORTANT CORRECTION — Perseus's citation ("Aesch. Frag. 350") is WRONG, apparently a copy-paste of the previous quotation's tag (rows for 383b). This text is Achilles' words to Odysseus in the underworld, quoted by Plato as "τοῦδε τοῦ ἔπους" with no mention of Aeschylus anywhere nearby; it is 100% verbatim *Odyssey* 11.489–491 (already independently and correctly cited elsewhere in this report at 516d). |
| 386d | LITERARY | Homer, *Iliad* 20.64–65 | yes — Iliad 20.64-65 (on-disk, exact) | Hom. Il. 20.64 |  |
| 386d | LITERARY | Homer, *Odyssey* 10.495 | yes — Odyssey 10.495 (on-disk; ὢ πόποι, ἦ ῥά τις for source's οἴῳ πεπνῦσθαι word order variant, and τοὶ/ἀτὰρ φρένες variants) | Hom. Od. 10.495 |  |
| 386d | LITERARY | Homer, *Iliad* 23.103 | yes — Iliad 23.103 (on-disk, close paraphrase-quote) | Hom. Il. 23.103 |  |
| 386d | LITERARY | Homer, *Iliad* 16.856–857 | yes — Iliad 16.856-857 (on-disk, exact) | Hom. Il. 16.856 |  |
| 387a | LITERARY | Homer, *Iliad* 23.100–101 | yes — Iliad 23.100-101 (on-disk, exact) | Hom. Il. 23.100 |  |
| 387a | LITERARY | Homer, *Odyssey* 24.6–7 | yes — Odyssey 24.6-7 (on-disk, exact) | Hom. Od. 24.6-10 |  |
| 387c | UNIDENTIFIED |  |  |  | Not a specific citable line: Plato lists it alongside Κωκυτούς/Στύγας/ἀλίβαντας as an example of the frightening vocabulary poets use for the underworld, without quoting a particular verse. Generic epic/tragic term for the dead below (νέρτεροι), literary in character but no single source to cite. |
| 387c | UNIDENTIFIED |  |  |  | Same list as the previous row (ἀλίβας, "bloodless one," a rare word for the dead attested in ancient lexica as Homeric, but not found verbatim in the on-disk Odyssey/Iliad text in this form). Literary in character; no specific citable source located. |
| 387c | DIALOGUE |  |  |  | Not a quotation: this is a verb in Plato's own sentence ("ὡς οἴεται... ποιεῖ"), evidently a mis-extraction by the source tool. |
| 388a | LITERARY | Homer, *Iliad* 24.10 | yes — Iliad 24.10 (on-disk, close paraphrase) | Hom. Il. 24.10-12 |  |
| 388b | LITERARY | Homer, *Iliad* 24.12 | yes — Iliad 24.12 (on-disk, close paraphrase) | Hom. Il. 24.12 |  |
| 388b | LITERARY | Homer, *Iliad* 18.23–24 | yes — Iliad 18.23-24 (on-disk, close paraphrase) | Hom. Il. 18.23-24 |  |
| 388b | LITERARY | Homer, *Iliad* 22.414–415 | yes — Iliad 22.414-415 (on-disk, close paraphrase) | Hom. Il. 22.414-415 |  |
| 388c | LITERARY | Homer, *Iliad* 18.54 | yes — Iliad 18.54 (on-disk, exact) | Hom. Il. 18.54 |  |
| 388c | LITERARY | Homer, *Iliad* 22.168–169 | yes — Iliad 22.168-169 (on-disk, close paraphrase) | Hom. Il. 22.168 |  |
| 388c | LITERARY | Homer, *Iliad* 16.433 | yes — Iliad 16.433 (on-disk; Plato's αἲ αἲ ἐγών for the source's ὤ μοι ἐγών) |  | First line of the same couplet completed at the next row (Il. 16.433–434); Perseus's combined citation is tagged only there. |
| 388d | LITERARY | Homer, *Iliad* 16.434 | yes — Iliad 16.434 (on-disk, exact) | Hom. Il. 16.433-434 |  |
| 389a | LITERARY | Homer, *Iliad* 1.599–600 | yes — Iliad 1.599-600 (on-disk, exact) | Hom. Il. 1.599-600 |  |
| 389d | LITERARY | Homer, *Odyssey* 17.383–384 | yes — Odyssey 17.383-384 (on-disk, exact) | Hom. Od. 17.383-384 |  |
| 389e | LITERARY | Homer, *Iliad* 4.412 | yes — Iliad 4.412 (on-disk, exact) | Hom. Il. 4.412 |  |
| 389e | LITERARY | Homer, *Iliad* 3.8 | yes — Iliad 3.8 (on-disk, close paraphrase; word order οἳ δʼ ἄρʼ ἴσαν σιγῇ vs Plato's ἴσαν μένεα πνείοντες Ἀχαιοί, σιγῇ) | Hom. Il. 3.8 |  |
| 389e | LITERARY | Homer, *Iliad* 1.225 | yes — Iliad 1.225 (on-disk, exact) | Hom. Il. 1.225 |  |
| 390a | LITERARY | Homer, *Odyssey* 9.8 | yes — thematic/contextual match confirmed (banquet scene, immediately preceding the already-cited Od. 9.9-10); wording is a variant of the standard text (πλεῖαι ὦσι for πλήθωσι) |  | New identification Perseus missed entirely (no bibl given). Precedes the Od. 9.9–10 quotation at the next rows; together they quote Odyssey 9.8–10. |
| 390b | LITERARY | Homer, *Odyssey* 9.9–10 | yes — Odyssey 9.9-10 (on-disk, exact) | Hom. Od. 9.8-10 |  |
| 390b | LITERARY | Homer, *Odyssey* 12.342 | yes — Odyssey 12.342 (on-disk, exact) | Hom. Od. 12.342 |  |
| 390c | LITERARY | Homer, *Iliad* 14.296 | yes — Iliad 14.296 (on-disk, exact) | Hom. Il. 14.296 |  |
| 390d | LITERARY | Homer, *Odyssey* 20.17–18 | yes — Odyssey 20.17-18 (on-disk, exact) | Hom. Od. 20.17-18 |  |
| 390e | UNIDENTIFIED |  |  | unknown | Perseus itself flags this "unknown." An old gnomic hexameter proverb ("gifts persuade the gods, gifts persuade revered kings"), also quoted independently by Clement of Alexandria; no ancient source assigns it to a named author, and no verbatim match was found in any on-disk text. |
| 391a | LITERARY | Homer, *Iliad* 22.15 | yes — Iliad 22.15 (on-disk, exact opening) | Hom. Il. 22.15 |  |
| 391b | LITERARY | Homer, *Iliad* 23.151 | yes — Iliad 23.151 (on-disk, first half of the line) |  | Forms one continuous line with the next row (Il. 23.151); Perseus's citation is tagged only there. |
| 391b | LITERARY | Homer, *Iliad* 23.151 | yes — Iliad 23.151 (on-disk, exact) | Hom. Il. 23.151 |  |
| 391e | LITERARY | Aeschylus, *Niobe*, fr. 154a.15 Radt | no — fragment not in on-disk corpus (papyrus-supplemented; not part of this canonical-greekLit corpus) | Aesch. Niobe Fr. | Confirmed via scholarship: the two lines Plato quotes at 391e (this row and the next) are fr. 154a lines 15–16 Radt. A 20th-century papyrus find later showed the lines' original dramatic context to be more nuanced than Plato's tendentious use of them here — a scholarly footnote worth preserving. |
| 391e | LITERARY | Aeschylus, *Niobe*, fr. 154a.16 Radt | no — fragment not in on-disk corpus | Aesch. Niobe | Continuation of the previous row's fragment. |
| 393a | LITERARY | Homer, *Iliad* 1.15–16 | yes — Iliad 1.15-16 (on-disk, close paraphrase) | Hom. Il. 1.15 |  |
| 408a | LITERARY | Homer, *Iliad* 4.218 | yes — Iliad 4.218 (on-disk, close paraphrase) | Hom. Il. 4.218 |  |
| 411b | DIALOGUE |  |  |  |  |
| 420e | DIALOGUE |  |  |  |  |
| 421a | DIALOGUE |  |  |  |  |
| 422d | DIALOGUE |  |  |  |  |
| 424b | LITERARY | Homer, *Odyssey* 1.351–352 | yes — same passage confirmed (Odyssey 1.351-352), but with two word-level variants from the standard modern text (ἐπιφρονέουσʼ/ἐπικλείουσʼ and ἀειδόντεσσι/ἀκουόντεσσι) | Hom. Od. 1.351 | A known crux: Plato's quoted wording differs from the standard (e.g. OCT/Loeb) Odyssey text at two words, more than ordinary loose quotation — likely reflects a genuinely divergent manuscript tradition available to Plato, not a transcription error in this report. |
| 430e | DIALOGUE |  |  |  | Idiomatic phrase ("stronger than oneself") that Socrates examines philosophically as ordinary usage, not attributed to any poet. |
| 431a | DIALOGUE |  |  |  | Same idiom as the previous row, continued. |
| 440a | DIALOGUE |  |  |  |  |
| 440a | DIALOGUE |  |  |  |  |
| 441b | LITERARY | Homer, *Odyssey* 20.17 | yes — Odyssey 20.17 (on-disk, exact; repeats the line already quoted at 390d) | Hom. Od. 20.17 |  |
| 453b | DIALOGUE |  |  |  |  |
| 453b | DIALOGUE |  |  |  |  |
| 453b | DIALOGUE |  |  |  |  |
| 453c | DIALOGUE |  |  |  |  |
| 453c | DIALOGUE |  |  |  |  |
| 457b | DIALOGUE |  |  |  | Plato's own metaphor ("an incomplete fruit of the ridiculous"), not attributed to any poet. |
| 457b | DIALOGUE |  |  |  | Same sentence as the previous row, continued. |
| 466c | LITERARY | Hesiod, *Works and Days* 40 | yes — WD 40 (on-disk, exact: ὅσῳ πλέον ἥμισυ παντός) |  | Explicitly names Hesiod in Plato's own sentence ("γνώσεται τὸν Ἡσίοδον ὅτι τῷ ὄντι ἦν σοφὸς λέγων..."); same citation as the next row, continued there. |
| 466c | LITERARY | Hesiod, *Works and Days* 40 | yes — WD 40 (on-disk, exact) | Hes. WD 40 |  |
| 468d | LITERARY | Homer, *Iliad* 7.321 | yes — Iliad 7.321 (on-disk, exact word); same citation as the next row |  | Embedded in Plato's own indirect-discourse sentence naming Homer ("καὶ γὰρ Ὅμηρος τὸν εὐδοκιμήσαντα... νώτοισιν... ἔφη διηνεκέεσσι γεραίρεσθαι"); same reference as the next row. |
| 468d | LITERARY | Homer, *Iliad* 7.321–322 | yes — Iliad 7.321-322 (on-disk, exact) | Hom. Il. 7.321-322 |  |
| 468d | LITERARY | Homer, *Iliad* 8.162 | yes — Iliad 8.162 (on-disk, exact: ἕδρῃ τε κρέασίν τε for Plato's ἕδραις τε καὶ κρέασιν) |  | Same citation as the next row, continued (the two halves of Iliad 8.162 together). |
| 468e | LITERARY | Homer, *Iliad* 8.162 | yes — Iliad 8.162 (on-disk, exact) | Hom. Il. 8.162 |  |
| 469a | LITERARY | Hesiod, *Works and Days* 122 | yes — WD 122 (on-disk; τελέθουσιν for source's καλέονται) | Hes. WD 121 | Perseus's line number (121) is one line early: WD 121 is the line just before the quotation begins (τοῦτο γένος κατὰ γαῖʼ ἐκάλυψε—); the quoted words start at 122. May reflect a differing line-numbering convention between editions rather than a simple error. |
| 479a | DIALOGUE |  |  |  |  |
| 507b | DIALOGUE |  |  |  |  |
| 516d | LITERARY | Homer, *Odyssey* 11.489 | yes — Odyssey 11.489-490 (on-disk, close paraphrase) | Hom. Od. 11.489 |  |
| 526a | DIALOGUE |  |  |  |  |
| 545d | DIALOGUE |  |  |  | Part of Socrates's own mock-Homeric invocation (see the next two rows); not itself a quotation. |
| 545e | DIALOGUE |  |  |  | Same mock-invocation, continued. |
| 545e | DIALOGUE |  |  | Hom. Il. 1.6 | Perseus's citation ("Hom. Il. 1.6") is WRONG — *Iliad* 1.6 reads ἐξ οὗ δὴ τὰ πρῶτα διαστήτην ἐρίσαντε and contains no form of ἔμπεσε. This is not a Homeric quotation at all: Socrates says explicitly "ὥσπερ Ὅμηρος, εὐχώμεθα ταῖς Μούσαις" ("let us, like Homer, pray to the Muses...") — he is composing his OWN mock-epic invocation in Homer's style (echoing the Iliad's proem type-scene generally), not quoting a specific line. |
| 547a | LITERARY | Homer, *Iliad* 6.211 | yes — Iliad 6.211 (on-disk, exact opening) | Hom. Il. 6.211 |  |
| 550c | LITERARY | Aeschylus, *Seven Against Thebes* 451 (adapted) | yes — located at Seven 451, but the wording differs from Plato's quotation | Aesch. Seven 451 | Plato explicitly says "τὸ τοῦ Αἰσχύλου" ("the phrase of Aeschylus"), but deliberately adapts the source's ἄλλον ἄλλαις ἐν πύλαις εἰληχότα ("posted at different gates") to ἄλλον ἄλλῃ πρὸς πόλει τεταγμένον ("posted at a different city"), substituting πόλει for πύλαις to fit his own argument about examining one city/type at a time. Not a Perseus error — a deliberate Platonic adaptation of the source line, confirmed correct by Perseus. |
| 556e | DIALOGUE |  |  |  |  |
| 563c | DIALOGUE |  |  |  |  |
| 566c | LITERARY | Herodotus, *Histories* 1.55 (the Delphic oracle to Croesus) | yes — Hdt. 1.55 (on-disk, exact sense; Plato recasts the oracle's infinitives φεύγειν/μένειν/αἰδεῖσθαι into indicatives φεύγει/μένει/αἰδεῖται to fit his own sentence) | Hdt. 1.55 |  |
| 566c | LITERARY | Homer, *Iliad* 16.776 | yes — Iliad 16.776 (on-disk, exact word); same citation as the next row |  | Part of the same two-word ironic echo as the next row (μέγας μεγαλωστί, negated by Plato as οὐ κεῖται — "he does NOT lie 'great and mightily [dead]'", inverting the Homeric line about Cebriones' corpse). |
| 566d | LITERARY | Homer, *Iliad* 16.776 | yes — Iliad 16.776 (on-disk, exact) | Hom. Il. 16.776 |  |
| 568b | DIALOGUE |  |  |  |  |
| 568b | DIALOGUE |  |  |  |  |
| 607b | UNIDENTIFIED |  |  |  | Perseus gave no identification. Widely recognized in the scholarly literature as untraceable — one of a set of anonymous jabs at philosophers Plato quotes at 607b–c as evidence of "the ancient quarrel between philosophy and poetry," almost certainly drawn from a lost comedy. No fragment number is agreed in modern scholarship; treated as comica adespota. Do not invent an attribution. |
| 607b | UNIDENTIFIED |  |  | Unknown | Same passage as the previous row ("barking, snapping" — completes the image of the dog barking at her master). Perseus itself says "Unknown." |
| 607b | UNIDENTIFIED |  |  |  | Same set of anonymous anti-philosophy tags as the two previous rows ("great among the empty talk of fools"); untraceable to any known author or play. |
| 607c | UNIDENTIFIED |  |  |  | Same set ("the mob of the over-clever, prevailing"); untraceable. |
| 607c | UNIDENTIFIED |  |  |  | Same set ("those who worry subtly..."); untraceable. |
| 607c | UNIDENTIFIED |  |  |  | Completes the previous row's clause ("...that they are poor"); untraceable. Together, rows 119–124 form one continuous run of anonymous quotations at 607b–c; all six should be treated as a single editorial note rather than six separate footnotes. |
| 615d | DIALOGUE |  |  |  |  |
| 615e | DIALOGUE |  |  |  |  |
| 616a | DIALOGUE |  |  |  |  |
| 617e | DIALOGUE |  |  |  | Speech of the prophet/Lachesis within the Myth of Er — Socrates's own invented mythic narration, not a quotation of another author. |
| 619b | DIALOGUE |  |  |  | Same myth, continued. |
