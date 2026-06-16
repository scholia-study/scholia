//! Curated sonnet MD (md_modernized + md_reviewed) → struct JSON.
//!
//! The two layers are parsed independently and paired line-by-line (modern →
//! text/html, old → original_text/original_html), exactly as Kant pairs its two
//! editions. The canonical 154-sonnet TOC (`common::shakespeare1`) is the
//! guard: a missing, extra, or misnamed curated file — or front-matter that
//! doesn't match the expected position/label/depth — is a hard error.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use clap::Parser;
use common::shakespeare1 as sonnets;

use shakespeare1_md_to_struct::html::{md_to_html, md_to_plain};
use shakespeare1_md_to_struct::model::*;

#[derive(Parser)]
#[command(about = "Parse curated Shakespeare sonnet markdown into DB-ready JSON")]
struct Cli {
    #[arg(long, default_value = "assets/shakespeare1/curated/md_modernized")]
    modernized_dir: String,
    #[arg(long, default_value = "assets/shakespeare1/curated/md_reviewed")]
    reviewed_dir: String,
    #[arg(long, default_value = "assets/shakespeare1/derived/output.json")]
    output_file: String,
}

struct FrontMatter {
    position: u32,
    label: String,
    depth: i16,
}

/// A parsed content block from a sonnet body: a `## N` heading or a verse stanza.
#[derive(Clone, Copy, PartialEq)]
enum BlockKind {
    Heading,
    Verse,
}

struct ParsedBlock {
    kind: BlockKind,
    lines: Vec<String>,
}

type Stanzas = Vec<ParsedBlock>;

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(&cli) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let modernized_dir = Path::new(&cli.modernized_dir);
    let reviewed_dir = Path::new(&cli.reviewed_dir);

    let expected = sonnets::all_filenames();
    let expected_names: HashSet<&str> = expected.iter().map(|(_, f)| f.as_str()).collect();

    // Guard: both dirs must contain exactly the canonical file set.
    let modernized_files = scan_dir(modernized_dir)?;
    let reviewed_files = scan_dir(reviewed_dir)?;
    for (n, fname) in &expected {
        if !modernized_files.contains(fname) {
            return Err(format!("missing {fname} in {} (sonnet {n})", cli.modernized_dir).into());
        }
        if !reviewed_files.contains(fname) {
            return Err(format!("missing {fname} in {} (sonnet {n})", cli.reviewed_dir).into());
        }
    }
    for f in modernized_files.iter().chain(reviewed_files.iter()) {
        if !expected_names.contains(f.as_str()) {
            return Err(format!("unexpected curated file: {f}").into());
        }
    }

    let mut toc_nodes = Vec::new();
    let mut sentence_number = 1i32; // global per-book line enumeration

    // The "Sonnets" work: a depth-0, source-anchored node (Bible-shape pill).
    // Pure navigation container — no content of its own; the 154 sonnets are its
    // children.
    toc_nodes.push(TocNodeData {
        source_ref: sonnets::SONNETS_SOURCE_REF.to_string(),
        slug: sonnets::SONNETS_SLUG.to_string(),
        path: sonnets::SONNETS_PATH.to_string(),
        sort_order: 0,
        depth: 0,
        label: sonnets::SONNETS_LABEL.to_string(),
        label_html: sonnets::SONNETS_LABEL.to_string(),
        parent_source_ref: None,
        source: Some(NodeSource {
            title: sonnets::SONNETS_SOURCE_TITLE.to_string(),
            publication_year: Some(sonnets::SONNETS_YEAR),
        }),
        content_blocks: vec![],
    });

    for (n, fname) in &expected {
        let (m_fm, m_blocks) = parse_file(modernized_dir, fname)?;
        let (r_fm, r_blocks) = parse_file(reviewed_dir, fname)?;

        validate_front_matter(&m_fm, *n, fname, "modernized")?;
        validate_front_matter(&r_fm, *n, fname, "reviewed")?;

        if m_blocks.len() != r_blocks.len() {
            return Err(format!(
                "{fname}: block count mismatch — modernized {}, reviewed {}",
                m_blocks.len(),
                r_blocks.len()
            )
            .into());
        }

        toc_nodes.push(build_node(
            *n,
            &m_fm,
            &m_blocks,
            &r_blocks,
            &mut sentence_number,
        )?);
    }

    let output = Output {
        book: BookData {
            slug: sonnets::BOOK_SLUG.into(),
            title: sonnets::BOOK_TITLE.into(),
            author: "William Shakespeare".into(),
            language: "en".into(),
            source: "A Scholia compilation; each work carries its own source provenance.".into(),
            source_date: String::new(),
        },
        reference_systems: vec![ReferenceSystemData {
            slug: "line".into(),
            label: "Line".into(),
            ref_type: "block".into(),
        }],
        toc_nodes,
    };

    let json = serde_json::to_string_pretty(&output)?;
    if let Some(parent) = Path::new(&cli.output_file).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&cli.output_file, json)?;

    let lines: usize = output
        .toc_nodes
        .iter()
        .flat_map(|nd| &nd.content_blocks)
        .map(|b| b.sentences.len())
        .sum();
    eprintln!("=== output summary ===");
    eprintln!("  toc_nodes:      {}", output.toc_nodes.len());
    eprintln!("  lines:          {lines}");
    eprintln!("  wrote {}", cli.output_file);
    Ok(())
}

fn build_node(
    n: u32,
    fm: &FrontMatter,
    modern: &[ParsedBlock],
    reviewed: &[ParsedBlock],
    sentence_number: &mut i32,
) -> Result<TocNodeData, Box<dyn std::error::Error>> {
    let mut content_blocks = Vec::with_capacity(modern.len());
    let mut line_in_node = 0i32; // canonical line number, reset per sonnet

    for (block_pos, (mb, rb)) in modern.iter().zip(reviewed.iter()).enumerate() {
        if mb.kind != rb.kind {
            return Err(format!(
                "Sonnet {n} block {block_pos}: block-kind mismatch between layers"
            )
            .into());
        }
        if mb.lines.len() != rb.lines.len() {
            return Err(format!(
                "Sonnet {n} block {block_pos}: line count mismatch — modernized {}, reviewed {}",
                mb.lines.len(),
                rb.lines.len()
            )
            .into());
        }

        match mb.kind {
            BlockKind::Heading => {
                // The `## N` sonnet-number heading. One non-numbered sentence;
                // no line marker (it isn't a verse line).
                let m_text = mb.lines[0].trim();
                let r_text = rb.lines[0].trim();
                content_blocks.push(ContentBlockData {
                    position: block_pos as i16,
                    block_type: "heading".into(),
                    paragraph_number: None,
                    figure_number: None,
                    text: md_to_plain(m_text),
                    html: md_to_html(m_text),
                    original_text: Some(md_to_plain(r_text)),
                    original_html: Some(md_to_html(r_text)),
                    sentences: vec![SentenceData {
                        position: 0,
                        sentence_number: None,
                        segment: None,
                        indent: None,
                        text: md_to_plain(m_text),
                        html: md_to_html(m_text),
                        original_text: Some(md_to_plain(r_text)),
                        original_html: Some(md_to_html(r_text)),
                        page_markers: vec![],
                    }],
                });
            }
            BlockKind::Verse => {
                let mut sentences = Vec::with_capacity(mb.lines.len());
                for (i, (m_raw, r_raw)) in mb.lines.iter().zip(rb.lines.iter()).enumerate() {
                    let (indent, m_text) = strip_indent(m_raw);
                    let r_text = r_raw.trim();
                    line_in_node += 1;

                    sentences.push(SentenceData {
                        position: i as i16,
                        sentence_number: Some(*sentence_number),
                        segment: None,
                        indent,
                        text: md_to_plain(&m_text),
                        html: md_to_html(&m_text),
                        original_text: Some(md_to_plain(r_text)),
                        original_html: Some(md_to_html(r_text)),
                        page_markers: vec![PageMarkerData {
                            system: "line".into(),
                            ref_value: line_in_node.to_string(),
                            sort_order: line_in_node - 1,
                            char_offset: 0,
                        }],
                    });
                    *sentence_number += 1;
                }

                let m_clean: Vec<String> = mb.lines.iter().map(|l| strip_indent(l).1).collect();
                let r_clean: Vec<String> = rb.lines.iter().map(|l| l.trim().to_string()).collect();
                content_blocks.push(ContentBlockData {
                    position: block_pos as i16,
                    block_type: "verse".into(),
                    paragraph_number: None,
                    figure_number: None,
                    text: m_clean.join("\n"),
                    html: join_html(&m_clean),
                    original_text: Some(r_clean.join("\n")),
                    original_html: Some(join_html(&r_clean)),
                    sentences,
                });
            }
        }
    }

    Ok(TocNodeData {
        source_ref: sonnets::source_ref(n),
        slug: sonnets::slug(n),
        path: sonnets::path(n),
        sort_order: n as i32,
        depth: fm.depth,
        label: fm.label.clone(),
        label_html: fm.label.clone(),
        parent_source_ref: Some(sonnets::SONNETS_SOURCE_REF.to_string()),
        source: None,
        content_blocks,
    })
}

fn join_html(lines: &[String]) -> String {
    lines
        .iter()
        .map(|l| md_to_html(l))
        .collect::<Vec<_>>()
        .join("<br>\n")
}

/// Leading whitespace → indent level (2 spaces per level), and the de-indented
/// line. Sonnets are flush, so this is normally `(None, line)`.
fn strip_indent(line: &str) -> (Option<i16>, String) {
    let trimmed = line.trim_start();
    let spaces = line.len() - trimmed.len();
    let indent = (spaces >= 2).then_some((spaces / 2) as i16);
    (indent, trimmed.trim_end().to_string())
}

fn scan_dir(dir: &Path) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let mut out = HashSet::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))? {
        let name = entry?.file_name().to_string_lossy().to_string();
        if name.ends_with(".md") && name != "000_toc.md" {
            out.insert(name);
        }
    }
    Ok(out)
}

fn parse_file(
    dir: &Path,
    fname: &str,
) -> Result<(FrontMatter, Stanzas), Box<dyn std::error::Error>> {
    let path = dir.join(fname);
    let content =
        fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let (fm, body) = parse_front_matter(&content)
        .ok_or_else(|| format!("no front matter in {}", path.display()))?;
    Ok((fm, parse_blocks(body)))
}

fn parse_front_matter(content: &str) -> Option<(FrontMatter, &str)> {
    let rest = content.trim_start_matches('\u{feff}').strip_prefix("---")?;
    let close = rest.find("\n---")?;
    let fm_text = &rest[..close];
    let body = rest[close + "\n---".len()..]
        .trim_start_matches('\n')
        .trim_end_matches('\n');

    let (mut position, mut label, mut depth) = (None, None, None);
    for line in fm_text.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("position:") {
            position = v.trim().parse().ok();
        } else if let Some(v) = line.strip_prefix("label:") {
            label = Some(v.trim().trim_matches('"').to_string());
        } else if let Some(v) = line.strip_prefix("depth:") {
            depth = v.trim().parse().ok();
        }
    }
    Some((
        FrontMatter {
            position: position?,
            label: label?,
            depth: depth?,
        },
        body,
    ))
}

/// Split a body into blocks: a `## …` line is its own heading block; runs of
/// other non-blank lines (separated by blank lines) are verse stanzas. Leading
/// whitespace on verse lines is preserved for indent detection.
fn parse_blocks(body: &str) -> Vec<ParsedBlock> {
    let mut blocks = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            flush_verse(&mut blocks, &mut cur);
        } else if let Some(heading) = trimmed.strip_prefix("## ") {
            flush_verse(&mut blocks, &mut cur);
            blocks.push(ParsedBlock {
                kind: BlockKind::Heading,
                lines: vec![heading.trim().to_string()],
            });
        } else {
            cur.push(line.to_string());
        }
    }
    flush_verse(&mut blocks, &mut cur);
    blocks
}

fn flush_verse(blocks: &mut Vec<ParsedBlock>, cur: &mut Vec<String>) {
    if !cur.is_empty() {
        blocks.push(ParsedBlock {
            kind: BlockKind::Verse,
            lines: std::mem::take(cur),
        });
    }
}

fn validate_front_matter(
    fm: &FrontMatter,
    n: u32,
    fname: &str,
    layer: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if fm.position != n {
        return Err(format!(
            "{fname} ({layer}): position {} != expected {n}",
            fm.position
        )
        .into());
    }
    if fm.label != sonnets::label(n) {
        return Err(format!(
            "{fname} ({layer}): label {:?} != expected {:?}",
            fm.label,
            sonnets::label(n)
        )
        .into());
    }
    if fm.depth != sonnets::DEPTH {
        return Err(format!(
            "{fname} ({layer}): depth {} != expected {}",
            fm.depth,
            sonnets::DEPTH
        )
        .into());
    }
    Ok(())
}
