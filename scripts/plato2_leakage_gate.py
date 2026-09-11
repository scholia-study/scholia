#!/usr/bin/env python3
"""Per-sentence leakage gate for the plato2 English translation.

Scores every candidate sentence against three independent control
translations and fails on any sentence that shares too long a run of words
with one of them.

The rule is deliberately per-sentence and absolute: a file-level average hides
exactly the failure kant3 shipped, where a handful of copied sentences drowned
in a page of clean ones. One sentence over the line fails the run.

Controls live in assets/plato2/control/ and are never shipped. The three are
keyed three different ways (see assets/plato2/control/README.md):

  - lamb_loeb_1925 — exact Stephanus section ("447a"), 404 keys.
  - horan_2026     — a Stephanus page RANGE ("447a-447d"), 64 keys tiling
                     447a-527e with no gap and no overlap.
  - jowett_1871    — the whole dialogue, one key ("full").

A candidate sentence is scored against the control text for its OWN location
where a control's keying allows it — a phrase matching in the right place is
signal, the same phrase somewhere in 39k words is mostly noise. Each
sentence's Stephanus section is read off the `{{{ }}}` markers in its own
file: a sentence belongs to the most recent marker at or before it.

Thresholds come from measurement, not intuition. Scoring the four plato1
controls against each other — independent translations of one Greek text —
gave the natural convergence floor:

    median 3-4 words | p95 6-8 | p99 9-12 | prose tail ~19

Quoted VERSE converges far harder: metre and the Greek constrain the
rendering, and runs of 24-30 identical words occur between controls that
certainly did not copy each other. Verse is therefore scored on its own,
higher threshold rather than excluded, so leakage there is still visible
without the gate crying wolf on every poet quoted (Pindar, Euripides — 10
lines corpus-wide, `| `-prefixed rather than plato1's `+ `). plato2's own
material has not been measured yet (`--calibrate` does that); until then this
gate keeps plato1's measured numbers rather than inventing new ones.
"""

import argparse
import json
import os
import re
import sys
from pathlib import Path

CONTROL_DIR = Path("assets/plato2/control")
TRANSLATED_DIR = Path("assets/plato2/curated/md_modernized_translated")

WORD_RE = re.compile(r"[a-z']+")
MARKER_RE = re.compile(r"\{\{\{\s*([^}]+?)\s*\}\}\}")
STEPHANUS_RE = re.compile(r"^(\d+)([a-e])$")
FRONT_STEPHANUS_RE = re.compile(r'^page_stephanus:\s*"([^"]+)"', re.M)
# `!` splits where the Greek does not (RULESET-plato2 translation hygiene), and
# `## ` headings, `@ Speaker` lines and front matter are not translatable
# prose — SENT_SPLIT_RE also covers `…`, a real terminator for this corpus's
# English splitter even though it is not one in the Greek.
SENT_SPLIT_RE = re.compile(r"(?<=[.!?…])\s+")


def words(text):
    """Lowercased word tokens; punctuation and markup dropped.

    Overlap is measured on words alone so that a difference of comma or
    capitalisation cannot disguise a copied clause.
    """
    return WORD_RE.findall(text.lower())


def ngrams(toks, n):
    return {tuple(toks[i : i + n]) for i in range(len(toks) - n + 1)}


def shared_span(cand, control_ngrams_by_n, n):
    """The actual words shared, so a reviewer can judge the run rather than
    just its length. An enumeration converges by force; a distinctive
    construction does not, and only the text tells them apart."""
    grams = control_ngrams_by_n.get(n)
    if not grams:
        return ""
    common = ngrams(cand, n) & grams
    return " ".join(next(iter(common))) if common else ""


def longest_shared_run(cand, control_ngrams_by_n, max_n):
    """Longest word run the candidate shares with a control, capped at max_n.

    Walks down from max_n so the common case (no long match) costs one set
    lookup per length rather than a quadratic scan.
    """
    for n in range(max_n, 2, -1):
        grams = control_ngrams_by_n.get(n)
        if grams and ngrams(cand, n) & grams:
            return n
    return 0


def load_controls():
    controls = {}
    for path in sorted(CONTROL_DIR.glob("*.json")):
        controls[path.stem] = json.loads(path.read_text(encoding="utf-8"))
    if not controls:
        sys.exit(f"no controls found in {CONTROL_DIR}")
    return controls


def control_index(controls, max_n):
    """Pre-hash every control's n-grams once, per key and per length."""
    index = {}
    for name, data in controls.items():
        for key, text in data.items():
            toks = words(text)
            index[(name, key)] = {n: ngrams(toks, n) for n in range(3, max_n + 1)}
    return index


def parse_section(ref):
    m = STEPHANUS_RE.match(ref)
    return int(m.group(1)) * 10 + "abcde".index(m.group(2)) if m else None


def parse_key_range(key):
    """A control key is either one Stephanus section or a `lo-hi` range."""
    lo_s, sep, hi_s = key.partition("-")
    lo = parse_section(lo_s)
    hi = parse_section(hi_s) if sep else lo
    return lo, hi


def section_schemes(controls):
    """Classify each control's own keying, since the three differ:
    Lamb by exact section, Horan by page range, Jowett as one whole-dialogue
    key. Structural, not name-based, so a control swapped for another with the
    same shape needs no code change."""
    schemes = {}
    for name, data in controls.items():
        keys = list(data)
        if keys == ["full"]:
            schemes[name] = ("whole", None)
        elif any("-" in k for k in keys):
            ranges = sorted(parse_key_range(k) + (k,) for k in keys)
            schemes[name] = ("range", ranges)
        else:
            order = sorted((k for k in keys if parse_section(k) is not None),
                            key=parse_section)
            schemes[name] = ("section", order)
    return schemes


def targets_for_section(section, schemes):
    """Which control entries a candidate sentence should be scored against,
    for its OWN Stephanus section.

    Lamb is section-keyed, so score the sentence's own section plus one on
    either side — translations distribute a clause across a section boundary
    differently than Burnet's Greek does. This mirrors plato1's Shorey
    windowing (scoring against every section a candidate spans) but narrowed
    to the sentence's immediate neighbourhood rather than its whole file,
    since plato2's per-sentence marker tracking makes that neighbourhood
    known precisely. Horan's ranges are coarser (~6 sections each) so the one
    range containing the section is enough. Jowett is a single key covering
    the whole dialogue.
    """
    sec_num = parse_section(section) if section else None
    keys = []
    for name, (kind, data) in schemes.items():
        if kind == "whole":
            keys.append((name, "full"))
        elif kind == "range" and sec_num is not None:
            for lo, hi, key in data:
                if lo <= sec_num <= hi:
                    keys.append((name, key))
                    break
        elif kind == "section" and section in data:
            i = data.index(section)
            keys += [(name, s) for s in data[max(0, i - 1) : i + 2]]
    return keys


def sectioned_units(path):
    """Yield (section, kind, text) for every prose sentence and verse line in
    one file, in document order.

    A sentence/line belongs to the most recent `{{{ }}}` marker at or before
    it; text before any inline marker inherits the file's declared
    page_stephanus. `@ Speaker` lines and `## ` headings are not prose and are
    dropped before splitting.
    """
    raw = path.read_text(encoding="utf-8")
    _, fm, body = raw.split("---", 2)
    m = FRONT_STEPHANUS_RE.search(fm)
    section = m.group(1) if m else None

    body = re.sub(r"^##.*$", " ", body, flags=re.M)
    body = re.sub(r"^@ .*$", " ", body, flags=re.M)

    units = []
    for i, part in enumerate(MARKER_RE.split(body)):
        if i % 2 == 1:
            section = part.strip()
            continue
        verse_lines, prose_lines = [], []
        for line in part.splitlines():
            (verse_lines if line.startswith("| ") else prose_lines).append(
                line[2:] if line.startswith("| ") else line
            )
        for vline in verse_lines:
            if vline.strip():
                units.append((section, "verse", vline.strip()))
        for sent in SENT_SPLIT_RE.split("\n".join(prose_lines)):
            sent = sent.strip()
            if sent:
                units.append((section, "prose", sent))
    return units


def scan_file(path, index, schemes, max_n):
    findings = []
    for section, kind, text in sectioned_units(path):
        toks = words(text)
        if len(toks) < 6:
            continue
        worst, worst_key = 0, None
        for key in targets_for_section(section, schemes):
            run = longest_shared_run(toks, index[key], min(max_n, len(toks)))
            if run > worst:
                worst, worst_key = run, key
        if worst:
            span = shared_span(toks, index[worst_key], worst)
            findings.append((worst, worst_key, kind, span))
    return findings


def _keys_overlapping(kind, data, lo, hi):
    if kind == "whole":
        return ["full"]
    if lo is None:
        return []
    if kind == "section":
        return [k for k in data if lo <= parse_section(k) <= hi]
    if kind == "range":
        return [k for rlo, rhi, k in data if rlo <= hi and lo <= rhi]
    return []


def _all_keys(kind, data):
    if kind == "whole":
        return ["full"]
    if kind == "section":
        return list(data)
    return [k for _, _, k in data]


def calibrate(controls, index, schemes, max_n):
    """Score each control against the others to learn what natural
    convergence looks like. Independent translations of one Greek text share
    proper nouns and stock phrases; the gate must sit above that floor, not on
    it."""
    import statistics

    print("Calibration — control vs. control (independent translations):\n")
    rows = []
    for name, data in controls.items():
        kind, payload = schemes[name]
        runs = []
        for key, text in data.items():
            if kind == "section":
                lo = hi = parse_section(key)
            elif kind == "range":
                lo, hi = parse_key_range(key)
            else:
                lo = hi = None
            for sentence in SENT_SPLIT_RE.split(text):
                toks = words(sentence)
                if len(toks) < 6:
                    continue
                worst = 0
                for other, (okind, odata) in schemes.items():
                    if other == name:
                        continue
                    # Jowett carries no location, so a Jowett sentence has no
                    # neighbourhood to check — fall back to the whole of the
                    # other control rather than silently comparing it to
                    # nothing (which would read as suspiciously clean).
                    cand_keys = (
                        _all_keys(okind, odata)
                        if lo is None
                        else _keys_overlapping(okind, odata, lo, hi)
                    )
                    for ok in cand_keys:
                        worst = max(worst, longest_shared_run(
                            toks, index[(other, ok)], min(max_n, len(toks))))
                runs.append(worst)
        if not runs:
            continue
        runs.sort()
        rows.append((name, runs))
        pct = lambda p: runs[min(len(runs) - 1, int(len(runs) * p))]
        print(
            f"  {name:<18} n={len(runs):>5}  median={statistics.median(runs):>4.1f} "
            f"p95={pct(0.95):>3}  p99={pct(0.99):>3}  max={runs[-1]:>3}"
        )
    allruns = sorted(r for _, rs in rows for r in rs)
    if allruns:
        p999 = allruns[min(len(allruns) - 1, int(len(allruns) * 0.999))]
        print(
            f"\n  pooled p99.9 = {p999} words. Natural convergence between "
            f"independent\n  translations rarely exceeds this; a threshold at or "
            f"just above it\n  flags copying without firing on coincidence."
        )
    return allruns


def self_test(controls, index, schemes, args):
    """A gate nobody has seen fire is not a gate.

    Feeds it a passage lifted verbatim from a control (must fail) and one
    written independently of all of them (must pass), so a change that
    quietly stops it detecting anything is caught.
    """
    import tempfile

    leaked = " ".join(controls["lamb_loeb_1925"]["447a"].split())[:400]
    clean = (
        "Chaerephon, tell me plainly whether you think the man worth "
        "hearing, because for my part I would rather walk on toward the "
        "harbour than sit through one more display that settles nothing at "
        "all."
    )
    header = (
        '---\nposition: 2\nlabel: "x"\ndepth: 1\npage_stephanus: "447a"\n'
        '---\n\n## x\n\n@ Socrates\n\n{{{ 447a }}} '
    )
    ok = True
    with tempfile.TemporaryDirectory() as tmp:
        for name, text, must_fail in (("leaked", leaked, True), ("clean", clean, False)):
            path = Path(tmp) / f"002_gorgias_{name}.md"
            path.write_text(header + text, encoding="utf-8")
            findings = scan_file(path, index, schemes, args.max_n)
            worst = max((f[0] for f in findings), default=0)
            failed = any(
                run >= (args.verse_threshold if kind == "verse" else args.threshold)
                for run, _, kind, _ in findings
            )
            verdict = "FAIL" if failed else "PASS"
            expect = "FAIL" if must_fail else "PASS"
            mark = "ok" if failed == must_fail else "BROKEN"
            if failed != must_fail:
                ok = False
            print(f"  {name:<7} longest run {worst:>3} -> {verdict} "
                  f"(expected {expect}) [{mark}]")
    print("self-test passed" if ok else "SELF-TEST BROKEN — the gate is not detecting")
    return 0 if ok else 1


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--threshold", type=int, default=20,
                    help="fail a PROSE sentence sharing this many consecutive "
                         "words with a control (default: 20; measured plato1 "
                         "prose convergence between independent translations "
                         "tops out at 19 — plato2's own material is not yet "
                         "measured, so this keeps plato1's number rather than "
                         "inventing one; run --calibrate once translated)")
    ap.add_argument("--verse-threshold", type=int, default=31,
                    help="same for quoted verse, which converges much harder "
                         "(default: 31; plato1 controls that did not copy "
                         "each other reach 30)")
    ap.add_argument("--review", type=int, default=13,
                    help="report but do not fail at this length (default: 13, "
                         "just above the p99 of natural convergence)")
    ap.add_argument("--max-n", type=int, default=40,
                    help="longest run to measure; only affects reporting detail")
    ap.add_argument("--dir", type=Path, default=TRANSLATED_DIR)
    ap.add_argument("--calibrate", action="store_true",
                    help="score the controls against each other and exit")
    ap.add_argument("--self-test", action="store_true",
                    help="prove the gate still fires: score a verbatim control "
                         "passage (must fail) and an independent one (must pass)")
    args = ap.parse_args()

    os.chdir(Path(__file__).resolve().parent.parent)
    controls = load_controls()
    index = control_index(controls, args.max_n)
    schemes = section_schemes(controls)
    print(f"controls: {', '.join(sorted(controls))}")

    if args.calibrate:
        calibrate(controls, index, schemes, args.max_n)
        return 0

    if args.self_test:
        return self_test(controls, index, schemes, args)

    if not args.dir.exists():
        print(f"\nNothing to gate: {args.dir} does not exist yet.")
        print("The translation has not been written. Gate is ready.")
        return 0

    files = sorted(args.dir.glob("*.md"))
    if not files:
        print(f"\nNothing to gate: no .md files in {args.dir}.")
        return 0

    failures, review = [], []
    for path in files:
        findings = scan_file(path, index, schemes, args.max_n)
        for run, key, kind, text in findings:
            limit = args.verse_threshold if kind == "verse" else args.threshold
            if run >= limit:
                failures.append((path.name, run, key, kind, text))
            elif run >= args.review:
                review.append((path.name, run, key, kind, text))

    print(f"\nscanned {len(files)} files "
          f"(prose threshold {args.threshold}, verse {args.verse_threshold})")

    if review:
        print(f"\nreview — {len(review)} passage(s) over {args.review} words, "
              f"not failing:\n")
        for name, run, key, kind, text in sorted(review, key=lambda f: -f[1])[:20]:
            print(f"  {run:>3} {kind:<5} vs {key[0]} [{key[1]}]  {name}")
            print(f"      shared: {text}")

    if failures:
        print(f"\nFAIL — {len(failures)} passage(s) at or over the threshold:\n")
        for name, run, key, kind, text in sorted(failures, key=lambda f: -f[1]):
            print(f"  {run:>3} {kind:<5} vs {key[0]} [{key[1]}]  {name}")
            print(f"      shared: {text}")
        return 1

    print("PASS — no passage shares a run at or over its threshold.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
