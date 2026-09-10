# plato1 golden values

Regression baseline for `scripts/plato1_verify.sh`. These are facts about the
curated Greek as it stands; a change here should be a deliberate edit with a
reason, never a surprise.

The distribution matters more than any single number. If the Greek splitter ever
regressed to a Latin-script one it would find roughly 2% of Greek sentence
boundaries — silently, with no error — and the median would jump from single
digits to paragraph length. That is what the median band catches.

| value | expected |
|---|---:|
| toc nodes | 116 |
| Stephanus markers | 1355 |
| total sentences | 5667 |
| median words/sentence | 9 |
| max words/sentence | 230 |

| book | sentences |
|---|---:|
| Book I | 715 |
| Book II | 525 |
| Book III | 622 |
| Book IV | 575 |
| Book V | 727 |
| Book VI | 520 |
| Book VII | 496 |
| Book VIII | 532 |
| Book IX | 485 |
| Book X | 470 |

Book X carries two divisions fewer than the original design. `The Spindle of
Necessity` held no text at all and `The River of Forgetfulness` held a single
short paragraph; both were collapsed into their neighbours, since a division
that opens onto a blank or near-blank page is not a division. Each removal
takes its heading sentence with it, hence 5667 rather than 5669.

Two further sentences come off because a sentence must contain a letter.
At 562a and 573b Burnet suspends a construction across the interlocutor's
reply by leaving a bare dash after the question mark; the Greek splitter used
to break there, yielding a letterless "sentence" that would have reached the
reader as an empty numbered, individually quotable unit. One per book, in
Books VIII and IX.

A textual correction accounts for one sentence. At 521e the Perseus TEI we
ingested prints a full stop between φθίσεως and ἐπιστατεῖ, leaving ἐπιστατεῖ
as a one-word sentence stripped of the genitives it governs. The error is
upstream, in the Perseus file itself, not in our converter; the curated Greek
now reads `σώματος γὰρ αὔξης καὶ φθίσεως ἐπιστατεῖ.` as one sentence, and the
counts above reflect that.

The longest sentence is the Ship of State (488a–b), which Burnet prints as a
single period; the 169-word Great Beast passage (493a–c) is the next. Both are
genuinely one Greek sentence, so the max is a real property of the text rather
than a splitting failure.
