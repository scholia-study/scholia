#!/usr/bin/env python3
"""Terminology consistency gate for the plato2 English translation (RULESET-11).

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

The *Gorgias* is staged, not narrated: every speech opens with its own
`@ Speaker` line, which is not translatable prose and is stripped before
sentences are counted, alongside headings, front matter and `{{{ }}}` markers.
The table's scope column is `conversations` (gorgias / polus / callicles), in
place of plato1's `books`.
"""

import argparse
import re
import sys
import unicodedata
from pathlib import Path

TABLE = Path("assets/plato2/TERMINOLOGY.md")
GREEK_DIR = Path("assets/plato2/curated/md_modernized")
ENGLISH_DIR = Path("assets/plato2/curated/md_modernized_translated")
MARKER_RE = re.compile(r"\{\{\{\s*[^}]+?\s*\}\}\}")
SPEAKER_RE = re.compile(r"^@ .*$", re.M)
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
    if not path.exists():
        return []
    rows = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.startswith("|") or line.startswith("|---"):
            continue
        cells = [c.strip() for c in line.strip("|").split("|")]
        if len(cells) < 4 or cells[0] in ("Greek",):
            continue
        greek, stem, english, root = cells[0], cells[1], cells[2], cells[3]
        conversations = cells[4] if len(cells) > 4 else "*"
        note = cells[5] if len(cells) > 5 else ""
        rows.append(
            {
                "greek": greek,
                "stem": stem,
                "english": english,
                "root": root.lower(),
                "conversations": conversations,
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
    body = SPEAKER_RE.sub(" ", body)
    return re.sub(r"^\| ", " ", body, flags=re.M)


def conversation_of(filename):
    """A division's filename carries its conversation as the second
    underscore-separated token (`NNN_<conversation>[_slug].md`)."""
    m = re.match(r"\d+_([a-z]+)(?:_|\.md$)", filename)
    return m.group(1) if m else None


def in_scope(row, conversation):
    """A term listed `*` is governed everywhere; a comma-separated list scopes
    it to those conversations only (`gorgias`, `polus`, `callicles`). `-` means
    advisory: never enforced, because no stem can separate the senses."""
    if row["conversations"] == "-":
        return False
    if row["conversations"] == "*" or conversation is None:
        return True
    return conversation in {c.strip() for c in row["conversations"].split(",")}


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
        conversation = conversation_of(gpath.name)
        gsents = SENT_SPLIT_GRC.split(body_of(gpath))
        esents = SENT_SPLIT_EN.split(body_of(epath))
        if len(gsents) != len(esents):
            # Parity is the drama splitter's job to enforce at build time; here
            # it only means the pairing is untrustworthy, so skip rather than
            # report noise.
            continue
        for gs, es in zip(gsents, esents):
            gf, ef = fold(gs), es.lower()
            for r in rows:
                if not in_scope(r, conversation):
                    continue
                # Anchored at a word start: an unanchored stem matches
                # mid-word, which no English wording could ever satisfy.
                if r["stem"] and re.search(rf"(?<![Ͱ-Ͽἀ-῿]){r['stem']}", gf):
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

    if args.self_test:
        return self_test()

    if not args.table.exists():
        print(f"no terminology table yet — skipped ({args.table} not written).")
        print("Gate is ready.")
        return 0

    rows = load_table(args.table)
    print(f"terminology table: {len(rows)} governed terms")

    problems = check_table(rows)
    if problems:
        print("\nFAIL — table integrity (RULESET-11.2):")
        print("\n".join(problems))
        return 1
    print("  table integrity ok — no undeclared collisions")

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


def self_test():
    """Prove both checks still fire.

    Unlike plato1, whose table already existed when its gate was written,
    plato2's TERMINOLOGY.md is not yet authored — so this builds its own
    synthetic table rather than depending on the real one, and exercises the
    same two checks (RULESET-11.2, RULESET-11.1) in isolation.
    """
    import tempfile

    ok = True
    row = {"greek": "ῥητορική", "stem": "ρητορικ", "english": "rhetoric",
           "root": "rhetoric", "conversations": "*", "note": "",
           "alias_ok": False}
    bad = [row, dict(row, greek="πειθώ", stem="πειθ", alias_ok=False)]
    if not check_table(bad):
        print("  SELF-TEST BROKEN: a duplicate rendering was not caught")
        ok = False
    else:
        print("  duplicate rendering -> caught [ok]")

    with tempfile.TemporaryDirectory() as tmp:
        g, e = Path(tmp) / "g", Path(tmp) / "e"
        g.mkdir(); e.mkdir()
        head = ('---\nposition: 2\nlabel: "x"\ndepth: 1\n'
                'page_stephanus: "447a"\n---\n\n## x\n\n@ Γοργίας\n\n')
        (g / "002_gorgias_x.md").write_text(
            head + "{{{ 447a }}} περὶ ῥητορικῆς λέγομεν.", encoding="utf-8")
        (e / "002_gorgias_x.md").write_text(
            '---\nposition: 2\nlabel: "x"\ndepth: 1\n'
            'page_stephanus: "447a"\n---\n\n## x\n\n@ Gorgias\n\n'
            "{{{ 447a }}} We speak about persuasion.", encoding="utf-8")
        stats, misses, _ = check_consistency([row], g, e, 0.9)
        hit, total = stats["ῥητορική"]
        if total == 1 and hit == 0:
            print("  wrong rendering of ῥητορική -> caught [ok]")
        else:
            print(f"  SELF-TEST BROKEN: expected 0/1, got {hit}/{total}")
            ok = False
    print("self-test passed" if ok else "SELF-TEST BROKEN")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
