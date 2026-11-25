#!/usr/bin/env python3
"""Translation-independence gate for hegel4 English files (ported from hegel2).

Measures verbatim word-8-gram overlap and the longest common word run
between a translated markdown file and EACH control translation separately
(every `*.txt` in `assets/hegel4/control/`: bd, gsh, petry, inwood,
wallace) — from-memory leakage can converge on any one of them
individually. PASS requires overlap < 3% AND longest run < 15 words
against every control.

Text the English carries verbatim from the German source file (Hegel's
Latin, French, Greek and English quotations, formulas, citation numbers)
is discounted: every 8-gram also present in the same-named
`md_modernized` file is masked before measuring, since the controls carry
those quotations verbatim too.

Calibration (2026-08-09): the published translations against each other run
Baillie↔Miller 7.1% / 35, Baillie↔Pinkard 2.5% / 34, Miller↔Pinkard
3.3% / 32 — so this bar is stricter than published practice, deliberately.

Usage:
  overlap_gate.py <file.md> [<file.md> ...]
  overlap_gate.py --spans K <file.md>   # print matched runs >= K words

Exit code 1 if any file fails against any control.
"""

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
CONTROL_DIR = REPO / "assets/hegel4/control"
SOURCE_DIR = REPO / "assets/hegel4/curated/md_modernized"

N = 8
MAX_OVERLAP = 0.03
MAX_RUN = 15


def tokenize(text: str) -> list:
    return re.findall(r"[a-z0-9']+", text.lower())


def md_tokens(path: Path) -> list:
    text = path.read_text(encoding="utf-8")
    text = re.sub(r"^---.*?---", "", text, count=1, flags=re.S)  # frontmatter
    text = re.sub(r"\{\{\{?\s*[^}]+?\s*\}?\}\}", " ", text)  # page markers
    text = re.sub(r"<[^>]+>", " ", text)  # intraword <i>
    return tokenize(text)


def mask_carried(tokens: list, source: Path) -> tuple:
    """Replace tokens covered by an 8-gram of the German source with unique
    sentinels, so verbatim-carried quotations can match no control."""
    if not source.exists():
        sys.exit(f"German source {source} not found")
    carried = set(ngrams(md_tokens(source), N))
    covered = [False] * len(tokens)
    for i in range(len(tokens) - N + 1):
        if tuple(tokens[i : i + N]) in carried:
            for k in range(i, i + N):
                covered[k] = True
    masked = [f"\0{i}" if c else t for i, (t, c) in enumerate(zip(tokens, covered))]
    return masked, sum(covered)


def controls() -> list:
    sets = [
        (f.stem, tokenize(f.read_text(encoding="utf-8", errors="ignore")))
        for f in sorted(CONTROL_DIR.glob("*.txt"))
    ]
    if not sets:
        sys.exit(f"no *.txt controls in {CONTROL_DIR} — extract them first")
    return sets


def ngrams(tokens: list, n: int):
    return (tuple(tokens[i : i + n]) for i in range(len(tokens) - n + 1))


def positions(control: list) -> dict:
    pos = {}
    for j in range(len(control) - N + 1):
        pos.setdefault(tuple(control[j : j + N]), []).append(j)
    return pos


def longest_run(tokens: list, control: list, pos: dict) -> int:
    """Exact longest common run: every start position is scanned, so an
    overlap-masked run can never be underestimated."""
    best = 0
    for i in range(len(tokens) - N + 1):
        for j in pos.get(tuple(tokens[i : i + N]), []):
            k = N
            while (
                i + k < len(tokens)
                and j + k < len(control)
                and tokens[i + k] == control[j + k]
            ):
                k += 1
            best = max(best, k)
    return best


def matched_runs(tokens: list, control: list, pos: dict, min_len: int) -> list:
    """Maximal matched runs >= min_len, greedily non-overlapping."""
    out, i = [], 0
    while i <= len(tokens) - N:
        best = 0
        for j in pos.get(tuple(tokens[i : i + N]), []):
            k = N
            while (
                i + k < len(tokens)
                and j + k < len(control)
                and tokens[i + k] == control[j + k]
            ):
                k += 1
            best = max(best, k)
        if best >= min_len:
            out.append((best, " ".join(tokens[i : i + best])))
            i += best
        else:
            i += 1
    return out


def main():
    args = sys.argv[1:]
    span_min = None
    if args and args[0] == "--spans":
        span_min = int(args[1])
        args = args[2:]
    files = [Path(a) for a in args]
    if not files:
        sys.exit(__doc__)
    sets = [(name, toks, positions(toks), set(ngrams(toks, N))) for name, toks in controls()]
    failed = False
    for f in files:
        toks, carried = mask_carried(md_tokens(f), SOURCE_DIR / f.name)
        if carried:
            print(f"{f.name}: {carried} tokens carried verbatim from the German, discounted")
        grams = list(ngrams(toks, N))
        if not grams:
            print(f"{f.name}: too short to measure — MANUAL CHECK")
            continue
        for name, control, pos, control_grams in sets:
            hits = sum(1 for g in grams if g in control_grams)
            overlap = hits / len(grams)
            longest = longest_run(toks, control, pos) if hits else 0
            ok = overlap < MAX_OVERLAP and longest < MAX_RUN
            status = "PASS" if ok else "FAIL"
            print(
                f"{f.name} vs {name}: {overlap:.1%} 8-gram overlap, "
                f"longest run {longest} words — {status}"
            )
            if span_min and hits:
                for length, text in matched_runs(toks, control, pos, span_min):
                    print(f"    [{length}] {text[:160]}")
            failed |= not ok
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
