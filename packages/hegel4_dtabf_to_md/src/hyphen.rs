//! Line-end hyphen decisions. The edition's JSON paragraph text is already
//! dehyphenated, so a candidate form that occurs there (on the same page or
//! a neighbour — a word can straddle a page break) settles the join. Where
//! the JSON does not settle it, a ruling in `hyphen_rulings.tsv` must.

use std::collections::{HashMap, HashSet};

use serde_json::Value;

use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// `Bewußt-` + `seyn` → `Bewußtseyn`.
    Join,
    /// The hyphen belongs to the word: `Experimental-` + `Physik`.
    Keep,
    /// A suspended hyphen: `an-` + `und`, kept with the word space.
    Space,
    /// Old `k-k` line break of `ck`: `rük-` + `ken` → `rücken`.
    Ck,
}

impl Decision {
    pub fn parse(s: &str) -> Result<Decision> {
        match s {
            "join" => Ok(Decision::Join),
            "keep" => Ok(Decision::Keep),
            "space" => Ok(Decision::Space),
            "ck" => Ok(Decision::Ck),
            other => Err(format!("unknown hyphen action {other:?}")),
        }
    }
}

#[derive(Debug, Default)]
pub struct Rulings {
    map: HashMap<(u32, String, String), Decision>,
}

impl Rulings {
    pub fn parse(tsv: &str) -> Result<Rulings> {
        let mut map = HashMap::new();
        for line in tsv.lines() {
            if line.trim().is_empty() || line.starts_with('#') || line.starts_with("page\t") {
                continue;
            }
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() < 4 {
                return Err(format!("malformed hyphen ruling: {line}"));
            }
            let page = f[0].parse().map_err(|_| format!("bad page in {line}"))?;
            map.insert(
                (page, f[1].to_string(), f[2].to_string()),
                Decision::parse(f[3])?,
            );
        }
        Ok(Rulings { map })
    }

    fn get(&self, page: u32, left: &str, right: &str) -> Option<Decision> {
        self.map
            .get(&(page, left.to_string(), right.to_string()))
            .copied()
    }
}

#[derive(Debug, Default)]
pub struct Oracle {
    keys: HashMap<u32, HashSet<String>>,
}

impl Oracle {
    /// Build from the edition JSON (`work-gw-20.js`: a JS assignment around a
    /// JSON object whose `pages[].units[].text` carry the dehyphenated text).
    pub fn from_json(js: &str) -> Result<Oracle> {
        let start = js.find('{').ok_or("no JSON object in work-gw-20.js")?;
        let end = js.rfind('}').ok_or("unterminated JSON in work-gw-20.js")?;
        let root: Value = serde_json::from_str(&js[start..=end]).map_err(|e| e.to_string())?;
        let pages = root["pages"].as_array().ok_or("JSON has no pages[]")?;
        let mut oracle = Oracle::default();
        for page in pages {
            let Some(gw_page) = page["gw_page"].as_u64() else {
                continue;
            };
            let set = oracle.keys.entry(gw_page as u32).or_default();
            for unit in page["units"].as_array().into_iter().flatten() {
                if let Some(text) = unit["text"].as_str() {
                    set.extend(index(text));
                }
            }
        }
        Ok(oracle)
    }

    #[cfg(test)]
    pub fn from_page_texts(pages: &[(u32, &str)]) -> Oracle {
        let mut oracle = Oracle::default();
        for (page, text) in pages {
            oracle.keys.entry(*page).or_default().extend(index(text));
        }
        oracle
    }

    fn has(&self, page: u32, radius: u32, form: &str) -> bool {
        self.within(page, radius).any(|set| set.contains(form))
    }

    /// A word whose own continuation is another line-end hyphen is only
    /// known up to there, so the candidate must begin some token.
    fn has_prefix(&self, page: u32, radius: u32, form: &str) -> bool {
        self.within(page, radius)
            .any(|set| set.iter().any(|t| t.starts_with(form)))
    }

    fn within(&self, page: u32, radius: u32) -> impl Iterator<Item = &HashSet<String>> {
        self.keys
            .iter()
            .filter(move |(p, _)| p.abs_diff(page) <= radius)
            .map(|(_, set)| set)
    }

    /// Settle one join. `left` is the hyphen-final word fragment as printed
    /// (`Bewußt-`), `right` the word core that continues it (`seyn`).
    pub fn decide(
        &self,
        rulings: &Rulings,
        page: u32,
        left: &str,
        right: &str,
        chained: bool,
    ) -> Result<(Decision, &'static str)> {
        if let Some(d) = rulings.get(page, left, right) {
            return Ok((d, "ruling"));
        }
        let stem = left.trim_end_matches('-');
        // A paragraph's text sits on the page it starts on, so a join can be
        // several pages from its line break: widen until some form turns up.
        for radius in SEARCH_RADII {
            let present = |form: &str| {
                if chained {
                    self.has_prefix(page, radius, form)
                } else {
                    self.has(page, radius, form)
                }
            };
            let mut found: Vec<Decision> = Vec::new();
            if present(&format!("{stem}{right}")) {
                found.push(Decision::Join);
            }
            if present(&format!("{left}{right}")) {
                found.push(Decision::Keep);
            }
            if self.has(page, radius, &suspended_key(left, right)) {
                found.push(Decision::Space);
            }
            if let (Some(s), Some(r)) = (stem.strip_suffix('k'), right.strip_prefix('k'))
                && present(&format!("{s}ck{r}"))
            {
                found.push(Decision::Ck);
            }
            match found.as_slice() {
                [] => continue,
                [d] => return Ok((*d, "json")),
                many => {
                    return Err(format!(
                        "hyphen join ambiguous on GW 20.{page}: {left} | {right} ({many:?} within {radius} pages; add a ruling)"
                    ));
                }
            }
        }
        Err(format!(
            "hyphen join unsettled on GW 20.{page}: {left} | {right} (no candidate in the JSON; add a ruling)"
        ))
    }
}

const SEARCH_RADII: [u32; 3] = [1, 4, u32::MAX];

/// Whitespace tokens (leading non-alphanumerics and trailing
/// non-alphanumerics other than a hyphen trimmed), plus a key for every
/// hyphen-final token followed by another word: the JSON keeps a suspended
/// hyphen (`an- und`) as two tokens, and the pair is what tells it from a
/// word that merely occurs hyphen-final elsewhere.
fn index(text: &str) -> Vec<String> {
    let tokens: Vec<String> = text
        .split_whitespace()
        .filter_map(|w| {
            let w = w.trim_start_matches(|c: char| !c.is_alphanumeric());
            let w = w.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '-');
            (!w.is_empty()).then(|| w.to_string())
        })
        .collect();
    let mut keys = tokens.clone();
    for pair in tokens.windows(2) {
        if pair[0].ends_with('-') {
            let next: String = pair[1].chars().take_while(|c| c.is_alphabetic()).collect();
            keys.push(suspended_key(&pair[0], &next));
        }
    }
    keys
}

fn suspended_key(left: &str, right: &str) -> String {
    format!("{left}\u{1} {right}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decide(oracle: &Oracle, left: &str, right: &str) -> Result<Decision> {
        oracle
            .decide(&Rulings::default(), 10, left, right, false)
            .map(|(d, _)| d)
    }

    #[test]
    fn the_json_settles_join_keep_space_and_ck() {
        let o = Oracle::from_page_texts(&[(
            10,
            "das Bewußtseyn, die Experimental-Physik, an- und Blut-, sie rücken.",
        )]);
        assert_eq!(decide(&o, "Bewußt-", "seyn"), Ok(Decision::Join));
        assert_eq!(decide(&o, "Experimental-", "Physik"), Ok(Decision::Keep));
        assert_eq!(decide(&o, "an-", "und"), Ok(Decision::Space));
        assert_eq!(decide(&o, "rük-", "ken"), Ok(Decision::Ck));
    }

    #[test]
    fn a_hyphen_final_token_elsewhere_is_not_a_suspended_hyphen() {
        let o = Oracle::from_page_texts(&[(10, "Ur- und Grund, der Urgrund")]);
        assert_eq!(decide(&o, "Ur-", "grund"), Ok(Decision::Join));
    }

    #[test]
    fn a_word_hyphenated_twice_is_settled_one_hop_at_a_time() {
        let o = Oracle::from_page_texts(&[(10, "das Bewußtseyn")]);
        let rulings = Rulings::default();
        assert_eq!(
            o.decide(&rulings, 10, "Be-", "wußt", true).unwrap().0,
            Decision::Join
        );
        assert_eq!(decide(&o, "Bewußt-", "seyn"), Ok(Decision::Join));
    }

    #[test]
    fn a_word_straddling_a_page_break_is_found_on_the_neighbour() {
        let o = Oracle::from_page_texts(&[(11, "Beweis")]);
        assert_eq!(decide(&o, "Be-", "weis"), Ok(Decision::Join));
    }

    #[test]
    fn unsettled_and_ambiguous_joins_are_refused_until_ruled() {
        let o = Oracle::from_page_texts(&[(10, "Wort-Stück WortStück")]);
        assert!(decide(&o, "Wort-", "Stück").is_err());
        assert!(decide(&o, "Foo-", "bar").is_err());
        let rulings = Rulings::parse("10\tWort-\tStück\tkeep\n").unwrap();
        assert_eq!(
            o.decide(&rulings, 10, "Wort-", "Stück", false).unwrap().0,
            Decision::Keep
        );
    }
}
