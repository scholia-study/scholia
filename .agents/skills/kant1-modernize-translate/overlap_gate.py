#!/usr/bin/env python3
"""Translation-independence gate for kant1 English files.

Measures verbatim word-8-gram overlap and the longest common word run
between a translated markdown file and the copyrighted Cambridge/Guyer-Wood
control text. PASS requires overlap < 3% AND longest run < 15 words
(independent-translation baseline, calibrated with Meiklejohn vs G/W:
<1% overlap, longest run 12 words).

Usage: overlap_gate.py <file.md> [<file.md> ...]
Exit code 1 if any file fails.
"""

import re
import sys
from html.parser import HTMLParser
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
CONTROL_DIR = REPO / "assets/kant1/control/text"

N = 8
MAX_OVERLAP = 0.03
MAX_RUN = 15


class TextExtractor(HTMLParser):
    def __init__(self):
        super().__init__()
        self.parts = []

    def handle_data(self, data):
        self.parts.append(data)


def tokenize(text: str) -> list[str]:
    return re.findall(r"[a-z0-9']+", text.lower())


def md_tokens(path: Path) -> list[str]:
    text = path.read_text(encoding="utf-8")
    text = re.sub(r"^---.*?---", "", text, count=1, flags=re.S)  # frontmatter
    text = re.sub(r"\{\{\{?\s*[^}]+?\s*\}?\}\}", " ", text)      # page markers
    text = re.sub(r"\[\^[^\]]+\]:?", " ", text)                   # footnote refs
    text = text.replace("|||", " ")
    text = re.sub(r"<[^>]+>", " ", text)                          # figure HTML
    return tokenize(text)


def control_tokens() -> list[str]:
    if not CONTROL_DIR.is_dir():
        sys.exit(f"control text not found: {CONTROL_DIR}")
    tokens: list[str] = []
    for f in sorted(CONTROL_DIR.rglob("*")):
        if f.suffix.lower() not in (".html", ".xhtml", ".htm", ".txt"):
            continue
        raw = f.read_text(encoding="utf-8", errors="ignore")
        if f.suffix.lower() == ".txt":
            tokens.extend(tokenize(raw))
        else:
            p = TextExtractor()
            p.feed(raw)
            tokens.extend(tokenize(" ".join(p.parts)))
    if not tokens:
        sys.exit(f"no text extracted from {CONTROL_DIR}")
    return tokens


def ngrams(tokens: list[str], n: int):
    return (tuple(tokens[i : i + n]) for i in range(len(tokens) - n + 1))


def longest_run(tokens: list[str], control: list[str], seed: set, n: int) -> int:
    best = 0
    i = 0
    positions: dict[tuple, list[int]] = {}
    for j in range(len(control) - n + 1):
        positions.setdefault(tuple(control[j : j + n]), []).append(j)
    while i <= len(tokens) - n:
        gram = tuple(tokens[i : i + n])
        if gram in seed:
            for j in positions.get(gram, []):
                k = n
                while i + k < len(tokens) and j + k < len(control) and tokens[i + k] == control[j + k]:
                    k += 1
                best = max(best, k)
        i += 1
    return best


def main():
    files = [Path(a) for a in sys.argv[1:]]
    if not files:
        sys.exit(__doc__)
    control = control_tokens()
    control_grams = set(ngrams(control, N))
    failed = False
    for f in files:
        toks = md_tokens(f)
        grams = list(ngrams(toks, N))
        if not grams:
            print(f"{f.name}: too short to measure — MANUAL CHECK")
            continue
        hits = sum(1 for g in grams if g in control_grams)
        overlap = hits / len(grams)
        run = longest_run(toks, control, control_grams, N) if hits else 0
        ok = overlap < MAX_OVERLAP and run < MAX_RUN
        status = "PASS" if ok else "FAIL"
        print(f"{f.name}: {overlap:.1%} 8-gram overlap, longest run {run} words — {status}")
        failed |= not ok
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
