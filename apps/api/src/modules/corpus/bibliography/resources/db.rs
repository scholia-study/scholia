use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::modules::corpus::bibliography::models::{
    ParentSourceResponse, ResourceResponse, ResourceReviewStatus, ResourceSubmissionListResponse,
    ResourceSubmissionResponse, ResourceSubmitter, SourcePersonResponse, SourceResponse,
};
use crate::modules::corpus::bibliography::sources::db::fetch_source_persons;
use crate::system::error::{AppError, SqlxResultExt};

struct ResourceRow {
    id: Uuid,
    resource_type: String,
    verbatim_kind: Option<String>,
    start_number: Option<i32>,
    end_number: Option<i32>,
    sentence_kind: String,
    quoted_text: Option<String>,
    editor_note: Option<String>,
    source_page_start: Option<i32>,
    source_page_end: Option<i32>,
    source_location_freeform: Option<String>,
    is_featured: bool,
    admin_notes: Option<String>,
    review_status: String,
    scope: String,
    created_at: time::OffsetDateTime,
    // Source fields (joined)
    src_id: Option<Uuid>,
    src_type: Option<String>,
    src_title: Option<String>,
    src_title_display: Option<String>,
    src_year: Option<i16>,
    src_original_year: Option<i16>,
    src_publisher: Option<String>,
    src_publication_place: Option<String>,
    src_isbn: Option<Vec<String>>,
    src_doi: Option<String>,
    src_edition: Option<String>,
    src_volume: Option<String>,
    src_journal_name: Option<String>,
    src_url: Option<String>,
    src_page_start: Option<i32>,
    src_page_end: Option<i32>,
    src_parent_id: Option<Uuid>,
    src_translation_of_id: Option<Uuid>,
    src_created_by: Option<Uuid>,
    src_protected: Option<bool>,
}

pub async fn list_resources(
    pool: &PgPool,
    book_id: Uuid,
    start: i32,
    end: i32,
    kind: &str,
    // The reader only sees approved resources, plus the caller's own pending
    // submissions (so a submitter gets immediate feedback that their suggestion
    // landed). `None` for anonymous callers → approved only. Rejected rows are
    // never surfaced here; they live in the editor queue.
    viewer_id: Option<Uuid>,
    // `admin_notes` is an editor-internal field. This endpoint is public, so
    // callers without ResourcesManage must never receive it. Editor callers
    // (create/update re-fetch) pass `true`.
    include_admin_notes: bool,
) -> Result<Vec<ResourceResponse>, AppError> {
    let rows = sqlx::query_as!(
        ResourceRow,
        r#"SELECT r.id,
                  r.resource_type::TEXT AS "resource_type!",
                  r.verbatim_kind::TEXT AS "verbatim_kind?",
                  ss.sentence_number AS "start_number?",
                  se.sentence_number AS "end_number?",
                  r.sentence_kind::TEXT AS "sentence_kind!",
                  r.quoted_text, r.editor_note,
                  r.source_page_start, r.source_page_end, r.source_location_freeform,
                  r.is_featured, r.admin_notes,
                  r.review_status::TEXT AS "review_status!",
                  r.scope::TEXT AS "scope!",
                  r.created_at,
                  s.id AS "src_id?",
                  s.source_type::TEXT AS "src_type?",
                  s.title AS "src_title?",
                  s.title_display AS "src_title_display?",
                  s.publication_year AS "src_year?",
                  s.original_year AS "src_original_year?",
                  s.publisher AS "src_publisher?",
                  s.publication_place AS "src_publication_place?",
                  s.isbn AS "src_isbn?",
                  s.doi AS "src_doi?",
                  s.edition AS "src_edition?",
                  s.volume AS "src_volume?",
                  s.journal_name AS "src_journal_name?",
                  s.url AS "src_url?",
                  s.page_start AS "src_page_start?",
                  s.page_end AS "src_page_end?",
                  s.parent_source_id AS "src_parent_id?",
                  s.translation_of_id AS "src_translation_of_id?",
                  s.created_by AS "src_created_by?",
                  s.protected AS "src_protected?"
           FROM resources r
           JOIN sentences ss ON ss.id = r.anchor_sentence_start_id
           LEFT JOIN sentences se ON se.id = r.anchor_sentence_end_id
           LEFT JOIN sources s ON s.id = r.source_id
           WHERE r.book_id = $1
             AND r.archived_at IS NULL
             AND r.sentence_kind = $2::sentence_kind
             AND ss.sentence_number <= $4
             AND COALESCE(se.sentence_number, ss.sentence_number) >= $3
             AND (r.review_status = 'approved'
                  OR (r.submitted_by = $5 AND r.review_status = 'pending'))
           ORDER BY r.is_featured DESC,
                    s.publication_year DESC NULLS LAST,
                    r.source_page_start ASC NULLS LAST"#,
        book_id,
        kind as _,
        start,
        end,
        viewer_id,
    )
    .fetch_all(pool)
    .await?;

    let mut resources = build_resource_responses(pool, rows, include_admin_notes).await?;

    // Sibling-edition resources projected through canonical passages
    // (ADR 0008). Approved only — pending stays visible solely to its
    // submitter on its origin edition.
    let projected = list_projected_resources(pool, book_id, start, end, kind).await?;
    if !projected.is_empty() {
        resources.extend(projected);
        sort_resources(&mut resources);
    }

    Ok(resources)
}

/// The SQL ordering of the same-book query, applied over the merged
/// native + projected list: featured first, then source year (newest
/// first, unknown last), then source page (unknown last).
fn sort_resources(resources: &mut [ResourceResponse]) {
    resources.sort_by(|a, b| {
        b.is_featured
            .cmp(&a.is_featured)
            .then_with(|| {
                let ya = a.source.as_ref().and_then(|s| s.publication_year);
                let yb = b.source.as_ref().and_then(|s| s.publication_year);
                match (ya, yb) {
                    (Some(x), Some(y)) => y.cmp(&x),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                }
            })
            .then_with(|| match (a.source_page_start, b.source_page_start) {
                (Some(x), Some(y)) => x.cmp(&y),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            })
    });
}

/// Map raw resource rows into their nested response shape, batch-fetching the
/// joined source persons and parent sources. Shared by the reader listing and
/// the editor submission queue so both render sources identically.
async fn build_resource_responses(
    pool: &PgPool,
    rows: Vec<ResourceRow>,
    include_admin_notes: bool,
) -> Result<Vec<ResourceResponse>, AppError> {
    // Collect unique source IDs and parent IDs for batch fetching
    let source_ids: Vec<Uuid> = rows.iter().filter_map(|r| r.src_id).collect();
    let parent_ids: Vec<Uuid> = rows
        .iter()
        .filter_map(|r| r.src_parent_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let persons_map = fetch_source_persons(pool, &source_ids).await?;

    // Batch fetch parent sources
    let mut parent_persons_map: HashMap<Uuid, Vec<SourcePersonResponse>> = HashMap::new();
    if !parent_ids.is_empty() {
        parent_persons_map = fetch_source_persons(pool, &parent_ids).await?;
    }

    let parent_rows = if !parent_ids.is_empty() {
        sqlx::query_as!(
            ParentRow,
            r#"SELECT id, source_type::TEXT AS "source_type!", title, publication_year, publisher
               FROM sources
               WHERE id = ANY($1)"#,
            &parent_ids,
        )
        .fetch_all(pool)
        .await?
    } else {
        vec![]
    };

    let parent_map: HashMap<Uuid, ParentSourceResponse> = parent_rows
        .into_iter()
        .map(|r| {
            let persons = parent_persons_map.get(&r.id).cloned().unwrap_or_default();
            (
                r.id,
                ParentSourceResponse {
                    id: r.id.to_string(),
                    source_type: r.source_type,
                    title: r.title,
                    publication_year: r.publication_year,
                    publisher: r.publisher,
                    persons,
                },
            )
        })
        .collect();

    // Build responses
    Ok(rows
        .into_iter()
        .map(|r| {
            let source = r.src_id.map(|sid| {
                let persons = persons_map.get(&sid).cloned().unwrap_or_default();
                let parent = r
                    .src_parent_id
                    .and_then(|pid| parent_map.get(&pid).cloned())
                    .map(Box::new);
                SourceResponse {
                    id: sid.to_string(),
                    source_type: r.src_type.unwrap_or_default(),
                    title: r.src_title.unwrap_or_default(),
                    title_display: r.src_title_display,
                    publication_year: r.src_year,
                    original_year: r.src_original_year,
                    publisher: r.src_publisher,
                    publication_place: r.src_publication_place,
                    isbn: r.src_isbn,
                    doi: r.src_doi,
                    edition: r.src_edition,
                    volume: r.src_volume,
                    journal_name: r.src_journal_name,
                    url: r.src_url,
                    page_start: r.src_page_start,
                    page_end: r.src_page_end,
                    translation_of_id: r.src_translation_of_id.map(|id| id.to_string()),
                    created_by: r
                        .src_created_by
                        .map(|id| id.to_string())
                        .unwrap_or_default(),
                    protected: r.src_protected.unwrap_or(false),
                    persons,
                    parent,
                }
            });

            ResourceResponse {
                id: r.id.to_string(),
                resource_type: r.resource_type,
                verbatim_kind: r.verbatim_kind,
                anchor_sentence_start_number: r.start_number.unwrap_or(0),
                anchor_sentence_end_number: r.end_number,
                sentence_kind: r.sentence_kind,
                quoted_text: r.quoted_text,
                editor_note: r.editor_note,
                source,
                source_page_start: r.source_page_start,
                source_page_end: r.source_page_end,
                source_location_freeform: r.source_location_freeform,
                is_featured: r.is_featured,
                admin_notes: if include_admin_notes {
                    r.admin_notes
                } else {
                    None
                },
                review_status: r.review_status,
                scope: r.scope,
                is_projected: false,
                origin_book_slug: None,
                origin_language: None,
                projected_sentence_start_number: None,
                projected_sentence_end_number: None,
                created_at: r
                    .created_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
            }
        })
        .collect())
}

/// A projected row: every `ResourceRow` column plus the origin edition and
/// the target-local placement range.
struct PeerResourceRow {
    id: Uuid,
    resource_type: String,
    verbatim_kind: Option<String>,
    start_number: Option<i32>,
    end_number: Option<i32>,
    sentence_kind: String,
    quoted_text: Option<String>,
    editor_note: Option<String>,
    source_page_start: Option<i32>,
    source_page_end: Option<i32>,
    source_location_freeform: Option<String>,
    is_featured: bool,
    admin_notes: Option<String>,
    review_status: String,
    scope: String,
    created_at: time::OffsetDateTime,
    src_id: Option<Uuid>,
    src_type: Option<String>,
    src_title: Option<String>,
    src_title_display: Option<String>,
    src_year: Option<i16>,
    src_original_year: Option<i16>,
    src_publisher: Option<String>,
    src_publication_place: Option<String>,
    src_isbn: Option<Vec<String>>,
    src_doi: Option<String>,
    src_edition: Option<String>,
    src_volume: Option<String>,
    src_journal_name: Option<String>,
    src_url: Option<String>,
    src_page_start: Option<i32>,
    src_page_end: Option<i32>,
    src_parent_id: Option<Uuid>,
    src_translation_of_id: Option<Uuid>,
    src_created_by: Option<Uuid>,
    src_protected: Option<bool>,
    origin_book_slug: String,
    origin_language: String,
    projected_start: Option<i32>,
    projected_end: Option<i32>,
}

struct PeerExtra {
    origin_book_slug: String,
    origin_language: String,
    projected_start: Option<i32>,
    projected_end: Option<i32>,
}

impl PeerResourceRow {
    fn split(self) -> (ResourceRow, PeerExtra) {
        let extra = PeerExtra {
            origin_book_slug: self.origin_book_slug,
            origin_language: self.origin_language,
            projected_start: self.projected_start,
            projected_end: self.projected_end,
        };
        let row = ResourceRow {
            id: self.id,
            resource_type: self.resource_type,
            verbatim_kind: self.verbatim_kind,
            start_number: self.start_number,
            end_number: self.end_number,
            sentence_kind: self.sentence_kind,
            quoted_text: self.quoted_text,
            editor_note: self.editor_note,
            source_page_start: self.source_page_start,
            source_page_end: self.source_page_end,
            source_location_freeform: self.source_location_freeform,
            is_featured: self.is_featured,
            admin_notes: self.admin_notes,
            review_status: self.review_status,
            scope: self.scope,
            created_at: self.created_at,
            src_id: self.src_id,
            src_type: self.src_type,
            src_title: self.src_title,
            src_title_display: self.src_title_display,
            src_year: self.src_year,
            src_original_year: self.src_original_year,
            src_publisher: self.src_publisher,
            src_publication_place: self.src_publication_place,
            src_isbn: self.src_isbn,
            src_doi: self.src_doi,
            src_edition: self.src_edition,
            src_volume: self.src_volume,
            src_journal_name: self.src_journal_name,
            src_url: self.src_url,
            src_page_start: self.src_page_start,
            src_page_end: self.src_page_end,
            src_parent_id: self.src_parent_id,
            src_translation_of_id: self.src_translation_of_id,
            src_created_by: self.src_created_by,
            src_protected: self.src_protected,
        };
        (row, extra)
    }
}

// Peer split above; the submission split below mirrors it.

/// Approved resources anchored on sibling editions of the same work whose
/// canonical-passage span overlaps the requested range (ADR 0008). The
/// target span resolves the requested sentences to canonical ordinals;
/// a peer anchor matches when its own ordinal span overlaps. Sentences
/// without a passage (standalone works, figure anchors, DARBY title
/// verses) never resolve, so such selections and anchors simply don't
/// project. `admin_notes` stays origin-side (never returned here).
async fn list_projected_resources(
    pool: &PgPool,
    book_id: Uuid,
    start: i32,
    end: i32,
    kind: &str,
) -> Result<Vec<ResourceResponse>, AppError> {
    let rows = sqlx::query_as!(
        PeerResourceRow,
        r#"WITH target_span AS (
               SELECT cp.work_root, MIN(cp.ordinal) AS lo, MAX(cp.ordinal) AS hi
               FROM sentences ts
               JOIN canonical_passages cp ON cp.id = ts.canonical_passage_id
               WHERE ts.book_id = $1
                 AND cp.sentence_kind = $2::sentence_kind
                 AND ts.sentence_number BETWEEN $3 AND $4
               GROUP BY cp.work_root
           )
           SELECT r.id,
                  r.resource_type::TEXT AS "resource_type!",
                  r.verbatim_kind::TEXT AS "verbatim_kind?",
                  ss.sentence_number AS "start_number?",
                  se.sentence_number AS "end_number?",
                  r.sentence_kind::TEXT AS "sentence_kind!",
                  r.quoted_text, r.editor_note,
                  r.source_page_start, r.source_page_end, r.source_location_freeform,
                  r.is_featured,
                  NULL::TEXT AS "admin_notes?",
                  r.review_status::TEXT AS "review_status!",
                  r.scope::TEXT AS "scope!",
                  r.created_at,
                  s.id AS "src_id?",
                  s.source_type::TEXT AS "src_type?",
                  s.title AS "src_title?",
                  s.title_display AS "src_title_display?",
                  s.publication_year AS "src_year?",
                  s.original_year AS "src_original_year?",
                  s.publisher AS "src_publisher?",
                  s.publication_place AS "src_publication_place?",
                  s.isbn AS "src_isbn?",
                  s.doi AS "src_doi?",
                  s.edition AS "src_edition?",
                  s.volume AS "src_volume?",
                  s.journal_name AS "src_journal_name?",
                  s.url AS "src_url?",
                  s.page_start AS "src_page_start?",
                  s.page_end AS "src_page_end?",
                  s.parent_source_id AS "src_parent_id?",
                  s.translation_of_id AS "src_translation_of_id?",
                  s.created_by AS "src_created_by?",
                  s.protected AS "src_protected?",
                  rb.slug AS origin_book_slug,
                  rb.language AS origin_language,
                  (SELECT MIN(t2.sentence_number)
                   FROM sentences t2
                   JOIN canonical_passages c2 ON c2.id = t2.canonical_passage_id
                   WHERE t2.book_id = $1
                     AND c2.work_root = tsp.work_root
                     AND c2.sentence_kind = $2::sentence_kind
                     AND c2.ordinal BETWEEN cps.ordinal
                                        AND COALESCE(cpe.ordinal, cps.ordinal)
                  ) AS "projected_start?",
                  (SELECT MAX(t2.sentence_number)
                   FROM sentences t2
                   JOIN canonical_passages c2 ON c2.id = t2.canonical_passage_id
                   WHERE t2.book_id = $1
                     AND c2.work_root = tsp.work_root
                     AND c2.sentence_kind = $2::sentence_kind
                     AND c2.ordinal BETWEEN cps.ordinal
                                        AND COALESCE(cpe.ordinal, cps.ordinal)
                  ) AS "projected_end?"
           FROM resources r
           JOIN books rb ON rb.id = r.book_id
           JOIN sentences ss ON ss.id = r.anchor_sentence_start_id
           JOIN canonical_passages cps ON cps.id = ss.canonical_passage_id
           LEFT JOIN sentences se ON se.id = r.anchor_sentence_end_id
           LEFT JOIN canonical_passages cpe ON cpe.id = se.canonical_passage_id
           JOIN target_span tsp ON tsp.work_root = cps.work_root
           LEFT JOIN sources s ON s.id = r.source_id
           WHERE r.book_id <> $1
             AND r.archived_at IS NULL
             AND r.review_status = 'approved'
             AND r.sentence_kind = $2::sentence_kind
             AND cps.ordinal <= tsp.hi
             AND COALESCE(cpe.ordinal, cps.ordinal) >= tsp.lo"#,
        book_id,
        kind as _,
        start,
        end,
    )
    .fetch_all(pool)
    .await?;

    let (resource_rows, extras): (Vec<ResourceRow>, Vec<PeerExtra>) =
        rows.into_iter().map(PeerResourceRow::split).unzip();

    let resources = build_resource_responses(pool, resource_rows, false).await?;

    Ok(resources
        .into_iter()
        .zip(extras)
        .map(|(mut resource, extra)| {
            resource.is_projected = true;
            resource.origin_book_slug = Some(extra.origin_book_slug);
            resource.origin_language = Some(extra.origin_language);
            resource.projected_sentence_start_number = extra.projected_start;
            resource.projected_sentence_end_number = extra.projected_end;
            resource
        })
        .collect())
}

struct SentenceLookup {
    id: Uuid,
    node_id: Uuid,
}

pub struct ResourceCreate<'a> {
    pub resource_type: &'a str,
    pub verbatim_kind: Option<&'a str>,
    pub sentence_start: i32,
    pub sentence_end: Option<i32>,
    pub sentence_kind: &'a str,
    pub source_id: Option<Uuid>,
    pub source_page_start: Option<i32>,
    pub source_page_end: Option<i32>,
    pub source_location_freeform: Option<&'a str>,
    pub quoted_text: Option<&'a str>,
    pub editor_note: Option<&'a str>,
    pub is_featured: bool,
    pub admin_notes: Option<&'a str>,
    /// "approved" when an editor creates directly, "pending" for a community
    /// submission awaiting review.
    pub review_status: &'a str,
    /// The community submitter, when this arrived through the review flow.
    pub submitted_by: Option<Uuid>,
    /// "work" | "language" | "edition" (ADR 0008).
    pub scope: &'a str,
}

pub async fn create_resource(
    pool: &PgPool,
    book_id: Uuid,
    entry: ResourceCreate<'_>,
) -> Result<Uuid, AppError> {
    // Validate range
    let actual_end = entry.sentence_end.unwrap_or(entry.sentence_start);
    if actual_end - entry.sentence_start + 1 > 15 {
        return Err(AppError::BadRequest(
            "Sentence range cannot exceed 15 sentences".to_string(),
        ));
    }

    // Resolve start sentence
    let is_body = entry.sentence_kind == "body";
    let start_sent = if is_body {
        sqlx::query_as!(
            SentenceLookup,
            r#"SELECT id, node_id FROM sentences
               WHERE book_id = $1 AND sentence_number = $2 AND block_id IS NOT NULL"#,
            book_id,
            entry.sentence_start,
        )
        .fetch_one(pool)
        .await
    } else {
        sqlx::query_as!(
            SentenceLookup,
            r#"SELECT id, node_id FROM sentences
               WHERE book_id = $1 AND sentence_number = $2 AND footnote_id IS NOT NULL"#,
            book_id,
            entry.sentence_start,
        )
        .fetch_one(pool)
        .await
    }
    .map_err(|_| {
        AppError::BadRequest(format!(
            "Sentence {} not found for kind '{}'",
            entry.sentence_start, entry.sentence_kind
        ))
    })?;

    // Resolve end sentence (if range)
    let end_sent_id = if let Some(end_num) = entry.sentence_end {
        let end_sent = if is_body {
            sqlx::query_as!(
                SentenceLookup,
                r#"SELECT id, node_id FROM sentences
                   WHERE book_id = $1 AND sentence_number = $2 AND block_id IS NOT NULL"#,
                book_id,
                end_num,
            )
            .fetch_one(pool)
            .await
        } else {
            sqlx::query_as!(
                SentenceLookup,
                r#"SELECT id, node_id FROM sentences
                   WHERE book_id = $1 AND sentence_number = $2 AND footnote_id IS NOT NULL"#,
                book_id,
                end_num,
            )
            .fetch_one(pool)
            .await
        }
        .map_err(|_| {
            AppError::BadRequest(format!(
                "Sentence {end_num} not found for kind '{}'",
                entry.sentence_kind
            ))
        })?;
        Some(end_sent.id)
    } else {
        None
    };

    let id = sqlx::query_scalar!(
        r#"INSERT INTO resources (
               book_id, resource_type, anchor_node_id,
               anchor_sentence_start_id, anchor_sentence_end_id,
               sentence_kind, source_id,
               source_page_start, source_page_end, source_location_freeform,
               verbatim_kind, quoted_text, editor_note, is_featured, admin_notes,
               review_status, submitted_by, scope
           ) VALUES (
               $1, $2::resource_type, $3, $4, $5,
               $6::sentence_kind, $7, $8, $9, $10,
               $11::verbatim_kind, $12, $13, $14, $15,
               $16::resource_review_status, $17, $18::resource_scope
           )
           RETURNING id"#,
        book_id,
        entry.resource_type as _,
        start_sent.node_id,
        start_sent.id,
        end_sent_id,
        entry.sentence_kind as _,
        entry.source_id,
        entry.source_page_start,
        entry.source_page_end,
        entry.source_location_freeform,
        entry.verbatim_kind as _,
        entry.quoted_text,
        entry.editor_note,
        entry.is_featured,
        entry.admin_notes,
        entry.review_status as _,
        entry.submitted_by,
        entry.scope as _,
    )
    .fetch_one(pool)
    .await?;

    Ok(id)
}

pub struct ResourceUpdate<'a> {
    pub resource_type: Option<&'a str>,
    pub verbatim_kind: Option<Option<&'a str>>,
    pub sentence_start: Option<i32>,
    pub sentence_end: Option<i32>,
    pub sentence_kind: Option<&'a str>,
    pub source_id: Option<Option<Uuid>>,
    pub source_page_start: Option<Option<i32>>,
    pub source_page_end: Option<Option<i32>>,
    pub source_location_freeform: Option<Option<&'a str>>,
    pub quoted_text: Option<Option<&'a str>>,
    pub editor_note: Option<Option<&'a str>>,
    pub is_featured: Option<bool>,
    pub admin_notes: Option<Option<&'a str>>,
    pub scope: Option<&'a str>,
}

pub async fn update_resource(
    pool: &PgPool,
    resource_id: Uuid,
    book_id: Uuid,
    patch: ResourceUpdate<'_>,
) -> Result<(), AppError> {
    // sentence_kind only makes sense alongside a re-resolved anchor: it
    // decides whether the anchor is a body or footnote sentence, so changing
    // it without a sentence_start would leave the stored anchor inconsistent
    // (and silently no-op, since kind is only written in the range branch).
    if patch.sentence_kind.is_some() && patch.sentence_start.is_none() {
        return Err(AppError::BadRequest(
            "sentence_kind can only be changed together with sentence_start".into(),
        ));
    }

    // Scope the update to the book in the URL. The UPDATEs below match on id
    // only, so without this a resource could be re-anchored to another book's
    // sentences (leaving book_id stale and the resource invisible in its own
    // book), and an id from a different book — or a nonexistent one — would
    // silently succeed.
    let belongs = sqlx::query_scalar!(
        r#"SELECT EXISTS(
               SELECT 1 FROM resources WHERE id = $1 AND book_id = $2
           ) AS "e!""#,
        resource_id,
        book_id,
    )
    .fetch_one(pool)
    .await?;
    if !belongs {
        return Err(AppError::NotFound("Resource not found".into()));
    }

    // If sentence range is being updated, resolve IDs
    if let Some(start) = patch.sentence_start {
        let end = patch.sentence_end.unwrap_or(start);
        if end - start + 1 > 15 {
            return Err(AppError::BadRequest(
                "Sentence range cannot exceed 15 sentences".to_string(),
            ));
        }

        let kind = patch.sentence_kind.unwrap_or("body");
        let is_body = kind == "body";

        let start_sent = if is_body {
            sqlx::query_as!(
                SentenceLookup,
                r#"SELECT id, node_id FROM sentences
                   WHERE book_id = $1 AND sentence_number = $2 AND block_id IS NOT NULL"#,
                book_id,
                start,
            )
            .fetch_one(pool)
            .await
        } else {
            sqlx::query_as!(
                SentenceLookup,
                r#"SELECT id, node_id FROM sentences
                   WHERE book_id = $1 AND sentence_number = $2 AND footnote_id IS NOT NULL"#,
                book_id,
                start,
            )
            .fetch_one(pool)
            .await
        }
        .on_missing(|| AppError::BadRequest(format!("Sentence {start} not found")))?;

        let end_sent_id = if patch.sentence_end.is_some() {
            let end_sent = if is_body {
                sqlx::query_as!(
                    SentenceLookup,
                    r#"SELECT id, node_id FROM sentences
                       WHERE book_id = $1 AND sentence_number = $2 AND block_id IS NOT NULL"#,
                    book_id,
                    end,
                )
                .fetch_one(pool)
                .await
            } else {
                sqlx::query_as!(
                    SentenceLookup,
                    r#"SELECT id, node_id FROM sentences
                       WHERE book_id = $1 AND sentence_number = $2 AND footnote_id IS NOT NULL"#,
                    book_id,
                    end,
                )
                .fetch_one(pool)
                .await
            }
            .on_missing(|| AppError::BadRequest(format!("Sentence {end} not found")))?;
            Some(end_sent.id)
        } else {
            None
        };

        sqlx::query!(
            r#"UPDATE resources
               SET anchor_node_id = $2,
                   anchor_sentence_start_id = $3,
                   anchor_sentence_end_id = $4,
                   sentence_kind = COALESCE($5::sentence_kind, sentence_kind),
                   updated_at = now()
               WHERE id = $1"#,
            resource_id,
            start_sent.node_id,
            start_sent.id,
            end_sent_id,
            patch.sentence_kind as _,
        )
        .execute(pool)
        .await?;
    }

    // Update the remaining fields, touching only those present in the patch.
    // Nullable columns carry an inner Option, so an explicit null binds NULL
    // and clears the column while an absent field is left unchanged.
    let mut qb = sqlx::QueryBuilder::new("UPDATE resources SET updated_at = now()");
    if let Some(v) = patch.resource_type {
        qb.push(", resource_type = ")
            .push_bind(v)
            .push("::resource_type");
    }
    if let Some(v) = patch.verbatim_kind {
        qb.push(", verbatim_kind = ")
            .push_bind(v)
            .push("::verbatim_kind");
    }
    if let Some(v) = patch.source_id {
        qb.push(", source_id = ").push_bind(v);
    }
    if let Some(v) = patch.source_page_start {
        qb.push(", source_page_start = ").push_bind(v);
    }
    if let Some(v) = patch.source_page_end {
        qb.push(", source_page_end = ").push_bind(v);
    }
    if let Some(v) = patch.source_location_freeform {
        qb.push(", source_location_freeform = ").push_bind(v);
    }
    if let Some(v) = patch.quoted_text {
        qb.push(", quoted_text = ").push_bind(v);
    }
    if let Some(v) = patch.editor_note {
        qb.push(", editor_note = ").push_bind(v);
    }
    if let Some(v) = patch.is_featured {
        qb.push(", is_featured = ").push_bind(v);
    }
    if let Some(v) = patch.admin_notes {
        qb.push(", admin_notes = ").push_bind(v);
    }
    if let Some(v) = patch.scope {
        qb.push(", scope = ").push_bind(v).push("::resource_scope");
    }
    qb.push(" WHERE id = ").push_bind(resource_id);
    qb.build().execute(pool).await?;

    Ok(())
}

pub async fn soft_delete_resource(
    pool: &PgPool,
    resource_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let result = sqlx::query!(
        r#"UPDATE resources
           SET archived_at = now(), archived_by = $2
           WHERE id = $1 AND archived_at IS NULL"#,
        resource_id,
        user_id,
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Resource not found".to_string()));
    }

    Ok(())
}

// No caller as of the modular refactor — previously masked as reachable
// `pub` db API. Retained (not deleted) to keep this a pure reorganization;
// dead-code cleanup is tracked separately.
#[allow(dead_code)]
pub async fn get_resource_book_id(pool: &PgPool, resource_id: Uuid) -> Result<Uuid, AppError> {
    let book_id = sqlx::query_scalar!(
        r#"SELECT book_id FROM resources WHERE id = $1"#,
        resource_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(book_id)
}

struct ParentRow {
    id: Uuid,
    source_type: String,
    title: String,
    publication_year: Option<i16>,
    publisher: Option<String>,
}

// ── Community submission queue ──────────────────────────────────────────

/// Count resources submitted (via the review flow) by `user_id` in the last
/// 24 hours. Feeds the rate-limit gate on non-editor submissions.
pub async fn count_recent_submissions_by_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<i64, AppError> {
    let count: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!"
           FROM resources
           WHERE submitted_by = $1
             AND created_at > now() - INTERVAL '24 hours'"#,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}

/// A submission-queue row: every column `ResourceRow` needs plus the book,
/// submitter, and review metadata the editor sees alongside the proposal.
struct SubmissionRow {
    // Resource core (mirrors ResourceRow).
    id: Uuid,
    resource_type: String,
    verbatim_kind: Option<String>,
    start_number: Option<i32>,
    end_number: Option<i32>,
    sentence_kind: String,
    quoted_text: Option<String>,
    editor_note: Option<String>,
    source_page_start: Option<i32>,
    source_page_end: Option<i32>,
    source_location_freeform: Option<String>,
    is_featured: bool,
    admin_notes: Option<String>,
    review_status: String,
    scope: String,
    created_at: time::OffsetDateTime,
    src_id: Option<Uuid>,
    src_type: Option<String>,
    src_title: Option<String>,
    src_title_display: Option<String>,
    src_year: Option<i16>,
    src_original_year: Option<i16>,
    src_publisher: Option<String>,
    src_publication_place: Option<String>,
    src_isbn: Option<Vec<String>>,
    src_doi: Option<String>,
    src_edition: Option<String>,
    src_volume: Option<String>,
    src_journal_name: Option<String>,
    src_url: Option<String>,
    src_page_start: Option<i32>,
    src_page_end: Option<i32>,
    src_parent_id: Option<Uuid>,
    src_translation_of_id: Option<Uuid>,
    src_created_by: Option<Uuid>,
    src_protected: Option<bool>,
    // Submission context.
    book_slug: String,
    book_title: String,
    submitter_id: Option<Uuid>,
    submitter_display_name: Option<String>,
    review_note: Option<String>,
    reviewed_at: Option<time::OffsetDateTime>,
}

struct SubmissionExtra {
    book_slug: String,
    book_title: String,
    submitter: Option<ResourceSubmitter>,
    review_note: Option<String>,
    reviewed_at: Option<String>,
}

impl SubmissionRow {
    fn split(self) -> (ResourceRow, SubmissionExtra) {
        let submitter = match (self.submitter_id, self.submitter_display_name) {
            (Some(id), Some(display_name)) => Some(ResourceSubmitter {
                id: id.to_string(),
                display_name,
            }),
            _ => None,
        };
        let reviewed_at = self.reviewed_at.map(|t| {
            t.format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default()
        });
        let extra = SubmissionExtra {
            book_slug: self.book_slug,
            book_title: self.book_title,
            submitter,
            review_note: self.review_note,
            reviewed_at,
        };
        let row = ResourceRow {
            id: self.id,
            resource_type: self.resource_type,
            verbatim_kind: self.verbatim_kind,
            start_number: self.start_number,
            end_number: self.end_number,
            sentence_kind: self.sentence_kind,
            quoted_text: self.quoted_text,
            editor_note: self.editor_note,
            source_page_start: self.source_page_start,
            source_page_end: self.source_page_end,
            source_location_freeform: self.source_location_freeform,
            is_featured: self.is_featured,
            admin_notes: self.admin_notes,
            review_status: self.review_status,
            scope: self.scope,
            created_at: self.created_at,
            src_id: self.src_id,
            src_type: self.src_type,
            src_title: self.src_title,
            src_title_display: self.src_title_display,
            src_year: self.src_year,
            src_original_year: self.src_original_year,
            src_publisher: self.src_publisher,
            src_publication_place: self.src_publication_place,
            src_isbn: self.src_isbn,
            src_doi: self.src_doi,
            src_edition: self.src_edition,
            src_volume: self.src_volume,
            src_journal_name: self.src_journal_name,
            src_url: self.src_url,
            src_page_start: self.src_page_start,
            src_page_end: self.src_page_end,
            src_parent_id: self.src_parent_id,
            src_translation_of_id: self.src_translation_of_id,
            src_created_by: self.src_created_by,
            src_protected: self.src_protected,
        };
        (row, extra)
    }
}

async fn wrap_submissions(
    pool: &PgPool,
    rows: Vec<SubmissionRow>,
) -> Result<Vec<ResourceSubmissionResponse>, AppError> {
    let (resource_rows, extras): (Vec<ResourceRow>, Vec<SubmissionExtra>) =
        rows.into_iter().map(SubmissionRow::split).unzip();

    // Editor context: include admin_notes so edit-before-approve round-trips.
    let resources = build_resource_responses(pool, resource_rows, true).await?;

    Ok(resources
        .into_iter()
        .zip(extras)
        .map(|(resource, extra)| ResourceSubmissionResponse {
            resource,
            book_slug: extra.book_slug,
            book_title: extra.book_title,
            submitter: extra.submitter,
            review_note: extra.review_note,
            reviewed_at: extra.reviewed_at,
        })
        .collect())
}

pub async fn list_submissions(
    pool: &PgPool,
    statuses: &[ResourceReviewStatus],
    page: u32,
    per_page: u32,
) -> Result<ResourceSubmissionListResponse, AppError> {
    let offset = ((page.saturating_sub(1)) as i64) * per_page as i64;
    let limit = per_page as i64;
    let status_strs: Vec<String> = statuses
        .iter()
        .map(|s| review_status_str(*s).to_string())
        .collect();

    // The `?`-typed aliases inside SUBMISSION_SELECT require the compile-time
    // macro path, so the shared column list is concatenated into each literal.
    let rows = sqlx::query_as!(
        SubmissionRow,
        r#"SELECT
    r.id,
    r.resource_type::TEXT AS "resource_type!",
    r.verbatim_kind::TEXT AS "verbatim_kind?",
    ss.sentence_number AS "start_number?",
    se.sentence_number AS "end_number?",
    r.sentence_kind::TEXT AS "sentence_kind!",
    r.quoted_text, r.editor_note,
    r.source_page_start, r.source_page_end, r.source_location_freeform,
    r.is_featured, r.admin_notes,
    r.review_status::TEXT AS "review_status!",
    r.scope::TEXT AS "scope!",
    r.created_at,
    s.id AS "src_id?",
    s.source_type::TEXT AS "src_type?",
    s.title AS "src_title?",
    s.title_display AS "src_title_display?",
    s.publication_year AS "src_year?",
    s.original_year AS "src_original_year?",
    s.publisher AS "src_publisher?",
    s.publication_place AS "src_publication_place?",
    s.isbn AS "src_isbn?",
    s.doi AS "src_doi?",
    s.edition AS "src_edition?",
    s.volume AS "src_volume?",
    s.journal_name AS "src_journal_name?",
    s.url AS "src_url?",
    s.page_start AS "src_page_start?",
    s.page_end AS "src_page_end?",
    s.parent_source_id AS "src_parent_id?",
    s.translation_of_id AS "src_translation_of_id?",
    s.created_by AS "src_created_by?",
    s.protected AS "src_protected?",
    b.slug AS book_slug,
    bs.title AS book_title,
    su.id AS "submitter_id?",
    su.display_name AS "submitter_display_name?",
    r.review_note,
    r.reviewed_at
           FROM resources r
           JOIN books b ON b.id = r.book_id
           JOIN sources bs ON bs.id = b.source_id
           JOIN sentences ss ON ss.id = r.anchor_sentence_start_id
           LEFT JOIN sentences se ON se.id = r.anchor_sentence_end_id
           LEFT JOIN sources s ON s.id = r.source_id
           LEFT JOIN users su ON su.id = r.submitted_by
           WHERE r.submitted_by IS NOT NULL
             AND r.archived_at IS NULL
             AND r.review_status::TEXT = ANY($1)
           ORDER BY r.created_at DESC
           LIMIT $2 OFFSET $3"#,
        &status_strs,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!"
           FROM resources r
           WHERE r.submitted_by IS NOT NULL
             AND r.archived_at IS NULL
             AND r.review_status::TEXT = ANY($1)"#,
        &status_strs,
    )
    .fetch_one(pool)
    .await?;

    Ok(ResourceSubmissionListResponse {
        submissions: wrap_submissions(pool, rows).await?,
        total,
        page,
        per_page,
    })
}

pub async fn get_submission(
    pool: &PgPool,
    id: Uuid,
) -> Result<ResourceSubmissionResponse, AppError> {
    let rows = sqlx::query_as!(
        SubmissionRow,
        r#"SELECT
    r.id,
    r.resource_type::TEXT AS "resource_type!",
    r.verbatim_kind::TEXT AS "verbatim_kind?",
    ss.sentence_number AS "start_number?",
    se.sentence_number AS "end_number?",
    r.sentence_kind::TEXT AS "sentence_kind!",
    r.quoted_text, r.editor_note,
    r.source_page_start, r.source_page_end, r.source_location_freeform,
    r.is_featured, r.admin_notes,
    r.review_status::TEXT AS "review_status!",
    r.scope::TEXT AS "scope!",
    r.created_at,
    s.id AS "src_id?",
    s.source_type::TEXT AS "src_type?",
    s.title AS "src_title?",
    s.title_display AS "src_title_display?",
    s.publication_year AS "src_year?",
    s.original_year AS "src_original_year?",
    s.publisher AS "src_publisher?",
    s.publication_place AS "src_publication_place?",
    s.isbn AS "src_isbn?",
    s.doi AS "src_doi?",
    s.edition AS "src_edition?",
    s.volume AS "src_volume?",
    s.journal_name AS "src_journal_name?",
    s.url AS "src_url?",
    s.page_start AS "src_page_start?",
    s.page_end AS "src_page_end?",
    s.parent_source_id AS "src_parent_id?",
    s.translation_of_id AS "src_translation_of_id?",
    s.created_by AS "src_created_by?",
    s.protected AS "src_protected?",
    b.slug AS book_slug,
    bs.title AS book_title,
    su.id AS "submitter_id?",
    su.display_name AS "submitter_display_name?",
    r.review_note,
    r.reviewed_at
           FROM resources r
           JOIN books b ON b.id = r.book_id
           JOIN sources bs ON bs.id = b.source_id
           JOIN sentences ss ON ss.id = r.anchor_sentence_start_id
           LEFT JOIN sentences se ON se.id = r.anchor_sentence_end_id
           LEFT JOIN sources s ON s.id = r.source_id
           LEFT JOIN users su ON su.id = r.submitted_by
           WHERE r.id = $1 AND r.submitted_by IS NOT NULL"#,
        id,
    )
    .fetch_all(pool)
    .await?;

    wrap_submissions(pool, rows)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound("Submission not found".into()))
}

pub async fn review_submission(
    pool: &PgPool,
    id: Uuid,
    reviewer_id: Uuid,
    status: ResourceReviewStatus,
    review_note: Option<&str>,
) -> Result<ResourceSubmissionResponse, AppError> {
    let result = sqlx::query!(
        r#"UPDATE resources
           SET review_status = $2::resource_review_status,
               reviewed_by   = $3,
               reviewed_at   = now(),
               review_note   = $4,
               updated_at    = now()
           WHERE id = $1 AND submitted_by IS NOT NULL"#,
        id,
        review_status_str(status) as _,
        reviewer_id,
        review_note,
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Submission not found".into()));
    }

    get_submission(pool, id).await
}

fn review_status_str(s: ResourceReviewStatus) -> &'static str {
    match s {
        ResourceReviewStatus::Pending => "pending",
        ResourceReviewStatus::Approved => "approved",
        ResourceReviewStatus::Rejected => "rejected",
    }
}
