#!/usr/bin/env bash
# Verify the plato1 curated Greek and, when it exists, its English translation.
#
# Gates:
#   1. file set + front matter — every TOC node has a file; position, label,
#      depth and page_stephanus match the checked-in table exactly
#   2. Stephanus markers — 1355, each exactly once, strictly ascending, and
#      never on a depth-0 book node
#   3. markup hygiene — no `{{ }}` secondary marker (this corpus has no second
#      system), no U+02BC, balanced braces, blockquote artefacts reported
#   4. golden values — sentence counts and the length distribution that would
#      collapse if the Greek splitter ever regressed
#   5. terminology gate (RULESET-11)
#   6. leakage gate (RULESET-10)
#   7. translation hygiene — markers carry, no `!`, quote style
#   8. struct tree shape — divisions nest under their book (depth-0 root)
#   9. division content — no empty division; content inside its declared span
#
# Usage: bash scripts/plato1_verify.sh
set -uo pipefail
cd "$(dirname "$0")/.."

DIR=assets/plato1/curated/md_modernized
FAIL=0

echo "== gates 1-3: structure, front matter, markers, hygiene =="
python3 - <<'PYEOF' || FAIL=1
import os, re, sys, unicodedata

DIR = "assets/plato1/curated/md_modernized"

# Oracle: parse the Rust TOC as TEXT, not through the crate, so a converter and
# common regressing together still has to get past this file.
rs = open("packages/common/src/plato1/toc.rs", encoding="utf-8").read()
entries = re.findall(
    r'FlatEntry \{\s*stephanus: "([^"]+)",\s*depth: (\d+),\s*label: "((?:[^"\\]|\\.)*)",',
    rs,
)
if len(entries) != 116:
    sys.exit(f"oracle parsed {len(entries)} TOC entries, expected 116")


def slugify(label):
    out = []
    for ch in label:
        if ch.isascii() and ch.isalnum():
            out.append(ch.lower())
        elif ch in "'’":
            continue
        elif not (out and out[-1] == "_"):
            out.append("_")
    return "".join(out).strip("_")


# A division's filename carries its book, so the ten stay grouped in a flat
# directory; a book node needs no prefix.
expected, book = {}, None
for i, (steph, depth, label) in enumerate(entries):
    label = label.replace('\\"', '"')
    if int(depth) == 0:
        # `Book I` -> `bk_i`: every one of the 116 filenames carries it.
        book = slugify(label).replace("book_", "bk_")
        name = f"{i + 1:03}_{book}.md"
    else:
        name = f"{i + 1:03}_{book}_{slugify(label)}.md"
    expected[name] = (i + 1, label, int(depth), steph)

ok = True
files = sorted(f for f in os.listdir(DIR) if f.endswith(".md"))
for f in set(files) - set(expected):
    print(f"  UNEXPECTED FILE {f}"); ok = False
for f in sorted(set(expected) - set(files)):
    print(f"  MISSING FILE {f}"); ok = False

markers, depth0_with_marker, hygiene = [], [], []
for fname in files:
    if fname not in expected:
        continue
    pos, label, depth, steph = expected[fname]
    raw = open(os.path.join(DIR, fname), encoding="utf-8").read()
    fm = dict(re.findall(r"^(\w+):\s*(.*)$", raw.split("---")[1], re.M))
    got = (
        int(fm.get("position", -1)),
        fm.get("label", "").strip().strip('"'),
        int(fm.get("depth", -1)),
        fm.get("page_stephanus", "").strip().strip('"'),
    )
    if got != (pos, label, depth, steph):
        print(f"  FRONT MATTER {fname}: {got} != {(pos, label, depth, steph)}"); ok = False

    body = raw.split("---", 2)[-1]
    found = re.findall(r"\{\{\{\s*([^}]+?)\s*\}\}\}", body)
    markers += found
    if depth == 0 and found:
        depth0_with_marker.append(fname)
    # A `{{ }}` would route to a second marker system this corpus does not have.
    if re.search(r"(?<!\{)\{\{(?!\{)", body):
        hygiene.append(f"  SECONDARY MARKER in {fname}"); ok = False
    if "ʼ" in body:
        hygiene.append(f"  U+02BC (unnormalised elision) in {fname}"); ok = False
    if body.count("{") != body.count("}"):
        hygiene.append(f"  UNBALANCED BRACES in {fname}"); ok = False
    for line in body.splitlines():
        if line.startswith(">"):
            print(f"  NOTE blockquote-leading '>' in {fname}: {line[:60]!r}")

for h in hygiene:
    print(h)
for f in depth0_with_marker:
    print(f"  MARKER ON BOOK NODE {f} — markers belong to divisions"); ok = False

if len(markers) != 1355:
    print(f"  MARKER COUNT {len(markers)} != 1355"); ok = False
if len(set(markers)) != len(markers):
    dupes = [m for m in set(markers) if markers.count(m) > 1]
    print(f"  DUPLICATE MARKERS {dupes[:5]}"); ok = False
key = lambda r: (int(re.match(r"\d+", r).group()), r)
if any(key(a) >= key(b) for a, b in zip(markers, markers[1:])):
    print("  MARKERS NOT STRICTLY ASCENDING"); ok = False

print(f"  {len(files)} files, {len(markers)} markers, front matter matches TOC"
      if ok else "  gate failed")
sys.exit(0 if ok else 1)
PYEOF

echo "== gate 4: golden values =="
if [ ! -f assets/plato1/derived/output.json ]; then
    echo "  derived/output.json absent — run: cargo run -p md_prose_to_struct -- --corpus plato1"
    FAIL=1
else
python3 - <<'PYEOF' || FAIL=1
import json, re, statistics, sys

golden = {}
for line in open("assets/plato1/GOLDEN.md", encoding="utf-8"):
    m = re.match(r"\|\s*([^|]+?)\s*\|\s*(\d+)\s*\|", line)
    if m and not line.startswith("|---"):
        golden[m.group(1)] = int(m.group(2))

o = json.load(open("assets/plato1/derived/output.json", encoding="utf-8"))
nodes = o["toc_nodes"]
books, cur = {}, None
for n in nodes:
    if n["depth"] == 0:
        cur = n["label"]; books[cur] = 0
    books[cur] += sum(len(b.get("sentences") or []) for b in (n.get("content_blocks") or []))
lens = [len(s["text"].split()) for n in nodes
        for b in (n.get("content_blocks") or []) for s in (b.get("sentences") or [])]

ok = True
def check(name, got):
    global ok
    want = golden.get(name)
    if want is None:
        return
    if got != want:
        print(f"  {name}: {got} != golden {want}"); ok = False

check("toc nodes", len(nodes))
check("total sentences", sum(books.values()))
check("max words/sentence", max(lens))
for book, n in books.items():
    check(book, n)

median = statistics.median(lens)
# The real regression detector: a Latin-script splitter finds ~2% of Greek
# boundaries silently, and the median jumps to paragraph length.
if not 5 <= median <= 20:
    print(f"  MEDIAN {median} words/sentence outside 5-20 — splitter regression?"); ok = False

print(f"  {sum(books.values())} sentences, median {median:.0f}, max {max(lens)} words"
      if ok else "  gate failed")
sys.exit(0 if ok else 1)
PYEOF
fi

echo "== gate 5: terminology =="
python3 scripts/plato1_terminology_gate.py || FAIL=1
python3 scripts/plato1_terminology_gate.py --self-test >/dev/null || { echo "  terminology self-test BROKEN"; FAIL=1; }

echo "== gate 6: leakage =="
python3 scripts/plato1_leakage_gate.py || FAIL=1
python3 scripts/plato1_leakage_gate.py --self-test >/dev/null || { echo "  leakage self-test BROKEN"; FAIL=1; }

echo "== gate 7: translation hygiene =="
python3 - <<'PYEOF' || FAIL=1
import os, re, sys

G = "assets/plato1/curated/md_modernized"
T = "assets/plato1/curated/md_modernized_translated"
if not os.path.isdir(T) or not os.listdir(T):
    print("  no translation yet — skipped")
    sys.exit(0)

MARK = re.compile(r"\{\{\{\s*([^}]+?)\s*\}\}\}")
ok = True
checked = 0
for name in sorted(os.listdir(T)):
    if not name.endswith(".md"):
        continue
    tr = open(os.path.join(T, name), encoding="utf-8").read()
    src = open(os.path.join(G, name), encoding="utf-8").read()
    tb, sb = tr.split("---", 2)[-1], src.split("---", 2)[-1]
    checked += 1

    # Markers must carry across exactly. Sentence parity would not catch a
    # dropped marker — it would simply vanish from the corpus.
    if MARK.findall(tb) != MARK.findall(sb):
        print(f"  MARKER MISMATCH {name}: {len(MARK.findall(tb))} vs source {len(MARK.findall(sb))}")
        ok = False

    # `!` is a hard terminator for the English splitter and Greek has none, so
    # it splits the English where the Greek does not.
    if "!" in tb:
        print(f"  EXCLAMATION MARK in {name} — splits where the Greek does not")
        ok = False

    # Ordinary dialogue turns are unquoted (the Greek has no quotation marks);
    # double quotes are reserved for genuinely nested speech. A file far above
    # the corpus norm is quoting ordinary turns.
    words = len(tb.split())
    rate = 1000 * tb.count('"') / max(words, 1)
    if rate > 20:
        print(f"  QUOTE DENSITY {name}: {rate:.0f}/1k — quoting ordinary turns?")
        ok = False

    # A single quote next to a terminator silently defeats the split.
    if re.search(r"['\u2019]\s*[.?]\s+[A-Z]", tb):
        print(f"  SINGLE QUOTE at a sentence boundary in {name}")
        ok = False

print(f"  {checked} translated files: markers carry, no `!`, quote style consistent"
      if ok else "  gate failed")
sys.exit(0 if ok else 1)
PYEOF

echo "== gate 8: struct tree shape =="
python3 - <<'PYEOF' || FAIL=1
import json, os, sys

OUT = "assets/plato1/derived/output.json"
if not os.path.exists(OUT):
    print("  no derived output.json — run `just struct plato1` first")
    sys.exit(1)

nodes = json.load(open(OUT, encoding="utf-8"))["toc_nodes"]
ok = True

# plato1 roots its tree at depth 0. A depth-1 division must carry a dotted
# ltree path under its book AND a parent_source_ref; when the shared path
# builder assumed a depth-1 root, both silently came out flat and the reader
# would have shown 116 sibling nodes instead of ten books.
flat = [n for n in nodes if n["depth"] == 1 and "." not in n["path"]]
orphan = [n for n in nodes if n["depth"] == 1 and not n.get("parent_source_ref")]
roots = [n for n in nodes if n["depth"] == 0]

if flat:
    print(f"  {len(flat)} depth-1 nodes have a flat path (first: {flat[0]['path']})")
    ok = False
if orphan:
    print(f"  {len(orphan)} depth-1 nodes have no parent_source_ref")
    ok = False
if len(roots) != 10:
    print(f"  expected 10 depth-0 books, found {len(roots)}")
    ok = False
print(f"  {len(roots)} books, {len(nodes) - len(roots)} divisions, every division parented"
      if ok else "  gate failed")
sys.exit(0 if ok else 1)
PYEOF

echo "== gate 9: division content and span =="
python3 - <<'PYEOF' || FAIL=1
import os, re, sys

D = "assets/plato1/curated/md_modernized"

def key(ref):
    m = re.match(r"(\d+)([a-e])$", ref)
    return int(m.group(1)) * 10 + "abcde".index(m.group(2)) if m else None

backward_snap = set(re.findall(
    r'"(\d+[a-e])"',
    (re.search(r"BACKWARD_SNAP[^=]*=\s*&\[(.*?)\];",
               open("packages/common/src/plato1/toc.rs", encoding="utf-8").read(),
               re.S) or type("", (), {"group": lambda *a: ""})()).group(1)))

rows = []
for f in sorted(os.listdir(D)):
    t = open(os.path.join(D, f), encoding="utf-8").read()
    ps = re.search(r'^page_stephanus:\s*"([^"]+)"', t, re.M)
    depth = re.search(r"^depth:\s*(\d+)", t, re.M)
    if not ps or not depth:
        continue
    rows.append((f, int(depth.group(1)), ps.group(1),
                 re.findall(r"\{\{\{\s*([^}]+?)\s*\}\}\}", t)))

ok = True
divisions = [r for r in rows if r[1] == 1]

# A division that carries no text is a table-of-contents entry opening onto a
# blank page. Its text has not been lost — it is sitting in a neighbour — so
# nothing else in the pipeline notices.
for f, _, ps, marks in divisions:
    if not marks:
        print(f"  EMPTY division {f} (declared {ps})")
        ok = False

# Content must fall inside the span the TOC declares for it. A division may
# open partway through a section, so its first marker legitimately follows the
# declared start; it may never precede it, nor run past the next division.
starts = [key(r[2]) for r in rows]
for i, (f, d, ps, marks) in enumerate(rows):
    if d != 1 or not marks:
        continue
    lo = key(ps)
    nxt = next((s for s in starts[i + 1:] if s is not None), None)
    first, last = key(marks[0]), key(marks[-1])
    if None in (lo, first, last):
        continue
    # A division whose boundary was snapped BACKWARD legitimately opens before
    # its declared page: the break was moved back to the start of the sentence
    # it fell inside. toc.rs::BACKWARD_SNAP is the authority on which ones.
    if first < lo and ps not in backward_snap:
        print(f"  {f}: first marker {marks[0]} precedes declared start {ps}")
        ok = False
    # The section a division opens in the middle of has its marker emitted in
    # the PREVIOUS file, so last == next declared start is normal. Only
    # overrunning it means content is misfiled.
    if nxt is not None and last > nxt:
        print(f"  {f}: last marker {marks[-1]} overruns the next division's start")
        ok = False

print(f"  {len(divisions)} divisions, all non-empty and within their declared span"
      if ok else "  gate failed")
sys.exit(0 if ok else 1)
PYEOF

echo
if [ "$FAIL" -eq 0 ]; then echo "plato1: all gates pass"; else echo "plato1: FAILURES above"; fi
exit "$FAIL"
