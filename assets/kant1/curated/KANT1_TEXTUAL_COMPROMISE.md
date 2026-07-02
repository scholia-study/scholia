# Kant1 Curated Text — Textual Compromises

A register of places where the curated text in `md_reviewed` / `md_modernized` /
`md_modernized_translated` **deliberately departs from a faithful reading** of the
1911 reprint in order to satisfy a pipeline constraint (sentence splitter, struct
parity, etc.). These are not OCR errors to fix — they are knowing trade-offs.
Each entry records the faithful reading so it can be restored once the underlying
tooling/policy issue is resolved.

> Scope note: this is distinct from the audit reports (`AUDIT_TEXT.md`,
> `AUDIT_TEXT_074_114.md`), which track *corrections*. This file tracks
> *intentional infidelities*.

---

## C1 — File 113, omitted abbreviation period after `ꝛc` (B 871 / AA 544)

| Field | Value |
|---|---|
| **Date** | 2026-06-13 |
| **Files** | `md_reviewed/113_…architektonik….md`, `md_modernized/113_….md` |
| **Layers** | REV + MOD (TRA unaffected — English expands ꝛc. to "and so on") |
| **Faithful reading** | `… das zehnte ꝛc. Jahrhundert auch zu den ersten?` (REV) / `… das zehnte etc. Jahrhundert …` (MOD) — abbreviation period **present**, as in the print |
| **Curated reading** | `… das zehnte ꝛc Jahrhundert …` (REV) / `… das zehnte etc Jahrhundert …` (MOD) — period **omitted** |
| **Sentence** | "Gehört das fünfte, das zehnte ꝛc. Jahrhundert auch zu den ersten?" ("Do the fifth, the tenth etc. century also belong among the first ones?") |

### Why

The sentence splitter (`packages/common/src/sentences.rs`) breaks a sentence on
**`. ` + capital letter**. German capitalizes every noun, so `ꝛc. Jahrhundert`
is indistinguishable from a real sentence boundary — but here ꝛc. (= *etc.*) sits
**mid-sentence**, with "Jahrhundert" continuing the same clause. Left with the
period, the German layers split into one more sentence than the English
(`… and so on …`, which never splits), so `kant1_md_translation_to_struct` panics
on a MOD↔TRA sentence-count mismatch in this block (German 17, English 16).

ꝛc./etc. genuinely **does** end sentences elsewhere and must keep splitting there
(REV `ꝛc. Obgleich` in 051; MOD `etc. Denn` in 065/104, `etc. Durch` in 078), so
it cannot simply be added to the splitter's no-split abbreviation list. 113 is the
only place in the whole corpus where ꝛc. precedes a capital **mid-sentence**.
Dropping the period at this one spot removes the false trigger; the rest of the
text and all three layers' sentence counts stay aligned, and the struct gates pass.

### Proper fix (deferred)

Restore the period and adopt the project's existing `usw.`-style design corpus-wide
(audit policy **#315**, "canonicalize ꝛc. and sweep"): mark `ꝛc.`/`etc.` as
non-splitting abbreviations in `SINGLE_ABBREVS`, then add the invisible `|||`
forced-split sentinel at every site where they genuinely end a sentence
(REV 051, 078; MOD 051, 065, 078, 104). That change is output-neutral for the
already-imported files but reaches outside the 074–114 audit range, hence the
deferral. The #315 sweep also covers the other ꝛc. renderings still in the text
("sc." one clause later in 112, plus "zc.", "z.", "u. ſ. w.").

### Restore checklist

- [ ] Add `"ꝛc."` and `"etc."` to `SINGLE_ABBREVS` in `packages/common/src/sentences.rs`.
- [ ] Add `|||` after the abbreviation at the sentence-ending sites: REV 051/078, MOD 051/065/078/104.
- [ ] Restore the period: `ꝛc Jahrhundert` → `ꝛc. Jahrhundert` (REV 113), `etc Jahrhundert` → `etc. Jahrhundert` (MOD 113).
- [ ] `just struct kant1` passes (both source and translation modes); no `|||` in the output JSON.

### Other

Line 16 in 095 has an extra comma after soll (just before the page change) for better flow, even though page displays no such comma.
