//! `indent_runs.tsv`: main-text paragraphs of one § that the print sets as a
//! single paragraph of indented items, emitted as `+ ` runs (the numbered
//! list under § 76). Paragraphs are counted per §, main text only, from 1,
//! after page joins. A row that matches no such range stops the run.

use std::collections::BTreeMap;

use crate::Result;

#[derive(Debug, Default)]
pub struct IndentRuns {
    rows: BTreeMap<u32, (usize, usize)>,
}

impl IndentRuns {
    pub fn parse(tsv: &str) -> Result<IndentRuns> {
        let mut rows = BTreeMap::new();
        for line in tsv.lines() {
            if line.trim().is_empty() || line.starts_with('#') || line.starts_with("section\t") {
                continue;
            }
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() < 3 {
                return Err(format!("malformed indent run: {line}"));
            }
            let num = |s: &str| {
                s.parse::<usize>()
                    .map_err(|_| format!("bad number in {line}"))
            };
            let section = num(f[0])? as u32;
            let (first, last) = (num(f[1])?, num(f[2])?);
            if first == 0 || last <= first {
                return Err(format!("indent run needs 1 <= first < last: {line}"));
            }
            if rows.insert(section, (first, last)).is_some() {
                return Err(format!("duplicate indent run for § {section}"));
            }
        }
        Ok(IndentRuns { rows })
    }

    pub fn get(&self, section: u32) -> Option<(usize, usize)> {
        self.rows.get(&section).copied()
    }

    pub fn stale(&self, used: &[u32]) -> Vec<u32> {
        self.rows
            .keys()
            .filter(|s| !used.contains(s))
            .copied()
            .collect()
    }
}
