use serde::Serialize;
use utoipa::openapi::schema::{ArrayBuilder, ObjectBuilder, SchemaType, Type};
use utoipa::openapi::{RefOr, Schema};

#[derive(Debug, Serialize)]
pub struct TocNodeResponse {
    pub id: String,
    pub ncx_id: String,
    pub label: String,
    pub depth: i16,
    pub play_order: i32,
    pub has_content: bool,
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
                "ncx_id",
                ObjectBuilder::new().schema_type(SchemaType::new(Type::String)),
            )
            .required("ncx_id")
            .property(
                "label",
                ObjectBuilder::new().schema_type(SchemaType::new(Type::String)),
            )
            .required("label")
            .property(
                "depth",
                ObjectBuilder::new().schema_type(SchemaType::new(Type::Integer)),
            )
            .required("depth")
            .property(
                "play_order",
                ObjectBuilder::new().schema_type(SchemaType::new(Type::Integer)),
            )
            .required("play_order")
            .property(
                "has_content",
                ObjectBuilder::new().schema_type(SchemaType::new(Type::Boolean)),
            )
            .required("has_content")
            .property(
                "children",
                ArrayBuilder::new().items(RefOr::Ref(utoipa::openapi::Ref::new(
                    "#/components/schemas/TocNodeResponse",
                ))),
            )
            .required("children")
            .build();

        schemas.push((
            "TocNodeResponse".to_string(),
            RefOr::T(Schema::Object(obj)),
        ));
    }
}
