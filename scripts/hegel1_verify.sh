#!/usr/bin/env bash
#
# Verify the hegel1 diplomatic layer — assets/hegel1/curated/md_reviewed/ —
# against the DTA linguistic TEI and the authoritative 50-node TOC table.
#
# Seven gates: file set, front matter, paragraph counts, page-marker sequence,
# TEI leakage, failed hyphen rejoins, long-s survival. Each prints PASS/FAIL;
# any failure exits non-zero.
#
# The TOC table is embedded below rather than read from packages/common on
# purpose: an oracle that shares its source of truth with the converter it
# checks only proves self-consistency. --toc swaps in another table.
#
# Usage: hegel1_verify.sh [--md <dir>] [--tei <xml>] [--toc <tsv>]
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"

md="$root/assets/hegel1/curated/md_reviewed"
tei="$root/assets/hegel1/raw/hegel_phaenomenologie_1807_TEI_P5_inkl_att_linguistic.xml"
toc=""

usage() {
    echo "usage: hegel1_verify.sh [--md <dir>] [--tei <xml>] [--toc <tsv>]"
}

while [ $# -gt 0 ]; do
    case "$1" in
        --md) md="${2:?--md needs a path}"; shift 2 ;;
        --tei) tei="${2:?--tei needs a path}"; shift 2 ;;
        --toc) toc="${2:?--toc needs a path}"; shift 2 ;;
        -h | --help) usage; exit 0 ;;
        *) echo "error: unknown argument '$1'" >&2; usage >&2; exit 1 ;;
    esac
done

if ! command -v python3 >/dev/null 2>&1; then
    echo "error: python3 not on PATH — hegel1_verify.sh needs it for the XML gates" >&2
    exit 1
fi

if [ ! -d "$md" ] || [ -z "$(find "$md" -maxdepth 1 -name '*.md' -print -quit)" ]; then
    echo "hegel1_verify: output not generated yet — no markdown under" >&2
    echo "  $md" >&2
    echo "run the hegel1 TEI-to-Markdown converter first, then re-run this script." >&2
    exit 1
fi

if [ ! -f "$tei" ]; then
    echo "hegel1_verify: source TEI missing — no file at" >&2
    echo "  $tei" >&2
    echo "fetch it from https://www.deutschestextarchiv.de/hegel_phaenomenologie_1807" >&2
    exit 1
fi

if [ -n "$toc" ] && [ ! -f "$toc" ]; then
    echo "error: --toc file not found: $toc" >&2
    exit 1
fi

HEGEL1_MD="$md" HEGEL1_TEI="$tei" HEGEL1_TOC="$toc" python3 - <<'PY'
import os
import re
import sys
import xml.etree.ElementTree as ET
from bisect import bisect_right
from pathlib import Path

MD = Path(os.environ["HEGEL1_MD"])
TEI = Path(os.environ["HEGEL1_TEI"])
TOC_OVERRIDE = os.environ.get("HEGEL1_TOC") or ""

TOC_TSV = """\
pos	depth	page	paras	supplied	slug	label
1	1		72	no	vorrede	Vorrede
2	1	XCI	0	no	erster_theil_wissenschaft_der_erfahrung_des_bewusstseyns	Erster Theil. Wissenschaft der Erfahrung des Bewuſstseyns
3	2	XCI	17	yes	einleitung	Einleitung
4	2	22	21	no	i_die_sinnliche_gewissheit_oder_das_diese_und_das_meynen	I. Die sinnliche Gewiſsheit; oder das Diese und das Meynen
5	2	38	21	no	ii_die_wahrnehmung_oder_das_ding_und_die_taeuschung	II. Die Wahrnehmung; oder das Ding, und die Täuschung
6	2	59	34	no	iii_krafft_und_verstand_erscheinung_und_uebersinnliche_welt	III. Krafft und Verstand, Erscheinung und übersinnliche Welt
7	2	101	12	no	iv_die_wahrheit_der_gewissheit_seiner_selbst	IV. Die Wahrheit der Gewiſsheit seiner selbst
8	3	114	19	no	a_selbststaendigkeit_und_unselbststaendigkeit_des_selbstbewusstseyns_herrschafft_und_knechtschafft	A. Selbstständigkeit und Unselbstständigkeit des Selbstbewuſstseyns; Herrschafft und Knechtschafft
9	3	129	34	no	b_freyheit_des_selbstbewusstseyns_stoicismus_skepticismus_und_das_unglueckliche_bewusstseyn	B. Freyheit des Selbstbewuſstseyns; Stoicismus, Skepticismus, und das unglückliche Bewuſstseyn
10	2	162	9	no	v_gewissheit_und_wahrheit_der_vernunft	V. Gewiſsheit und Wahrheit der Vernunft
11	3	174	4	no	a_beobachtende_vernunft	A. Beobachtende Vernunft
12	4	177	54	no	a_beobachtung_der_natur	a. Beobachtung der Natur
13	4	234	11	no	b_die_beobachtung_des_selbstbewusstseyns_in_seiner_reinheit_und_seiner_beziehung_auf_aeussre_wirklichkeit_logische_und_psychologische_gesetze	b. Die Beobachtung des Selbstbewuſstseyns in seiner Reinheit und seiner Beziehung auf äuſsre Wirklichkeit; logische und psychologische Gesetze
14	4	243	38	no	c_beobachtung_der_beziehung_des_selbstbewusstseyns_auf_seine_unmittelbare_wirklichkeit_physiognomik_und_schaedellehre	c. Beobachtung der Beziehung des Selbstbewuſstseyns auf seine unmittelbare Wirklichkeit; Physiognomik und Schädellehre
15	3	286	13	no	b_die_verwirklichung_des_vernuenftigen_selbstbewusstseyns_durch_sich_selbst	B. Die Verwirklichung des vernünftigen Selbstbewuſstseyns durch sich selbst
16	4	298	7	no	a_die_lust_und_die_nothwendigkeit	a. Die Luſt und die Nothwendigkeit
17	4	305	14	no	b_das_gesetz_des_herzens_und_der_wahnsinn_des_eigenduenkels	b. Das Gesetz des Herzens, und der Wahnsinn des Eigendünkels
18	4	317	13	no	c_die_tugend_und_der_weltlauff	c. Die Tugend und der Weltlauff
19	3	330	3	no	c_die_individualitaet_welche_sich_an_und_fuer_sich_selbst_reell_ist	C. Die Individualität, welche sich an und für sich selbst reell ist
20	4	333	21	no	a_das_geistige_thierreich_und_der_betrug_oder_die_sache_selbs	a. Das geistige Thierreich und der Betrug, oder die Sache selbs’
21	4	358	10	no	b_die_gesetzgebende_vernunfft	b. Die gesetzgebende Vernunfft
22	4	365	9	no	c_gesetzprueffende_vernunfft	c. Gesetzprüffende Vernunfft
23	2	376	6	no	vi_der_geist	VI. Der Geist
24	3	382	2	no	a_der_wahre_geist_die_sittlichkeit	A. Der wahre Geist, die Sittlichkeit
25	4	383	18	no	a_die_sittliche_welt_das_menschliche_und_goettliche_gesetz_der_mann_und_das_weib	a. Die sittliche Welt, das menschliche und göttliche Gesetz, der Mann und das Weib
26	4	403	13	no	b_die_sittliche_handlung_das_menschliche_und_goettliche_wissen_die_schuld_und_das_schicksal	b. Die sittliche Handlung, das menschliche und göttliche Wissen, die Schuld und das Schicksal
27	4	422	7	no	c_rechtszustand	c. Rechtszustand
28	3	429	3	no	b_der_sich_entfremdete_geist_die_bildung	B. Der sich entfremdete Geist; die Bildung
29	4	434	1	no	i_die_welt_des_sich_entfremdeten_geistes	I. Die Welt des sich entfremdeten Geistes
30	5	435	39	no	a_die_bildung_und_ihr_reich_der_wirklichkeit	a. Die Bildung und ihr Reich der Wirklichkeit
31	5	474	11	no	b_der_glauben_und_die_reine_einsicht	b. Der Glauben und die reine Einsicht
32	4	486	4	no	ii_die_aufklaerung	II. Die Aufklärung
33	5	488	33	no	a_der_kampf_der_aufklaerung_mit_dem_aberglauben	a. Der Kampf der Aufklärung mit dem Aberglauben
34	5	522	8	no	b_die_wahrheit_der_aufklaerung	b. Die Wahrheit der Aufklärung
35	4	533	14	no	iii_die_absolute_freyheit_und_der_schrecken	III. Die abſolute Freyheit und der Schrecken
36	3	548	3	no	c_der_seiner_selbst_gewisse_geist_die_moralitaet	C. Der seiner selbst gewiſse Geist. Die Moralität
37	4	550	17	no	a_die_moralische_weltanschauung	a. Die moralische Weltanschauung
38	4	565	16	no	b_die_verstellung	b. Die Verstellung
39	4	581	40	no	c_das_gewissen_die_schoene_seele_das_boese_und_seine_verzeyhung	c. Das Gewiſſen, die ſchöne Seele, das Böſe und ſeine Verzeyhung
40	2	625	12	no	vii_die_religion	VII. Die Religion
41	3	637	1	no	a_natuerliche_religion	A. Natürliche Religion
42	4	640	4	no	a_das_lichtwesen	a, Das Lichtwesen
43	4	643	2	no	b_die_pflanze_und_das_thier	b. Die Pflanze und das Thier
44	4	645	8	no	c_der_werkmeister	c. Der Werkmeister
45	3	651	6	no	b_die_kunst_religion	B. Die Kunst-Religion
46	4	655	15	no	a_das_abstracte_kunstwerk	a. Das abstracte Kunstwerk
47	4	669	7	no	b_das_lebendige_kunstwerk	b. Das lebendige Kunſtwerk
48	4	676	21	no	c_das_geistige_kunstwerk	c. Das geiſtige Kunſtwerk
49	3	699	40	no	c_die_offenbare_religion	C. Die offenbare Religion
50	2	742	21	no	viii_das_absolute_wissen	VIII. Das abſolute Wiſſen
"""

NS = "{http://www.tei-c.org/ns/1.0}"
NON_CONTENT_DIV = {"contents", "imprint", "advertisement"}
LONG_S = "ſ"
MAX_REPORT = 12

MARKER_RE = re.compile(r"\{\{\{\s*([^{}]+?)\s*\}\}\}")
MARKER_ONLY_RE = re.compile(r"^(?:\{\{\{\s*[^{}]+?\s*\}\}\}[ \t]*\n?)+$")
TEI_TAG_RE = re.compile(
    r"</?(?:lb|pb|hi|w|s|seg|choice|sic|corr|supplied|lg|l|fw|milestone|head"
    r"|div|p|note|unclear|gap|add|del|figure|formula|table|row|cell|list|item"
    r"|space|foreign|persName|placeName|orgName|q|quote|cit|ref|graphic|c|g)"
    r"\b[^>]*/?>",
    re.IGNORECASE,
)
NORM_RE = re.compile(r"@norm\b|\bnorm\s*=")
HYPHEN_SPLIT_RE = re.compile(r"-\s+(\w+)")
# German suspended hyphenation ("allem Hin- und Herreden") is the printed
# reading, not a rejoin that got away: the hanging hyphen is followed by a
# function word, never by a word fragment. DTA tags those first halves
# pos="TRUNC"; the closed list below is that signal seen from the Markdown.
SUSPENDED_TAIL = {
    "und", "oder", "noch", "aber", "sondern", "sowie", "wie", "als", "bis",
    "zum", "zur", "theils",
}


def parse_toc(text):
    lines = [line for line in text.split("\n") if line.strip()]
    header = lines[0].split("\t")
    rows = []
    for line in lines[1:]:
        cells = line.split("\t")
        if len(cells) != len(header):
            sys.exit(f"error: malformed TOC row (expected {len(header)} columns): {line!r}")
        row = dict(zip(header, cells))
        row["pos"] = int(row["pos"])
        row["depth"] = int(row["depth"])
        row["paras"] = int(row["paras"])
        rows.append(row)
    return rows


def content_divs(tei_path):
    text_el = ET.parse(str(tei_path)).getroot().find(f"{NS}text")
    found = []

    def walk(el):
        for child in el:
            if child.tag == f"{NS}div":
                if child.get("type") not in NON_CONTENT_DIV:
                    found.append(child)
                    walk(child)
            else:
                walk(child)

    walk(text_el)
    return text_el, found


def source_pages(text_el, divs):
    inside = set()
    for div in divs:
        for pb in div.iter(f"{NS}pb"):
            inside.add(id(pb))
    return [
        pb.get("n")
        for pb in text_el.iter(f"{NS}pb")
        if pb.get("n") and id(pb) in inside
    ]


def split_front_matter(text):
    lines = text.split("\n")
    if not lines or lines[0].strip() != "---":
        return None, text, -1
    for i in range(1, len(lines)):
        if lines[i].strip() == "---":
            fm = {}
            for line in lines[1:i]:
                key, sep, value = line.partition(":")
                if sep:
                    fm[key.strip()] = value.strip()
            return fm, "\n".join(lines[i + 1:]), i
    return None, text, -1


def unquote(value):
    if len(value) >= 2 and value[0] == '"' and value[-1] == '"':
        return value[1:-1], True
    return value, False


def paragraph_blocks(body):
    blocks = []
    for raw in re.split(r"\n[ \t]*\n", body.strip()):
        block = raw.strip()
        if not block:
            continue
        if block.startswith("## "):
            continue
        if block == "---":
            continue
        if MARKER_ONLY_RE.match(block + "\n"):
            continue
        if all(line.strip().startswith("|") for line in block.split("\n")):
            continue
        blocks.append(block)
    return blocks


def line_index(offsets, position):
    return bisect_right(offsets, position) - 1


rows = parse_toc(Path(TOC_OVERRIDE).read_text(encoding="utf-8") if TOC_OVERRIDE else TOC_TSV)
expected_names = {row["pos"]: f"{row['pos']:03d}_{row['slug']}.md" for row in rows}
actual_names = sorted(p.name for p in MD.glob("*.md"))

print(f"md_reviewed : {MD}")
print(f"source TEI  : {TEI}")
print(f"toc table   : {TOC_OVERRIDE or 'embedded'} ({len(rows)} rows)")
print(f"files found : {len(actual_names)}")
print()

results = []


def record(number, name, ok, details=()):
    results.append((number, name, ok))
    if not ok:
        print(f"gate {number} — {name}")
        for line in list(details)[:MAX_REPORT]:
            print(f"    {line}")
        extra = len(details) - MAX_REPORT
        if extra > 0:
            print(f"    … and {extra} more")
        print()


missing = [name for name in expected_names.values() if name not in set(actual_names)]
unexpected = [name for name in actual_names if name not in set(expected_names.values())]
details = []
if len(actual_names) != len(rows):
    details.append(f"expected {len(rows)} files, found {len(actual_names)}")
details += [f"missing: {name}" for name in missing]
details += [f"unexpected: {name}" for name in unexpected]
record(1, f"file set matches the TOC ({len(rows)} × NNN_slug.md)", not details, details)

loaded = {}
for row in rows:
    path = MD / expected_names[row["pos"]]
    if path.is_file():
        loaded[row["pos"]] = path.read_text(encoding="utf-8")

parsed = {}
fm_details = []
for row in rows:
    pos = row["pos"]
    if pos not in loaded:
        continue
    name = expected_names[pos]
    fm, body, fm_end = split_front_matter(loaded[pos])
    parsed[pos] = (fm, body, fm_end)
    if fm is None:
        fm_details.append(f"{name}: no YAML front matter")
        continue
    if fm.get("position") != str(pos):
        fm_details.append(f"{name}: position is {fm.get('position')!r}, expected {pos}")
    if fm.get("depth") != str(row["depth"]):
        fm_details.append(f"{name}: depth is {fm.get('depth')!r}, expected {row['depth']}")
    label, quoted = unquote(fm.get("label", ""))
    if not quoted:
        fm_details.append(f"{name}: label must be a double-quoted string")
    if label != row["label"]:
        fm_details.append(f"{name}: label is {label!r}, expected {row['label']!r}")
    page = row["page"]
    raw_page = fm.get("page_1807")
    if not page:
        if raw_page is not None:
            fm_details.append(f"{name}: page_1807 present ({raw_page!r}) but this node has no page")
    elif raw_page is None:
        fm_details.append(f"{name}: page_1807 missing, expected {page!r}")
    else:
        value, quoted = unquote(raw_page)
        if value != page:
            fm_details.append(f"{name}: page_1807 is {value!r}, expected {page!r}")
        elif page.isdigit() and quoted:
            fm_details.append(f"{name}: Arabic page_1807 {raw_page} must be a bare number")
        elif not page.isdigit() and not quoted:
            fm_details.append(f"{name}: Roman page_1807 {raw_page} must be a quoted string")
if not parsed:
    fm_details.append("no files to inspect")
record(2, "front matter (position, label, depth, page_1807)", not fm_details, fm_details)

para_details = []
for row in rows:
    pos = row["pos"]
    if pos not in parsed:
        continue
    _, body, _ = parsed[pos]
    count = len(paragraph_blocks(body))
    if count != row["paras"]:
        para_details.append(f"{expected_names[pos]}: {count} paragraphs, expected {row['paras']}")
if not parsed:
    para_details.append("no files to inspect")
record(3, "paragraph count per file matches the TOC", not para_details, para_details)

text_el, divs = content_divs(TEI)
expected_pages = source_pages(text_el, divs)
page_details = []
if len(divs) != len(rows):
    page_details.append(f"source has {len(divs)} content divs, TOC has {len(rows)} rows")
actual_pages = []
for row in rows:
    pos = row["pos"]
    if pos not in parsed:
        page_details.append(f"{expected_names[pos]}: file absent, marker sequence incomplete")
        continue
    _, body, _ = parsed[pos]
    actual_pages += [(pos, m.group(1)) for m in MARKER_RE.finditer(body)]

flat = [value for _, value in actual_pages]
if len(flat) != len(expected_pages):
    page_details.append(f"{len(flat)} page markers emitted, source has {len(expected_pages)} numbered <pb>")
seen = set()
duplicates = [value for value in flat if value in seen or seen.add(value)]
if duplicates:
    page_details.append(f"duplicate markers: {', '.join(dict.fromkeys(duplicates))}")
absent = [value for value in expected_pages if value not in set(flat)]
if absent:
    page_details.append(f"missing from output: {', '.join(absent)}")
stray = [value for value in flat if value not in set(expected_pages)]
if stray:
    page_details.append(f"not in source: {', '.join(dict.fromkeys(stray))}")
for i, (got, want) in enumerate(zip(flat, expected_pages)):
    if got != want:
        pos = actual_pages[i][0]
        page_details.append(
            f"order breaks at marker #{i + 1} in {expected_names[pos]}: "
            f"got {got!r}, source has {want!r}"
        )
        page_details.append(f"  output : {' '.join(flat[max(0, i - 3):i + 4])}")
        page_details.append(f"  source : {' '.join(expected_pages[max(0, i - 3):i + 4])}")
        break
record(4, f"page markers reproduce the source <pb n> sequence ({len(expected_pages)})", not page_details, page_details)

leak_details = []
for row in rows:
    pos = row["pos"]
    if pos not in loaded:
        continue
    name = expected_names[pos]
    for number, line in enumerate(loaded[pos].split("\n"), start=1):
        for match in TEI_TAG_RE.finditer(line):
            leak_details.append(f"{name}:{number}: TEI tag {match.group(0)!r}")
        for match in NORM_RE.finditer(line):
            leak_details.append(f"{name}:{number}: @norm leak {line.strip()[:70]!r}")
if not loaded:
    leak_details.append("no files to inspect")
record(5, "no <lb/>, raw TEI tags or @norm values in the Markdown", not leak_details, leak_details)

# The heading and the front-matter label are exempt: gate 2 already pins them
# byte-for-byte against the TOC, so a defect there surfaces as a gate-2 failure
# rather than being counted twice here.
hyphen_details = []
suspended = 0
for row in rows:
    pos = row["pos"]
    if pos not in parsed:
        continue
    name = expected_names[pos]
    text = loaded[pos]
    _, _, fm_end = parsed[pos]
    lines = text.split("\n")
    offsets = []
    cursor = 0
    for line in lines:
        offsets.append(cursor)
        cursor += len(line) + 1
    for match in HYPHEN_SPLIT_RE.finditer(text):
        word = match.group(1)
        if not word[0].islower():
            continue
        i = line_index(offsets, match.start())
        if i <= fm_end or lines[i].strip() == "---" or lines[i].lstrip().startswith("## "):
            continue
        if word.lower() in SUSPENDED_TAIL:
            suspended += 1
            continue
        start = max(0, match.start() - 40)
        hyphen_details.append(f"{name}:{i + 1}: …{text[start:match.end() + 20]}…")
if not parsed:
    hyphen_details.append("no files to inspect")
record(
    6,
    f"no failed hyphen rejoins ({suspended} suspended compounds allowed)",
    not hyphen_details,
    hyphen_details,
)

long_s_total = sum(text.count(LONG_S) for text in loaded.values())
long_s_files = sum(1 for text in loaded.values() if LONG_S in text)
long_s_details = []
if not loaded:
    long_s_details.append("no files to inspect")
elif long_s_total == 0:
    long_s_details.append(
        "no long-s anywhere in md_reviewed — the diplomatic layer has been modernized"
    )
record(
    7,
    f"long-s preserved ({long_s_total} in {long_s_files} files)",
    not long_s_details,
    long_s_details,
)

width = max(len(name) for _, name, _ in results)
print("gate  check" + " " * (width - 5) + "result")
for number, name, ok in results:
    print(f"  {number}   {name.ljust(width)}  {'PASS' if ok else 'FAIL'}")
print()

failed = [number for number, _, ok in results if not ok]
if failed:
    print(f"hegel1_verify: FAILED ({len(failed)} of {len(results)} gates: {', '.join(map(str, failed))})")
    sys.exit(1)
print(f"hegel1_verify: OK ({len(results)} gates)")
PY
