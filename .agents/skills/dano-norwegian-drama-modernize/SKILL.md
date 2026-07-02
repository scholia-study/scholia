---
name: dano-norwegian-drama-modernize
description: Modernize 19th-century Dano-Norwegian plays (Ibsen-era) from a faithful md_reviewed layer into a modern Norwegian Bokmål md_modernized reading text. Holds the full Dano-Norwegian → Bokmål rule-set + a running record of lexical fixes. Use when modernizing any such play.
---

You are a careful editor digitizing classical texts. For an Ibsen-era
Dano-Norwegian play, you take the faithful original (the `md_reviewed` layer) and
produce a modern Norwegian **Bokmål** reading text (the `md_modernized` layer).

The work is **modernize the words only** — keep the playwright's syntax, word
order, and every structural marker. Depth is *fuller/lexical* (not conservative):
modern spelling + morphology, pronoun updates, AND archaic vocabulary swapped for
contemporary — while the two layers stay structurally identical.

The rule-set and lexical record below are **language-general** (Dano-Norwegian →
modern Bokmål) and apply to any such play, not just the current corpus.

## Corpora

- **ibsen1** (active) — Ibsen, *Kejser og Galilæer* (1873). 14 files = 2 parts
  (`cf` = Cæsars Frafald, `kj` = Kejser Julian): `001_cf_titelblad`,
  `002_cf_de_optraedende`, `003–007_cf_*_handling`, `008_kj_titelblad`,
  `009_kj_de_optraedende`, `010–014_kj_*_handling`.
- Future Ibsen-era plays slot in the same way: a new `assets/<corpus>/` tree.

## Folder layout (per corpus)

- `assets/<corpus>/curated/md_reviewed/` — faithful original. **Source. Do not edit.**
- `assets/<corpus>/curated/md_modernized/` — modern Bokmål. **The target you produce.**

## Workflow (one act/unit per turn)

1. Read the reviewed file. Produce the modernized file (write in chunks for long acts).
2. **Parity is mandatory** — the two layers MUST be structurally identical so the
   future importer can pair them: same frontmatter, same block sequence + types,
   same sentence/verse-line boundaries, same `{{{ N }}}` page-marker sequence.
3. Run the **Verification** block below. Fix any skeleton / marker / residual hits.
4. **The USER owns review + fixes of finished `md_modernized` files.** Do NOT
   overwrite or regenerate a finished file unless expressly told — only produce
   the next un-done file. The user reviews each act and may hand you new fixes.
5. **When the user gives a new correction, append it to "Lexical change record".**
6. Flag kept literary words for the reviewer. When in doubt, stop and ask.
   Accuracy is paramount.

## Drama markup — preserve exactly

`## HEAD` (act/part heading) · `@ NAME *(opener)*SEP` (speech; SEP = literal
`.`/`:`/none) · flush lines under a speech = prose paragraphs · `| line` = verse ·
`@stage (…)` scene-level stage direction · `*(…)*` on its own line = speaker-owned ·
`*(…)*` inline / `*word*` = emphasis · `{{{ N }}}` = printed-page marker.

## Modernization rules (Dano-Norwegian → Bokmål)

**Orthography.**
- Soft → hard consonant: kejser → keiser, skib → skip, gade → gate, bog → bok, sag → sak,
  råbe → rope, løbe → løpe, byde → by, røbe → røpe.
- æ → e: træ → tre, læse → lese, færd → ferd. ej/aj → ei: vej → vei, nej → nei, sejle → seile.
  øj → øy: øje → øye, høj → høy, fløjte → fløyte.
- Double consonant for short vowel: slot → slott, vand → vann, nat → natt, op → opp, ind → inn.
- ld/nd → ll/nn: sind → sinn, mund → munn, anden → annen (hånd, holdt stay).
- Past -ede → -et: kastede → kastet. aa → å (usually already done in source).

**Pronouns / possessives.** mig/dig/sig → meg/deg/seg · I/eder/eders → dere/dere/deres ·
vor/vort → vår/vårt · hendes → hennes (hans/min/din stay).

**Function words.** thi → for · nu → nå · mere → mer · kun/blot → bare · end → enn ·
hvad → hva · hvorledes → hvordan · således/sådant → slik/slikt · iblandt/blandt → iblant/blant ·
bag/bagefter → bak/bakefter · endnu → ennå · idag/inat → i dag/i natt · at(infinitive) → å ·
måske/kanske → **kanskje** · lig → lik **everywhere** (likesom, likefrem, likeledes,
likefullt, like) · meget(quantity) → mye **but** meget(=very, intensifier) **KEPT**.
- **`der`**: relative (who/which) → `som`; existential ("der er/går/gives…") → `det`;
  locative ("there") → keep `der`.
- **Neuter numeral "one" = `ett`** when it means *a single* (counting): "ett skritts
  fremgang", "ett ord". The indefinite article stays `et`.

**Vocab swaps.** betler → tigger · vorde → bli · Grækenland → Hellas (but Græker → Greker,
gresk) · sten → stein · udørk → ødemark.

**Titles / roles.** Quæstoren → Kvestoren · Staldmesteren → Stallmesteren · Lægen → Legen ·
Prætorian → Pretorian · Fyrstinde → Fyrstinne · Slavinden → Slavinnen.
(Husmesteren, Hærføreren, Ridderen, Underføreren, Fanebæreren, and Latin proper
names like Cæsar/Cæsaræa stay.)

**Kept literary words** (don't over-modernize; flag for the reviewer):
hin/hint/hine, stundom, visselig, sikkerlig, frende, hvo, endog, fattedes,
**isinde** (NOT "i sinne" — false friend = anger).

**Typos / OCR sics are FIXED in md_modernized** (e.g. `JuIian` → `Julian`). The
sic-preservation rule applies only to `md_reviewed`.

**`{{{ N }}}` markers never split a word.** The reviewed layer breaks a word across
a printed page with a soft-hyphen + marker (`Sønder‐ {{{ 189 }}} knust`,
`for‐ {{{ 107 }}} sikring`). In md_modernized **rejoin the word and place the marker
immediately BEFORE the whole word** (`{{{ 189 }}} Sønderknust`). The marker *value
sequence* is unchanged, so parity still holds.

## Lexical change record (append new user fixes here)

- køter → kjøter
- affældig → avfeldig
- tilhobe → til hope
- Athen → Aten
- forliges → forlikes
- løvekulen → løvehulen (but "løvens hule" genitive stays)
- nys → nylig/nettopp/just (context — NOT kept literary)
- hård → hard
- såsom → så som
- venskab → vennskap
- deslige → desslike
- mindes → minnes
- traf → traff
- tilgiv → tilgi
- tillad → tillat
- afsind → avsinn
- spørg → spør
- tilfulde → til fulle
- sønlige → sønnlige
- **sejr(noun) → seier** (seiren → seieren, seirens → seierens; verb `seiret` and `seier-` compounds like seierherre/seiersinntog/seiervinninger/seierstog stay)
- uheld → uhell
- visdomsven → visdomsvenn
- ligefullt → likefullt
- "et skritts" → "ett skritts"
- måske/kanske → kanskje
- røbed → røpet (røbe → røpe)
- udørk → ødemark
- tilorde → til orde
- himlen → himmelen
- imøte → i møte
- iblinde → i blinde
- slog → slo
- forjob → fordrev
- tilfals → til fals
- opædt → oppspist
- jertegn → jærtegn
- tag → tak
- tak → takk
- Mark Aurel → Markus Aurelius
- end ikke → selv ikke
- utaknemlig → utakknemlig
- hadefuld → hatefull
- spurgt → spurt
- sandelig → sannelig
- forbillede → forbilde
- ven → venn
- stadse → stase
- løn → lønn
- mindst → minst
- trods → tross
- sandt → sant
- uduelige → udugelige
- hade → hate
- streg → strek
- tilhånde → til hånde
- hverv → verv
- kerne → kjerne 
- delagtig → delaktig
- besked → beskjed
- hede → hete
- palatset → palasset
- straf → straff
- talrik → tallrik
- betroede → betrodde
- hensigter → hensikter
- sigte → sikte (sigter → sikter)
- vidne → vitne (vidnesbyrd → vitnesbyrd)
- bever → skjelver
- beven → skjelvelse
- skrig → skrik
- pønse → pønske (pønser → pønsker)
- yndest → gunst
- vild → vill (vildledende → villedende, vildfarelse → villfarelse)
- forskel → forskjell (forskellen → forskjellen, forskellige → forskjellige)
- fortabt → fortapt

## Verification (run after each act)

```bash
r=assets/<corpus>/curated/md_reviewed/NNN.md
m=assets/<corpus>/curated/md_modernized/NNN.md
wc -l "$r" "$m"                                   # line counts must match
diff <(grep -oE '\{\{\{ [0-9]+ \}\}\}' "$r") \
     <(grep -oE '\{\{\{ [0-9]+ \}\}\}' "$m")       # page-marker sequence identical
skel() { sed -E 's/^(@stage|@ |## |\| ).*/\1/; t; s/^\*\(.*/*(/; t; s/^---$/---/; t; s/^$/BLANK/; t; s/^.*$/TEXT/'; }
diff <(skel < "$r") <(skel < "$m")                # block skeleton identical
# residual Danish sweep (expect 0; meget>0 ok only if intensifier "very"):
grep -coiE '\bmig\b|\bdig\b|\bsig\b|kejser|\bskib|\bhvad\b|\bnej\b|\baf\b|\beder\b|\bnu\b|gerning|\bgøre\b|\bnys\b|\bhård|\bseir\b|\bsejr|Athen|Grækenland|måske|udørk' "$m"
```

A clean run = identical line count, empty marker diff, empty skeleton diff, and 0
residual hits (intensifier `meget` excepted). Then hand the act back to the user
for review.
