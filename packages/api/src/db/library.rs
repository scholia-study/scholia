use std::collections::{HashMap, HashSet};

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::library::{
    LibraryAuthor, LibraryResponse, LibraryStats, LibraryVersion, LibraryWork,
};

struct SourceRow {
    id: Uuid,
    title: String,
    publication_year: Option<i16>,
    publisher: Option<String>,
    translation_of_id: Option<Uuid>,
}

struct BookRow {
    slug: String,
    language: String,
    source_id: Uuid,
}

struct PersonRow {
    source_id: Uuid,
    role: String,
    person_id: Uuid,
    name: String,
    sort_name: Option<String>,
}

struct PrimaryAuthor {
    id: Uuid,
    name: String,
    sort_name: String,
}

pub async fn get_library(pool: &PgPool) -> Result<LibraryResponse, AppError> {
    // Fetch every book-type source. This gives us the root metadata even for
    // phantom originals (sources with no matching books row).
    let source_rows = sqlx::query_as!(
        SourceRow,
        r#"SELECT id,
                  COALESCE(title_display, title) AS "title!",
                  publication_year,
                  publisher,
                  translation_of_id
           FROM sources
           WHERE source_type = 'book'::source_type"#,
    )
    .fetch_all(pool)
    .await?;
    let sources: HashMap<Uuid, SourceRow> = source_rows.into_iter().map(|s| (s.id, s)).collect();

    let book_rows = sqlx::query_as!(BookRow, r#"SELECT slug, language, source_id FROM books"#,)
        .fetch_all(pool)
        .await?;

    let source_ids: Vec<Uuid> = sources.keys().copied().collect();
    let sp_rows = sqlx::query_as!(
        PersonRow,
        r#"SELECT sp.source_id,
                  sp.role::TEXT AS "role!",
                  p.id AS person_id,
                  p.name,
                  p.sort_name
           FROM source_persons sp
           JOIN persons p ON p.id = sp.person_id
           WHERE sp.source_id = ANY($1)
           ORDER BY sp.position"#,
        &source_ids,
    )
    .fetch_all(pool)
    .await?;

    let mut authors_by_source: HashMap<Uuid, Vec<PersonRow>> = HashMap::new();
    let mut editors_by_source: HashMap<Uuid, Vec<PersonRow>> = HashMap::new();
    let mut translators_by_source: HashMap<Uuid, Vec<PersonRow>> = HashMap::new();
    for row in sp_rows {
        let bucket = match row.role.as_str() {
            "author" => &mut authors_by_source,
            "editor" => &mut editors_by_source,
            "translator" => &mut translators_by_source,
            _ => continue,
        };
        bucket.entry(row.source_id).or_default().push(row);
    }

    // Group books by their root source (walking translation_of_id).
    let mut versions_by_work: HashMap<Uuid, Vec<BookRow>> = HashMap::new();
    for book in book_rows {
        let root = root_of(book.source_id, &sources);
        versions_by_work.entry(root).or_default().push(book);
    }

    // Assemble works grouped under their primary author.
    let mut works_by_author: HashMap<Uuid, (PrimaryAuthor, Vec<LibraryWork>)> = HashMap::new();

    for (work_id, mut book_versions) in versions_by_work {
        let Some(root_source) = sources.get(&work_id) else {
            continue;
        };

        // Primary-author identity: strict role='author' on the root source,
        // fall back to role='editor' when the work has no author at all.
        let authors_of_root = authors_by_source.get(&work_id);
        let editors_of_root = editors_by_source.get(&work_id);
        let (primary_list, is_editor_fallback): (&Vec<PersonRow>, bool) = match authors_of_root {
            Some(list) if !list.is_empty() => (list, false),
            _ => match editors_of_root {
                Some(list) if !list.is_empty() => (list, true),
                _ => continue, // no author, no editor — drop silently
            },
        };

        let primary = &primary_list[0];
        let primary_author = PrimaryAuthor {
            id: primary.person_id,
            name: primary.name.clone(),
            sort_name: primary
                .sort_name
                .clone()
                .unwrap_or_else(|| primary.name.clone()),
        };

        let co_authors: Vec<String> = if is_editor_fallback {
            Vec::new()
        } else {
            primary_list[1..].iter().map(|p| p.name.clone()).collect()
        };

        let editor_names: Option<Vec<String>> = if is_editor_fallback {
            Some(primary_list.iter().map(|p| p.name.clone()).collect())
        } else {
            None
        };

        // Sort versions: original always first, then language-alphabetical.
        book_versions.sort_by(|a, b| {
            let a_root = a.source_id == work_id;
            let b_root = b.source_id == work_id;
            match (a_root, b_root) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a
                    .language
                    .cmp(&b.language)
                    .then_with(|| a.slug.cmp(&b.slug)),
            }
        });

        let versions: Vec<LibraryVersion> = book_versions
            .into_iter()
            .map(|b| {
                let is_original = b.source_id == work_id;
                let translators = translators_by_source
                    .get(&b.source_id)
                    .map(|v| v.iter().map(|p| p.name.clone()).collect())
                    .unwrap_or_default();
                let version_source = sources.get(&b.source_id);
                LibraryVersion {
                    book_slug: b.slug,
                    language: b.language,
                    is_original,
                    translator_names: translators,
                    publisher: version_source.and_then(|s| s.publisher.clone()),
                    publication_year: version_source.and_then(|s| s.publication_year),
                }
            })
            .collect();

        let work = LibraryWork {
            work_id: work_id.to_string(),
            title: root_source.title.clone(),
            publication_year: root_source.publication_year,
            co_authors,
            editor_names,
            versions,
        };

        works_by_author
            .entry(primary_author.id)
            .or_insert_with(|| (primary_author, Vec::new()))
            .1
            .push(work);
    }

    // Sort works within each author: publication_year asc, undated last,
    // title as tiebreaker.
    for (_, works) in works_by_author.values_mut() {
        works.sort_by(|a, b| match (a.publication_year, b.publication_year) {
            (Some(ay), Some(by)) => ay.cmp(&by).then_with(|| a.title.cmp(&b.title)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.title.cmp(&b.title),
        });
    }

    let mut authors: Vec<LibraryAuthor> = works_by_author
        .into_values()
        .map(|(person, works)| LibraryAuthor {
            id: person.id.to_string(),
            name: person.name,
            sort_name: person.sort_name,
            books: works,
        })
        .collect();
    authors.sort_by(|a, b| {
        a.sort_name
            .to_lowercase()
            .cmp(&b.sort_name.to_lowercase())
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    let works_count: i64 = authors.iter().map(|a| a.books.len() as i64).sum();
    let authors_count: i64 = authors.len() as i64;
    let mut langs: HashSet<String> = HashSet::new();
    for a in &authors {
        for w in &a.books {
            for v in &w.versions {
                langs.insert(v.language.clone());
            }
        }
    }
    let languages_count: i64 = langs.len() as i64;

    Ok(LibraryResponse {
        authors,
        stats: LibraryStats {
            works: works_count,
            authors: authors_count,
            languages: languages_count,
        },
    })
}

fn root_of(start: Uuid, sources: &HashMap<Uuid, SourceRow>) -> Uuid {
    let mut current = start;
    let mut visited: HashSet<Uuid> = HashSet::new();
    while let Some(src) = sources.get(&current) {
        if !visited.insert(current) {
            break;
        }
        match src.translation_of_id {
            Some(next) if sources.contains_key(&next) => current = next,
            _ => break,
        }
    }
    current
}
