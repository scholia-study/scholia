use std::env;
use std::fs;

use common::model::{BlockType, Book, TocNode};
use sqlx::PgPool;
use uuid::Uuid;

struct NodeWork<'a> {
    node: &'a TocNode,
    parent_id: Option<Uuid>,
    parent_path: String,
}

struct Counts {
    nodes: u32,
    blocks: u32,
    sentences: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let json_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "assets/wdl.json".to_string());

    let data = fs::read_to_string(&json_path)?;
    let book: Book = serde_json::from_str(&data)?;

    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;

    let counts = import(&pool, &book).await?;

    println!("Imported:");
    println!("  1 book");
    println!("  {} toc_nodes", counts.nodes);
    println!("  {} content_blocks", counts.blocks);
    println!("  {} sentences", counts.sentences);

    Ok(())
}

async fn import(pool: &PgPool, book: &Book) -> Result<Counts, Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;

    sqlx::query("TRUNCATE sentences, content_blocks, toc_nodes, books CASCADE")
        .execute(&mut *tx)
        .await?;

    let book_id = Uuid::new_v4();
    let slug = slugify(&book.title);

    sqlx::query(
        "INSERT INTO books (id, slug, title, author, language, source, source_date)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(book_id)
    .bind(&slug)
    .bind(&book.title)
    .bind(&book.author)
    .bind(&book.language)
    .bind(&book.source)
    .bind(&book.date)
    .execute(&mut *tx)
    .await?;

    // Iterative DFS over the node tree
    let mut stack: Vec<NodeWork> = Vec::new();
    let mut counts = Counts {
        nodes: 0,
        blocks: 0,
        sentences: 0,
    };

    // Push root nodes in reverse so they process in order
    for node in book.nodes.iter().rev() {
        stack.push(NodeWork {
            node,
            parent_id: None,
            parent_path: String::new(),
        });
    }

    while let Some(work) = stack.pop() {
        let node = work.node;
        let node_id = Uuid::new_v4();
        let ltree_label = ncx_to_ltree(&node.ncx_id);
        let path = if work.parent_path.is_empty() {
            ltree_label.clone()
        } else {
            format!("{}.{}", work.parent_path, ltree_label)
        };

        sqlx::query(
            "INSERT INTO toc_nodes (id, book_id, parent_id, ncx_id, path, play_order, depth, label)
             VALUES ($1, $2, $3, $4, $5::ltree, $6, $7, $8)",
        )
        .bind(node_id)
        .bind(book_id)
        .bind(work.parent_id)
        .bind(&node.ncx_id)
        .bind(&path)
        .bind(node.play_order as i32)
        .bind(node.depth as i16)
        .bind(&node.label)
        .execute(&mut *tx)
        .await?;
        counts.nodes += 1;

        for block in &node.content {
            let block_id = Uuid::new_v4();
            let block_type_str = match block.block_type {
                BlockType::Paragraph => "paragraph",
                BlockType::Heading => "heading",
                BlockType::Footnote => "footnote",
                BlockType::Separator => "separator",
            };

            sqlx::query(
                "INSERT INTO content_blocks (id, book_id, node_id, position, block_type, paragraph_number, text, html, page_ref)
                 VALUES ($1, $2, $3, $4, $5::block_type, $6, $7, $8, $9)",
            )
            .bind(block_id)
            .bind(book_id)
            .bind(node_id)
            .bind(block.position as i16)
            .bind(block_type_str)
            .bind(block.paragraph_number.map(|n| n as i32))
            .bind(&block.text)
            .bind(&block.html)
            .bind(block.page_ref.as_deref())
            .execute(&mut *tx)
            .await?;
            counts.blocks += 1;

            for sentence in &block.sentences {
                let sentence_id = Uuid::new_v4();

                sqlx::query(
                    "INSERT INTO sentences (id, book_id, node_id, block_id, position, sentence_number, text, html)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                )
                .bind(sentence_id)
                .bind(book_id)
                .bind(node_id)
                .bind(block_id)
                .bind(sentence.position as i16)
                .bind(sentence.sentence_number as i32)
                .bind(&sentence.text)
                .bind(&sentence.html)
                .execute(&mut *tx)
                .await?;
                counts.sentences += 1;
            }
        }

        // Push children in reverse so they process in order
        for child in node.children.iter().rev() {
            stack.push(NodeWork {
                node: child,
                parent_id: Some(node_id),
                parent_path: path.clone(),
            });
        }
    }

    tx.commit().await?;
    Ok(counts)
}

fn slugify(title: &str) -> String {
    let lower = title.to_lowercase();
    let slug: String = lower
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    // Collapse runs of hyphens
    let mut result = String::with_capacity(slug.len());
    let mut prev_hyphen = false;
    for c in slug.chars() {
        if c == '-' {
            if !prev_hyphen {
                result.push('-');
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }
    result.trim_matches('-').to_string()
}

fn ncx_to_ltree(ncx_id: &str) -> String {
    ncx_id.replace('-', "_")
}
