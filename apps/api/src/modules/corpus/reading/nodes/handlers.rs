use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::modules::corpus::reading::nodes::models::{NodeDetail, NodeMetaResponse};
use crate::system::error::AppError;
use crate::system::state::AppState;

#[derive(Deserialize, IntoParams)]
pub struct NodeParams {
    /// include original_text/original_html fields
    #[serde(default)]
    original: Option<bool>,
}

/// Get node content (blocks + sentences)
#[utoipa::path(
    get,
    path = "/api/books/{slug}/nodes/{node_slug}",
    params(
        ("slug" = String, Path, description = "Book slug"),
        ("node_slug" = String, Path, description = "Node slug"),
        NodeParams,
    ),
    responses(
        (status = 200, description = "Node with content blocks and sentences", body = NodeDetail),
        (status = 404, description = "Node not found")
    ),
    tag = "nodes"
)]
pub async fn get_node(
    State(state): State<AppState>,
    Path((slug, node_slug)): Path<(String, String)>,
    Query(params): Query<NodeParams>,
) -> Result<Json<NodeDetail>, AppError> {
    let include_original = params.original.unwrap_or(false);
    let node = crate::modules::corpus::reading::nodes::db::get_node_content(
        &state.pool,
        &slug,
        &node_slug,
        include_original,
    )
    .await?;
    Ok(Json(node))
}

/// Excerpt length served to the web app. The frontend clamps meta
/// descriptions to ~160 chars; the headroom covers OG previews.
const EXCERPT_MAX_CHARS: usize = 300;

/// Get node metadata (SEO excerpt)
#[utoipa::path(
    get,
    path = "/api/books/{slug}/nodes/{node_slug}/meta",
    params(
        ("slug" = String, Path, description = "Book slug"),
        ("node_slug" = String, Path, description = "Node slug"),
    ),
    responses(
        (status = 200, description = "Node metadata", body = NodeMetaResponse),
        (status = 404, description = "Node not found")
    ),
    tag = "nodes"
)]
pub async fn get_node_meta(
    State(state): State<AppState>,
    Path((slug, node_slug)): Path<(String, String)>,
) -> Result<Json<NodeMetaResponse>, AppError> {
    let texts = crate::modules::corpus::reading::nodes::db::get_node_opening_text(
        &state.pool,
        &slug,
        &node_slug,
    )
    .await?;
    Ok(Json(NodeMetaResponse {
        excerpt: texts.map(|t| build_excerpt(&t, EXCERPT_MAX_CHARS)),
    }))
}

/// Join opening block texts, collapse whitespace, and truncate at a
/// word boundary with an ellipsis.
fn build_excerpt(texts: &[String], max_chars: usize) -> String {
    let joined = texts.join(" ");
    let mut collapsed = String::with_capacity(joined.len());
    let mut last_was_space = true;
    for ch in joined.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                collapsed.push(' ');
                last_was_space = true;
            }
        } else {
            collapsed.push(ch);
            last_was_space = false;
        }
    }
    let collapsed = collapsed.trim_end();

    if collapsed.chars().count() <= max_chars {
        return collapsed.to_string();
    }
    let cut: String = collapsed.chars().take(max_chars).collect();
    let trimmed = match cut.rfind(' ') {
        // Keep the word boundary unless it would gut the excerpt.
        Some(idx) if idx > max_chars / 2 => &cut[..idx],
        _ => cut.as_str(),
    };
    format!("{}…", trimmed.trim_end_matches([',', ';', ':', ' ']))
}

#[cfg(test)]
mod tests {
    use super::build_excerpt;

    #[test]
    fn short_text_passes_through() {
        let texts = vec!["In the beginning".to_string()];
        assert_eq!(build_excerpt(&texts, 300), "In the beginning");
    }

    #[test]
    fn collapses_whitespace_across_blocks() {
        let texts = vec!["First  block\n\ntext".to_string(), "second".to_string()];
        assert_eq!(build_excerpt(&texts, 300), "First block text second");
    }

    #[test]
    fn truncates_at_word_boundary_with_ellipsis() {
        let texts = vec!["alpha beta gamma delta".to_string()];
        let out = build_excerpt(&texts, 16);
        assert_eq!(out, "alpha beta…");
        assert!(out.chars().count() <= 17);
    }

    #[test]
    fn multibyte_safe_truncation() {
        let texts = vec!["ſelbſt größer — Verſtand und Vernunft überall".to_string()];
        let out = build_excerpt(&texts, 20);
        assert!(out.ends_with('…'));
        assert!(out.chars().count() <= 21);
    }
}
