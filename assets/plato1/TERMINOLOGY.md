# plato1 terminology standard

Governed renderings for the English edition. A term listed here renders one
way across all ten books; `scripts/plato1_terminology_gate.py` enforces that
mechanically (RULESET-11), and this file is the only place the decision lives.

**Status: DRAFT — needs editorial sign-off.** These are the terms whose
rendering propagates furthest through the argument, with a first proposal for
each. Change any of them here and the gate follows; the reasoning belongs in
the `plato1-translate` skill.

`stem` is a REGEX matched against accent-folded Greek, so it catches inflected
forms. A Greek stem also catches the whole word FAMILY, which is usually what
you want (`φυλαξ` finds both the noun and the verb "to guard against") but
sometimes is not: `δοξα` would also match δοξάζω "to think", whose correct
English is not "opinion" — hence `δοξα(?!ζ)`, excluding the verb. Note the stems are deliberately narrow where a word family spans senses:
`φυλακ` catches the noun φύλαξ "guardian" in its oblique forms but not the verb
φυλάξαι "to preserve", whose English is not "guard".

`books` scopes a term to a range of books, for words whose sense changes: in
Book I λογιστικός is "the reckoner", a craftsman skilled at calculation, and
only from Book IV is τὸ λογιστικόν the soul's rational part. `*` means all ten.

**`-` means ADVISORY: documented here but NOT machine-enforced.** Some terms
cannot be governed by stem at all. εἶδος and ἰδέα are the clear case: Plato
uses them for the technical Form *and* for an ordinary "kind, class", often in
the same book — at 357c a "third εἶδος of good" is a third KIND, not a third
Form. Enforcing "form" there produced a mistranslation, so these are guidance
for the translator's judgement, not a rule a script can check.

`root` is the English substring that must then appear — the SHORTEST root all
legitimate renderings share, not the headword. A Greek stem catches its whole
word family, verbs included (`φυλαξ` matches both φύλαξ "guardian" and
φυλάξασθαι "to guard against"), so the root must be short enough to cover them:
`guard`, not `guardian`; `just`, which covers "unjust" and "injustice" alike.

| Greek | stem | English | root | books | note |
|---|---|---|---|---|---|
| δικαιοσύνη | δικαιοσυν | justice | just | * | The dialogue's subject. Never 'righteousness' — moralises it. |
| ἀδικία | αδικι | injustice | just | * | Kept morphologically parallel to δικαιοσύνη. |
| εἶδος | ειδοσ | form | form |  - | NOT 'idea'. Reserve 'idea' for nothing; it imports Locke. |
| ἰδέα | ιδεα | form | form |  - | Same rendering as εἶδος; Plato uses them interchangeably here. |
| ψυχή | ψυχη | soul | soul | * | Not 'mind' — the tripartite argument needs the whole person. |
| ἀρετή | αρετη | virtue | virtue | * | Not 'excellence'; 'virtue' carries the ethical weight in Bk I. |
| πόλις | πολισ | city | cit | * | Not 'state' — anachronistic. 'city' throughout. |
| πολιτεία | πολιτει | constitution | constitution | 4-10 | The title is Republic; the noun in text is 'constitution'. The constitution as a technical object from Bk IV; earlier simply civic life. |
| λογιστικόν | λογιστικο | rational | rational | 4-10 | The reasoning part of the soul (Bk IV). |
| θυμοειδές | θυμοειδε | spirited | spirited | 4-10 | The spirited part. Distinct from θυμός below. |
| ἐπιθυμητικόν | επιθυμητικο | appetitive | appetit | 4-10 | The appetitive part. |
| σωφροσύνη | σωφροσυν | moderation | moderation | * | Not 'temperance' — Victorian and narrowing. |
| ἀνδρεία | ανδρεια | courage | courage | 4-10 | The NOUN only. The adjective family keeps an ordinary sense throughout — at 459c ἀνδρειοτέρου of a physician is "bolder", nerve to administer drugs, not the cardinal virtue. |
| σοφία | σοφια | wisdom | wisdom | 4-10 | Named as one of the four virtues from Bk IV; earlier σοφία is ordinary skill or cleverness. |
| φρόνησις | φρονησ(?!αι) | practical wisdom | practical wisdom | * | Kept distinct from σοφία. Excludes φρονῆσαι, the aorist infinitive of the VERB φρονέω — ἡ τοῦ φρονῆσαι ἀρετή at 518e is "the virtue of thought", not the noun. The noun's forms (φρόνησις, φρονήσεως, φρονήσει, φρόνησιν) never begin φρονησαι, so the two separate cleanly. |
| ἐπιστήμη | επιστημ | knowledge | knowledge | 5-10 | Against δόξα in Bk V. Technical against δόξα from Bk V; earlier it is ordinary "knowing how". |
| δόξα | (?<!παρα )δοξα(?!ζ) | opinion | opin | 5-5 | Not 'belief' — the Bk V knowledge/opinion argument turns on the pairing with ἐπιστήμη, and that is the ONLY book where it is enforced. Two reasons it is not enforced beyond it. (1) The aorist stem of δοκέω is δοξ-, so δόξαι, δόξαν and δόξας are each simultaneously a noun form and a verb form ("it would seem"); no regex separates them. (2) The reputation sense persists — πρὸς δόξαν καὶ ἔριν at 499a is "reputation and rivalry", and enforcing "opinion" there produced an actual mistranslation. In Bks II-III δόξα is REPUTATION throughout. Root is `opin`, not `opinion`, so it covers δοξαστόν "opinable". The idiom παρὰ δόξαν ("contrary to expectation") is excluded by lookbehind. |
| τέχνη | τεχνη | craft | craft | * | Not 'art'. The craft analogy runs from Bk I to Bk X. |
| μίμησις | μιμησι | imitation | imitat | 3-10 | Bk III and X. Technical from the diction argument in Bk III. |
| ἀγαθόν | αγαθο | good | good | * | The Good in Bk VI. |
| φύλαξ | φυλακ(?!η) | guardian | guard | * | Excludes φυλακή, the abstract noun "safekeeping, preservation" (φρόνησίς τε καὶ φυλακὴ at 433d), which is not the guardian class. The personal noun's oblique forms (φύλακος, φύλακες, φύλακας, φυλακικοῦ) never begin φυλακη, so the two separate cleanly. Root is `guard`, not `guardian`, to cover "guard" and "guardianship". |
| παιδεία | παιδει | education | educat | * |  |
| γένεσις | γενεσ[ιε] | becoming | becoming | 6-6 | Against being. Enforced in Bk VI ONLY, where the sun and the divided line establish the being/becoming contrast. Beyond it the word reverts to ordinary senses no root can cover alongside "becoming": at 533b (Bk VII) πρὸς γενέσεις τε καὶ συνθέσεις is what crafts PRODUCE, and at 546d (Bk VIII) γενέσεων is BIRTHS, of children. The stem catches both γενεσι- and γενεσε- forms — plain `γενεσι` silently missed every oblique form — while excluding γενέσθαι, the aorist infinitive of the far commoner verb γίγνομαι. |
| ἔρως | ερωσ(?!ι) | erotic love | erotic | 5-10 | Distinguished from φιλία. Ordinary "desire/love" earlier; the tyrant's eros is Bk IX. The NOUN only: `ερωσ` also matched the contract verb ἐρῶσιν "they love", whose English shares no substring with "erotic". |
