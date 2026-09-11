# plato2 golden values

Regression baseline for `scripts/plato2_verify.sh`. These are facts about the
curated Greek as it stands; a change here should be a deliberate edit with a
reason, never a surprise.

The distribution matters more than any single number. If the Greek splitter ever
regressed to a Latin-script one it would find roughly 2% of Greek sentence
boundaries — silently, with no error — and the median would jump from single
digits to paragraph length. That is what the median band catches.

| value | expected |
|---|---:|
| toc nodes | 22 |
| Stephanus markers | 404 |
| total sentences | 1725 |
| median words/sentence | 9 |
| max words/sentence | 174 |

Sentences are the quotable ones: `speaker` and `heading` blocks are inert and
excluded, so this counts dialogue and verse only.

| conversation | sentences |
|---|---:|
| The Conversation with Gorgias | 334 |
| The Conversation with Polus | 544 |
| The Conversation with Callicles | 847 |

## On the longest sentence

The 174-word maximum is not a missed split. It is Socrates recapitulating
the whole argument at 507a–c, where Burnet punctuates the imagined replies with
em-dashes inside a single speech (`…κακή ἐστιν· ἦν δὲ αὕτη ἡ ἄφρων τε καὶ
ἀκόλαστος.—πάνυ γε.—…`) rather than as separate turns. The Greek splitter
requires whitespace after a terminator, so `.—` correctly does not split. The
pattern occurs 14 times corpus-wide. The Republic's own maximum is 230 words for
the same reason, so this is the family norm, not an outlier.
