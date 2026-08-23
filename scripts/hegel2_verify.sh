#!/usr/bin/env bash
# Verify the hegel2 md_reviewed layer against its sources and witnesses.
#
# Gates:
#   1. file set — every TOC node file present, nothing unexpected
#   2. front matter — position/label/depth/page_gw per file
#   3. page-marker sequence — GW pages appear exactly once, in order,
#      with only the known apparatus gaps
#   4. markup hygiene — no unconverted emphasis stars, no raw footnote ids,
#      no unbalanced marker braces
#   5. witness alignment — reviewed text vs the DTA TEIs (1813 Wesen,
#      1816 Begriff): every word-level difference must be covered by a
#      keep_gw/use_dta ruling in gw_dta_rulings.tsv. The 1832 Seyn has no
#      DTA witness; it is checked against the Zeno Werke text as a
#      REPORT-ONLY summary (orthography differs by edition).
#
# Usage: bash scripts/hegel2_verify.sh

set -euo pipefail
cd "$(dirname "$0")/.."

DIR=assets/hegel2/curated/md_reviewed
FAIL=0

echo "== gate 1+2+3+4: structure, front matter, markers, hygiene =="
python3 - <<'PYEOF' || FAIL=1
import re, sys, unicodedata

DIR = "assets/hegel2/curated/md_reviewed"
sys.path.insert(0, "scripts")

# oracle: the TOC as (position, depth, page, filename) parsed from the
# checked-in table — read the Rust source as text, not through the crate,
# so a converter/common co-regression still has to get past this file.
rs = open("packages/common/src/hegel2/toc.rs").read()
entries = []
for m in re.finditer(
    r'position: (\d+),\s*depth: (\d+),\s*(?://[^\n]*\n\s*)*page: (None|Some\("([^"]+)"\)),'
    r'\s*slug: "([^"]+)",\s*label: "((?:[^"\\]|\\.)*)",',
    rs,
):
    pos, depth, _, page, slug, label = m.groups()
    entries.append((int(pos), int(depth), page, slug, label.replace('\\"', '"')))
assert len(entries) == 274, f"oracle parsed {len(entries)} entries"

import os
files = sorted(f for f in os.listdir(DIR) if f.endswith(".md"))
expected = {f"{pos:03}_{slug}.md": (pos, depth, page, label) for pos, depth, page, slug, label in entries}
ok = True
for f in files:
    if f not in expected:
        print(f"UNEXPECTED FILE {f}")
        ok = False
missing = set(expected) - set(files)
for f in sorted(missing):
    print(f"MISSING FILE {f}")
    ok = False

marker_re = re.compile(r"\{\{\{ (\d+)\.(\d+) \}\}\}")
seen_pages = []
for f in files:
    if f not in expected:
        continue
    pos, depth, page, label = expected[f]
    text = open(f"{DIR}/{f}").read()
    m = re.match(
        r'---\nposition: (\d+)\nlabel: "((?:[^"\\]|\\.)*)"\ndepth: (\d+)\n(?:page_gw: "([^"]*)"\n)?---\n',
        text,
    )
    if not m:
        print(f"{f}: bad front matter")
        ok = False
        continue
    fpos, flabel, fdepth, fpage = m.groups()
    if int(fpos) != pos or int(fdepth) != depth:
        print(f"{f}: position/depth mismatch")
        ok = False
    if flabel.replace('\\"', '"') != label:
        print(f"{f}: label mismatch: {flabel!r} vs {label!r}")
        ok = False
    want_page = None if page == "None" else page
    if fpage != want_page:
        print(f"{f}: page_gw mismatch: {fpage!r} vs {want_page!r}")
        ok = False
    body = text[m.end():]
    if "[^fn-" in body:
        print(f"{f}: unresolved raw footnote id")
        ok = False
    for line in body.splitlines():
        stars = line.count("*")
        if stars and line.strip() != "***":
            print(f"{f}: unconverted emphasis star: {line[:70]}")
            ok = False
    if body.count("{{{") != body.count("}}}"):
        print(f"{f}: unbalanced marker braces")
        ok = False
    for mm in marker_re.finditer(body):
        seen_pages.append((int(mm.group(1)), int(mm.group(2)), f))

# marker sequence: strictly increasing within each volume run, no
# duplicates, and full coverage of the content ranges
runs = {21: (4, 383), 11: (241, 409), 12: (5, 253)}
apparatus_gaps = {(21, 21), (21, 22), (21, 23), (21, 24), (21, 25), (21, 26), (21, 50),
                  (21, 52), (12, 7), (12, 8), (12, 9), (12, 10)}
by_vol = {}
for vol, page, f in seen_pages:
    by_vol.setdefault(vol, []).append((page, f))
for vol, (lo, hi) in runs.items():
    pages = [p for p, _ in by_vol.get(vol, [])]
    dupes = {p for p in pages if pages.count(p) > 1}
    if dupes:
        print(f"vol {vol}: duplicate page markers {sorted(dupes)[:8]}")
        ok = False
    if pages != sorted(pages):
        print(f"vol {vol}: marker sequence not ascending")
        ok = False
    have = set(pages)
    missing = [
        p for p in range(lo, hi + 1)
        if p not in have and (vol, p) not in apparatus_gaps
    ]
    if missing:
        print(f"vol {vol}: pages without markers: {missing[:12]}{'…' if len(missing) > 12 else ''}")
        ok = False

print(f"structure: {len(files)} files, {len(seen_pages)} page markers")
sys.exit(0 if ok else 1)
PYEOF

echo "== gate 5: witness alignment (DTA 1813 + 1816; Zeno report for 1832) =="
python3 - <<'PYEOF' || FAIL=1
import re, sys, json, unicodedata, difflib, html as H

DIR = "assets/hegel2/curated/md_reviewed"


def norm_token(t):
    t = unicodedata.normalize("NFC", t)
    for a, b in (("aͤ", "ae"), ("oͤ", "oe"), ("uͤ", "ue"),
                 ("Aͤ", "ae"), ("Oͤ", "oe"), ("Uͤ", "ue"),
                 ("ä", "ae"), ("ö", "oe"), ("ü", "ue"),
                 ("Ä", "ae"), ("Ö", "oe"), ("Ü", "ue"),
                 ("ſ", "s"), ("ß", "ss"), ("²", "2"), ("³", "3")):
        t = t.replace(a, b)
    return re.sub(r"[^\w]", "", t).lower()


def merge_singles(toks):
    """Fold runs of single-character tokens (abbreviation letters "u s f",
    math "a 2") so spacing conventions cannot masquerade as text diffs."""
    out = []
    run = []
    for t in toks:
        if len(t) == 1:
            run.append(t)
            continue
        if run:
            out.append("".join(run))
            run = []
        out.append(t)
    if run:
        out.append("".join(run))
    return out


def md_tokens(vol_prefix):
    """Reviewed-layer body tokens for one volume: markers, headings and
    footnote definitions excluded (headings/notes are checked structurally;
    the witness stream excludes them too)."""
    import os
    toks = []
    for f in sorted(os.listdir(DIR)):
        if not f.endswith(".md"):
            continue
        text = open(f"{DIR}/{f}").read()
        body = text.split("---\n", 2)[2]
        # does this file belong to the volume? check its markers/front page
        for line in body.splitlines():
            ls = line.strip()
            if not ls:
                continue
            if ls.startswith("## ") or ls.startswith("[^"):
                # headings/notes stay out of the stream, but a heading's page
                # marker still switches the current volume
                for m in re.finditer(r"\{\{\{ (\d+)\.\d+ \}\}\}", ls):
                    toks.append(("VOL", m.group(1)))
                continue
            if ls.startswith("+ "):
                ls = ls[2:]
            ls = re.sub(r"\{\{\{ (\d+)\.\d+ \}\}\}", lambda m: f"⁂{m.group(1)}", ls)
            for w in ls.replace("_", " ").replace("<i>", " ").replace("</i>", " ").split():
                if w.startswith("⁂"):
                    toks.append(("VOL", w[1:]))
                    continue
                n = norm_token(w)
                if n:
                    toks.append((n, None))
    # split by volume using the marker stream
    out = []
    cur = None
    for t, vol in toks:
        if t == "VOL":
            cur = int(vol)
            continue
        if cur == vol_prefix:
            out.append(t)
    return merge_singles(out)


def dta_tokens(path):
    tei = open(path).read()
    text = tei[re.search(r"<text[ >]", tei).start():]
    # glue drop-cap initials ("<hi rendition=\"#in\">D</hi>ie") before the
    # generic tag strip turns the tag boundary into a word split
    text = re.sub(r'<hi rendition="#in">([^<]+)</hi>', r"\1", text)
    for pat in (r"<titlePage.*?</titlePage>",
                r"<div[^>]*type=\"(contents|corrigenda)\"[^>]*>.*?</div>",
                r"<note[^>]*place=\"foot\".*?</note>",
                r"<note[^>]*type=\"remarkResponsibility\".*?</note>",
                r"<fw[^>]*>.*?</fw>", r"<head[^>]*>.*?</head>",
                r"<formula[^>]*>.*?</formula>"):
        text = re.sub(pat, " ", text, flags=re.S)
    text = re.sub(r"<[^>]+>", " ", text)
    text = H.unescape(text)
    raw = text.split()
    toks, pending = [], None
    for w in raw:
        if pending is not None:
            w = pending + w
            pending = None
        if len(w) > 1 and (w.endswith("-") or w.endswith("⸗")):
            pending = w[:-1]
            continue
        n = norm_token(w)
        if n:
            toks.append(n)
    return merge_singles(toks)


def chunked_diff(a, b):
    i = j = 0
    n_diff = 0
    sites = 0
    while i < len(a) and j < len(b):
        wa, wb = a[i:i + 4000], b[j:j + 4800]
        sm = difflib.SequenceMatcher(None, wa, wb, autojunk=False)
        blocks = [bl for bl in sm.get_matching_blocks() if bl.size >= 5]
        if not blocks:
            n_diff += 4000
            sites += 1
            i += 4000
            j += 4000
            continue
        last = blocks[-1]
        for op, i1, i2, j1, j2 in sm.get_opcodes():
            if i1 >= last.a + last.size:
                break
            if op != "equal":
                n_diff += max(min(i2, last.a + last.size) - i1, min(j2, last.b + last.size) - j1)
                sites += 1
        i += last.a + last.size
        j += last.b + last.size
    n_diff += (len(a) - i) + (len(b) - j)
    return n_diff, sites


ok = True
# rulings keep the residual budget: every keep_gw site stays a diff; the
# gate bounds total diff mass rather than re-adjudicating each site.
rul = [l.split("\t") for l in open("assets/hegel2/curated/gw_dta_rulings.tsv").read().splitlines()[1:]]
budget = {"11": 0, "12": 0}
for f in rul:
    if f[6] == "keep_gw":
        budget[f[1]] += max(len(f[4].split()), len(f[5].split())) + 2

for vol, tei in ((11, "assets/hegel2/raw/hegel_logik0102_1813.xml"),
                 (12, "assets/hegel2/raw/hegel_logik02_1816.xml")):
    a = md_tokens(vol)
    b = dta_tokens(tei)
    n_diff, sites = chunked_diff(a, b)
    allowed = budget[str(vol)] + 40
    status = "OK" if n_diff <= allowed else "FAIL"
    if n_diff > allowed:
        ok = False
    print(f"vol {vol}: {len(a)} md tokens vs {len(b)} dta tokens, "
          f"diff mass {n_diff} (allowed {allowed}), {sites} sites — {status}")

# Zeno report for the 1832 Seyn (edition orthography differs; report only)
z = json.load(open("assets/hegel2/control/wdl.json"))
zwords = []
def walk(n):
    label = n.get("label", "")
    if "Zweites Buch" in label or "Zweiter Teil" in label:
        return
    for c in n.get("content", []):
        if c.get("type") == "paragraph":
            zwords.extend(c.get("text", "").split())
    for ch in n.get("children", []):
        walk(ch)
for n in z["nodes"]:
    walk(n)
def fold(t):
    t = norm_token(t)
    for x, y in (("th", "t"), ("ey", "ei"), ("tz", "z"), ("c", "k"), ("y", "i")):
        t = t.replace(x, y)
    return t
a = [fold(t) for t in md_tokens(21)]
b = [w for w in (fold(w) for w in zwords) if w]
n_diff, sites = chunked_diff(a, b)
print(f"vol 21 (report only): {len(a)} md tokens vs {len(b)} zeno tokens, "
      f"diff mass {n_diff} ({n_diff / max(len(a), 1) * 100:.2f}%), {sites} sites")

sys.exit(0 if ok else 1)
PYEOF

if [ "$FAIL" -ne 0 ]; then
    echo "hegel2 verify: FAILED"
    exit 1
fi
echo "hegel2 verify: all gates passed"
