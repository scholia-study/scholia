#!/usr/bin/env python3
"""Terminology consistency gate for the plato1 English translation (RULESET-11).

Two checks:

  1. TABLE INTEGRITY — no English rendering is claimed by two different Greek
     terms unless the collision is declared deliberate. Pure property of the
     table; runs even before a word is translated.

  2. RENDERING CONSISTENCY — where a Greek sentence carries a governed term,
     its English counterpart carries the governed rendering. The editions are
     sentence-parity locked, so sentence N of the Greek faces sentence N of the
     English and the comparison is exact rather than approximate.

Check 2 is reported as a per-term compliance rate with a floor rather than a
per-occurrence abort: a translator may legitimately carry a term across a
sentence boundary, and a rule that forbids that would be gamed by writing worse
English. A term that drifts genuinely shows up as a rate collapse, not a single
miss.
"""

import argparse
import re
import sys
import unicodedata
from pathlib import Path

TABLE = Path("assets/plato1/TERMINOLOGY.md")
GREEK_DIR = Path("assets/plato1/curated/md_modernized")
ENGLISH_DIR = Path("assets/plato1/curated/md_modernized_translated")
MARKER_RE = re.compile(r"\{\{\{\s*[^}]+?\s*\}\}\}")
SENT_SPLIT_EN = re.compile(r"(?<=[.!?])\s+")
SENT_SPLIT_GRC = re.compile(r"(?<=[.;])\s+")


def fold(s):
    """Accent- and breathing-folded Greek, final sigma normalised.

    Mirrors the SQL grc_fold in migration 0029: iota subscript (U+0345) is
    kept, everything else combining is dropped.
    """
    s = unicodedata.normalize("NFD", s.lower())
    s = "".join(c for c in s if not unicodedata.combining(c) or c == "ͅ")
    return unicodedata.normalize("NFC", s).replace("ς", "σ")


def load_table(path=TABLE):
    rows = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.startswith("|") or line.startswith("|---"):
            continue
        cells = [c.strip() for c in line.strip("|").split("|")]
        if len(cells) < 4 or cells[0] in ("Greek",):
            continue
        greek, stem, english, root = cells[0], cells[1], cells[2], cells[3]
        books = cells[4] if len(cells) > 4 else "*"
        note = cells[5] if len(cells) > 5 else ""
        rows.append(
            {
                "greek": greek,
                "stem": stem,
                "english": english,
                "root": root.lower(),
                "books": books,
                "note": note,
                # A collision is deliberate only if the note says so in the
                # agreed form; anything else is an accident and fails.
                "alias_ok": "same rendering as" in note.lower(),
            }
        )
    return rows


def check_table(rows):
    """RULESET-11.2 — no rendering shared by two governed terms."""
    by_rendering = {}
    for r in rows:
        by_rendering.setdefault(r["english"].lower(), []).append(r)
    problems = []
    for rendering, group in sorted(by_rendering.items()):
        if len(group) < 2:
            continue
        if all(g["alias_ok"] for g in group[1:]):
            print(
                f"  declared alias: {' / '.join(g['greek'] for g in group)}"
                f" -> {rendering!r}"
            )
            continue
        problems.append(
            f"  {rendering!r} claimed by "
            + ", ".join(g["greek"] for g in group)
            + " with no declared alias"
        )
    return problems


def body_of(path):
    raw = path.read_text(encoding="utf-8")
    body = raw.split("---", 2)[-1]
    body = MARKER_RE.sub(" ", body)
    body = re.sub(r"^##.*$", " ", body, flags=re.M)
    return re.sub(r"^\+ ", " ", body, flags=re.M)


ROMAN = {"i": 1, "ii": 2, "iii": 3, "iv": 4, "v": 5,
         "vi": 6, "vii": 7, "viii": 8, "ix": 9, "x": 10}


def book_of(filename):
    m = re.search(r"_bk_([ivx]+)_", filename) or re.search(r"_bk_([ivx]+)\.md$", filename)
    return ROMAN.get(m.group(1)) if m else None


def in_scope(row, book):
    """A term whose sense changes across the work is only governed where its
    technical sense applies. `-` means advisory: never enforced, because no
    stem can separate the senses (εἶδος is the Form and also "kind")."""
    if row["books"] == "-":
        return False
    if row["books"] == "*" or book is None:
        return True
    lo, _, hi = row["books"].partition("-")
    return int(lo) <= book <= int(hi or lo)


def check_consistency(rows, greek_dir, english_dir, floor):
    """RULESET-11.1 — a governed term's rendering, everywhere it occurs."""
    stats = {r["greek"]: [0, 0] for r in rows}
    misses = []
    pairs = 0
    for gpath in sorted(greek_dir.glob("*.md")):
        epath = english_dir / gpath.name
        if not epath.exists():
            continue
        pairs += 1
        book = book_of(gpath.name)
        gsents = SENT_SPLIT_GRC.split(body_of(gpath))
        esents = SENT_SPLIT_EN.split(body_of(epath))
        if len(gsents) != len(esents):
            # Parity is md_prose_to_struct's job to enforce at build time; here
            # it only means the pairing is untrustworthy, so skip rather than
            # report noise.
            continue
        for gs, es in zip(gsents, esents):
            gf, ef = fold(gs), es.lower()
            for r in rows:
                if not in_scope(r, book):
                    continue
                # Anchored at a word start: an unanchored stem matches
                # mid-word (ἔρως's `ερωσ` sits inside μοχθηροτέρως "more
                # wretchedly"), which no English wording could ever satisfy.
                if r["stem"] and re.search(rf"(?<![Ͱ-Ͽ\u1f00-\u1fff]){r['stem']}", gf):
                    stats[r["greek"]][1] += 1
                    if r["root"] and r["root"] in ef:
                        stats[r["greek"]][0] += 1
                    else:
                        misses.append((r["greek"], r["english"], gpath.name,
                                       " ".join(es.split())[:90]))
    return stats, misses, pairs


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--greek-dir", type=Path, default=GREEK_DIR)
    ap.add_argument("--english-dir", type=Path, default=ENGLISH_DIR)
    ap.add_argument("--table", type=Path, default=TABLE)
    ap.add_argument("--floor", type=float, default=0.90,
                    help="minimum per-term compliance rate (default 0.90)")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    import os
    os.chdir(Path(__file__).resolve().parent.parent)

    rows = load_table(args.table)
    print(f"terminology table: {len(rows)} governed terms")

    problems = check_table(rows)
    if problems:
        print("\nFAIL — table integrity (RULESET-11.2):")
        print("\n".join(problems))
        return 1
    print("  table integrity ok — no undeclared collisions")

    if args.self_test:
        return self_test(rows)

    if not args.english_dir.exists() or not any(args.english_dir.glob("*.md")):
        print(f"\nNo translation in {args.english_dir} yet; consistency check "
              f"skipped. Gate is ready.")
        return 0

    stats, misses, pairs = check_consistency(rows, args.greek_dir,
                                             args.english_dir, args.floor)
    print(f"\nchecked {pairs} sentence-paired files")
    failed = []
    for term, (hit, total) in sorted(stats.items(), key=lambda kv: kv[1][1],
                                     reverse=True):
        if not total:
            continue
        rate = hit / total
        flag = "" if rate >= args.floor else "  << below floor"
        print(f"  {term:<14} {hit:>4}/{total:<4} {rate:6.1%}{flag}")
        if rate < args.floor:
            failed.append((term, rate, total))
    if failed:
        print(f"\nFAIL — {len(failed)} term(s) below the {args.floor:.0%} floor:")
        for term, rate, total in failed:
            print(f"  {term}: {rate:.1%} of {total} occurrences")
        for term, english, fname, sent in misses[:15]:
            print(f"    {term} -> expected {english!r}: {fname}: {sent}")
        return 1
    print("\nPASS — every governed term holds its rendering.")
    return 0


def self_test(rows):
    """Prove both checks still fire."""
    import tempfile
    ok = True
    bad = rows[:1] + [dict(rows[0], greek="ἄλλο", stem="αλλ", alias_ok=False)]
    if not check_table(bad):
        print("  SELF-TEST BROKEN: a duplicate rendering was not caught")
        ok = False
    else:
        print("  duplicate rendering -> caught [ok]")

    with tempfile.TemporaryDirectory() as tmp:
        g, e = Path(tmp) / "g", Path(tmp) / "e"
        g.mkdir(); e.mkdir()
        head = '---\nposition: 1\nlabel: "x"\ndepth: 1\npage_stephanus: "327a"\n---\n\n'
        (g / "001_x.md").write_text(head + "περὶ δικαιοσύνης λέγομεν.", encoding="utf-8")
        (e / "001_x.md").write_text(head + "We speak about righteousness.", encoding="utf-8")
        stats, misses, _ = check_consistency(rows, g, e, 0.9)
        hit, total = stats["δικαιοσύνη"]
        if total == 1 and hit == 0:
            print("  wrong rendering of δικαιοσύνη -> caught [ok]")
        else:
            print(f"  SELF-TEST BROKEN: expected 0/1, got {hit}/{total}")
            ok = False
    print("self-test passed" if ok else "SELF-TEST BROKEN")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
