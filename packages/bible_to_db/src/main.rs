use std::fs;
use std::path::PathBuf;

use clap::Parser;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Parser)]
#[command(about = "Import KJV/WEB Bible chapters from assets/bible into PostgreSQL")]
struct Cli {
    /// Translation slug: kjv | web
    #[arg(long)]
    translation: String,

    /// Override DATABASE_URL
    #[arg(long)]
    database_url: Option<String>,

    /// Root directory for cached chapter JSON
    #[arg(long, default_value = "assets/bible")]
    assets_dir: String,
}

#[derive(Deserialize)]
struct Chapter {
    verses: Vec<Verse>,
}

#[derive(Deserialize)]
struct Verse {
    chapter: u32,
    verse: u32,
    text: String,
}

struct TranslationMeta {
    slug: &'static str,
    full_title: &'static str,
    publication_year: i16,
    book_slug: &'static str,
    publisher: &'static str,
}

const TRANSLATIONS: &[TranslationMeta] = &[
    // `publisher` doubles as the same-language version-pill label (KJV /
    // WEB). Both translations are public domain; if a real publisher
    // matters in citations later, restructure with a short_label column.
    TranslationMeta {
        slug: "kjv",
        full_title: "King James Bible",
        publication_year: 1611,
        book_slug: "kjv-bible",
        publisher: "KJV",
    },
    TranslationMeta {
        slug: "web",
        full_title: "World English Bible",
        publication_year: 2000,
        book_slug: "web-bible",
        publisher: "WEB",
    },
];

struct BibleBook {
    slug: &'static str,
    label: &'static str,
    chapters: u32,
}

// Reading order matters — sort_order is assigned monotonically below.
// Slugs are space-less lowercase to match bible-api.com's normalized
// book identifiers (so the same string serves as DB slug, filesystem
// dir, and URL fragment).
const BIBLE_BOOKS: &[BibleBook] = &[
    // Old Testament
    BibleBook {
        slug: "genesis",
        label: "Genesis",
        chapters: 50,
    },
    BibleBook {
        slug: "exodus",
        label: "Exodus",
        chapters: 40,
    },
    BibleBook {
        slug: "leviticus",
        label: "Leviticus",
        chapters: 27,
    },
    BibleBook {
        slug: "numbers",
        label: "Numbers",
        chapters: 36,
    },
    BibleBook {
        slug: "deuteronomy",
        label: "Deuteronomy",
        chapters: 34,
    },
    BibleBook {
        slug: "joshua",
        label: "Joshua",
        chapters: 24,
    },
    BibleBook {
        slug: "judges",
        label: "Judges",
        chapters: 21,
    },
    BibleBook {
        slug: "ruth",
        label: "Ruth",
        chapters: 4,
    },
    BibleBook {
        slug: "1samuel",
        label: "1 Samuel",
        chapters: 31,
    },
    BibleBook {
        slug: "2samuel",
        label: "2 Samuel",
        chapters: 24,
    },
    BibleBook {
        slug: "1kings",
        label: "1 Kings",
        chapters: 22,
    },
    BibleBook {
        slug: "2kings",
        label: "2 Kings",
        chapters: 25,
    },
    BibleBook {
        slug: "1chronicles",
        label: "1 Chronicles",
        chapters: 29,
    },
    BibleBook {
        slug: "2chronicles",
        label: "2 Chronicles",
        chapters: 36,
    },
    BibleBook {
        slug: "ezra",
        label: "Ezra",
        chapters: 10,
    },
    BibleBook {
        slug: "nehemiah",
        label: "Nehemiah",
        chapters: 13,
    },
    BibleBook {
        slug: "esther",
        label: "Esther",
        chapters: 10,
    },
    BibleBook {
        slug: "job",
        label: "Job",
        chapters: 42,
    },
    BibleBook {
        slug: "psalms",
        label: "Psalms",
        chapters: 150,
    },
    BibleBook {
        slug: "proverbs",
        label: "Proverbs",
        chapters: 31,
    },
    BibleBook {
        slug: "ecclesiastes",
        label: "Ecclesiastes",
        chapters: 12,
    },
    BibleBook {
        slug: "songofsolomon",
        label: "Song of Solomon",
        chapters: 8,
    },
    BibleBook {
        slug: "isaiah",
        label: "Isaiah",
        chapters: 66,
    },
    BibleBook {
        slug: "jeremiah",
        label: "Jeremiah",
        chapters: 52,
    },
    BibleBook {
        slug: "lamentations",
        label: "Lamentations",
        chapters: 5,
    },
    BibleBook {
        slug: "ezekiel",
        label: "Ezekiel",
        chapters: 48,
    },
    BibleBook {
        slug: "daniel",
        label: "Daniel",
        chapters: 12,
    },
    BibleBook {
        slug: "hosea",
        label: "Hosea",
        chapters: 14,
    },
    BibleBook {
        slug: "joel",
        label: "Joel",
        chapters: 3,
    },
    BibleBook {
        slug: "amos",
        label: "Amos",
        chapters: 9,
    },
    BibleBook {
        slug: "obadiah",
        label: "Obadiah",
        chapters: 1,
    },
    BibleBook {
        slug: "jonah",
        label: "Jonah",
        chapters: 4,
    },
    BibleBook {
        slug: "micah",
        label: "Micah",
        chapters: 7,
    },
    BibleBook {
        slug: "nahum",
        label: "Nahum",
        chapters: 3,
    },
    BibleBook {
        slug: "habakkuk",
        label: "Habakkuk",
        chapters: 3,
    },
    BibleBook {
        slug: "zephaniah",
        label: "Zephaniah",
        chapters: 3,
    },
    BibleBook {
        slug: "haggai",
        label: "Haggai",
        chapters: 2,
    },
    BibleBook {
        slug: "zechariah",
        label: "Zechariah",
        chapters: 14,
    },
    BibleBook {
        slug: "malachi",
        label: "Malachi",
        chapters: 4,
    },
    // New Testament
    BibleBook {
        slug: "matthew",
        label: "Matthew",
        chapters: 28,
    },
    BibleBook {
        slug: "mark",
        label: "Mark",
        chapters: 16,
    },
    BibleBook {
        slug: "luke",
        label: "Luke",
        chapters: 24,
    },
    BibleBook {
        slug: "john",
        label: "John",
        chapters: 21,
    },
    BibleBook {
        slug: "acts",
        label: "Acts",
        chapters: 28,
    },
    BibleBook {
        slug: "romans",
        label: "Romans",
        chapters: 16,
    },
    BibleBook {
        slug: "1corinthians",
        label: "1 Corinthians",
        chapters: 16,
    },
    BibleBook {
        slug: "2corinthians",
        label: "2 Corinthians",
        chapters: 13,
    },
    BibleBook {
        slug: "galatians",
        label: "Galatians",
        chapters: 6,
    },
    BibleBook {
        slug: "ephesians",
        label: "Ephesians",
        chapters: 6,
    },
    BibleBook {
        slug: "philippians",
        label: "Philippians",
        chapters: 4,
    },
    BibleBook {
        slug: "colossians",
        label: "Colossians",
        chapters: 4,
    },
    BibleBook {
        slug: "1thessalonians",
        label: "1 Thessalonians",
        chapters: 5,
    },
    BibleBook {
        slug: "2thessalonians",
        label: "2 Thessalonians",
        chapters: 3,
    },
    BibleBook {
        slug: "1timothy",
        label: "1 Timothy",
        chapters: 6,
    },
    BibleBook {
        slug: "2timothy",
        label: "2 Timothy",
        chapters: 4,
    },
    BibleBook {
        slug: "titus",
        label: "Titus",
        chapters: 3,
    },
    BibleBook {
        slug: "philemon",
        label: "Philemon",
        chapters: 1,
    },
    BibleBook {
        slug: "hebrews",
        label: "Hebrews",
        chapters: 13,
    },
    BibleBook {
        slug: "james",
        label: "James",
        chapters: 5,
    },
    BibleBook {
        slug: "1peter",
        label: "1 Peter",
        chapters: 5,
    },
    BibleBook {
        slug: "2peter",
        label: "2 Peter",
        chapters: 3,
    },
    BibleBook {
        slug: "1john",
        label: "1 John",
        chapters: 5,
    },
    BibleBook {
        slug: "2john",
        label: "2 John",
        chapters: 1,
    },
    BibleBook {
        slug: "3john",
        label: "3 John",
        chapters: 1,
    },
    BibleBook {
        slug: "jude",
        label: "Jude",
        chapters: 1,
    },
    BibleBook {
        slug: "revelation",
        label: "Revelation",
        chapters: 22,
    },
];

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Import failed: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    let translation = TRANSLATIONS
        .iter()
        .find(|t| t.slug == cli.translation)
        .ok_or_else(|| format!("Unknown translation: {}", cli.translation))?;

    let db_url = cli
        .database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .ok_or("No database URL")?;

    let pool = PgPool::connect(&db_url).await?;
    let mut tx = pool.begin().await?;

    // System user owns all seed-imported sources/persons.
    let system_user_id: Uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    // 1a. Canonical "The Bible" source — the translation root that KJV and WEB
    //     fold under. SELECT-then-INSERT so importing both translations is
    //     order-independent; the unique constraint can't dedup because
    //     publication_year is NULL (NULLs aren't equal in btree).
    let canonical_bible_source_id: Uuid = match sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM sources
         WHERE title = 'The Bible' AND source_type = 'book' AND publication_year IS NULL",
    )
    .fetch_optional(&mut *tx)
    .await?
    {
        Some(id) => id,
        None => {
            sqlx::query_scalar(
                "INSERT INTO sources (source_type, title, protected, created_by)
             VALUES ('book', 'The Bible', true, $1)
             RETURNING id",
            )
            .bind(system_user_id)
            .fetch_one(&mut *tx)
            .await?
        }
    };

    // 1b. Translation source (KJV / WEB), linked to the canonical Bible source.
    let bible_source_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sources (source_type, title, publication_year, publisher, translation_of_id, protected, created_by)
         VALUES ('book', $1, $2, $3, $4, true, $5)
         RETURNING id",
    )
    .bind(translation.full_title)
    .bind(translation.publication_year)
    .bind(translation.publisher)
    .bind(canonical_bible_source_id)
    .bind(system_user_id)
    .fetch_one(&mut *tx)
    .await?;

    // 1c. Pre-load existing siblings (translations imported earlier) so we
    //     can guard structural parity (Q5 import guard / parity caveat).
    //     For each Bible-book this run will insert, we'll cross-check:
    //       - the depth=0 toc_node slug matches what an earlier translation used
    //       - per-chapter verse counts match
    //     If any of these drift, we refuse the import — silent versification
    //     drift would silently break Q7/Q9 (cross-translation projection).
    let canonical_books: Vec<(String, String, Uuid)> = sqlx::query_as(
        "SELECT tn.source_ref, tn.slug, tn.id
         FROM toc_nodes tn
         JOIN books b ON b.id = tn.book_id
         JOIN sources s ON s.id = b.source_id
         WHERE s.translation_of_id = $1 AND tn.depth = 0",
    )
    .bind(canonical_bible_source_id)
    .fetch_all(&mut *tx)
    .await?;
    // Indexed by source_ref → (slug, parent_node_id) for O(1) lookup.
    let canonical_book_index: std::collections::HashMap<String, (String, Uuid)> = canonical_books
        .into_iter()
        .map(|(source_ref, slug, id)| (source_ref, (slug, id)))
        .collect();

    let book_id: Uuid = sqlx::query_scalar(
        "INSERT INTO books (slug, source_id, language)
         VALUES ($1, $2, 'en')
         RETURNING id",
    )
    .bind(translation.book_slug)
    .bind(bible_source_id)
    .fetch_one(&mut *tx)
    .await?;

    eprintln!(
        "Translation {} -> source {} / book {}",
        translation.slug, bible_source_id, book_id
    );

    // 2. Verse reference system (one per book, applies across all Bible-books loaded)
    let verse_system_id: Uuid = sqlx::query_scalar(
        "INSERT INTO reference_systems (book_id, slug, label, ref_type)
         VALUES ($1, 'verse', 'Verse', 'inline')
         RETURNING id",
    )
    .bind(book_id)
    .fetch_one(&mut *tx)
    .await?;

    // 3. Per Bible-book: child source, parent toc_node, then per-chapter toc_node + content
    let mut sentence_number: i32 = 1;
    let mut marker_sort_order: i32 = 1;
    // Single monotonic counter so the linear reader scroll order is
    // [Genesis-parent, Genesis-1, ..., Genesis-50, John-parent, John-1, ...].
    // Earlier we used `bb.sort_order * 1000` for chapters which placed both
    // book-parents (sort_order 1, 2) before any chapters — the reader would
    // scroll up from Genesis 1 and hit "John" out of nowhere.
    let mut sort_order: i32 = 0;
    let next_order = |so: &mut i32| {
        *so += 1;
        *so
    };
    let mut totals = Totals::default();

    for bb in BIBLE_BOOKS {
        // Q5/Q9 import guard: enforce slug agreement with the canonical
        // translation. We check upfront before inserting anything for
        // this book so the error surfaces with a clean message.
        if let Some((canonical_slug, _)) = canonical_book_index.get(bb.slug) {
            if canonical_slug != bb.slug {
                return Err(format!(
                    "Slug drift on Bible-book '{}': existing translation \
                     uses depth=0 slug '{}' but this importer is using '{}'. \
                     Slugs must match across translations so cross-translation \
                     navigation (Q5/Q9) keeps working.",
                    bb.slug, canonical_slug, bb.slug
                )
                .into());
            }
        }

        // 3a. Per-Bible-book source so citations like "Gen 1:1" anchor on Genesis,
        //     not on the whole translation.
        let bb_source_id: Uuid = sqlx::query_scalar(
            "INSERT INTO sources (source_type, title, publication_year, publisher, parent_source_id, protected, created_by)
             VALUES ('chapter', $1, $2, $3, $4, true, $5)
             RETURNING id",
        )
        .bind(format!("{} ({})", bb.label, translation.full_title))
        .bind(translation.publication_year)
        .bind(translation.publisher)
        .bind(bible_source_id)
        .bind(system_user_id)
        .fetch_one(&mut *tx)
        .await?;

        // 3b. Top-level toc_node for the Bible-book (e.g. "Genesis"). Owns the
        //     bibliographic anchor via source_id.
        let bb_path = bb.slug.replace('-', "_");
        let bb_node_id: Uuid = sqlx::query_scalar(
            "INSERT INTO toc_nodes (book_id, parent_id, source_id, source_ref, slug, path, sort_order, depth, label)
             VALUES ($1, NULL, $2, $3, $4, $5::ltree, $6, 0, $7)
             RETURNING id",
        )
        .bind(book_id)
        .bind(bb_source_id)
        .bind(bb.slug)
        .bind(bb.slug)
        .bind(&bb_path)
        .bind(next_order(&mut sort_order))
        .bind(bb.label)
        .fetch_one(&mut *tx)
        .await?;
        totals.nodes += 1;

        // 3c. Heading block on the parent so the reader has something to render
        //     (otherwise the node shows as an empty bordered divider).
        let bb_heading_block_id: Uuid = sqlx::query_scalar(
            "INSERT INTO content_blocks (book_id, node_id, position, block_type, text, html)
             VALUES ($1, $2, 0, 'heading', $3, $3)
             RETURNING id",
        )
        .bind(book_id)
        .bind(bb_node_id)
        .bind(bb.label)
        .fetch_one(&mut *tx)
        .await?;
        totals.blocks += 1;

        // Heading block needs at least one sentence so margin/anchor logic
        // still has a target. Heading sentences carry no sentence_number
        // (those count body text only).
        sqlx::query(
            "INSERT INTO sentences (book_id, node_id, block_id, position, text, html)
             VALUES ($1, $2, $3, 0, $4, $4)",
        )
        .bind(book_id)
        .bind(bb_node_id)
        .bind(bb_heading_block_id)
        .bind(bb.label)
        .execute(&mut *tx)
        .await?;
        totals.sentences += 1;

        for chapter_num in 1..=bb.chapters {
            let chapter_path = format!("{}.ch_{}", bb_path, chapter_num);
            let chapter_slug = format!("{}-{}", bb.slug, chapter_num);
            let chapter_label = format!("Chapter {}", chapter_num);
            let chapter_source_ref = format!("{}:{}", bb.slug, chapter_num);

            let chapter_node_id: Uuid = sqlx::query_scalar(
                "INSERT INTO toc_nodes (book_id, parent_id, source_ref, slug, path, sort_order, depth, label)
                 VALUES ($1, $2, $3, $4, $5::ltree, $6, 1, $7)
                 RETURNING id",
            )
            .bind(book_id)
            .bind(bb_node_id)
            .bind(&chapter_source_ref)
            .bind(&chapter_slug)
            .bind(&chapter_path)
            .bind(next_order(&mut sort_order))
            .bind(&chapter_label)
            .fetch_one(&mut *tx)
            .await?;
            totals.nodes += 1;

            // Load verses for this chapter
            let path: PathBuf = [
                &cli.assets_dir,
                translation.slug,
                bb.slug,
                &format!("{}.json", chapter_num),
            ]
            .iter()
            .collect();
            let raw = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            let chapter: Chapter = serde_json::from_str(&raw)?;

            // Verse-count parity guard: cross-translation features key off
            // matching verse identity. Count *distinct verse markers* in the
            // canonical translation's chapter — this is verse-level, not
            // sentence-level (a single verse can carry multiple sentences
            // post-segmentation, so counting sentences would be wrong).
            if let Some((_, canonical_parent_id)) = canonical_book_index.get(bb.slug) {
                let canonical_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(DISTINCT pm.ref_value)
                     FROM page_markers pm
                     JOIN sentences s ON s.id = pm.sentence_id
                     JOIN toc_nodes tn ON tn.id = s.node_id
                     JOIN reference_systems rs ON rs.id = pm.system_id
                     WHERE tn.parent_id = $1
                       AND tn.source_ref = $2
                       AND rs.slug = 'verse'",
                )
                .bind(canonical_parent_id)
                .bind(&chapter_source_ref)
                .fetch_one(&mut *tx)
                .await?;
                let new_count = chapter.verses.len() as i64;
                if canonical_count > 0 && canonical_count != new_count {
                    // Known drift cases (Romans 14 doxology, some Psalm
                    // titles, 3 John) shouldn't block the entire import.
                    // Cross-translation visual hints degrade gracefully:
                    // verses that don't exist in both translations simply
                    // won't show projection markers.
                    eprintln!(
                        "WARN: verse-count drift on '{}': canonical={}, this={} — \
                         cross-translation hints will not align on this chapter.",
                        chapter_source_ref, canonical_count, new_count
                    );
                    totals.parity_warnings += 1;
                }
            }

            // One paragraph block per chapter; verses become sentences within it.
            let block_id: Uuid = sqlx::query_scalar(
                "INSERT INTO content_blocks (book_id, node_id, position, block_type, paragraph_number, text, html)
                 VALUES ($1, $2, 0, 'paragraph', $3, '', '')
                 RETURNING id",
            )
            .bind(book_id)
            .bind(chapter_node_id)
            .bind(sentence_number) // paragraph_number — global, equals first sentence_number
            .fetch_one(&mut *tx)
            .await?;
            totals.blocks += 1;

            // sentence-position counter within this chapter's block;
            // increments once per grammatical sentence, regardless of which
            // verse the sentence belongs to.
            let mut block_position: i16 = 0;
            // Track the running paragraph text and per-sentence char offsets
            // so verse markers can pinpoint the start of their verse inside
            // the joined paragraph if a renderer ever uses block.text.
            for verse in &chapter.verses {
                if verse.chapter != chapter_num {
                    return Err(format!(
                        "Chapter mismatch in {}: expected {}, got {}",
                        path.display(),
                        chapter_num,
                        verse.chapter
                    )
                    .into());
                }
                let verse_text = clean_verse(&verse.text);
                // Verses can carry multiple grammatical sentences (e.g.
                // KJV Gen 5:1 is two). Splitting them at the importer keeps
                // the reader's selection unit aligned with what users
                // intuit as "a sentence" — and lets verse markers anchor
                // *each* sentence in the verse.
                let sentences = segment_sentences(&verse_text);
                for sentence_text in &sentences {
                    let html = html_escape(sentence_text);

                    let sentence_id: Uuid = sqlx::query_scalar(
                        "INSERT INTO sentences (book_id, node_id, block_id, position, sentence_number, text, html)
                         VALUES ($1, $2, $3, $4, $5, $6, $7)
                         RETURNING id",
                    )
                    .bind(book_id)
                    .bind(chapter_node_id)
                    .bind(block_id)
                    .bind(block_position)
                    .bind(sentence_number)
                    .bind(sentence_text)
                    .bind(&html)
                    .fetch_one(&mut *tx)
                    .await?;
                    totals.sentences += 1;
                    sentence_number += 1;
                    block_position += 1;

                    // Verse marker: every sentence in a verse carries the
                    // verse marker so verse identity is preserved at the
                    // sentence level (and so the inline marker continues
                    // to render at every sentence as before).
                    sqlx::query(
                        "INSERT INTO page_markers (system_id, sentence_id, ref_value, sort_order, char_offset)
                         VALUES ($1, $2, $3, $4, NULL)",
                    )
                    .bind(verse_system_id)
                    .bind(sentence_id)
                    .bind(format!("{}:{}", chapter_num, verse.verse))
                    .bind(marker_sort_order)
                    .execute(&mut *tx)
                    .await?;
                    totals.markers += 1;
                    marker_sort_order += 1;
                }
            }

            // Update the block's text/html with the concatenated paragraph after
            // we know all verses, so the reader has a fallback rendering.
            let para_text = chapter
                .verses
                .iter()
                .map(|v| clean_verse(&v.text))
                .collect::<Vec<_>>()
                .join(" ");
            let para_html = format!("<p>{}</p>", html_escape(&para_text));
            sqlx::query("UPDATE content_blocks SET text = $1, html = $2 WHERE id = $3")
                .bind(&para_text)
                .bind(&para_html)
                .bind(block_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;

    eprintln!();
    eprintln!("=== Import complete: {} ===", translation.full_title);
    eprintln!("  toc_nodes:      {}", totals.nodes);
    eprintln!("  content_blocks: {}", totals.blocks);
    eprintln!("  sentences:      {}", totals.sentences);
    eprintln!("  page_markers:   {}", totals.markers);
    if totals.parity_warnings > 0 {
        eprintln!(
            "  parity_warnings:{}  (chapters where verse count differs from \
             canonical translation; cross-translation hints will not align there)",
            totals.parity_warnings
        );
    }

    Ok(())
}

#[derive(Default)]
struct Totals {
    nodes: u32,
    blocks: u32,
    sentences: u32,
    markers: u32,
    parity_warnings: u32,
}

fn clean_verse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Split a verse into grammatical sentences.
///
/// Heuristic: cut after `.`, `!`, or `?` that is followed by whitespace
/// and an uppercase letter (or end-of-text). Punctuation stays with the
/// preceding sentence. `;` and `:` do NOT end sentences — biblical text
/// uses them as internal pauses inside a single thought.
///
/// Conservative on purpose. Edge cases like "Mr. Jones" don't appear in
/// KJV/WEB. If a verse has no sentence-ending punctuation at all (ends
/// in `,` `;` `:` or unterminated), the whole verse is returned as one
/// sentence — that matches the user's mental model of "this verse is
/// part of a longer thought."
fn segment_sentences(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut sentences = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if matches!(c, '.' | '!' | '?') {
            // Walk past any closing quotes/brackets that hug the punct.
            let mut end_of_punct = i + 1;
            while end_of_punct < chars.len()
                && matches!(chars[end_of_punct], '"' | '\'' | ')' | ']' | '”' | '’')
            {
                end_of_punct += 1;
            }
            // Walk past whitespace.
            let mut j = end_of_punct;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            // Sentence break only if next non-space char starts a new
            // sentence (uppercase letter) or we're at end-of-verse.
            let is_break = j == chars.len()
                || chars[j].is_uppercase()
                // Also break before opening quote+capital, e.g. ' "And...'
                || (matches!(chars[j], '"' | '\'' | '“' | '‘')
                    && j + 1 < chars.len()
                    && chars[j + 1].is_uppercase());
            if is_break {
                let chunk: String = chars[start..end_of_punct].iter().collect();
                let chunk = chunk.trim();
                if !chunk.is_empty() {
                    sentences.push(chunk.to_string());
                }
                start = j;
                i = j;
                continue;
            }
        }
        i += 1;
    }
    if start < chars.len() {
        let chunk: String = chars[start..].iter().collect();
        let chunk = chunk.trim();
        if !chunk.is_empty() {
            sentences.push(chunk.to_string());
        }
    }
    if sentences.is_empty() {
        sentences.push(text.trim().to_string());
    }
    sentences
}

#[cfg(test)]
mod tests {
    use super::segment_sentences;

    #[test]
    fn splits_kjv_gen_5_1() {
        let input = "This is the book of the generations of Adam. In the day that God created man, in the likeness of God made he him;";
        let out = segment_sentences(input);
        assert_eq!(
            out,
            vec![
                "This is the book of the generations of Adam.",
                "In the day that God created man, in the likeness of God made he him;",
            ]
        );
    }

    #[test]
    fn keeps_single_sentence_intact() {
        let input = "Male and female created he them; and blessed them, and called their name Adam, in the day when they were created.";
        assert_eq!(segment_sentences(input), vec![input]);
    }

    #[test]
    fn unterminated_verse_returns_one_sentence() {
        let input = "And Adam lived an hundred and thirty years, and begat a son in his own likeness, after his image; and called his name Seth:";
        assert_eq!(segment_sentences(input), vec![input]);
    }

    #[test]
    fn empty_verse_returns_empty_list_marker() {
        let out = segment_sentences("   ");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0], "");
    }

    #[test]
    fn handles_quoted_sentences() {
        let input = "God said, \"Let there be light.\" And there was light.";
        assert_eq!(
            segment_sentences(input),
            vec!["God said, \"Let there be light.\"", "And there was light.",]
        );
    }
}
