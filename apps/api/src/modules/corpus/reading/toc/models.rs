use serde::Serialize;
use utoipa::openapi::schema::{ArrayBuilder, ObjectBuilder, SchemaType, Type};
use utoipa::openapi::{RefOr, Schema};

#[derive(Debug, Serialize)]
pub struct TocNodeResponse {
    pub id: String,
    pub source_ref: String,
    pub slug: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_html: Option<String>,
    pub depth: i16,
    pub sort_order: i32,
    pub has_content: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_node_id: Option<String>,
    /// Bibliographic anchor: when set, this node IS the toc entry for a
    /// discrete child work hosted inside the book (e.g. Genesis under a
    /// Bible). Used by the frontend to render compilation children as
    /// chapter-pickable pills.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    pub children: Vec<TocNodeResponse>,
}

impl utoipa::__dev::ComposeSchema for TocNodeResponse {
    fn compose(_: Vec<RefOr<Schema>>) -> RefOr<Schema> {
        RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/TocNodeResponse",
        ))
    }
}

impl utoipa::ToSchema for TocNodeResponse {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("TocNodeResponse")
    }

    fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
        let obj = ObjectBuilder::new()
            .property(
                "id",
                ObjectBuilder::new().schema_type(SchemaType::new(Type::String)),
            )
            .required("id")
            .property(
                "source_ref",
                ObjectBuilder::new().schema_type(SchemaType::new(Type::String)),
            )
            .required("source_ref")
            .property(
                "slug",
                ObjectBuilder::new().schema_type(SchemaType::new(Type::String)),
            )
            .required("slug")
            .property(
                "label",
                ObjectBuilder::new().schema_type(SchemaType::new(Type::String)),
            )
            .required("label")
            .property(
                "label_html",
                ObjectBuilder::new().schema_type(SchemaType::new(Type::String)),
            )
            .property(
                "depth",
                ObjectBuilder::new().schema_type(SchemaType::new(Type::Integer)),
            )
            .required("depth")
            .property(
                "sort_order",
                ObjectBuilder::new().schema_type(SchemaType::new(Type::Integer)),
            )
            .required("sort_order")
            .property(
                "has_content",
                ObjectBuilder::new().schema_type(SchemaType::new(Type::Boolean)),
            )
            .required("has_content")
            .property(
                "source_node_id",
                ObjectBuilder::new().schema_type(SchemaType::new(Type::String)),
            )
            .property(
                "source_id",
                ObjectBuilder::new().schema_type(SchemaType::new(Type::String)),
            )
            .property(
                "children",
                ArrayBuilder::new().items(RefOr::Ref(utoipa::openapi::Ref::new(
                    "#/components/schemas/TocNodeResponse",
                ))),
            )
            .required("children")
            .build();

        schemas.push(("TocNodeResponse".to_string(), RefOr::T(Schema::Object(obj))));
    }
}
