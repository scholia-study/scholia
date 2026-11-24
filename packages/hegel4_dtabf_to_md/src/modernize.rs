//! Orthographic modernization of the reviewed layer: every word core is
//! replaced per `modernize_rulings.tsv`; markers, footnote references and
//! front-matter keys pass through. A word with a pre-reform trace (`ey`, `ſ`)
//! that the table does not rule on is an error rather than a guess.

use std::collections::HashMap;

use regex::Regex;

use crate::Result;

pub struct Modernizer {
    map: HashMap<String, String>,
    words: Regex,
    protected: Regex,
    pub untabled: Vec<String>,
}

impl Modernizer {
    pub fn parse(tsv: &str) -> Result<Modernizer> {
        let mut map = HashMap::new();
        for line in tsv.lines().skip(1) {
            if line.trim().is_empty() {
                continue;
            }
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() < 2 {
                return Err(format!("malformed modernize row: {line}"));
            }
            if map.insert(f[0].to_string(), f[1].to_string()).is_some() {
                return Err(format!("duplicate modernize surface {:?}", f[0]));
            }
        }
        Ok(Modernizer {
            map,
            words: Regex::new(r"\p{L}+").unwrap(),
            protected: Regex::new(r"\{\{\{.*?\}\}\}|\{\{.*?\}\}|\[\^\d+\]").unwrap(),
            untabled: Vec::new(),
        })
    }

    pub fn text(&mut self, text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut last = 0;
        let spans: Vec<(usize, usize)> = self
            .protected
            .find_iter(text)
            .map(|m| (m.start(), m.end()))
            .collect();
        for (start, end) in spans {
            out.push_str(&self.words(&text[last..start]));
            out.push_str(&text[start..end]);
            last = end;
        }
        out.push_str(&self.words(&text[last..]));
        out
    }

    fn words(&mut self, text: &str) -> String {
        let (map, untabled) = (&self.map, &mut self.untabled);
        self.words
            .replace_all(text, |caps: &regex::Captures| {
                let w = caps.get(0).unwrap().as_str();
                match map.get(w) {
                    Some(r) => r.clone(),
                    None => {
                        if w.to_lowercase().contains("ey") || w.contains('ſ') {
                            untabled.push(w.to_string());
                        }
                        w.to_string()
                    }
                }
            })
            .into_owned()
    }

    /// One reviewed node file: the front matter's `label:` and the body are
    /// modernized, the other front-matter lines are structural.
    pub fn file(&mut self, content: &str) -> String {
        let mut in_front = false;
        let mut out = Vec::new();
        for (i, line) in content.lines().enumerate() {
            if line == "---" && (i == 0 || in_front) {
                in_front = i == 0;
                out.push(line.to_string());
            } else if in_front {
                match line.strip_prefix("label: ") {
                    Some(value) => out.push(format!("label: {}", self.text(value))),
                    None => out.push(line.to_string()),
                }
            } else {
                out.push(self.text(line));
            }
        }
        let mut joined = out.join("\n");
        if content.ends_with('\n') {
            joined.push('\n');
        }
        joined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TSV: &str = "surface\tresult\tcount\tsource\tcertainty\tnote\nSeyn\tSein\t1\tx\tsure\t\nTheil\tTeil\t1\tx\tsure\t\n";

    #[test]
    fn words_change_but_markers_and_references_do_not() {
        let mut m = Modernizer::parse(TSV).unwrap();
        assert_eq!(
            m.text("## {{{ 20.5 }}} {{ 1 }} Das _Seyn_[^1] im Theil"),
            "## {{{ 20.5 }}} {{ 1 }} Das _Sein_[^1] im Teil"
        );
    }

    #[test]
    fn front_matter_changes_only_its_label() {
        let mut m = Modernizer::parse(TSV).unwrap();
        let src = "---\nposition: 1\nlabel: \"Seyn\"\npage_gw: \"20.5\"\n---\n\nTheil\n";
        assert_eq!(
            m.file(src),
            "---\nposition: 1\nlabel: \"Sein\"\npage_gw: \"20.5\"\n---\n\nTeil\n"
        );
    }

    #[test]
    fn an_untabled_pre_reform_word_is_collected() {
        let mut m = Modernizer::parse(TSV).unwrap();
        m.text("die Policey");
        assert_eq!(m.untabled, vec!["Policey"]);
    }
}
