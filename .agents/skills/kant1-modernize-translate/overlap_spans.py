#!/usr/bin/env python3
"""Per-file overlap + maximal matched runs vs a kant3 control (companion to overlap_gate.py).

Usage:
  gm_spans.py --audit                 # full-corpus JSON dump -> gm_audit.json
  gm_spans.py --spans K file [...]    # print runs >= K words for given files
"""

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import overlap_gate as g

REPO = Path(__file__).resolve().parents[3]
TRANSLATED = REPO / "assets/kant3/curated/md_modernized_translated"
N = g.N

if "--bernard" in sys.argv:
    sys.argv.remove("--bernard")
    CONTROL = REPO / "assets/kant3/control/bernard_1892_pg48433.txt"
else:
    CONTROL = REPO / "assets/kant3/control/guyer_matthews_2000_cambridge.txt"

_gm = g.tokenize(CONTROL.read_text(encoding="utf-8"))
_pos = {}
for _j in range(len(_gm) - N + 1):
    _pos.setdefault(tuple(_gm[_j : _j + N]), []).append(_j)


def maximal_runs(tokens, min_len):
    """Maximal matched runs >= min_len, greedily non-overlapping."""
    out, i = [], 0
    while i <= len(tokens) - N:
        best = 0
        for j in _pos.get(tuple(tokens[i : i + N]), []):
            k = N
            while (
                i + k < len(tokens)
                and j + k < len(_gm)
                and tokens[i + k] == _gm[j + k]
            ):
                k += 1
            best = max(best, k)
        if best >= min_len:
            out.append({"len": best, "text": " ".join(tokens[i : i + best])})
            i += best
        else:
            i += 1
    return out


def measure(path, min_len=10):
    toks = g.md_tokens(path)
    grams = list(g.ngrams(toks, N))
    if not grams:
        return None
    hits = sum(1 for gr in grams if gr in _pos)
    return {
        "overlap": round(hits / len(grams), 4),
        "grams": len(grams),
        "runs": maximal_runs(toks, min_len),
    }


def main():
    if sys.argv[1] == "--audit":
        report = {}
        for f in sorted(TRANSLATED.glob("*.md")):
            report[f.name] = measure(f)
        json.dump(report, open("gm_audit.json", "w"), indent=1)
        measurable = {k: v for k, v in report.items() if v}
        fails = {
            k: v
            for k, v in measurable.items()
            if v["overlap"] >= 0.03 or any(r["len"] >= 15 for r in v["runs"])
        }
        print(f"measurable {len(measurable)}, fail {len(fails)}")
        print(
            f"mean overlap {sum(v['overlap'] for v in measurable.values()) / len(measurable):.2%}"
        )
    elif sys.argv[1] == "--spans":
        k = int(sys.argv[2])
        for name in sys.argv[3:]:
            m = measure(TRANSLATED / name, k)
            print(f"## {name}: {m['overlap']:.1%}, {len(m['runs'])} runs >= {k}")
            for r in m["runs"]:
                print(f"[{r['len']}w] {r['text']}")


if __name__ == "__main__":
    main()
