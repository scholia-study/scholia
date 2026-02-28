use common::content::{extract_all_content, number_and_split};
use common::epub_reader::EpubReader;
use common::model::{BlockType, Book};
use common::ncx::{ncx_to_toc_nodes, parse_ncx};
use common::opf::parse_opf;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let epub_path = Path::new("assets/wdl.epub");
    let output_path = Path::new("assets/wdl.json");

    println!("Opening EPUB: {}", epub_path.display());
    let mut reader = EpubReader::open(epub_path)?;

    // Phase 1: Parse NCX
    println!("Parsing NCX...");
    let ncx_xml = reader.read_file("toc.ncx")?;
    let (title, ncx_nodes) = parse_ncx(&ncx_xml)?;
    println!("Title: {}", title);
    println!("Top-level nodes: {}", ncx_nodes.len());

    // Phase 2: Parse OPF metadata
    println!("Parsing OPF metadata...");
    let opf_xml = reader.read_file("inhalt.opf")?;
    let metadata = parse_opf(&opf_xml)?;
    println!("Author: {}", metadata.author);
    println!("Language: {}", metadata.language);

    // Phase 3: Build TocNode tree and extract content
    println!("Extracting content...");
    let mut toc_nodes = ncx_to_toc_nodes(&ncx_nodes, 1);
    extract_all_content(&mut reader, &mut toc_nodes, &ncx_nodes)?;

    // Phase 4: Number paragraphs and split sentences
    println!("Numbering paragraphs and splitting sentences...");
    number_and_split(&mut toc_nodes);

    // Phase 5: Serialize
    let book = Book {
        title,
        author: metadata.author,
        language: metadata.language,
        publisher: metadata.publisher,
        date: metadata.date,
        nodes: toc_nodes,
    };

    let json = serde_json::to_string_pretty(&book)?;
    fs::write(output_path, &json)?;
    println!("Wrote {} bytes to {}", json.len(), output_path.display());

    // Print summary stats
    print_stats(&book);

    Ok(())
}

fn print_stats(book: &Book) {
    let mut total_nodes = 0;
    let mut total_blocks = 0;
    let mut nodes_with_content = 0;
    let mut total_paragraphs = 0;
    let mut total_sentences = 0;

    fn count(
        nodes: &[common::model::TocNode],
        total: &mut usize,
        blocks: &mut usize,
        with_content: &mut usize,
        paragraphs: &mut usize,
        sentences: &mut usize,
    ) {
        for node in nodes {
            *total += 1;
            *blocks += node.content.len();
            if !node.content.is_empty() {
                *with_content += 1;
            }
            for block in &node.content {
                if block.block_type == BlockType::Paragraph {
                    *paragraphs += 1;
                    *sentences += block.sentences.len();
                }
            }
            count(
                &node.children,
                total,
                blocks,
                with_content,
                paragraphs,
                sentences,
            );
        }
    }

    count(
        &book.nodes,
        &mut total_nodes,
        &mut total_blocks,
        &mut nodes_with_content,
        &mut total_paragraphs,
        &mut total_sentences,
    );
    println!("\nStats:");
    println!("  Total TOC nodes: {}", total_nodes);
    println!("  Nodes with content: {}", nodes_with_content);
    println!("  Total content blocks: {}", total_blocks);
    println!("  Total paragraphs: {}", total_paragraphs);
    println!("  Total sentences: {}", total_sentences);
}
