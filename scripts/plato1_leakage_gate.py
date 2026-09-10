#!/usr/bin/env python3
"""Per-sentence leakage gate for the plato1 English translation.

Scores every candidate sentence against four independent control translations
and fails on any sentence that shares too long a run of words with one of them.

The rule is deliberately per-sentence and absolute: a file-level average hides
exactly the failure kant3 shipped, where a handful of copied sentences drowned
in a page of clean ones. One sentence over the line fails the run.

Controls live in assets/plato1/control/ and are never shipped. Shorey is keyed
by Stephanus section, so a candidate is scored against the control text for its
own section; the others are keyed by book.

Thresholds come from measurement, not intuition. Scoring the four controls
against each other — independent translations of one Greek text — gives the
natural convergence floor:

    median 3-4 words | p95 6-8 | p99 9-12 | prose tail ~19

Quoted VERSE converges far harder: metre and the Greek constrain the rendering,
and runs of 24-30 identical words occur between controls that certainly did not
copy each other. Verse is therefore scored on its own, higher threshold rather
than excluded, so leakage there is still visible without the gate crying wolf
on every poet Plato quotes.
"""

import argparse
import json
import os
import re
import sys
from pathlib import Path

CONTROL_DIR = Path("assets/plato1/control")
TRANSLATED_DIR = Path("assets/plato1/curated/md_modernized_translated")
TOC_RS = Path("packages/common/src/plato1/toc.rs")

WORD_RE = re.compile(r"[a-z']+")
MARKER_RE = re.compile(r"\{\{\{\s*([^}]+?)\s*\}\}\}")
SENT_SPLIT_RE = re.compile(r"(?<=[.!?])\s+")


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
    just its length. An enumeration ("twice six or three times four") converges
    by force; a distinctive construction ("the more I trust you the more I am
    at a loss") does not, and only the text tells them apart."""
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


def book_of_section(ref, book_ranges):
    page = int(re.match(r"(\d+)", ref).group(1))
    for book, (lo, hi) in book_ranges.items():
        if lo <= page <= hi:
            return book
    return None


def toc_book_ranges():
    """Map book -> (first page, last page), read from the Rust TOC as text.

    Reading the source rather than importing the crate keeps this gate honest
    if the TOC and the converter ever regress together.
    """
    rs = TOC_RS.read_text(encoding="utf-8")
    entries = re.findall(
        r'FlatEntry \{\s*stephanus: "([^"]+)",\s*depth: (\d+),', rs
    )
    ranges, book = {}, 0
    for steph, depth in entries:
        page = int(re.match(r"(\d+)", steph).group(1))
        if depth == "0":
            book += 1
            ranges[f"book{book}"] = [page, page]
        else:
            ranges[f"book{book}"][1] = max(ranges[f"book{book}"][1], page)
    return {k: tuple(v) for k, v in ranges.items()}


def control_index(controls, max_n):
    """Pre-hash every control's n-grams once, per key and per length."""
    index = {}
    for name, data in controls.items():
        for key, text in data.items():
            toks = words(text)
            index[(name, key)] = {n: ngrams(toks, n) for n in range(3, max_n + 1)}
    return index


def relevant_control_keys(controls, sections, book):
    """Which control entries a candidate should be scored against.

    Shorey is section-keyed, so only the sections the candidate actually covers
    are relevant — a phrase matching in the right place is signal, the same
    phrase anywhere in 124k words is mostly noise. The others are book-keyed.
    """
    keys = []
    for name, data in controls.items():
        if any(s in data for s in sections):
            keys += [(name, s) for s in sections if s in data]
        elif book and book in data:
            keys.append((name, book))
    return keys


def scan_file(path, controls, index, book_ranges, max_n):
    raw = path.read_text(encoding="utf-8")
    body = raw.split("---", 2)[-1]
    sections = MARKER_RE.findall(body)
    book = book_of_section(sections[0], book_ranges) if sections else None
    body = MARKER_RE.sub(" ", body)
    body = re.sub(r"^##.*$", " ", body, flags=re.M)

    # `+ `-prefixed lines are quoted verse (RULESET-6) and are scored on their
    # own threshold; splitting them out here is what makes that possible.
    verse_lines, prose_lines = [], []
    for line in body.splitlines():
        (verse_lines if line.startswith("+ ") else prose_lines).append(
            line[2:] if line.startswith("+ ") else line
        )

    keys = relevant_control_keys(controls, sections, book)

    def score(units, kind):
        out = []
        for unit in units:
            toks = words(unit)
            if len(toks) < 6:
                continue
            worst, worst_key = 0, None
            for key in keys:
                run = longest_shared_run(toks, index[key], min(max_n, len(toks)))
                if run > worst:
                    worst, worst_key = run, key
            if worst:
                span = shared_span(toks, index[worst_key], worst)
                out.append((worst, worst_key, kind, span))
        return out

    findings = score(SENT_SPLIT_RE.split("\n".join(prose_lines)), "prose")
    findings += score(verse_lines, "verse")
    return findings, book


def calibrate(controls, index, max_n):
    """Score each control against the others to learn what natural convergence
    looks like. Independent translations of one Greek text share proper nouns
    and stock phrases; the gate must sit above that floor, not on it."""
    import statistics

    print("Calibration — control vs. control (independent translations):\n")
    rows = []
    for name, data in controls.items():
        others = [k for k in index if k[0] != name]
        runs = []
        for key, text in data.items():
            for sentence in SENT_SPLIT_RE.split(text):
                toks = words(sentence)
                if len(toks) < 6:
                    continue
                cand_keys = [k for k in others if k[1] == key] or [
                    k for k in others if not k[1][0].isdigit()
                ]
                worst = 0
                for ck in cand_keys[:4]:
                    worst = max(
                        worst, longest_shared_run(toks, index[ck], min(max_n, len(toks)))
                    )
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


def self_test(controls, index, args):
    """A gate nobody has seen fire is not a gate.

    Feeds it a passage lifted verbatim from a control (must fail) and one
    written independently of all of them (must pass), so a change that
    quietly stops it detecting anything is caught.
    """
    import tempfile

    book_ranges = toc_book_ranges()
    leaked = " ".join(controls["shorey_loeb"]["327a"].split())[:600]
    clean = (
        "Yesterday I walked down to the harbour town together with Ariston's "
        "boy, meaning to say my prayers before the goddess and at the same "
        "time to watch how they would keep a festival they had never kept "
        "before. The local people made a fine showing, though to my eye the "
        "visitors from the north did no worse."
    )
    header = (
        '---\nposition: 2\nlabel: "The Descent to the Piraeus"\ndepth: 1\n'
        'page_stephanus: "327a"\n---\n\n## The Descent to the Piraeus\n\n'
        "{{{ 327a }}} "
    )
    ok = True
    with tempfile.TemporaryDirectory() as tmp:
        for name, text, must_fail in (
            ("leaked", leaked, True),
            ("clean", clean, False),
        ):
            path = Path(tmp) / f"002_{name}.md"
            path.write_text(header + text, encoding="utf-8")
            findings, _ = scan_file(path, controls, index, book_ranges, args.max_n)
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
            path.unlink()
    print("self-test passed" if ok else "SELF-TEST BROKEN — the gate is not detecting")
    return 0 if ok else 1


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--threshold", type=int, default=20,
                    help="fail a PROSE sentence sharing this many consecutive "
                         "words with a control (default: 20; measured prose "
                         "convergence between independent translations tops "
                         "out at 19)")
    ap.add_argument("--verse-threshold", type=int, default=31,
                    help="same for quoted verse, which converges much harder "
                         "(default: 31; controls that did not copy each other "
                         "reach 30)")
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
    print(f"controls: {', '.join(sorted(controls))}")

    if args.calibrate:
        calibrate(controls, index, args.max_n)
        return 0

    if args.self_test:
        return self_test(controls, index, args)

    if not args.dir.exists():
        print(f"\nNothing to gate: {args.dir} does not exist yet.")
        print("The translation has not been written. Gate is ready.")
        return 0

    book_ranges = toc_book_ranges()
    files = sorted(args.dir.glob("*.md"))
    if not files:
        print(f"\nNothing to gate: no .md files in {args.dir}.")
        return 0

    failures, review = [], []
    for path in files:
        findings, _ = scan_file(path, controls, index, book_ranges, args.max_n)
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
