use uuid::Uuid;

/// System user that owns all seed-imported persons/sources (see db schema).
/// `00000000-0000-0000-0000-000000000001` == u128 `1`.
pub const SYSTEM_USER_ID: Uuid = Uuid::from_u128(1);

#[cfg(test)]
mod tests {
    use super::SYSTEM_USER_ID;
    use uuid::Uuid;

    #[test]
    fn system_user_id_matches_canonical_string() {
        assert_eq!(
            SYSTEM_USER_ID,
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
        );
    }
}
