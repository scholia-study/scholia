use std::collections::{HashMap, HashSet};

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::library::{
    BookPill, LibraryGroup, LibraryResponse, LibraryStats, LibraryVersion, LibraryWork,
};

// ── Row types ───────────────────────────────────────────────

struct SourceRow {
    id: Uuid,
    title: String,
    publication_year: Option<i16>,
    publisher: Option<String>,
    translation_of_id: Option<Uuid>,
    parent_source_id: Option<Uuid>,
}

struct BookRow {
    id: Uuid,
    slug: String,
    language: String,
    source_id: Uuid,
}

struct TocAnchorRow {
    book_id: Uuid,
    node_slug: String,
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
    name: String,
    sort_name: String,
}

// ── Internal types ──────────────────────────────────────────

/// A "version" of a work — either a top-level books row or a nested
/// toc-node anchor inside some host book.
struct VersionInstance {
    book_slug: String,
    book_language: String,
    /// Set when the version is anchored at a nested toc node (Shape 3
    /// compilation child). None for top-level books.
    node_slug: Option<String>,
    /// The source row backing this version (used for is_original /
    /// translator lookup). May differ from the work's root source when
    /// the work has multiple translations.
    source_id: Uuid,
}

/// Identifies which group a work belongs to.
enum GroupKey {
    Author(Uuid),
    SelfNamed(Uuid),
}

// ── Public ──────────────────────────────────────────────────

pub async fn get_library(pool: &PgPool) -> Result<LibraryResponse, AppError> {
    // Every book-type source. title_display for display, parent for
    // Shape-3 compilation walks, translation_of for version grouping.
    let source_rows = sqlx::query_as!(
        SourceRow,
        r#"SELECT id,
                  COALESCE(title_display, title) AS "title!",
                  publication_year,
                  publisher,
                  translation_of_id,
                  parent_source_id
           FROM sources
           WHERE source_type = 'book'::source_type"#,
    )
    .fetch_all(pool)
    .await?;
    let sources: HashMap<Uuid, SourceRow> = source_rows.into_iter().map(|s| (s.id, s)).collect();

    let book_rows = sqlx::query_as!(
        BookRow,
        r#"SELECT id, slug, language, source_id FROM books"#,
    )
    .fetch_all(pool)
    .await?;
    let books_by_id: HashMap<Uuid, BookRow> = book_rows
        .iter()
        .map(|b| {
            (
                b.id,
                BookRow {
                    id: b.id,
                    slug: b.slug.clone(),
                    language: b.language.clone(),
                    source_id: b.source_id,
                },
            )
        })
        .collect();

    // Shape-3 nested anchors: toc nodes that carry their own source_id.
    let anchor_rows = sqlx::query_as!(
        TocAnchorRow,
        r#"SELECT book_id, slug AS "node_slug!", source_id AS "source_id!"
           FROM toc_nodes
           WHERE source_id IS NOT NULL"#,
    )
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

    // Build VersionInstance lists, keyed by translation-root source id.
    // Top-level versions come from books rows; nested versions come
    // from toc-node anchors. The translation-root grouping merges
    // multiple translations of the same work into one LibraryWork.
    // Splits into top-level vs nested up front based on whether the
    // root source has a parent.
    let mut top_versions_by_work: HashMap<Uuid, Vec<VersionInstance>> = HashMap::new();
    let mut nested_versions_by_work: HashMap<Uuid, Vec<VersionInstance>> = HashMap::new();

    let push_version = |map_top: &mut HashMap<Uuid, Vec<VersionInstance>>,
                        map_nested: &mut HashMap<Uuid, Vec<VersionInstance>>,
                        root: Uuid,
                        v: VersionInstance| {
        let target = match sources.get(&root) {
            Some(s) if s.parent_source_id.is_some() => map_nested,
            Some(_) => map_top,
            None => return, // dangling translation_of_id; skip
        };
        target.entry(root).or_default().push(v);
    };

    for book in &book_rows {
        let root = root_of(book.source_id, &sources);
        push_version(
            &mut top_versions_by_work,
            &mut nested_versions_by_work,
            root,
            VersionInstance {
                book_slug: book.slug.clone(),
                book_language: book.language.clone(),
                node_slug: None,
                source_id: book.source_id,
            },
        );
    }

    for anchor in anchor_rows {
        let Some(host) = books_by_id.get(&anchor.book_id) else {
            continue;
        };
        let root = root_of(anchor.source_id, &sources);
        push_version(
            &mut top_versions_by_work,
            &mut nested_versions_by_work,
            root,
            VersionInstance {
                book_slug: host.slug.clone(),
                book_language: host.language.clone(),
                node_slug: Some(anchor.node_slug),
                source_id: anchor.source_id,
            },
        );
    }

    // Assemble groups. A group is keyed by either an author (person id)
    // or by the top-level source itself ("self").
    let mut author_groups: HashMap<Uuid, (PrimaryAuthor, Vec<LibraryWork>)> = HashMap::new();
    let mut self_groups: HashMap<Uuid, Vec<LibraryWork>> = HashMap::new();

    // For each top-level source, what's its primary group? Computed
    // eagerly so the nested pass can route children correctly under the
    // compilation-primary rule.
    let mut top_level_primary: HashMap<Uuid, GroupKey> = HashMap::new();

    // Pass 1: top-level works.
    for (work_id, mut version_list) in top_versions_by_work {
        let Some(root_source) = sources.get(&work_id) else {
            continue;
        };

        let key = compute_primary_key(work_id, &authors_by_source, &editors_by_source);
        top_level_primary.insert(work_id, clone_key(&key));

        let work = build_work(
            work_id,
            root_source,
            &mut version_list,
            &sources,
            &authors_by_source,
            &editors_by_source,
            &translators_by_source,
        );

        match key {
            GroupKey::Author(person_id) => {
                let primary = primary_author(work_id, &authors_by_source, &editors_by_source)
                    .expect("Author key implies a primary person");
                author_groups
                    .entry(person_id)
                    .or_insert_with(|| (primary, Vec::new()))
                    .1
                    .push(work);
            }
            GroupKey::SelfNamed(_) => {
                // Default: the compilation/singleton heading IS the work —
                // children populate the list in pass 2; singletons stay
                // empty so only the heading link is rendered.
                //
                // Exception: when the work has multiple versions (e.g. The
                // Bible with KJV + WEB translations), the heading alone
                // can't expose them. Push the work itself so the version
                // pills become visible. The frontend hides the redundant
                // title when work.title == group.primary_label.
                let entry = self_groups.entry(work_id).or_default();
                if work.versions.len() > 1 {
                    entry.push(work);
                }
            }
        }
    }

    // Pass 2: nested works. Each goes under the *top-level ancestor's*
    // primary group (compilation-primary rule).
    for (work_id, mut version_list) in nested_versions_by_work {
        let Some(work_source) = sources.get(&work_id) else {
            continue;
        };

        let top_level = top_level_of(work_id, &sources);
        let Some(parent_key) = top_level_primary.get(&top_level) else {
            // Top-level not discovered (no books anchored to it) — skip.
            continue;
        };

        let work = build_work(
            work_id,
            work_source,
            &mut version_list,
            &sources,
            &authors_by_source,
            &editors_by_source,
            &translators_by_source,
        );

        match parent_key {
            GroupKey::Author(person_id) => {
                if let Some((_, list)) = author_groups.get_mut(person_id) {
                    list.push(work);
                }
            }
            GroupKey::SelfNamed(top_id) => {
                self_groups.entry(*top_id).or_default().push(work);
            }
        }
    }

    // Sort works within each group: publication_year asc (undated last),
    // title as tiebreaker.
    let work_sort =
        |a: &LibraryWork, b: &LibraryWork| match (a.publication_year, b.publication_year) {
            (Some(ay), Some(by)) => ay.cmp(&by).then_with(|| a.title.cmp(&b.title)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.title.cmp(&b.title),
        };
    for (_, works) in author_groups.values_mut() {
        works.sort_by(work_sort);
    }
    for works in self_groups.values_mut() {
        works.sort_by(work_sort);
    }

    // Build the flat group list (author groups + self groups merged).
    let mut groups: Vec<LibraryGroup> = Vec::new();

    for (person_id, (person, works)) in author_groups {
        groups.push(LibraryGroup {
            id: person_id.to_string(),
            primary_kind: "author".to_string(),
            primary_label: person.name,
            sort_name: person.sort_name,
            primary_slug: None,
            books: works,
            book_pills: Vec::new(),
        });
    }

    for (source_id, works) in self_groups {
        let Some(src) = sources.get(&source_id) else {
            continue;
        };
        // Heading link: any books row hosting this source directly.
        // (The Bible's books row has source_id = Bible source.)
        let primary_slug = book_rows
            .iter()
            .find(|b| b.source_id == source_id)
            .map(|b| format!("/books/{}", b.slug));

        groups.push(LibraryGroup {
            id: source_id.to_string(),
            primary_kind: "self".to_string(),
            primary_label: src.title.clone(),
            sort_name: src.title.clone(),
            primary_slug,
            books: works,
            book_pills: Vec::new(),
        });
    }

    // Populate book_pills for "Bible-shape" groups: a SelfNamed group
    // whose single work is available in 2+ translations and whose
    // representative book carries depth=0 toc-anchored child sources
    // (the compilation pattern). Pills are sourced from the first
    // version's book under the assumption — guarded at import time —
    // that all sibling translations agree on depth=0 node slugs.
    populate_book_pills(pool, &mut groups).await?;

    // Sort by sort_name (case-insensitive), then label as tiebreaker.
    groups.sort_by(|a, b| {
        a.sort_name
            .to_lowercase()
            .cmp(&b.sort_name.to_lowercase())
            .then_with(|| {
                a.primary_label
                    .to_lowercase()
                    .cmp(&b.primary_label.to_lowercase())
            })
    });

    // Stats.
    let works_count: i64 = groups.iter().map(|g| g.books.len() as i64).sum::<i64>()
        // Self-named singletons (heading IS the work, books empty)
        // still count as one work each.
        + groups
            .iter()
            .filter(|g| g.primary_kind == "self" && g.books.is_empty())
            .count() as i64;

    let authors_count: i64 = groups.iter().filter(|g| g.primary_kind == "author").count() as i64;

    let mut langs: HashSet<String> = HashSet::new();
    for g in &groups {
        for w in &g.books {
            for v in &w.versions {
                langs.insert(v.language.clone());
            }
        }
    }
    // Self-named singletons have no listed works; pick up languages from
    // the host books row directly.
    for g in &groups {
        if g.primary_kind == "self"
            && g.books.is_empty()
            && let Some(book) = book_rows.iter().find(|b| b.source_id.to_string() == g.id)
        {
            langs.insert(book.language.clone());
        }
    }
    let languages_count: i64 = langs.len() as i64;

    Ok(LibraryResponse {
        groups,
        stats: LibraryStats {
            works: works_count,
            authors: authors_count,
            languages: languages_count,
        },
    })
}

// ── Bible-shape book_pills ──────────────────────────────────

async fn populate_book_pills(pool: &PgPool, groups: &mut [LibraryGroup]) -> Result<(), AppError> {
    for group in groups.iter_mut() {
        if group.primary_kind != "self" {
            continue;
        }
        // Exactly one work, with multiple translations of it.
        let Some(work) = group.books.first() else {
            continue;
        };
        if work.versions.len() < 2 {
            continue;
        }
        // First version is the representative — versions are sorted in
        // build_work so [0] is the original (or first by language) which
        // is the most stable reference point.
        let Some(rep) = work.versions.first() else {
            continue;
        };
        let pills = sqlx::query!(
            r#"SELECT tn.slug, tn.label, tn.sort_order
               FROM toc_nodes tn
               JOIN books b ON b.id = tn.book_id
               WHERE b.slug = $1 AND tn.depth = 0 AND tn.source_id IS NOT NULL
               ORDER BY tn.sort_order"#,
            rep.book_slug,
        )
        .fetch_all(pool)
        .await?;
        if pills.is_empty() {
            continue;
        }
        group.book_pills = pills
            .into_iter()
            .map(|r| BookPill {
                node_slug: r.slug,
                label: r.label,
                sort_order: r.sort_order,
            })
            .collect();
    }
    Ok(())
}

// ── Helpers ─────────────────────────────────────────────────

fn compute_primary_key(
    work_id: Uuid,
    authors_by_source: &HashMap<Uuid, Vec<PersonRow>>,
    editors_by_source: &HashMap<Uuid, Vec<PersonRow>>,
) -> GroupKey {
    if let Some(list) = authors_by_source.get(&work_id)
        && let Some(first) = list.first()
    {
        return GroupKey::Author(first.person_id);
    }

    if let Some(list) = editors_by_source.get(&work_id)
        && let Some(first) = list.first()
    {
        return GroupKey::Author(first.person_id);
    }

    GroupKey::SelfNamed(work_id)
}

fn clone_key(key: &GroupKey) -> GroupKey {
    match key {
        GroupKey::Author(id) => GroupKey::Author(*id),
        GroupKey::SelfNamed(id) => GroupKey::SelfNamed(*id),
    }
}

fn primary_author(
    work_id: Uuid,
    authors_by_source: &HashMap<Uuid, Vec<PersonRow>>,
    editors_by_source: &HashMap<Uuid, Vec<PersonRow>>,
) -> Option<PrimaryAuthor> {
    let list = authors_by_source
        .get(&work_id)
        .filter(|l| !l.is_empty())
        .or_else(|| editors_by_source.get(&work_id).filter(|l| !l.is_empty()))?;
    let first = list.first()?;
    Some(PrimaryAuthor {
        name: first.name.clone(),
        sort_name: first
            .sort_name
            .clone()
            .unwrap_or_else(|| first.name.clone()),
    })
}

fn build_work(
    work_id: Uuid,
    root_source: &SourceRow,
    versions: &mut [VersionInstance],
    sources: &HashMap<Uuid, SourceRow>,
    authors_by_source: &HashMap<Uuid, Vec<PersonRow>>,
    editors_by_source: &HashMap<Uuid, Vec<PersonRow>>,
    translators_by_source: &HashMap<Uuid, Vec<PersonRow>>,
) -> LibraryWork {
    let authors_of_root = authors_by_source.get(&work_id);
    let editors_of_root = editors_by_source.get(&work_id);

    let (co_authors, editor_names): (Vec<String>, Option<Vec<String>>) =
        match (authors_of_root, editors_of_root) {
            (Some(list), _) if !list.is_empty() => {
                (list[1..].iter().map(|p| p.name.clone()).collect(), None)
            }
            (_, Some(list)) if !list.is_empty() => (
                Vec::new(),
                Some(list.iter().map(|p| p.name.clone()).collect()),
            ),
            _ => (Vec::new(), None),
        };

    // Sort versions: original (= the root source itself) first, then by
    // language and slug.
    versions.sort_by(|a, b| {
        let a_root = a.source_id == work_id;
        let b_root = b.source_id == work_id;
        match (a_root, b_root) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a
                .book_language
                .cmp(&b.book_language)
                .then_with(|| a.book_slug.cmp(&b.book_slug)),
        }
    });

    let library_versions: Vec<LibraryVersion> = versions
        .iter()
        .map(|v| {
            let translators = translators_by_source
                .get(&v.source_id)
                .map(|list| list.iter().map(|p| p.name.clone()).collect())
                .unwrap_or_default();
            let version_source = sources.get(&v.source_id);
            LibraryVersion {
                book_slug: v.book_slug.clone(),
                node_slug: v.node_slug.clone(),
                language: v.book_language.clone(),
                is_original: v.source_id == work_id,
                translator_names: translators,
                publisher: version_source.and_then(|s| s.publisher.clone()),
                publication_year: version_source.and_then(|s| s.publication_year),
            }
        })
        .collect();

    LibraryWork {
        work_id: work_id.to_string(),
        title: root_source.title.clone(),
        publication_year: root_source.publication_year,
        co_authors,
        editor_names,
        versions: library_versions,
    }
}

/// Walk `translation_of_id` to find the canonical (translation-root)
/// source. Used to merge multiple translations of one work into a
/// single LibraryWork with multiple versions.
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

/// Walk `parent_source_id` to find the topmost compilation parent (the
/// source whose `parent_source_id` is None). For top-level sources the
/// result equals the input.
fn top_level_of(start: Uuid, sources: &HashMap<Uuid, SourceRow>) -> Uuid {
    let mut current = start;
    let mut visited: HashSet<Uuid> = HashSet::new();
    while let Some(src) = sources.get(&current) {
        if !visited.insert(current) {
            break;
        }
        match src.parent_source_id {
            Some(parent) if sources.contains_key(&parent) => current = parent,
            _ => break,
        }
    }
    current
}
