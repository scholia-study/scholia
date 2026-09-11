# plato2 terminology standard

Governed renderings for the English edition. A term listed here renders one way
across the whole dialogue; `scripts/plato2_terminology_gate.py` enforces that
mechanically, and this file is the only place the decision lives.

Where a term is shared with the *Republic*, plato2 follows
`assets/plato1/TERMINOLOGY.md`. A reader moving between the two dialogues on the
same shelf should not find justice renamed.

`stem` is a REGEX matched against accent-folded Greek, so it catches inflected
forms, and a Greek stem catches the whole word FAMILY. That is usually what you
want but sometimes is not, so several stems here are deliberately narrow:

- `καλ` would match **Καλλικλῆς** — one of the five speakers — and also καλέω
  "to call". Both would fire on every page.
- `νομ` would match νομίζω "to think", whose English is not "law".
- `ιατρ` covers both ἰατρική the craft and ἰατρός the man, which are
  "medicine" and "doctor".
- `δοξα` would match δοξάζω "to think", whose English is not "opinion".

Enforcement is kept conservative on purpose. A gate that misfires gets switched
off, and a narrow stem that catches the noun is worth more than a wide one that
has to be argued with.

`root` is the English substring that must then appear — the SHORTEST root all
legitimate renderings share, not the headword: `flatter`, which covers
"flattery", "flattering" and "flatterer" alike.

`conversations` scopes a term to `gorgias`, `polus`, `callicles` (the three
depth-0 nodes) or `*` for the whole dialogue.

**`-` in the root column means ADVISORY: documented here but NOT machine-enforced.**

| Greek | stem | English | root | conversations | note |
|---|---|---|---|---|---|
| ῥητορική | ρητορικ(η\|ην\|ησ\|ῃ\|α) | rhetoric | rhetoric | * | The dialogue's subject. Feminine forms only — ἡ ῥητορική is the craft. |
| ῥήτωρ | ρητωρ\|ρητορ(?!ικ)\|ρητορικ(ο\|ου\|οι\|ον\|ουσ\|οισ) | orator | orator | * | The practitioner, never 'rhetorician' — keeps him distinct from the craft. Includes ῥητορικός/ῥητορικόν **applied to a person**: 'make someone ῥητορικόν' is making him an orator, not making him rhetoric. The adverb ῥητορικῶς is left ungoverned. |
| τέχνη | τεχν(η\|α\|ῃ\|ω) | craft | craft | * | As plato1. Never 'art' — imports the aesthetic sense. |
| ἐμπειρία | εμπειρια | knack | knack | * | What rhetoric IS, against τέχνη (465a). The contrast is the dialogue's hinge, so the two words must not blur. |
| κολακεία | κολακ | flattery | flatter | * | The genus of which rhetoric is a branch (463b). |
| ὀψοποιική | οψοποι | cookery | cook | * | Flattery's counterpart to medicine in the fourfold analogy (465c). |
| κομμωτική | κομμωτ | cosmetics | cosmetic | * | Flattery's counterpart to physical training. |
| σοφιστική | σοφιστ | sophistry | sophist | * | Flattery's counterpart to legislation. |
| ἰατρική | ιατρικ | medicine | medicine | * | The craft. |
| ἰατρός | ιατρ(ο\|ω) | doctor | doctor | * | The man. Distinct from the craft above. |
| γυμναστική | γυμναστ | physical training | training | * | Never 'gymnastics' — a modern sport. |
| δικαιοσύνη | δικαιοσυν | justice | justice | * | As plato1. Never 'righteousness' — moralises it. |
| ψυχή | ψυχ | soul | soul | * | As plato1. |
| σωφροσύνη | σωφρο(συν\|ν(?!ιζ)) | moderation | moderat | * | Never 'temperance' — a Victorian abstinence word. Governs the noun and the adjective σώφρων, but NOT the verb σωφρονίζω 'chasten, bring to one's senses', whose English is not 'moderate'. A wider stem forced a stilted rendering at 478d before this was narrowed. |
| ἀρετή | - | virtue | - | * | ADVISORY — see below. |
| ἐπιστήμη | επιστημ | knowledge | know | * | As plato1. |
| δόξα | - | opinion | - | * | As plato1, but ADVISORY here — see below. |
| ἡδονή | ηδον | pleasure | pleas | * | Callicles' good. |
| ἀγαθόν | αγαθ | good | good | * | Distinguished from ἡδονή at 495a — the two must never trade words. |
| αἰσχρόν | αισχρ | shameful | shame | * | Half of the pair the Polus refutation turns on (474c). |
| κόσμος | κοσμ | order | order | * | The soul's ordering (504b) and the world's (508a). NOTE the myth uses the word's other classical sense at 523e — the worldly **finery** a soul is stripped of before judgement, not its order. If that sense recurs, move this entry to advisory rather than forcing 'order' onto it. |
| τάξις | - | arrangement | - | * | ADVISORY — see below. Kept distinct from κόσμος: two governed terms may not share one English word. |
| φύσις | φυσ | nature | natur | * | Half of Callicles' νόμος/φύσις antithesis. |
| εὐδαιμονία | ευδαιμ | happiness | happ | * | Never 'flourishing' — an Aristotelian term of art Plato is not using here. |
| κόλασις | κολα(ζ\|σ) | correction | correct | * | Punishment as medicine for the soul (478d). |
| τιμωρία | τιμωρ | punishment | punish | * | Kept distinct from κόλασις. |
| παρρησία | παρρησ | frankness | frank | * | Callicles' claimed virtue as an interlocutor (487a). |
| πόλις | πολι(σ\|ν) | city | cit | * | As plato1. Never 'state'. Narrow stem: the πολιτικ- family is advisory. |
| δύναμις | δυναμι(σ\|ν\|ω) | power | power | * | The Polus conversation's subject. Narrow stem: δύναμαι 'I am able' is not this word. |
| καλόν | - | fine | - | * | See below — not stem-enforceable. |
| νόμος | - | law / convention | - | * | See below — the sense shifts. |
| ἀδικεῖν | - | do wrong | - | * | See below. |
| πλεονεξία | - | getting more | - | callicles | Callicles' claim for the superior man (483c). No stable English root. |
| πειθώ | - | persuasion | - | * | The noun is 'persuasion', the verb 'persuade'. Not stem-enforceable: πείσομαι is the future of both πείθομαι 'I shall obey' and πάσχω 'I shall suffer', and this dialogue is full of suffering. |

## The advisory entries, and why they cannot be rules

**καλόν = "fine"** (with αἰσχρόν = "shameful"). The refutation at 474c–475e
defines καλόν as pleasant-or-beneficial and runs the definition over bodies,
colours, shapes, sounds and practices alike, so the English word has to survive
"a fine body" and "a fine law" equally — which is why "noble" was rejected. It
is advisory rather than enforced for two reasons: no English root is safe
("fin" catches "find" and "finally"), and the Greek stem `καλ` collides with
both Καλλικλῆς and καλέω "to call".

**νόμος** is "law" everywhere except inside Callicles' antithesis, where νόμος
against φύσις is **convention** against nature (482e–484c). Enforcing either
word would mistranslate the other half of the dialogue. Use "law" by default and
"convention" where the antithesis is live, and never let the same sentence carry
both senses without saying so.

**δόξα** is "opinion", but it cannot be stem-governed in this dialogue. Of the
six sentences whose Greek carries a `δοξα` form, four are inflections of
**δοκέω** "to seem" — δόξαν as an aorist participle, δόξαι as an infinitive —
whose English is "seems best", not "opinion". No regex separates them, because
the two verbs genuinely share the stem.

**ἀρετή** is "virtue" of a person and **excellence** of a body or a tool, and
the *Gorgias* uses both: ἡ ἀρετὴ σώματος at 464a is the body's excellence, not
its virtue. "Virtue" and "excellence" share no English root, so no rule can
cover the pair. Use "virtue" for the human sense and "excellence" elsewhere.

**τάξις** is "arrangement" in the argument about the soul's ordering, but at
455b it is the military drawing-up of troops, whose English is "formation" or
"deployment". With two occurrences in the whole dialogue, a rule buys nothing
that the note does not.

**ἀδικεῖν** is the verb the whole Polus argument runs on: τὸ ἀδικεῖν is worse
than τὸ ἀδικεῖσθαι. "Do wrong" and "suffer wrong" are the natural English pair
and the division titles use them. But δικαιοσύνη is enforced as "justice", and
"do wrong" shares no root with it, so a rule would fail on one or the other.
Both "do wrong" and "act unjustly" are permitted; what is NOT permitted is
letting the noun drift off "justice".

## Register

Plato's Greek here is conversational and combative — the *Gorgias* has more
open hostility in it than any other dialogue. Callicles is rude. Socrates is
needling. Render that as speech people actually make, and avoid the Victorian
register that makes Plato sound like scripture: it is both bad English and the
single strongest attractor toward Jowett's phrasing.
