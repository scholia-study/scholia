//! Content hashing for incremental reconcile: a per-node hash plus a root hash
//! (over the ordered node hashes) that let a re-import skip unchanged nodes.
//!
//! Both importers and both code paths (fresh insert + reconcile) must produce
//! *identical* hashes, so the field order and separators live here. A node's hash
//! covers everything reconcile would write for it EXCEPT the recomputed
//! positional/numbering fields (`sentence_number`, `paragraph_number`,
//! `figure_number`, within-block `position`, `natural_key`) — excluding those
//! keeps a numbering shift from invalidating downstream hashes.
//!
//! Encoding (stable across runs/platforms): `blake3`, hex-encoded; every field
//! followed by a `0x1f` separator; `Option`s tagged `S`/`N` so `None` and
//! `Some("")` differ; every child list length-prefixed so a split/insert/delete
//! changes the hash. Changing any of this invalidates stored hashes — run
//! `--full-rewrite` once to rewrite them.

use blake3::Hasher;

/// Unit separator appended after every field.
const US: &[u8] = &[0x1f];

/// A page/reference marker on a sentence (Kant AA/B markers, Bible verse refs).
pub struct MarkerContent<'a> {
    pub system: &'a str,
    pub ref_value: &'a str,
    pub char_offset: Option<i32>,
}

/// One sentence's content identity (block sentence or footnote sentence).
pub struct SentenceContent<'a> {
    pub text: &'a str,
    pub html: &'a str,
    pub original_text: Option<&'a str>,
    pub original_html: Option<&'a str>,
    pub segment: Option<i16>,
    pub markers: Vec<MarkerContent<'a>>,
    pub footnotes: Vec<FootnoteContent<'a>>,
}

/// A footnote anchored to a sentence: its number plus its own sentences.
/// Footnote sentences carry no markers or nested footnotes of their own.
pub struct FootnoteContent<'a> {
    pub number: i32,
    pub sentences: Vec<SentenceContent<'a>>,
}

/// One content block (paragraph / heading / figure / separator).
pub struct BlockContent<'a> {
    pub block_type: &'a str,
    pub text: &'a str,
    pub html: &'a str,
    pub original_text: Option<&'a str>,
    pub original_html: Option<&'a str>,
    pub sentences: Vec<SentenceContent<'a>>,
}

/// One node subtree: its label plus its ordered blocks.
pub struct NodeContent<'a> {
    pub label: &'a str,
    pub label_html: &'a str,
    pub blocks: Vec<BlockContent<'a>>,
}

fn feed_str(h: &mut Hasher, v: &str) {
    h.update(v.as_bytes());
    h.update(US);
}

fn feed_int(h: &mut Hasher, v: i64) {
    h.update(v.to_string().as_bytes());
    h.update(US);
}

fn feed_opt_str(h: &mut Hasher, v: Option<&str>) {
    match v {
        Some(s) => {
            h.update(b"S");
            h.update(s.as_bytes());
        }
        None => {
            h.update(b"N");
        }
    }
    h.update(US);
}

fn feed_opt_int(h: &mut Hasher, v: Option<i64>) {
    match v {
        Some(n) => {
            h.update(b"S");
            h.update(n.to_string().as_bytes());
        }
        None => {
            h.update(b"N");
        }
    }
    h.update(US);
}

fn feed_sentence(h: &mut Hasher, s: &SentenceContent) {
    feed_str(h, s.text);
    feed_str(h, s.html);
    feed_opt_str(h, s.original_text);
    feed_opt_str(h, s.original_html);
    feed_opt_int(h, s.segment.map(i64::from));

    feed_int(h, s.markers.len() as i64);
    for m in &s.markers {
        feed_str(h, m.system);
        feed_str(h, m.ref_value);
        feed_opt_int(h, m.char_offset.map(i64::from));
    }

    feed_int(h, s.footnotes.len() as i64);
    for f in &s.footnotes {
        feed_int(h, i64::from(f.number));
        feed_int(h, f.sentences.len() as i64);
        for fs in &f.sentences {
            feed_sentence(h, fs);
        }
    }
}

/// Hash of one node's full content subtree. Hex-encoded blake3.
pub fn node_hash(node: &NodeContent) -> String {
    let mut h = Hasher::new();
    feed_str(&mut h, node.label);
    feed_str(&mut h, node.label_html);
    feed_int(&mut h, node.blocks.len() as i64);
    for b in &node.blocks {
        feed_str(&mut h, b.block_type);
        feed_str(&mut h, b.text);
        feed_str(&mut h, b.html);
        feed_opt_str(&mut h, b.original_text);
        feed_opt_str(&mut h, b.original_html);
        feed_int(&mut h, b.sentences.len() as i64);
        for s in &b.sentences {
            feed_sentence(&mut h, s);
        }
    }
    h.finalize().to_hex().to_string()
}

/// Root hash = hash of the ordered list of node hashes. Hex-encoded blake3.
pub fn root_hash(node_hashes: &[String]) -> String {
    let mut h = Hasher::new();
    feed_int(&mut h, node_hashes.len() as i64);
    for nh in node_hashes {
        feed_str(&mut h, nh);
    }
    h.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sentence(
        text: &'static str,
        markers: Vec<MarkerContent<'static>>,
    ) -> SentenceContent<'static> {
        SentenceContent {
            text,
            html: text,
            original_text: None,
            original_html: None,
            segment: None,
            markers,
            footnotes: Vec::new(),
        }
    }

    fn sample() -> NodeContent<'static> {
        NodeContent {
            label: "Chapter 1",
            label_html: "Chapter 1",
            blocks: vec![BlockContent {
                block_type: "paragraph",
                text: "Hello world. Bye.",
                html: "<p>Hello world. Bye.</p>",
                original_text: None,
                original_html: None,
                sentences: vec![
                    sentence(
                        "Hello world.",
                        vec![MarkerContent {
                            system: "verse",
                            ref_value: "1:1",
                            char_offset: None,
                        }],
                    ),
                    sentence("Bye.", Vec::new()),
                ],
            }],
        }
    }

    // Golden value: any change to field order / separators / the algorithm
    // breaks this on purpose. If the change is intentional, treat it as a data
    // migration — bump and run `--full` once to rewrite every stored hash.
    #[test]
    fn node_hash_is_stable() {
        assert_eq!(
            node_hash(&sample()),
            "60bbd1ebd752459c9c1e60cfdd4ea23b24015905ede4f13ffe6c106bffe2de92"
        );
    }

    #[test]
    fn splitting_a_sentence_changes_the_hash() {
        let mut other = sample();
        other.blocks[0].sentences = vec![
            sentence(
                "Hello.",
                vec![MarkerContent {
                    system: "verse",
                    ref_value: "1:1",
                    char_offset: None,
                }],
            ),
            sentence("World.", Vec::new()),
            sentence("Bye.", Vec::new()),
        ];
        assert_ne!(node_hash(&sample()), node_hash(&other));
    }

    #[test]
    fn none_differs_from_empty_string() {
        let mut a = sample();
        a.blocks[0].original_text = None;
        let mut b = sample();
        b.blocks[0].original_text = Some("");
        assert_ne!(node_hash(&a), node_hash(&b));
    }

    #[test]
    fn root_hash_depends_on_order() {
        let a = root_hash(&["x".to_string(), "y".to_string()]);
        let b = root_hash(&["y".to_string(), "x".to_string()]);
        assert_ne!(a, b);
    }
}
