use serde::Serialize;
use utoipa::ToSchema;

use super::node::NodeDetail;

#[derive(Debug, Serialize, ToSchema)]
pub struct NodePage {
    pub nodes: Vec<NodeDetail>,
    pub has_more: bool,
    pub has_previous: bool,
}
