//! A tiny regex-driven tokenizer for Perseus's EpiDoc TEI, not a general XML
//! parser. The body's element set is small and fixed — verified by counting
//! (see `ALLOWED_ELEMENTS`) — so a hand-rolled tag scanner plus an explicit
//! open-element stack is simpler and easier to audit than pulling in a DOM
//! crate for a document this regular. No entities, no CDATA, no comments and
//! no unquoted/single-quoted attributes appear in the source, so none of
//! that is handled.

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

/// Exhaustive, per the spec's element count. Anything else in the body is a
/// shape the converter was not told about and must not silently swallow.
pub const ALLOWED_ELEMENTS: &[&str] = &[
    "milestone",
    "div",
    "p",
    "said",
    "label",
    "l",
    "quote",
    "bibl",
    "q",
    "del",
    "add",
    "gap",
    "cit",
];

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Text(String),
    Element {
        name: String,
        attrs: HashMap<String, String>,
        children: Vec<Node>,
    },
}

static TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<(/?)([A-Za-z][A-Za-z0-9]*)([^>]*?)(/?)>").unwrap());
static ATTR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"([A-Za-z][\w:.\-]*)="([^"]*)""#).unwrap());

/// Locate the `<body ...> ... </body>` span in a raw TEI document. There is
/// exactly one `<body>` in an EpiDoc file (no nested `body`s), so a plain
/// substring search is safe.
pub fn extract_body(xml: &str) -> Result<&str, String> {
    let start = xml
        .find("<body")
        .ok_or("no <body> element found in TEI file")?;
    let end_tag = xml[start..]
        .find("</body>")
        .ok_or("no closing </body> found in TEI file")?;
    Ok(&xml[start..start + end_tag + "</body>".len()])
}

fn parse_attrs(raw: &str) -> HashMap<String, String> {
    ATTR_RE
        .captures_iter(raw)
        .map(|c| (c[1].to_string(), c[2].to_string()))
        .collect()
}

type Frame = (String, HashMap<String, String>, Vec<Node>);

fn push_node(stack: &mut [Frame], top: &mut Vec<Node>, node: Node) {
    match stack.last_mut() {
        Some((_, _, children)) => children.push(node),
        None => top.push(node),
    }
}

/// Parse the inside of `<body>...</body>` (pass the whole `<body>...</body>`
/// span; the wrapping tag itself is skipped like any other allowed element
/// would be — it isn't in `ALLOWED_ELEMENTS` on purpose, since "the body" is
/// the scope the spec's element count is verified against, not an element
/// inside it) into a flat list of top-level nodes.
pub fn parse_body(body_span: &str) -> Result<Vec<Node>, String> {
    let inner_start = body_span.find('>').map(|i| i + 1).unwrap_or(0);
    let inner_end = body_span.rfind("</body>").unwrap_or(body_span.len());
    let body = &body_span[inner_start..inner_end];

    let mut stack: Vec<Frame> = Vec::new();
    let mut top: Vec<Node> = Vec::new();
    let mut last_end = 0usize;

    for caps in TAG_RE.captures_iter(body) {
        let m = caps.get(0).unwrap();
        let text = &body[last_end..m.start()];
        if !text.is_empty() {
            push_node(&mut stack, &mut top, Node::Text(text.to_string()));
        }
        last_end = m.end();

        let is_close = &caps[1] == "/";
        let name = caps[2].to_string();
        let self_close = &caps[4] == "/";

        if !ALLOWED_ELEMENTS.contains(&name.as_str()) {
            return Err(format!(
                "unexpected element <{}{}> in TEI body — not in the verified element set",
                if is_close { "/" } else { "" },
                name
            ));
        }

        if is_close {
            let (open_name, attrs, children) = stack
                .pop()
                .ok_or_else(|| format!("unmatched closing tag </{name}>"))?;
            if open_name != name {
                return Err(format!(
                    "mismatched close: expected </{open_name}>, found </{name}>"
                ));
            }
            push_node(
                &mut stack,
                &mut top,
                Node::Element {
                    name: open_name,
                    attrs,
                    children,
                },
            );
        } else {
            let attrs = parse_attrs(&caps[3]);
            if self_close {
                push_node(
                    &mut stack,
                    &mut top,
                    Node::Element {
                        name,
                        attrs,
                        children: Vec::new(),
                    },
                );
            } else {
                stack.push((name, attrs, Vec::new()));
            }
        }
    }
    let tail = &body[last_end..];
    if !tail.is_empty() {
        push_node(&mut stack, &mut top, Node::Text(tail.to_string()));
    }

    if !stack.is_empty() {
        let open: Vec<&str> = stack.iter().map(|(n, _, _)| n.as_str()).collect();
        return Err(format!("unclosed element(s) at end of body: {open:?}"));
    }

    Ok(top)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flat_text_and_element() {
        let nodes = parse_body("<body>πολέμου <add>καὶ</add> μάχης.</body>").unwrap();
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0], Node::Text("πολέμου ".to_string()));
        match &nodes[1] {
            Node::Element { name, children, .. } => {
                assert_eq!(name, "add");
                assert_eq!(children, &[Node::Text("καὶ".to_string())]);
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn parses_self_closing_with_attrs() {
        let nodes =
            parse_body(r#"<body><milestone unit="section" resp="Stephanus" n="447a"/></body>"#)
                .unwrap();
        match &nodes[0] {
            Node::Element { name, attrs, .. } => {
                assert_eq!(name, "milestone");
                assert_eq!(attrs.get("unit").map(String::as_str), Some("section"));
                assert_eq!(attrs.get("n").map(String::as_str), Some("447a"));
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn parses_nested_elements() {
        let nodes =
            parse_body(r#"<body><quote type="verse"><l>a</l><l>b <add>c</add></l></quote></body>"#)
                .unwrap();
        let Node::Element { children, .. } = &nodes[0] else {
            panic!("expected element")
        };
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn rejects_unexpected_element() {
        let err = parse_body("<body><foreign>bar</foreign></body>").unwrap_err();
        assert!(
            err.contains("foreign"),
            "error should name the element: {err}"
        );
    }

    #[test]
    fn rejects_unclosed_element() {
        let err = parse_body("<body><add>unterminated</body>").unwrap_err();
        assert!(err.contains("unclosed") || err.contains("mismatched"));
    }

    #[test]
    fn extract_body_finds_span() {
        let xml = r#"<TEI><teiHeader><p>ignore me</p></teiHeader><text><body><p>real</p></body></text></TEI>"#;
        let span = extract_body(xml).unwrap();
        assert!(span.starts_with("<body"));
        assert!(span.ends_with("</body>"));
        assert!(!span.contains("ignore me"));
    }
}
