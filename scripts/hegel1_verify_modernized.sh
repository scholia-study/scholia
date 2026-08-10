#!/usr/bin/env bash
#
# Verify the hegel1 reading layer — assets/hegel1/curated/md_modernized/ —
# against the diplomatic layer it must stay 1:1 with (PLAN_HEGEL1.md, layer-2
# gates).
#
# Six gates: file set, front-matter identity (label field excepted), the
# modernized labels against an embedded copy of the toc_mod table, paragraph
# counts, page-marker sequence, and orthography (no long-s or ey anywhere,
# headings included). --regen rebuilds the layer with the converter +
# decision table + ops table and requires a byte-identical result — the
# reproducibility gate. The completeness gates (every residual pair ruled,
# every ss token ruled) are enforced by the converter itself, which refuses
# to emit otherwise.
#
# Usage: hegel1_verify_modernized.sh [--md <dir>] [--reviewed <dir>] [--regen]
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"

md="$root/assets/hegel1/curated/md_modernized"
reviewed="$root/assets/hegel1/curated/md_reviewed"
regen=""

usage() {
    echo "usage: hegel1_verify_modernized.sh [--md <dir>] [--reviewed <dir>] [--regen]"
}

while [ $# -gt 0 ]; do
    case "$1" in
        --md) md="${2:?--md needs a path}"; shift 2 ;;
        --reviewed) reviewed="${2:?--reviewed needs a path}"; shift 2 ;;
        --regen) regen=1; shift ;;
        -h | --help) usage; exit 0 ;;
        *) echo "error: unknown argument '$1'" >&2; usage >&2; exit 1 ;;
    esac
done

for dir in "$md" "$reviewed"; do
    if [ ! -d "$dir" ] || [ -z "$(find "$dir" -maxdepth 1 -name '*.md' -print -quit)" ]; then
        echo "hegel1_verify_modernized: no markdown under $dir" >&2
        exit 1
    fi
done

if [ -n "$regen" ]; then
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    (cd "$root" && cargo run -q -p hegel1_tei_to_md -- \
        assets/hegel1/raw/hegel_phaenomenologie_1807_TEI_P5_inkl_att_linguistic.xml \
        "$tmp" --layer modernized \
        --ops assets/hegel1/curated/modernize_ops.tsv >/dev/null)
    if diff -rq "$md" "$tmp" >/dev/null; then
        echo "regen: byte-identical PASS"
    else
        echo "regen: differs from converter output FAIL" >&2
        diff -rq "$md" "$tmp" >&2 || true
        exit 1
    fi
fi

MD_DIR="$md" REVIEWED_DIR="$reviewed" python3 - <<'PY'
import os
import re
import sys

md_dir = os.environ["MD_DIR"]
rev_dir = os.environ["REVIEWED_DIR"]

# Embedded copy of common::hegel1::toc_mod::MODERNIZED_LABELS, by position —
# an oracle that read the crate would only prove self-consistency.
MOD_LABELS = [
    "Vorrede",
    "Erster Teil. Wissenschaft der Erfahrung des Bewusstseins",
    "Einleitung",
    "I. Die sinnliche Gewissheit; oder das Diese und das Meinen",
    "II. Die Wahrnehmung; oder das Ding, und die Täuschung",
    "III. Kraft und Verstand, Erscheinung und übersinnliche Welt",
    "IV. Die Wahrheit der Gewissheit seiner selbst",
    "A. Selbstständigkeit und Unselbstständigkeit des Selbstbewusstseins; Herrschaft und Knechtschaft",
    "B. Freiheit des Selbstbewusstseins; Stoizismus, Skeptizismus, und das unglückliche Bewusstsein",
    "V. Gewissheit und Wahrheit der Vernunft",
    "A. Beobachtende Vernunft",
    "a. Beobachtung der Natur",
    "b. Die Beobachtung des Selbstbewusstseins in seiner Reinheit und seiner Beziehung auf äußre Wirklichkeit; logische und psychologische Gesetze",
    "c. Beobachtung der Beziehung des Selbstbewusstseins auf seine unmittelbare Wirklichkeit; Physiognomik und Schädellehre",
    "B. Die Verwirklichung des vernünftigen Selbstbewusstseins durch sich selbst",
    "a. Die Lust und die Notwendigkeit",
    "b. Das Gesetz des Herzens, und der Wahnsinn des Eigendünkels",
    "c. Die Tugend und der Weltlauf",
    "C. Die Individualität, welche sich an und für sich selbst reell ist",
    "a. Das geistige Tierreich und der Betrug, oder die Sache selbst",
    "b. Die gesetzgebende Vernunft",
    "c. Gesetzprüfende Vernunft",
    "VI. Der Geist",
    "A. Der wahre Geist, die Sittlichkeit",
    "a. Die sittliche Welt, das menschliche und göttliche Gesetz, der Mann und das Weib",
    "b. Die sittliche Handlung, das menschliche und göttliche Wissen, die Schuld und das Schicksal",
    "c. Rechtszustand",
    "B. Der sich entfremdete Geist; die Bildung",
    "I. Die Welt des sich entfremdeten Geistes",
    "a. Die Bildung und ihr Reich der Wirklichkeit",
    "b. Der Glauben und die reine Einsicht",
    "II. Die Aufklärung",
    "a. Der Kampf der Aufklärung mit dem Aberglauben",
    "b. Die Wahrheit der Aufklärung",
    "III. Die absolute Freiheit und der Schrecken",
    "C. Der seiner selbst gewisse Geist. Die Moralität",
    "a. Die moralische Weltanschauung",
    "b. Die Verstellung",
    "c. Das Gewissen, die schöne Seele, das Böse und seine Verzeihung",
    "VII. Die Religion",
    "A. Natürliche Religion",
    "a, Das Lichtwesen",
    "b. Die Pflanze und das Tier",
    "c. Der Werkmeister",
    "B. Die Kunst-Religion",
    "a. Das abstrakte Kunstwerk",
    "b. Das lebendige Kunstwerk",
    "c. Das geistige Kunstwerk",
    "C. Die offenbare Religion",
    "VIII. Das absolute Wissen",
]

results = []


def gate(name, ok, detail=""):
    results.append((name, ok, detail))


mod_files = sorted(f for f in os.listdir(md_dir) if f.endswith(".md"))
rev_files = sorted(f for f in os.listdir(rev_dir) if f.endswith(".md"))
gate(
    "file set matches md_reviewed (50 files)",
    mod_files == rev_files and len(mod_files) == 50,
    f"{len(mod_files)} vs {len(rev_files)}",
)

fm_ok, label_ok, para_ok, mark_ok = True, True, True, True
long_s, ey_tokens, leaks = 0, [], []
marker = re.compile(r"\{\{\{ ([^}]*) \}\}\}")
label_re = re.compile(r'^label: "(.*)"$', re.M)
for f in mod_files:
    if f not in rev_files:
        continue
    mod = open(os.path.join(md_dir, f)).read()
    rev = open(os.path.join(rev_dir, f)).read()
    fm_mod, fm_rev = mod.split("---\n")[1], rev.split("---\n")[1]
    if label_re.sub("label: -", fm_mod) != label_re.sub("label: -", fm_rev):
        fm_ok = False
    position = int(re.search(r"^position: (\d+)$", fm_mod, re.M).group(1))
    label = label_re.search(fm_mod).group(1)
    heading = next(l for l in mod.splitlines() if l.startswith("## "))
    if label != MOD_LABELS[position - 1] or not heading.endswith(label):
        label_ok = False
    mp = [b for b in mod.split("\n\n") if b.strip()]
    rp = [b for b in rev.split("\n\n") if b.strip()]
    if len(mp) != len(rp):
        para_ok = False
    if marker.findall(mod) != marker.findall(rev):
        mark_ok = False
    body = mod.split("---\n", 2)[2]
    long_s += body.count("ſ")
    ey_tokens += re.findall(r"[\wäöüÄÖÜ]*ey[\wäöü]*", body)
    leaks += re.findall(r"<(?!i>|/i>)[a-z]+|@norm|\x00", body)

gate("front matter matches md_reviewed (label excepted)", fm_ok)
gate("labels + headings match the modernized TOC table", label_ok)
gate("paragraph count per file matches md_reviewed", para_ok)
gate("page-marker sequence matches md_reviewed", mark_ok)
gate(
    "orthography incl. headings: no long-s, no ey, no TEI leakage",
    long_s == 0 and not ey_tokens and not leaks,
    f"ſ={long_s} ey={len(ey_tokens)} leaks={len(leaks)}",
)

# report, not a gate: th/ck tokens still present (legitimate in Greek/foreign
# stems — Methode, zurück — but worth an eyeball)
th = {}
for f in mod_files:
    body = open(os.path.join(md_dir, f)).read().split("---\n", 2)[2]
    for w in re.findall(r"[\wäöüÄÖÜß]+", body):
        if re.search(r"th|Th", w):
            th[w] = th.get(w, 0) + 1
top = sorted(th.items(), key=lambda kv: -kv[1])[:8]

width = max(len(r[0]) for r in results) + 2
print("gate  check" + " " * (width - 5) + "result")
failed = []
for i, (name, ok, detail) in enumerate(results, 1):
    mark = "PASS" if ok else f"FAIL  {detail}"
    print(f"  {i}   {name:<{width}}{mark}")
    if not ok:
        failed.append(i)
print()
print(f"th-sweep (report only): {sum(th.values())} tokens, top: " + ", ".join(f"{w}×{n}" for w, n in top))
if failed:
    print(f"hegel1_verify_modernized: FAILED (gates {failed})")
    sys.exit(1)
print(f"hegel1_verify_modernized: OK ({len(results)} gates)")
PY
