use crate::model::TocNode;
use quick_xml::de::from_str;
use serde::Deserialize;

// --- Raw NCX XML structures for deserialization ---

#[derive(Debug, Deserialize)]
#[serde(rename = "ncx")]
struct Ncx {
    #[serde(rename = "docTitle")]
    doc_title: DocTitle,
    #[serde(rename = "navMap")]
    nav_map: NavMap,
}

#[derive(Debug, Deserialize)]
struct DocTitle {
    text: String,
}

#[derive(Debug, Deserialize)]
struct NavMap {
    #[serde(rename = "navPoint", default)]
    nav_points: Vec<RawNavPoint>,
}

#[derive(Debug, Deserialize)]
struct RawNavPoint {
    #[serde(rename = "@playOrder")]
    play_order: u32,
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "navLabel")]
    nav_label: NavLabel,
    content: NavContent,
    #[serde(rename = "navPoint", default)]
    children: Vec<RawNavPoint>,
}

#[derive(Debug, Deserialize)]
struct NavLabel {
    text: String,
}

#[derive(Debug, Deserialize)]
struct NavContent {
    #[serde(rename = "@src")]
    src: String,
}

/// Parsed navigation source, split into file path and optional fragment.
#[derive(Debug, Clone)]
pub struct NavSrc {
    pub file: String,
    pub fragment: Option<String>,
}

/// Intermediate representation after NCX parsing, before content extraction.
#[derive(Debug, Clone)]
pub struct NcxNode {
    pub ncx_id: String,
    pub play_order: u32,
    pub label: String,
    pub src: NavSrc,
    pub children: Vec<NcxNode>,
}

/// Parse the toc.ncx XML string and return the document title and a tree of NcxNodes.
pub fn parse_ncx(xml: &str) -> Result<(String, Vec<NcxNode>), Box<dyn std::error::Error>> {
    let ncx: Ncx = from_str(xml)?;
    let title = ncx.doc_title.text;
    let nodes = ncx
        .nav_map
        .nav_points
        .into_iter()
        .map(convert_nav_point)
        .collect();
    Ok((title, nodes))
}

fn convert_nav_point(raw: RawNavPoint) -> NcxNode {
    let (file, fragment) = match raw.content.src.split_once('#') {
        Some((f, frag)) => (f.to_string(), Some(frag.to_string())),
        None => (raw.content.src, None),
    };
    let children = raw.children.into_iter().map(convert_nav_point).collect();
    NcxNode {
        ncx_id: raw.id,
        play_order: raw.play_order,
        label: raw.nav_label.text,
        src: NavSrc { file, fragment },
        children,
    }
}

/// Flatten the NCX tree into a depth-first ordered list, preserving depth info.
/// Returns (NcxNode-without-children, depth) tuples for boundary calculation.
pub fn flatten_ncx(nodes: &[NcxNode], depth: u16) -> Vec<(String, u32, String, NavSrc, u16)> {
    let mut result = Vec::new();
    for node in nodes {
        result.push((
            node.ncx_id.clone(),
            node.play_order,
            node.label.clone(),
            node.src.clone(),
            depth,
        ));
        result.extend(flatten_ncx(&node.children, depth + 1));
    }
    result
}

/// Convert NcxNode tree into TocNode tree (without content — that's filled in later).
pub fn ncx_to_toc_nodes(nodes: &[NcxNode], depth: u16) -> Vec<TocNode> {
    nodes
        .iter()
        .map(|n| TocNode {
            ncx_id: n.ncx_id.clone(),
            play_order: n.play_order,
            label: n.label.clone(),
            depth,
            children: ncx_to_toc_nodes(&n.children, depth + 1),
            content: Vec::new(),
        })
        .collect()
}
