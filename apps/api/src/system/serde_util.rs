use serde::{Deserialize, Deserializer};

/// Deserialize `Option<Option<T>>` so an absent field and an explicit `null`
/// are distinguishable — the three PATCH states a plain `Option<T>` can't
/// express. Use with `#[serde(default, deserialize_with = "double_option")]`:
///
/// - field absent    → `None`         (leave the column unchanged)
/// - field is `null`  → `Some(None)`   (clear the column to NULL)
/// - field has a value → `Some(Some(v))` (set the column)
///
/// Plain `Option<Option<T>>` without this doesn't work: serde folds a present
/// `null` into the outer `None`, collapsing "clear" back into "unchanged".
pub fn double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

#[cfg(test)]
mod tests {
    use super::double_option;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Patch {
        #[serde(default, deserialize_with = "double_option")]
        field: Option<Option<i32>>,
    }

    fn parse(json: &str) -> Option<Option<i32>> {
        serde_json::from_str::<Patch>(json).unwrap().field
    }

    #[test]
    fn absent_field_is_none() {
        assert_eq!(parse("{}"), None);
    }

    #[test]
    fn explicit_null_is_some_none() {
        assert_eq!(parse(r#"{"field": null}"#), Some(None));
    }

    #[test]
    fn value_is_some_some() {
        assert_eq!(parse(r#"{"field": 7}"#), Some(Some(7)));
    }
}
