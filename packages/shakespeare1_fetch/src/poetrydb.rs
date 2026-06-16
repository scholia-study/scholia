//! Modern-spelling layer: Shakespeare's sonnets from PoetryDB.
//!
//! One request to `author/William Shakespeare` returns every poem; the
//! sonnets are titled `"Sonnet N: <incipit>"`, so we exact-match `^Sonnet
//! (\d+):` to pull the 154 and index them by number. (A plain `title/Sonnet
//! 1` query can't be used — PoetryDB matches titles by substring and would
//! also return other poets' sonnets.)

use std::collections::BTreeMap;

use regex::Regex;
use serde::Deserialize;

#[derive(Deserialize)]
struct Poem {
    title: String,
    lines: Vec<String>,
}

/// Parse a PoetryDB author response into `sonnet number -> lines`.
pub fn parse(json: &str) -> Result<BTreeMap<u32, Vec<String>>, Box<dyn std::error::Error>> {
    let poems: Vec<Poem> = serde_json::from_str(json)?;
    let re = Regex::new(r"^Sonnet (\d+):").unwrap();

    let mut out: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    for poem in poems {
        if let Some(caps) = re.captures(&poem.title) {
            let n: u32 = caps[1].parse()?;
            let lines: Vec<String> = poem
                .lines
                .into_iter()
                .map(|l| l.trim().to_string())
                .collect();
            out.insert(n, lines);
        }
    }
    Ok(out)
}
