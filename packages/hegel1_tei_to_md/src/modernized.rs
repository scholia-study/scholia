//! The `md_modernized` layer: same TEI walk, same block machinery, but every
//! `<w>` token is re-spelled through the pass-1 rule table
//! (`common::hegel1::modernize`) plus the checked-in pass-2 decision table,
//! and `<choice>` flips from `<sic>` to `<corr>`.
//!
//! DTA splits one printed word into several `<w>` fragments where a line
//! break or markup interrupts it, putting the whole-word `@norm` on the head
//! fragment and none on the continuations. Modernization is decided on the
//! merged word, then distributed back onto the fragments by edit-distance
//! alignment, so page markers and emphasis spans stay exactly where the
//! print has them.

use std::collections::{BTreeMap, HashMap};

use common::hegel1::modernize::{Classification, classify};
use roxmltree::{Node, NodeId};

/// How a fragment connects to the next one inside the same printed word.
#[derive(Clone, Copy, PartialEq)]
enum Junction {
    /// Line-break split: the trailing hyphen rejoins away (`Bewuſst-|ſeyn`).
    Soft,
    /// Markup split, no hyphen involved (`<hi>Selbſt</hi>bewuſstseyn`) — or a
    /// printed compound hyphen that stays part of the text.
    Plain,
}

struct Fragment {
    id: NodeId,
    surface: String,
    /// Junction to the *next* fragment; `Plain` on the last.
    junction: Junction,
}

struct Group {
    fragments: Vec<Fragment>,
    /// Fragment surfaces joined, soft-junction hyphens dropped.
    merged: String,
    norm: String,
}

/// One printed word's replacement, sliced per fragment. A soft junction keeps
/// its marker so the emitter can hand the join to the existing `normalize`
/// machinery.
pub struct Replacement {
    pub text: String,
    pub soft_joined: bool,
}

/// The decision-table rows the text needs but the TSV lacks.
pub struct MissingRuling {
    pub surface: String,
    pub cab_norm: String,
    pub count: usize,
    pub class: &'static str,
}

/// A `<w>`'s surface with an internal line-break rejoin: a hyphen directly
/// before an inner `<lb/>` *with text after it* splits the word across lines
/// inside one `<w>` and rejoins away. A trailing `<lb/>` instead means the
/// continuation lives in the next `<w>` (`gegenwär-<lb/></w><pb/><w>tig`), so
/// the hyphen stays for the junction logic and the trailing break is
/// reported to it.
fn w_surface(el: Node) -> (String, bool) {
    let mut raw = String::new();
    collect(el, &mut raw);
    let stripped: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    let trailing_lb = stripped.ends_with('\u{0}');
    let inner = stripped.trim_end_matches('\u{0}');
    return (
        inner.replace("-\u{0}", "").replace('\u{0}', ""),
        trailing_lb,
    );

    fn collect(n: Node, out: &mut String) {
        for ch in n.children() {
            if ch.is_text() {
                out.push_str(ch.text().unwrap_or(""));
            } else if ch.is_element() && ch.tag_name().name() == "lb" {
                out.push('\u{0}');
            } else if ch.is_element() {
                collect(ch, out);
            }
        }
    }
}

fn has_lb(el: Node) -> bool {
    el.descendants()
        .any(|n| n.is_element() && n.tag_name().name() == "lb")
}

/// Document-order `<w>` and `<lb>` elements in modernized scope: inside
/// content divs, `<fw>` dropped, and — unlike the reviewed layer — `<sic>`
/// skipped in favour of `<corr>`.
fn scope_elements<'a, 'i>(root: Node<'a, 'i>) -> Vec<Node<'a, 'i>> {
    let mut out = Vec::new();
    collect(root, false, &mut out);
    return out;

    fn collect<'a, 'i>(el: Node<'a, 'i>, mut in_div: bool, out: &mut Vec<Node<'a, 'i>>) {
        if !el.is_element() && !el.is_root() {
            return;
        }
        match el.tag_name().name() {
            "fw" | "sic" => return,
            "div" => {
                if super::skipped_div(el) {
                    return;
                }
                in_div = true;
            }
            "w" | "lb" if in_div => {
                out.push(el);
                return;
            }
            _ => {}
        }
        for ch in el.children() {
            collect(ch, in_div, out);
        }
    }
}

/// Merge continuation fragments onto their norm-bearing heads — the inverse
/// of DTA's word splitting, mirroring `rejoining_words`: a junction is soft
/// only when a line break separates the pieces and a hyphen closes the
/// earlier one.
fn collect_groups(root: Node) -> Vec<Group> {
    let elements = scope_elements(root);
    let mut groups: Vec<Group> = Vec::new();
    let mut i = 0;
    while i < elements.len() {
        let el = elements[i];
        i += 1;
        if el.tag_name().name() == "lb" {
            continue;
        }
        let (surface, mut trailing_lb) = w_surface(el);
        let Some(norm) = el.attribute("norm") else {
            // A continuation no head claimed (document opening, apparatus):
            // treat as its own identity token.
            groups.push(Group {
                merged: surface.clone(),
                norm: surface.clone(),
                fragments: vec![Fragment {
                    id: el.id(),
                    surface,
                    junction: Junction::Plain,
                }],
            });
            continue;
        };
        let mut fragments = vec![Fragment {
            id: el.id(),
            surface: surface.clone(),
            junction: Junction::Plain,
        }];
        let mut merged = surface;
        loop {
            let mut j = i;
            let mut lb_between = false;
            while j < elements.len() && elements[j].tag_name().name() == "lb" {
                lb_between = true;
                j += 1;
            }
            let Some(&next) = elements.get(j) else { break };
            if next.tag_name().name() != "w" || next.attribute("norm").is_some() {
                break;
            }
            let (frag, frag_trailing_lb) = w_surface(next);
            let soft = merged.ends_with('-') && (trailing_lb || lb_between || has_lb(next));
            if soft {
                merged.pop();
                fragments.last_mut().expect("head exists").junction = Junction::Soft;
            }
            merged.push_str(&frag);
            fragments.push(Fragment {
                id: next.id(),
                surface: frag,
                junction: Junction::Plain,
            });
            trailing_lb = frag_trailing_lb;
            i = j + 1;
        }
        groups.push(Group {
            fragments,
            merged,
            norm: norm.to_string(),
        });
    }
    groups
}

/// The exact output token for every distinct surface: the decision table
/// first, then the mechanical rule table against the surface's majority CAB
/// norm. Surfaces the rules cannot explain and the table does not rule are
/// collected for the refuse-to-emit report.
fn build_lexicon(
    groups: &[Group],
    rulings: &HashMap<String, String>,
) -> (HashMap<String, String>, Vec<MissingRuling>) {
    let mut norm_votes: HashMap<&str, BTreeMap<&str, usize>> = HashMap::new();
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for g in groups {
        *norm_votes
            .entry(&g.merged)
            .or_default()
            .entry(&g.norm)
            .or_default() += 1;
        *counts.entry(&g.merged).or_default() += 1;
    }
    let mut lexicon: HashMap<String, String> = rulings
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let mut missing = Vec::new();
    for (surface, votes) in &norm_votes {
        let norm = votes
            .iter()
            .max_by_key(|(_, n)| **n)
            .map(|(norm, _)| *norm)
            .expect("at least one vote");
        if rulings.contains_key(*surface) {
            continue;
        }
        // CAB sometimes fails to normalize at all (norm `nichtansichseyende`,
        // `Daseyus`), which would sail through as Identity or a hollow
        // mechanical fold. Any unruled target still carrying archaic markers
        // needs a decision row like every other judgment call.
        let archaic = |s: &str| s.contains('ſ') || s.contains("ey") || s.contains("Ey");
        match classify(surface, norm) {
            Classification::Identity if archaic(norm) => missing.push(MissingRuling {
                surface: surface.to_string(),
                cab_norm: norm.to_string(),
                count: counts[*surface],
                class: "cab_unnormalized",
            }),
            Classification::Identity => {}
            Classification::Mechanical { result, .. } if archaic(&result) => {
                missing.push(MissingRuling {
                    surface: surface.to_string(),
                    cab_norm: norm.to_string(),
                    count: counts[*surface],
                    class: "cab_unnormalized",
                })
            }
            Classification::Mechanical { result, .. } => {
                lexicon.insert(surface.to_string(), result);
            }
            Classification::SsAudit { .. } => missing.push(MissingRuling {
                surface: surface.to_string(),
                cab_norm: norm.to_string(),
                count: counts[*surface],
                class: "ss_audit",
            }),
            Classification::Residual => missing.push(MissingRuling {
                surface: surface.to_string(),
                cab_norm: norm.to_string(),
                count: counts[*surface],
                class: "residual",
            }),
        }
    }
    missing.sort_by_key(|b| std::cmp::Reverse(b.count));
    (lexicon, missing)
}

/// Cut `target` at the alignment images of the fragment boundaries in
/// `merged`, so each fragment carries its share of the modernized word.
/// Plain Levenshtein alignment: the rules are local substitutions, so the
/// path stays honest; any cut point still concatenates to `target` exactly.
fn slice_by_alignment(merged: &str, target: &str, cuts: &[usize]) -> Vec<String> {
    let s: Vec<char> = merged.chars().collect();
    let t: Vec<char> = target.chars().collect();
    let (n, m) = (s.len(), t.len());
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for (i, row) in dp.iter_mut().enumerate() {
        row[0] = i as u32;
    }
    for (j, cell) in dp[0].iter_mut().enumerate() {
        *cell = j as u32;
    }
    for i in 1..=n {
        for j in 1..=m {
            let sub = dp[i - 1][j - 1] + u32::from(s[i - 1] != t[j - 1]);
            dp[i][j] = sub.min(dp[i - 1][j] + 1).min(dp[i][j - 1] + 1);
        }
    }
    // Walk back, recording for each surface index the matched target index.
    let mut image = vec![m; n + 1];
    let (mut i, mut j) = (n, m);
    image[n] = m;
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && dp[i][j] == dp[i - 1][j - 1] + u32::from(s[i - 1] != t[j - 1]) {
            i -= 1;
            j -= 1;
        } else if i > 0 && dp[i][j] == dp[i - 1][j] + 1 {
            i -= 1;
        } else {
            j -= 1;
        }
        image[i] = j;
    }
    let mut out = Vec::new();
    let mut start = 0;
    for &cut in cuts {
        let end = image[cut];
        out.push(t[start..end].iter().collect());
        start = end;
    }
    out.push(t[start..].iter().collect());
    out
}

/// The per-`<w>` replacement map plus the surface → output lexicon
/// (rulings ∪ mechanical) the ops pass re-spells phrases with.
pub type ReplacementPlan = (HashMap<NodeId, Replacement>, HashMap<String, String>);

/// Per-`<w>` replacement map for the whole document, plus the lexicon.
pub fn replacements(
    root: Node,
    rulings: &HashMap<String, String>,
) -> Result<ReplacementPlan, Vec<MissingRuling>> {
    let groups = collect_groups(root);
    let (lexicon, missing) = build_lexicon(&groups, rulings);
    if !missing.is_empty() {
        return Err(missing);
    }
    let mut map = HashMap::new();
    for g in &groups {
        let target = lexicon.get(&g.merged).unwrap_or(&g.merged);
        let slices = if g.fragments.len() == 1 {
            vec![target.clone()]
        } else {
            let mut cuts = Vec::new();
            let mut pos = 0;
            for f in &g.fragments[..g.fragments.len() - 1] {
                pos += f.surface.chars().count();
                if f.junction == Junction::Soft {
                    pos -= 1; // the dropped rejoin hyphen
                }
                cuts.push(pos);
            }
            slice_by_alignment(&g.merged, target, &cuts)
        };
        for (f, text) in g.fragments.iter().zip(slices) {
            map.insert(
                f.id,
                Replacement {
                    text,
                    soft_joined: f.junction == Junction::Soft,
                },
            );
        }
    }
    Ok((map, lexicon))
}

/// A pass-3 correction: the 1807 errata ("Verbesserungen") plus the few
/// print-split repairs, curated as exact search/replace strings against the
/// *reviewed* text. Both sides are re-spelled through the lexicon before
/// application, so one table serves both layers' orthographies.
pub struct Op {
    pub file: String,
    pub search: String,
    pub replace: String,
    pub note: String,
}

/// Parse the ops table; a row with an empty search is unresolved curation
/// and refuses the build.
pub fn load_ops(tsv: &str) -> Result<Vec<Op>, String> {
    let mut out = Vec::new();
    for (i, line) in tsv.lines().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 5 {
            return Err(format!("ops line {}: expected 5 columns", i + 1));
        }
        if cols[2].is_empty() || cols[3].is_empty() {
            return Err(format!(
                "ops line {}: unresolved (empty search/replace): {}",
                i + 1,
                cols[4]
            ));
        }
        out.push(Op {
            file: cols[0].to_string(),
            search: cols[2].to_string(),
            replace: cols[3].to_string(),
            note: cols[4].to_string(),
        });
    }
    Ok(out)
}

/// Re-spell an ops phrase through the lexicon, word run by word run. A
/// leftover long-s means a token the lexicon does not cover — an error, not
/// a silent archaism.
pub fn modernize_phrase(phrase: &str, lexicon: &HashMap<String, String>) -> Result<String, String> {
    let mut out = String::new();
    let mut run = String::new();
    let flush = |run: &mut String, out: &mut String| {
        if !run.is_empty() {
            match lexicon.get(run.as_str()) {
                Some(r) => out.push_str(r),
                None => out.push_str(run),
            }
            run.clear();
        }
    };
    for ch in phrase.chars() {
        if ch.is_alphabetic() {
            run.push(ch);
        } else {
            flush(&mut run, &mut out);
            out.push(ch);
        }
    }
    flush(&mut run, &mut out);
    if out.contains('ſ') {
        return Err(format!("ops phrase keeps long-s after mapping: {out}"));
    }
    Ok(out)
}

/// Apply every op to its file, requiring exactly one match each.
pub fn apply_ops(
    files: &mut [(String, String)],
    ops: &[Op],
    lexicon: &HashMap<String, String>,
) -> Result<usize, String> {
    let mut applied = 0;
    for op in ops {
        let search = modernize_phrase(&op.search, lexicon)?;
        let replace = modernize_phrase(&op.replace, lexicon)?;
        let (_, md) = files
            .iter_mut()
            .find(|(f, _)| *f == op.file)
            .ok_or_else(|| format!("ops: unknown file {}", op.file))?;
        let hits = md.matches(&search).count();
        if hits != 1 {
            return Err(format!(
                "ops: `{}` matches {} times in {} ({})",
                search, hits, op.file, op.note
            ));
        }
        *md = md.replacen(&search, &replace, 1);
        applied += 1;
    }
    Ok(applied)
}

/// Parse the checked-in decision table: `surface … ruling result …` rows.
pub fn load_rulings(tsv: &str) -> Result<HashMap<String, String>, String> {
    let mut out = HashMap::new();
    for (i, line) in tsv.lines().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 6 {
            return Err(format!("rulings line {}: expected ≥6 columns", i + 1));
        }
        let (surface, ruling, result) = (cols[0], cols[4], cols[5]);
        if !matches!(ruling, "accept" | "reject" | "rewrite" | "drop") {
            return Err(format!("rulings line {}: bad ruling `{ruling}`", i + 1));
        }
        // `drop` emits nothing (the DTA's empty apparatus tokens, whose CAB
        // norm is a placeholder like "[Formel]"); every other ruling must
        // carry a non-empty result.
        if ruling == "drop" {
            if !result.is_empty() {
                return Err(format!("rulings line {}: drop rows carry no result", i + 1));
            }
        } else if result.is_empty() {
            return Err(format!("rulings line {}: empty result", i + 1));
        }
        if out
            .insert(surface.to_string(), result.to_string())
            .is_some()
        {
            return Err(format!(
                "rulings line {}: duplicate surface `{surface}`",
                i + 1
            ));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignment_slices_concatenate_to_the_target() {
        // <i>Selbſt</i>bewuſstseyn → Selbstbewusstsein, cut after Selbſt
        let slices = slice_by_alignment("Selbſtbewuſstseyn", "Selbstbewusstsein", &[6]);
        assert_eq!(slices.concat(), "Selbstbewusstsein");
        assert_eq!(slices[0], "Selbst");
        assert_eq!(slices[1], "bewusstsein");
    }

    #[test]
    fn alignment_survives_length_changes_before_the_cut() {
        // Noth|wendigkeit → Notwendigkeit: the th→t sits left of the cut.
        let slices = slice_by_alignment("Nothwendigkeit", "Notwendigkeit", &[4]);
        assert_eq!(slices.concat(), "Notwendigkeit");
        assert_eq!(slices[0], "Not");
        assert_eq!(slices[1], "wendigkeit");
    }

    #[test]
    fn rulings_reject_unknown_vocabulary() {
        let tsv = "surface\tcab_norm\tcount\tclass\truling\tresult\treason\n\
                   itzt\tjetzt\t122\tresidual\tkeep\titzt\tterm\n";
        assert!(load_rulings(tsv).is_err());
    }

    #[test]
    fn rulings_load_surface_to_result() {
        let tsv = "surface\tcab_norm\tcount\tclass\truling\tresult\treason\n\
                   itzt\tjetzt\t122\tresidual\treject\titzt\tterm of art\n\
                   nemlich\tnämlich\t181\tresidual\taccept\tnämlich\tplain\n";
        let map = load_rulings(tsv).unwrap();
        assert_eq!(map["itzt"], "itzt");
        assert_eq!(map["nemlich"], "nämlich");
    }
}
