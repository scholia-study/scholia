//! `page_joins.tsv`: the decision for every page boundary at which the XML
//! closes a `<p>` and opens another. The edition usually keeps a paragraph
//! that crosses a page in one `<p>`, so each split is either a paragraph the
//! edition cut in two (`join`) or a genuine paragraph break (`keep`).

use std::collections::{BTreeMap, BTreeSet};

use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Join {
    Join,
    Keep,
}

#[derive(Debug, Default)]
pub struct PageJoins {
    rows: BTreeMap<u32, Join>,
}

impl PageJoins {
    pub fn parse(tsv: &str) -> Result<PageJoins> {
        let mut rows = BTreeMap::new();
        for line in tsv.lines() {
            if line.trim().is_empty() || line.starts_with('#') || line.starts_with("gw_page\t") {
                continue;
            }
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() < 3 {
                return Err(format!("malformed page join: {line}"));
            }
            let page: u32 = f[0].parse().map_err(|_| format!("bad page in {line}"))?;
            let decision = match f[1] {
                "join" => Join::Join,
                "keep" => Join::Keep,
                other => return Err(format!("unknown page join decision {other:?}")),
            };
            if !matches!(f[2], "certain" | "uncertain") {
                return Err(format!("unknown page join certainty {:?}", f[2]));
            }
            if rows.insert(page, decision).is_some() {
                return Err(format!("duplicate page join for GW 20.{page}"));
            }
        }
        Ok(PageJoins { rows })
    }

    pub fn get(&self, page: u32) -> Option<Join> {
        self.rows.get(&page).copied()
    }

    pub fn stale(&self, seen: &BTreeSet<u32>) -> Vec<u32> {
        self.rows
            .keys()
            .filter(|p| !seen.contains(p))
            .copied()
            .collect()
    }
}
