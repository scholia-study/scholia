#!/usr/bin/env bash
# Verify the plato2 curated Greek and, when it exists, its English translation.
#
# Gates:
#   1. file set + front matter — every TOC node has a file; position, label,
#      depth and page_stephanus match the checked-in table exactly
#   2. Stephanus markers — 404, each exactly once, strictly ascending, and
#      never on a depth-0 conversation node
#   3. markup hygiene — no `{{ }}` secondary marker (this corpus has no second
#      system), no U+02BC, balanced braces
#   4. speaker coverage — every division opens on a speaker, no dialogue is
#      orphaned from one, and the five speakers appear in the source's counts
#   5. golden values — sentence counts and the length distribution that would
#      collapse if the Greek splitter ever regressed
#   6. struct tree shape — divisions nest under their conversation (depth-0 root)
#   7. division content — no empty division; content inside its declared span
#   8. translation hygiene — markers and speakers carry, no `!`, quote style
#
# Usage: bash scripts/plato2_verify.sh
set -uo pipefail
cd "$(dirname "$0")/.."

FAIL=0

echo "== gates 1-4: structure, front matter, markers, hygiene, speakers =="
python3 - <<'PYEOF' || FAIL=1
import os, re, sys

DIR = "assets/plato2/curated/md_modernized"

# Oracle: parse the Rust TOC as TEXT, not through the crate, so a converter and
# common regressing together still has to get past this file.
rs = open("packages/common/src/plato2/toc.rs", encoding="utf-8").read()
entries = re.findall(
    r'FlatEntry \{\s*stephanus: "([^"]+)",\s*depth: (\d+),\s*label: "((?:[^"\\]|\\.)*)",',
    rs,
)
if len(entries) != 22:
    sys.exit(f"oracle parsed {len(entries)} TOC entries, expected 22")


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


# A division's filename carries its conversation, so the three stay grouped in a
# flat directory; a conversation node needs no prefix.
expected, conv = {}, None
for i, (steph, depth, label) in enumerate(entries):
    label = label.replace('\\"', '"')
    if int(depth) == 0:
        conv = slugify(label).removeprefix("the_conversation_with_")
        name = f"{i + 1:03}_{conv}.md"
    else:
        name = f"{i + 1:03}_{conv}_{slugify(label)}.md"
    expected[name] = (i + 1, label, int(depth), steph)

ok = True
files = sorted(f for f in os.listdir(DIR) if f.endswith(".md")) if os.path.isdir(DIR) else []
if not files:
    sys.exit(f"  no curated files in {DIR} — run the converter first")
for f in sorted(set(files) - set(expected)):
    print(f"  UNEXPECTED FILE {f}"); ok = False
for f in sorted(set(expected) - set(files)):
    print(f"  MISSING FILE {f}"); ok = False

SPEAKERS = {"Σωκράτης": 527, "Καλλίκλης": 236, "Πῶλος": 207, "Γοργίας": 97, "Χαιρεφῶν": 14}
markers, depth0_with_marker, hygiene = [], [], []
speaker_counts, orphans, headless = {}, [], []
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

    # Speaker coverage. A division opens at a genuine speech turn, so the first
    # block after its heading must be a speaker line; dialogue before any
    # speaker line would reach the reader unattributed.
    blocks = [b.strip() for b in re.split(r"\n\s*\n", body) if b.strip()]
    blocks = [b for b in blocks if not b.startswith("## ")]
    if depth == 1:
        if not blocks:
            continue
        if not blocks[0].startswith("@ "):
            headless.append(fname); ok = False
        seen_speaker = False
        for b in blocks:
            if b.startswith("@ "):
                seen_speaker = True
                who = b[2:].strip()
                speaker_counts[who] = speaker_counts.get(who, 0) + 1
            elif not seen_speaker:
                orphans.append(f"{fname}: {b[:48]!r}"); ok = False
    elif blocks:
        print(f"  CONVERSATION NODE {fname} carries body content"); ok = False

for h in hygiene:
    print(h)
for f in depth0_with_marker:
    print(f"  MARKER ON CONVERSATION NODE {f} — markers belong to divisions"); ok = False
for f in headless:
    print(f"  DIVISION DOES NOT OPEN ON A SPEAKER: {f}")
for o in orphans[:5]:
    print(f"  DIALOGUE BEFORE ANY SPEAKER in {o}")

if len(markers) != 404:
    print(f"  MARKER COUNT {len(markers)} != 404"); ok = False
if len(set(markers)) != len(markers):
    dupes = sorted({m for m in markers if markers.count(m) > 1})
    print(f"  DUPLICATE MARKERS {dupes[:5]}"); ok = False
key = lambda r: (int(re.match(r"\d+", r).group()), r)
if any(key(a) >= key(b) for a, b in zip(markers, markers[1:])):
    print("  MARKERS NOT STRICTLY ASCENDING"); ok = False
# Burnet's page 447 ends at d. 404 sections, not the 405 that 81x5 would imply.
if "447e" in markers:
    print("  447e EMITTED — Burnet's page 447 ends at d"); ok = False

if speaker_counts != SPEAKERS:
    print(f"  SPEAKER COUNTS {speaker_counts} != {SPEAKERS}"); ok = False

total_speakers = sum(speaker_counts.values())
print(f"  {len(files)} files, {len(markers)} markers, {total_speakers} speech turns, front matter matches TOC"
      if ok else "  gate failed")
sys.exit(0 if ok else 1)
PYEOF

echo "== gates 5-7: golden values, tree shape, division span =="
if [ ! -f assets/plato2/derived/output.json ]; then
    echo "  derived/output.json absent — run: just struct plato2"
    FAIL=1
else
python3 - <<'PYEOF' || FAIL=1
import json, os, re, statistics, sys

o = json.load(open("assets/plato2/derived/output.json", encoding="utf-8"))
nodes = o["toc_nodes"]
ok = True

roots = [n for n in nodes if n["depth"] == 0]
flat = [n for n in nodes if n["depth"] == 1 and "." not in n["path"]]
orphan = [n for n in nodes if n["depth"] == 1 and not n.get("parent_source_ref")]
if len(roots) != 3:
    print(f"  expected 3 depth-0 conversations, found {len(roots)}"); ok = False
if flat:
    print(f"  {len(flat)} depth-1 nodes have a flat path (first: {flat[0]['path']})"); ok = False
if orphan:
    print(f"  {len(orphan)} depth-1 nodes have no parent_source_ref"); ok = False

lens = [len(s["text"].split()) for n in nodes
        for b in (n.get("content_blocks") or []) for s in (b.get("sentences") or [])
        if b.get("block_type") not in ("speaker", "heading")]
sentences = len(lens)

golden_path = "assets/plato2/GOLDEN.md"
if os.path.exists(golden_path):
    golden = {}
    for line in open(golden_path, encoding="utf-8"):
        m = re.match(r"\|\s*([^|]+?)\s*\|\s*(\d+)\s*\|", line)
        if m and not line.startswith("|---"):
            golden[m.group(1)] = int(m.group(2))
    for name, got in (("toc nodes", len(nodes)), ("total sentences", sentences),
                      ("max words/sentence", max(lens) if lens else 0)):
        want = golden.get(name)
        if want is not None and got != want:
            print(f"  {name}: {got} != golden {want}"); ok = False
else:
    print(f"  {golden_path} absent — record the goldens once the corpus is settled")

median = statistics.median(lens) if lens else 0
# The real regression detector: a Latin-script splitter finds ~2% of Greek
# boundaries silently, and the median jumps to paragraph length.
if not 5 <= median <= 20:
    print(f"  MEDIAN {median} words/sentence outside 5-20 — splitter regression?"); ok = False

print(f"  {len(roots)} conversations, {len(nodes) - len(roots)} divisions, "
      f"{sentences} sentences, median {median:.0f}, max {max(lens) if lens else 0} words"
      if ok else "  gate failed")
sys.exit(0 if ok else 1)
PYEOF
fi

echo "== gate 7b: division content and span =="
python3 - <<'PYEOF' || FAIL=1
import os, re, sys

D = "assets/plato2/curated/md_modernized"

def key(ref):
    m = re.match(r"(\d+)([a-e])$", ref)
    return int(m.group(1)) * 10 + "abcde".index(m.group(2)) if m else None

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
if len(divisions) != 19:
    print(f"  {len(divisions)} divisions found, expected 19")
    sys.exit(1)

# A division that carries no text is a table-of-contents entry opening onto a
# blank page. Its text has not been lost — it is sitting in a neighbour — so
# nothing else in the pipeline notices.
for f, _, ps, marks in divisions:
    if not marks:
        print(f"  EMPTY division {f} (declared {ps})"); ok = False

# Content must fall inside the span the TOC declares for it. A division may open
# partway through a section — its opening marker was emitted in the previous
# file — so its first marker legitimately follows the declared start; it may
# never precede it, nor run past the next division.
starts = [key(r[2]) for r in rows]
for i, (f, d, ps, marks) in enumerate(rows):
    if d != 1 or not marks:
        continue
    lo = key(ps)
    nxt = next((s for s in starts[i + 1:] if s is not None), None)
    first, last = key(marks[0]), key(marks[-1])
    if None in (lo, first, last):
        continue
    if first < lo:
        print(f"  {f}: first marker {marks[0]} precedes declared start {ps}"); ok = False
    if nxt is not None and last > nxt:
        print(f"  {f}: last marker {marks[-1]} overruns the next division's start"); ok = False

print(f"  {len(divisions)} divisions, all non-empty and within their declared span"
      if ok else "  gate failed")
sys.exit(0 if ok else 1)
PYEOF

echo "== gate 8: translation hygiene =="
python3 - <<'PYEOF' || FAIL=1
import os, re, sys

G = "assets/plato2/curated/md_modernized"
T = "assets/plato2/curated/md_modernized_translated"
if not os.path.isdir(T) or not [f for f in os.listdir(T) if f.endswith(".md")]:
    print("  no translation yet — skipped")
    sys.exit(0)

MARK = re.compile(r"\{\{\{\s*([^}]+?)\s*\}\}\}")
EN = {"Σωκράτης": "Socrates", "Καλλίκλης": "Callicles", "Πῶλος": "Polus",
      "Γοργίας": "Gorgias", "Χαιρεφῶν": "Chaerephon"}
ok = True
checked = 0
for name in sorted(os.listdir(T)):
    if not name.endswith(".md"):
        continue
    tb = open(os.path.join(T, name), encoding="utf-8").read().split("---", 2)[-1]
    sb = open(os.path.join(G, name), encoding="utf-8").read().split("---", 2)[-1]
    checked += 1

    # Block parity fails before sentence parity does and is exact, so it is
    # checked here; per-sentence parity is the importer's job, which owns the
    # real splitters and must not be second-guessed by a reimplementation.
    gblocks = [b for b in re.split(r"\n\s*\n", sb) if b.strip()]
    tblocks = [b for b in re.split(r"\n\s*\n", tb) if b.strip()]
    if len(gblocks) != len(tblocks):
        print(f"  BLOCK COUNT {name}: {len(tblocks)} vs source {len(gblocks)}")
        ok = False

    # Markers must carry across exactly. Sentence parity would not catch a
    # dropped marker — it would simply vanish from the corpus.
    if MARK.findall(tb) != MARK.findall(sb):
        print(f"  MARKER MISMATCH {name}: {len(MARK.findall(tb))} vs source {len(MARK.findall(sb))}")
        ok = False

    # Speakers must pair one-to-one, in order, in their conventional English.
    src_who = re.findall(r"^@ (.+)$", sb, re.M)
    tr_who = re.findall(r"^@ (.+)$", tb, re.M)
    if [EN.get(w.strip(), w.strip()) for w in src_who] != [w.strip() for w in tr_who]:
        print(f"  SPEAKER MISMATCH {name}: {len(tr_who)} lines vs source {len(src_who)}")
        ok = False

    # An ellipsis terminates for the English splitter but not the Greek one, so
    # it is legitimate only where Burnet's one lost gap puts it in the source.
    if tb.count("\u2026") != sb.count("\u2026"):
        print(f"  ELLIPSIS COUNT {name}: {tb.count(chr(0x2026))} vs source {sb.count(chr(0x2026))}")
        ok = False

    # `!` is a hard terminator for the English splitter and Greek has none, so
    # it splits the English where the Greek does not.
    if "!" in tb:
        print(f"  EXCLAMATION MARK in {name} — splits where the Greek does not")
        ok = False

    # A closing quote or bracket between a terminator and its space defeats the
    # splitter (`[.!?…]+\s+` needs the whitespace to follow the terminator
    # directly), silently merging two English sentences into one.
    swallowed = re.findall(r"[.?\u2026][\"'\u201d\u2019\)\]]+\s", tb)
    if swallowed:
        print(f"  SWALLOWED BOUNDARY in {name}: {len(swallowed)} terminator(s) followed by a closing mark")
        ok = False

    # Ordinary dialogue turns are unquoted (the Greek has no quotation marks);
    # double quotes are reserved for genuinely nested speech.
    words = len(tb.split())
    rate = 1000 * tb.count('"') / max(words, 1)
    if rate > 20:
        print(f"  QUOTE DENSITY {name}: {rate:.0f}/1k — quoting ordinary turns?")
        ok = False

    # An abbreviation period inside a footnote splits it in two: the Greek
    # splitter has no abbreviation filter. Expanded citation form is mandatory.
    for fn in re.findall(r"^\[\^[^\]]+\]:\s*(.+)$", tb, re.M):
        if re.search(r"\b(?:Hom|Od|Il|Pind|fr|Eur|Aesch|Soph|Hes|cf|ed|vol|p|pp|ff)\.\s", fn):
            print(f"  ABBREVIATED CITATION in {name}: {fn[:50]!r} — expand it")
            ok = False

print(f"  {checked} translated files: markers and speakers carry, no `!`, quote style consistent"
      if ok else "  gate failed")
sys.exit(0 if ok else 1)
PYEOF

echo
if [ "$FAIL" -eq 0 ]; then echo "plato2: all gates pass"; else echo "plato2: FAILURES above"; fi
exit "$FAIL"
